use {
    serde::{Deserialize, Serialize},
    std::str::FromStr,
    strum_macros::Display,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DataType {
    Boolean,
    Int8,
    Int16,
    Int32,
    Int,
    Int128,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Float32,
    Float,
    Text,
    Bytea,
    Inet,
    Date,
    Timestamp,
    Time,
    Interval,
    Uuid,
    Map,
    List,
    Decimal,
    Point,
}

impl FromStr for DataType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "boolean" => Ok(DataType::Boolean),
            "int8" => Ok(DataType::Int8),
            "int16" => Ok(DataType::Int16),
            "int32" => Ok(DataType::Int32),
            "int" | "integer" => Ok(DataType::Int),
            "int128" => Ok(DataType::Int128),
            "uint8" => Ok(DataType::Uint8),
            "uint16" => Ok(DataType::Uint16),
            "uint32" => Ok(DataType::Uint32),
            "uint64" => Ok(DataType::Uint64),
            "uint128" => Ok(DataType::Uint128),
            "float32" => Ok(DataType::Float32),
            "float" => Ok(DataType::Float),
            "text" | "varchar" | "string" => Ok(DataType::Text),
            "bytea" => Ok(DataType::Bytea),
            "inet" => Ok(DataType::Inet),
            "date" => Ok(DataType::Date),
            "timestamp" => Ok(DataType::Timestamp),
            "time" => Ok(DataType::Time),
            "interval" => Ok(DataType::Interval),
            "uuid" => Ok(DataType::Uuid),
            "map" => Ok(DataType::Map),
            "list" => Ok(DataType::List),
            "decimal" => Ok(DataType::Decimal),
            "point" => Ok(DataType::Point),
            _ => Err(()),
        }
    }
}
