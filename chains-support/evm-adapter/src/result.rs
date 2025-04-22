use {serde::Serialize, std::fmt::Debug, thiserror::Error as ThisError};

pub use crate::common::{
    account::AccountError, block::BlockError, chain::EvmChainError, entity::EntityError,
    entity_id::EntityIdError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum Error {
    #[error("evm: {0}")]
    EvmChainError(EvmChainError),

    #[error("entity_id: {0}")]
    EntityIdError(EntityIdError),

    #[error("entity: {0}")]
    EntityError(EntityError),

    #[error("account: {0}")]
    AccountError(AccountError),

    #[error("error: {0}")]
    BlockError(BlockError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
