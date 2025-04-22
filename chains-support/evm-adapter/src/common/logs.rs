use {
    std::str::FromStr,
    super::{
        block::BlockRange,
        entity_id::{parse_block_number_or_tag, EntityIdError},
    },
    alloy::{
        eips::BlockNumberOrTag,
        hex::FromHexError,
        primitives::{Address, AddressError, B256},
        rpc::types::Filter,
    },
    eql_macros::EnumVariants,
    pest::iterators::{Pair, Pairs},
    serde::{Deserialize, Serialize},
};


#[derive(Debug)]
pub enum LogEntityError {
    InvalidField(String),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum EntityFilterError {
    #[error("Invalid block number")]
    InvalidBlockNumber,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Logs {
    filter: Vec<LogFilter>,
    fields: Vec<LogField>,
}

impl Logs {
    pub fn new(filter: Vec<LogFilter>, fields: Vec<LogField>) -> Self {
        Self { filter, fields }
    }

    pub fn filter(&self) -> &Vec<LogFilter> {
        &self.filter
    }

    pub fn fields(&self) -> &Vec<LogField> {
        &self.fields
    }

    pub fn build_bloom_filter(&self) -> Filter {
        LogFilter::build_filter(&self.filter)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LogsError {
    #[error("Invalid log filter {0}")]
    InvalidLogFilter(String),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    EntityIdError(#[from] EntityIdError),
    #[error(transparent)]
    AddressError(#[from] AddressError),
    #[error(transparent)]
    LogFieldError(#[from] LogFieldError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LogFilter {
    BlockRange(BlockRange),
    BlockHash(B256),
    EmitterAddress(Address),
    EventSignature(String),
    Topic0(B256),
    Topic1(B256),
    Topic2(B256),
    Topic3(B256),
}

impl LogFilter {
    // TODO: remove this method
    pub fn to_block_range(
        &self,
    ) -> Result<(BlockNumberOrTag, Option<BlockNumberOrTag>), EntityFilterError> {
        match self {
            LogFilter::BlockRange(block_id) => Ok(block_id.range()),
            _ => Err(EntityFilterError::InvalidBlockNumber),
        }
    }

    fn to_filter(&self, filter: Filter) -> Filter {
        match self {
            LogFilter::BlockRange(range) => {
                filter
                    .from_block(range.start())
                    // If end is None, range is actually one block. unwrap_or will reuse start as range
                    .to_block(range.end().unwrap_or(range.start()))
            }
            LogFilter::BlockHash(hash) => filter.at_block_hash(*hash),
            LogFilter::EmitterAddress(address) => filter.address(*address),
            LogFilter::EventSignature(signature) => filter.event(signature),
            LogFilter::Topic0(topic_hash) => filter.event_signature(*topic_hash),
            LogFilter::Topic1(topic_hash) => filter.topic1(*topic_hash),
            LogFilter::Topic2(topic_hash) => filter.topic2(*topic_hash),
            LogFilter::Topic3(topic_hash) => filter.topic3(*topic_hash),
        }
    }

    pub fn build_filter(entity_filters: &[LogFilter]) -> Filter {
        entity_filters
            .iter()
            .fold(Filter::new(), |filter, entity_filter| {
                entity_filter.to_filter(filter)
            })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumVariants)]
pub enum LogField {
    Address,
    Topic0,
    Topic1,
    Topic2,
    Topic3,
    Data,
    BlockHash,
    BlockNumber,
    BlockTimestamp,
    TransactionHash,
    TransactionIndex,
    LogIndex,
    Removed,
    Chain,
}

impl std::fmt::Display for LogField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogField::Address => write!(f, "address"),
            LogField::Topic0 => write!(f, "topic0"),
            LogField::Topic1 => write!(f, "topic1"),
            LogField::Topic2 => write!(f, "topic2"),
            LogField::Topic3 => write!(f, "topic3"),
            LogField::Data => write!(f, "data"),
            LogField::BlockHash => write!(f, "block_hash"),
            LogField::BlockNumber => write!(f, "block_number"),
            LogField::BlockTimestamp => write!(f, "block_timestamp"),
            LogField::TransactionHash => write!(f, "transaction_hash"),
            LogField::TransactionIndex => write!(f, "transaction_index"),
            LogField::LogIndex => write!(f, "log_index"),
            LogField::Removed => write!(f, "removed"),
            LogField::Chain => write!(f, "chain"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LogFieldError {
    #[error("Invalid log field: {0}")]
    InvalidLogField(String),
}
