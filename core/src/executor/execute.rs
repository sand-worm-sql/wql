use {
    super::{
        fetch::fetch,
        select::{select, select_with_labels},
    },
    crate::{
        ast::{
            AstLiteral, BinaryOperator, DataType, Dictionary, Expr, Query, SelectItem, SetExpr,
            Show, Statement, TableAlias, TableFactor, TableWithJoins, Variable,
        },
        data::{Key, Row, Schema, Value},
        error::Error,
        result::Result,
        store::{GStore, GStoreMut},
    },
    futures::{
        future::join_all,
        stream::{StreamExt, TryStreamExt},
    },
    reqwest::Client,
    serde::{Deserialize, Serialize},
    serde_json::{json, Error as SerdeError, Value as JsonValue},
    std::{collections::HashMap, env::var, fmt::Debug, hash::Hash, rc::Rc, str::FromStr},
    thiserror::Error as ThisError,
};

#[derive(Debug, Deserialize)]
struct Chain {
    name: String,
    short_code: String,
}

#[derive(Debug, Deserialize)]
struct Entity {
    name: String,
    description: String,
    live_preview: String,
    fields: HashMap<String, String>,
}

#[derive(ThisError, Serialize, Debug, PartialEq, Eq)]
pub enum ExecuteError {
    #[error("table not found: {0}")]
    TableNotFound(String),
    #[error("Network error: {0}")]
    RequestFailed(String),
    #[error("Invalid HTTP response: {0}")]
    InvalidResponse(String),
    #[error("Failed to parse JSON data: {0}")]
    ParseError(String),
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Payload {
    ShowColumns(Vec<(String, DataType)>),
    ShowChains(Vec<(String, String)>),
    ShowChainsEntities(Vec<(String, String)>),
    Select {
        labels: Vec<String>,
        rows: Vec<Vec<Value>>,
    },
    SelectMap(Vec<HashMap<String, Value>>),
    ShowVariable(PayloadVariable),
}

impl Payload {
    /// Exports `select` payloads as an [`std::iter::Iterator`].
    ///
    /// The items of the Iterator are `HashMap<Column, Value>`, and they are borrowed by default.
    /// If ownership is required, you need to acquire them directly.
    ///
    /// - Some: [`Payload::Select`], [`Payload::SelectMap`]
    /// - None: otherwise
    pub fn select(&self) -> Option<impl Iterator<Item = HashMap<&str, &Value>>> {
        #[derive(iter_enum::Iterator)]
        enum Iter<I1, I2> {
            Schema(I1),
            Schemaless(I2),
        }

        Some(match self {
            Payload::Select { labels, rows } => Iter::Schema(rows.iter().map(move |row| {
                labels
                    .iter()
                    .zip(row.iter())
                    .map(|(label, value)| (label.as_str(), value))
                    .collect::<HashMap<_, _>>()
            })),
            Payload::SelectMap(rows) => Iter::Schemaless(rows.iter().map(|row| {
                row.iter()
                    .map(|(k, v)| (k.as_str(), v))
                    .collect::<HashMap<_, _>>()
            })),
            _ => return None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum PayloadVariable {
    Tables(Vec<String>),
    Functions(Vec<String>),
    Version(String),
}

pub async fn execute<T: GStore + GStoreMut>(
    storage: &mut T,
    statement: &Statement,
) -> Result<Payload> {
    let autocommit = storage.begin(true).await?;
    let result = execute_inner(storage, statement).await;

    if !autocommit {
        return result;
    }

    match result {
        Ok(payload) => storage.commit().await.map(|_| payload),
        Err(error) => {
            storage.rollback().await?;

            Err(error)
        }
    }
}

async fn execute_inner<T: GStore + GStoreMut>(
    storage: &mut T,
    statement: &Statement,
) -> Result<Payload> {
    match statement {
        //- Selection
        Statement::Query(query) => {
            let (labels, rows) = select_with_labels(storage, query, None).await?;

            match labels {
                Some(labels) => rows
                    .map(|row| row?.try_into_vec())
                    .try_collect::<Vec<_>>()
                    .await
                    .map(|rows| Payload::Select { labels, rows }),
                None => rows
                    .map(|row| row?.try_into_map())
                    .try_collect::<Vec<_>>()
                    .await
                    .map(Payload::SelectMap),
            }
        }

        Statement::Show(show_stmt) => match show_stmt {
            Show::Variable(variable) => match variable {
                Variable::Version => {
                    let version = var("CARGO_PKG_VERSION")
                        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_owned());
                    Ok(Payload::ShowVariable(PayloadVariable::Version(version)))
                }
                Variable::Chains => {
                    let chains = fetch_chains().await?;
                    Ok(Payload::ShowChains(chains))
                }
                Variable::Tables => {
                    let tables: Vec<String> = Vec::new();
                    Ok(Payload::ShowVariable(PayloadVariable::Tables(tables)))
                }
            },

            Show::ChainEntities { chain_name } => {
                let entities = fetch_entity_names(&chain_name).await?;
                Ok(Payload::ShowChainsEntities(entities))
            }

            Show::ChainEntitiesColumns {
                chain_name,
                entity_name,
            } => {
                let schema_result = fetch_entity_schemas(chain_name, entity_name).await?;
                let columns: Vec<(String, DataType)> = schema_result
                    .into_iter()
                    .flat_map(|(k, v)| match DataType::from_str(&v) {
                        Ok(data_type) => Some((k, data_type)),
                        Err(_) => None,
                    })
                    .collect();
                Ok(Payload::ShowColumns(columns))
            }
        },
    }
}

async fn fetch_chains() -> Result<Vec<(String, String)>> {
    let url = "https://raw.githubusercontent.com/sand-worm-labs/chain_registry/refs/heads/main/data/chain/index.json";
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| Error::Execute(ExecuteError::RequestFailed(e.to_string())))?;

    if !response.status().is_success() {
        return Err(Error::Execute(ExecuteError::InvalidResponse(
            response.status().to_string(),
        )));
    }

    let networks: Vec<Chain> = response
        .json()
        .await
        .map_err(|e| Error::Execute(ExecuteError::ParseError(e.to_string())))?;

    Ok(networks
        .into_iter()
        .map(|c| (c.name, c.short_code))
        .collect())
}

pub async fn fetch_entity_names(chain_name: &str) -> Result<Vec<(String, String)>> {
    let client = Client::new();
    let sources = ["decoded", "projects", "raw"];
    let base_url = "https://raw.githubusercontent.com/sand-worm-labs/chain_registry/refs/heads/main/data/entities";

    let fetches: Vec<Option<(String, String)>> =
        futures::future::join_all(sources.iter().map(|&source| {
            let url = format!("{}/{}/{}.json", base_url, chain_name, source);
            let client = client.clone();
            async move {
                let response = client.get(&url).send().await.ok()?;
                if response.status().is_success() {
                    let data: Vec<Entity> = response.json().await.ok()?;
                    Some(
                        data.iter()
                            .map(|e| (e.name.to_string(), source.to_string()))
                            .collect(),
                    )
                } else {
                    None
                }
            }
        }))
        .await;
    let results = fetches
        .into_iter()
        .flatten()
        .collect::<Vec<(String, String)>>();
    Ok(results)
}

pub async fn fetch_entity_schemas(chain: &str, source: &str) -> Result<Vec<(String, String)>> {
    let client = Client::new();
    let base_url = "https://raw.githubusercontent.com/sand-worm-labs/chain_registry/refs/heads/main/data/entities";
    let url = format!("{}/{}/{}.json", base_url, chain, source);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| Error::Execute(ExecuteError::RequestFailed(e.to_string())))?;
    if !response.status().is_success() {
        return Err(Error::Execute(ExecuteError::InvalidResponse(
            response.status().to_string(),
        )));
    }
    let entitites: Vec<Entity> = response
        .json()
        .await
        .map_err(|e| Error::Execute(ExecuteError::ParseError(e.to_string())))?;
    let all_fields: Vec<(String, String)> = entitites
        .iter()
        .flat_map(|e| {
            e.fields
                .iter()
                .map(move |(k, v)| (format!("{}.{}", e.name, k), v.clone()))
        })
        .collect();
    Ok(all_fields)
}
