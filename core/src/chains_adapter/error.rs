use {serde::Serialize, std::fmt::Debug, thiserror::Error};

#[derive(Error, Serialize, Debug, PartialEq, Eq)]
pub enum ChainAdapterError {
    #[error("failed to parseInvalid chain :{0}")]
    InvalidChain(String),

    #[error("adapter failed to parse query: {0}")]
    ChainFailedToParse(String),

    #[error("failed to parse chain: {0}")]
    ChainConvertFailed(String),

}
