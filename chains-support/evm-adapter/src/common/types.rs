use {
    super::{
        chain::{Chain, ChainError, ChainOrRpc},
        dump::{Dump, DumpError},
        entity::{Entity, EntityError},
    },
    alloy::transports::http::reqwest::Url,
    pest::iterators::Pairs,
};


#[derive(Debug, PartialEq)]
pub enum Expression {
    Get(GetExpression),
    Count(CountExpression),
}

#[derive(Debug, PartialEq)]
pub struct CountExpression {
    pub query: GetExpression,
}

impl CountExpression {
    fn new(query: GetExpression) -> Self {
        Self { query }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ExpressionError {
    #[error(transparent)]
    Count(#[from] CountExpressionError),

    #[error(transparent)]
    Get(#[from] GetExpressionError),
}

#[derive(thiserror::Error, Debug)]
pub enum CountExpressionError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Nested Count expression not allowed. Unexpected token: {0}")]
    NextedCountToken(String),
    #[error("Missing Count query not allowed. Unexpected token: {0}")]
    MissingCountQuery(String),

}

impl TryFrom<Pairs<'_, Rule>> for CountExpression {
    type Error = ExpressionError; // Use the new enum

    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self, Self::Error> {
        let mut query: Option<GetExpression> = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::query => {
                  query = Some(GetExpression::try_from(pair.into_inner())?);
                }
                _ => {
                    return Err(CountExpressionError::UnexpectedToken(pair.as_str().to_string()).into());
                }
            }
        }

        Ok(CountExpression::new(
            query.ok_or(CountExpressionError::MissingCountQuery(String::from("No query provided")))?
        ))
    }
}


#[derive(Debug, PartialEq)]
pub struct GetExpression {
    pub entity: Entity,
    pub chains: Vec<ChainOrRpc>,
    pub dump: Option<Dump>,
}

impl GetExpression {
    fn new(entity: Entity, chains: Vec<ChainOrRpc>, dump: Option<Dump>) -> Self {
        Self {
            entity,
            chains,
            dump,
        }
    }
}
#[derive(thiserror::Error, Debug)]
pub enum GetExpressionError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Missing entity")]
    MissingEntity,
    #[error("Missing chain or RPC")]
    MissingChainOrRpc,
    #[error("URL parse error: {0}")]
    UrlParseError(String),
    #[error(transparent)]
    EntityError(#[from] EntityError),
    #[error(transparent)]
    ChainError(#[from] ChainError),
    #[error(transparent)]
    DumpError(#[from] DumpError),
}

impl TryFrom<Pairs<'_, Rule>> for GetExpression {
    type Error = GetExpressionError;

    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self, Self::Error> {
        
        let mut entity: Option<Entity> = None;
        let mut chains: Option<Vec<ChainOrRpc>> = None;
        let mut dump: Option<Dump> = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::entity => {
                    entity = Some(Entity::try_from(pair.into_inner())?);
                }
                Rule::chain_selector => {
                    let selector = pair.as_str();
                    chains = Some(Chain::from_selector(selector)?);
                }
                Rule::rpc_url => {
                    let url = Url::parse(pair.as_str())
                        .map_err(|e| GetExpressionError::UrlParseError(e.to_string()))?;
                    chains = Some(vec![ChainOrRpc::Rpc(url)]);
                }
                Rule::dump => {
                    dump = Some(Dump::try_from(pair.into_inner())?);
                }
                _ => {
                    return Err(GetExpressionError::UnexpectedToken(
                        pair.as_str().to_string(),
                    ))
                }
            }
        }

        Ok(GetExpression::new(
            entity.ok_or(GetExpressionError::MissingEntity)?,
            chains.ok_or(GetExpressionError::MissingChainOrRpc)?,
            dump,
        ))
    }
}
