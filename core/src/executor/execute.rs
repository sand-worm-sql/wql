use {
    super::{   
        fetch::fetch,
        select::{select, select_with_labels},
    },
    crate::{
        ast::{
            AstLiteral, BinaryOperator, DataType, Dictionary, Expr, Query, SelectItem, SetExpr,
            Statement, TableAlias, TableFactor, TableWithJoins, Variable,
        },
        data::{Key, Row, Schema, Value},
        result::Result,
        store::{GStore, GStoreMut},
    },
    futures::stream::{StreamExt, TryStreamExt},
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, env::var, fmt::Debug, rc::Rc},
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq, Eq)]
pub enum ExecuteError {
    #[error("table not found: {0}")]
    TableNotFound(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Payload {
    ShowColumns(Vec<(String, DataType)>),
    Create,
    Insert(usize),
    Select {
        labels: Vec<String>,
        rows: Vec<Vec<Value>>,
    },
    SelectMap(Vec<HashMap<String, Value>>),
    Delete(usize),
    Update(usize),
    DropTable(usize),
    DropFunction,
    AlterTable,
    CreateIndex,
    DropIndex,
    StartTransaction,
    Commit,
    Rollback,
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
    if matches!(
        statement,
        Statement::StartTransaction | Statement::Rollback | Statement::Commit
    ) {
        return execute_inner(storage, statement).await;
    }

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
        Statement::ShowColumns { table_name } => {
            let Schema { column_defs, .. } = storage
                .fetch_schema(table_name)
                .await?
                .ok_or_else(|| ExecuteError::TableNotFound(table_name.to_owned()))?;

            let output: Vec<(String, DataType)> = column_defs
                .unwrap_or_default()
                .into_iter()
                .map(|key| (key.name, key.data_type))
                .collect();

            Ok(Payload::ShowColumns(output))
        }
        Statement::ShowIndexes(table_name) => {
            let query = Query {
                body: SetExpr::Select(Box::new(crate::ast::Select {
                    projection: vec![SelectItem::Wildcard],
                    from: TableWithJoins {
                        relation: TableFactor::Dictionary {
                            dict: Dictionary::GlueIndexes,
                            alias: TableAlias {
                                name: "GLUE_INDEXES".to_owned(),
                                columns: Vec::new(),
                            },
                        },
                        joins: Vec::new(),
                    },
                    selection: Some(Expr::BinaryOp {
                        left: Box::new(Expr::Identifier("TABLE_NAME".to_owned())),
                        op: BinaryOperator::Eq,
                        right: Box::new(Expr::Literal(AstLiteral::QuotedString(
                            table_name.to_owned(),
                        ))),
                    }),
                    group_by: Vec::new(),
                    having: None,
                })),
                order_by: Vec::new(),
                limit: None,
                offset: None,
            };

            let (labels, rows) = select_with_labels(storage, &query, None).await?;
            let labels = labels.unwrap_or_default();
            let rows = rows
                .map(|row| row?.try_into_vec())
                .try_collect::<Vec<_>>()
                .await?;

            if rows.is_empty() {
                return Err(ExecuteError::TableNotFound(table_name.to_owned()).into());
            }

            Ok(Payload::Select { labels, rows })
        }
        Statement::ShowVariable(variable) => match variable {
            Variable::Tables => {
                let query = Query {
                    body: SetExpr::Select(Box::new(crate::ast::Select {
                        projection: vec![SelectItem::Expr {
                            expr: Expr::Identifier("TABLE_NAME".to_owned()),
                            label: "TABLE_NAME".to_owned(),
                        }],
                        from: TableWithJoins {
                            relation: TableFactor::Dictionary {
                                dict: Dictionary::GlueTables,
                                alias: TableAlias {
                                    name: "GLUE_TABLES".to_owned(),
                                    columns: Vec::new(),
                                },
                            },
                            joins: Vec::new(),
                        },
                        selection: None,
                        group_by: Vec::new(),
                        having: None,
                    })),
                    order_by: Vec::new(),
                    limit: None,
                    offset: None,
                };

                let table_names = select(storage, &query, None)
                    .await?
                    .map(|row| row?.try_into_vec())
                    .try_collect::<Vec<Vec<Value>>>()
                    .await?
                    .iter()
                    .flat_map(|values| values.iter().map(|value| value.into()))
                    .collect::<Vec<_>>();

                Ok(Payload::ShowVariable(PayloadVariable::Tables(table_names)))
            }
            Variable::Version => {
                let version = var("CARGO_PKG_VERSION")
                    .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_owned());
                let payload = Payload::ShowVariable(PayloadVariable::Version(version));

                Ok(payload)
            }
        }
    }
}
