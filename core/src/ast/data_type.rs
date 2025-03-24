use {
    serde::{Deserialize, Serialize},
    strum_macros::Display,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DataType {
    Boolean,
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Int256,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Uint256,
    Bytea,
    Text,
    Date,
    Timestamp,
    Time,
    Interval,
    Uuid,
    Map,
    List,
}
