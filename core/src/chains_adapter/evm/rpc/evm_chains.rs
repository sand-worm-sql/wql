use {
    crate::{
        data::{Interval, Value},
        result::{Error, Result},
    },
    chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike},
    serde::{Deserialize, Serialize},
    std::{cmp::Ordering, fmt::Debug},
    thiserror::Error as ThisError,
};

#[derive(ThisError, Debug, PartialEq, Eq, Serialize)]
pub enum KeyError {
    #[error("FLOAT data type cannot be converted to Big-Endian bytes for comparison")]
    FloatToCmpBigEndianNotSupported,

    #[error("MAP data type cannot be used as Key")]
    MapTypeKeyNotSupported,

    #[error("LIST data type cannot be used as Key")]
    ListTypeKeyNotSupported,

    #[error("POINT data type cannot be used as Key")]
    PointTypeKeyNotSupported,
}