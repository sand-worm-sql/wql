use {
    super::{
        account::{Account, AccountError},
        block::{Block, BlockError},
        logs::{Logs, LogsError},
        transaction::{Transaction, TransactionError},
    },
    crate::result::{Error, Result},
    serde::Serialize,
    thiserror::Error as ThisError,
    wql_core::ast::{Select, SetExpr, TableFactor},
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum EntityError {
    #[error("Unexpected token {0}")]
    UnexpectedToken(String),

    #[error("Missing entity")]
    MissingEntity,

    #[error("Entity value error: {0}")]
    EntityValueError(String),

    #[error(transparent)]
    TransactionError(#[from] TransactionError),

    #[error(transparent)]
    LogsError(#[from] LogsError),

    #[error(transparent)]
    BlockError(#[from] BlockError),

    #[error(transparent)]
    AccountError(#[from] AccountError),
}

#[derive(Debug, PartialEq)]
pub enum Entity {
    Account(Account),
    Block(Block),
    Transaction(Transaction),
    Logs(Logs),
}

impl TryFrom<Select> for Entity {
    type Error = Error;

    fn try_from(select: Select) -> Result<Self> {
        parse_table_factor(&select.from.relation)
    }
}

fn parse_table_factor(factor: &TableFactor) -> Result<Entity> {
    match factor {
        TableFactor::Table {
            chain_name,
            name,
            alias,
            index,
            existing_table,
        } => {
            let table = TableFactor::Table {
                chain_name: chain_name.clone(),
                name: name.clone(),
                alias: alias.clone(),
                index: index.clone(),
                existing_table: *existing_table,
            };

            match name.to_ascii_lowercase().as_str() {
                "account" => Ok(Entity::Account(Account::try_from(table)?)),
                "block" => Ok(Entity::Block(Block::try_from(table)?)),
                "transaction" | "tx" => Ok(Entity::Transaction(Transaction::try_from(table)?)),
                "logs" | "log" => Ok(Entity::Logs(Logs::try_from(table)?)),
                _ => Err(Error::EntityError(EntityError::MissingEntity)),
            }
        }

        TableFactor::Derived { subquery, .. } => match &subquery.body {
            SetExpr::Select(inner) => Entity::try_from(inner.as_ref().clone()),
            SetExpr::Values(_) => Err(Error::EntityError(EntityError::EntityValueError(
                "VALUES expressions not supported in FROM".into(),
            ))),
            _ => Err(Error::EntityError(EntityError::MissingEntity)),
        },

        _ => Err(Error::EntityError(EntityError::MissingEntity)),
    }
}
