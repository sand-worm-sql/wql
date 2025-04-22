use {serde::Serialize, std::fmt::Debug, thiserror::Error as ThisError};

pub use crate::common::chain::SuiChainError;

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum Error {
    #[error("storage: {0}")]
    SuiChainError(SuiChainError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
