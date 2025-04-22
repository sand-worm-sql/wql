use {
    super::entity_id::{parse_block_number_or_tag, EntityIdError},
    crate::result::{Error, Result},
    alloy::{
        eips::BlockNumberOrTag,
        providers::{Provider, RootProvider},
        rpc::types::BlockTransactionsKind,
        transports::http::{Client, Http},
    },
    eql_macros::EnumVariants,
    serde::{Deserialize, Serialize},
    std::{
        fmt::{self, Display, Formatter},
        sync::Arc,
    },
    thiserror::Error as ThisError,
    wql_core::ast::TableFactor,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum BlockError {
    #[error("Unexpected token {0} for block")]
    UnexpectedToken(String),

    #[error(transparent)]
    EntityIdError(#[from] EntityIdError),

    #[error(transparent)]
    BlockFilterError(#[from] BlockFilterError),

    #[error(transparent)]
    BlockFieldError(#[from] BlockFieldError),

    #[error(transparent)]
    BlockRangeError(#[from] BlockRangeError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BlockId {
    Number(BlockNumberOrTag),
    Range(BlockRange),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
    // TODO: ids should be mandatory
    // TODO: ids should be a HashSet
    ids: Option<Vec<BlockId>>,
    filter: Option<Vec<BlockFilter>>,
    fields: Vec<BlockField>,
}

impl Block {
    pub fn new(
        ids: Option<Vec<BlockId>>,
        filter: Option<Vec<BlockFilter>>,
        fields: Vec<BlockField>,
    ) -> Self {
        Self {
            ids,
            filter,
            fields,
        }
    }

    pub fn ids(&self) -> Option<&Vec<BlockId>> {
        self.ids.as_ref()
    }

    pub fn fields(&self) -> &Vec<BlockField> {
        &self.fields
    }

    pub fn filters(&self) -> Option<&Vec<BlockFilter>> {
        self.filter.as_ref()
    }
}

impl TryFrom<TableFactor> for Block {
    type Error = Error;

    fn try_from(value: TableFactor) -> Result<Self> {
        let mut fields: Vec<BlockField> = vec![];
        let mut ids: Vec<BlockId> = vec![];
        let mut filter: Option<Vec<BlockFilter>> = None;

        Ok(Block {
            ids: Some(ids),
            filter,
            fields,
        })
    }
}

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum BlockFilterError {
    #[error("Invalid block filter property: {0}")]
    InvalidBlockFilterProperty(String),

    #[error(transparent)]
    EntityIdError(#[from] EntityIdError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BlockFilter {
    Range(BlockRange),
}

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum BlockFieldError {
    #[error("Invalid property for entity block: {0}")]
    InvalidBlockField(String),
}

// TODO: should include nonce, transactions and withdrawals
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumVariants)]
pub enum BlockField {
    Number,
    Timestamp,
    Size,
    Hash,
    ParentHash,
    StateRoot,
    TransactionsRoot,
    ReceiptsRoot,
    LogsBloom,
    ExtraData,
    MixHash,
    TotalDifficulty,
    BaseFeePerGas,
    WithdrawalsRoot,
    BlobGasUsed,
    ExcessBlobGas,
    ParentBeaconBlockRoot,
    Chain,
}

impl Display for BlockField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockField::Number => write!(f, "number"),
            BlockField::Timestamp => write!(f, "timestamp"),
            BlockField::Size => write!(f, "size"),
            BlockField::Hash => write!(f, "hash"),
            BlockField::ParentHash => write!(f, "parent_hash"),
            BlockField::StateRoot => write!(f, "state_root"),
            BlockField::TransactionsRoot => write!(f, "transactions_root"),
            BlockField::ReceiptsRoot => write!(f, "receipts_root"),
            BlockField::LogsBloom => write!(f, "logs_bloom"),
            BlockField::ExtraData => write!(f, "extra_data"),
            BlockField::MixHash => write!(f, "mix_hash"),
            BlockField::TotalDifficulty => write!(f, "total_difficulty"),
            BlockField::BaseFeePerGas => write!(f, "base_fee_per_gas"),
            BlockField::WithdrawalsRoot => write!(f, "withdrawals_root"),
            BlockField::BlobGasUsed => write!(f, "blob_gas_used"),
            BlockField::ExcessBlobGas => write!(f, "excess_blob_gas"),
            BlockField::ParentBeaconBlockRoot => write!(f, "parent_beacon_block_root"),
            BlockField::Chain => write!(f, "chain"),
        }
    }
}

impl TryFrom<&str> for BlockField {
    type Error = BlockFieldError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "number" => Ok(BlockField::Number),
            "timestamp" => Ok(BlockField::Timestamp),
            "size" => Ok(BlockField::Size),
            "hash" => Ok(BlockField::Hash),
            "parent_hash" => Ok(BlockField::ParentHash),
            "state_root" => Ok(BlockField::StateRoot),
            "transactions_root" => Ok(BlockField::TransactionsRoot),
            "receipts_root" => Ok(BlockField::ReceiptsRoot),
            "logs_bloom" => Ok(BlockField::LogsBloom),
            "extra_data" => Ok(BlockField::ExtraData),
            "mix_hash" => Ok(BlockField::MixHash),
            "total_difficulty" => Ok(BlockField::TotalDifficulty),
            "base_fee_per_gas" => Ok(BlockField::BaseFeePerGas),
            "withdrawals_root" => Ok(BlockField::WithdrawalsRoot),
            "blob_gas_used" => Ok(BlockField::BlobGasUsed),
            "excess_blob_gas" => Ok(BlockField::ExcessBlobGas),
            "parent_beacon_block_root" => Ok(BlockField::ParentBeaconBlockRoot),
            "chain" => Ok(BlockField::Chain),
            invalid_field => Err(BlockFieldError::InvalidBlockField(
                invalid_field.to_string(),
            )),
        }
    }
}

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum BlockRangeError {
    #[error("Unable to fetch block number {0}")]
    UnableToFetchBlockNumber(BlockNumberOrTag),
    #[error("Start block must be less than end block")]
    StartBlockMustBeLessThanEndBlock,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockRange {
    start: BlockNumberOrTag,
    end: Option<BlockNumberOrTag>,
}

impl BlockRange {
    pub fn new(start: BlockNumberOrTag, end: Option<BlockNumberOrTag>) -> Self {
        Self { start, end }
    }

    pub fn range(&self) -> (BlockNumberOrTag, Option<BlockNumberOrTag>) {
        (self.start, self.end)
    }

    pub fn start(&self) -> BlockNumberOrTag {
        self.start
    }

    pub fn end(&self) -> Option<BlockNumberOrTag> {
        self.end
    }

    pub async fn resolve_block_numbers(
        &self,
        provider: &Arc<RootProvider<Http<Client>>>,
    ) -> Result<Vec<u64>> {
        let (start_block, end_block) = self.range();
        let start_block_number = get_block_number_from_tag(provider.clone(), &start_block).await?;

        let end_block_number = match end_block {
            Some(end) => Some(get_block_number_from_tag(provider.clone(), &end).await?),
            None => None,
        };

        if let Some(end) = end_block_number {
            if start_block_number > end {
                return Err(Error::BlockError(BlockError::BlockRangeError(
                    BlockRangeError::StartBlockMustBeLessThanEndBlock,
                )));
            }
        }

        match end_block_number {
            Some(end) => {
                let range = start_block_number..=end;
                return Ok(range.collect());
            }
            None => Ok(vec![start_block_number]),
        }
    }
}

impl Display for BlockRange {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let start = match &self.start {
            BlockNumberOrTag::Number(number) => number.to_string(),
            _ => self.start.to_string(),
        };

        if let Some(end) = &self.end {
            let end = match end {
                BlockNumberOrTag::Number(number) => number.to_string(),
                _ => end.to_string(),
            };
            write!(f, "{}:{}", start, end)
        } else {
            write!(f, "{}", start)
        }
    }
}

pub async fn get_block_number_from_tag(
    provider: Arc<RootProvider<Http<Client>>>,
    block_number_or_tag: &BlockNumberOrTag,
) -> Result<u64, Error> {
    match block_number_or_tag {
        BlockNumberOrTag::Number(number) => Ok(*number),
        tag => {
            let block_opt = provider
                .get_block_by_number(*tag, BlockTransactionsKind::Hashes)
                .await
                .map_err(|_| {
                    Error::BlockError(BlockError::BlockRangeError(
                        BlockRangeError::UnableToFetchBlockNumber(tag.clone()),
                    ))
                })?;

            if let Some(block) = block_opt {
                Ok(block.header.number)
            } else {
                Err(Error::BlockError(BlockError::BlockRangeError(
                    BlockRangeError::UnableToFetchBlockNumber(tag.clone()),
                )))
            }
        }
    }
}
