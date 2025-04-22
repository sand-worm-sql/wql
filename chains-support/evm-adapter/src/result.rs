use {serde::Serialize, std::fmt::Debug, thiserror::Error as ThisError};

pub use crate::common::{
    chain::EvmChainError,
    entity_id::EntityIdError,
    entity::EntityError,
    account::AccountError
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum Error {
    #[error("evm: {0}")]
    EvmChainError(EvmChainError),

    #[error("entity_id: {0}")]
    EntityIdError(EntityIdError),

    #[error("entity: {0}")]
    EntityError(EntityError),

    #[error("error: {0}")]
    AccountError(AccountError)
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
