use {
    crate::result::{Error, Result},
    alloy::eips::BlockNumberOrTag,
    serde::Serialize,
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

#[derive(Debug, PartialEq, Eq, ThisError, Serialize)]
pub enum EntityIdError {
    #[error("Invalid address")]
    InvalidAddress,
    #[error("Invalid tx hash")]
    InvalidTxHash,
    #[error("Invalid block number or tag: {0}")]
    InvalidBlockNumberOrTag(String),
    #[error("Unable resolve ENS name")]
    EnsResolution,
}

pub fn parse_block_number_or_tag(id: &str) -> Result<BlockNumberOrTag> {
    match id.trim().parse::<u64>() {
        Ok(id) => Ok(BlockNumberOrTag::Number(id)),
        Err(_) => id.parse::<BlockNumberOrTag>().map_err(|_| {
            Error::EntityIdError(EntityIdError::InvalidBlockNumberOrTag(id.to_string()))
        }),
    }
}
