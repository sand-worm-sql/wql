use {
    super::{
        date::{parse_date, parse_time, parse_timestamp},
        Value,
    },
    crate::ast::DataType,
    chrono::{NaiveDate, NaiveDateTime, NaiveTime},
    rust_decimal::prelude::ToPrimitive,
    serde::Serialize,
    uuid::Uuid,
};

type Result<T> = std::result::Result<T, ConvertError>;

#[derive(Debug, Serialize, thiserror::Error, PartialEq)]
#[error("failed to convert value({value:?}) to data type({data_type})")]
pub struct ConvertError {
    pub value: Value,
    pub data_type: DataType,
}

// implies `TryFrom<Value> for T` from `TryFrom<&Value> for T`
macro_rules! try_from_owned_value {
    ($($target:ty), *) => {$(
        impl TryFrom<Value> for $target {
            type Error = ConvertError;

            fn try_from(v: Value) -> Result<Self> {
                Self::try_from(&v)
            }
        }
    )*}
}

try_from_owned_value!(
    bool, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize
);

impl From<&Value> for String {
    fn from(v: &Value) -> Self {
        match v {
            Value::Str(value) => value.to_owned(),
            Value::Bytes(value) => hex::encode(value),
            Value::Bool(value) => (if *value { "TRUE" } else { "FALSE" }).to_owned(),
            Value::I8(value) => value.to_string(),
            Value::I16(value) => value.to_string(),
            Value::I32(value) => value.to_string(),
            Value::I64(value) => value.to_string(),
            Value::I128(value) => value.to_string(),
            Value::U8(value) => value.to_string(),
            Value::U16(value) => value.to_string(),
            Value::U32(value) => value.to_string(),
            Value::U64(value) => value.to_string(),
            Value::U128(value) => value.to_string(),
            Value::Date(value) => value.to_string(),
            Value::Timestamp(value) => value.to_string(),
            Value::Time(value) => value.to_string(),
            Value::Interval(value) => value.to_sql_str(),
            Value::Uuid(value) => Uuid::from_u128(*value).to_string(),
            Value::Map(_) => TryInto::<serde_json::Value>::try_into(v.clone())
                .unwrap_or_default()
                .to_string(),
            Value::List(_) => TryInto::<serde_json::Value>::try_into(v.clone())
                .unwrap_or_default()
                .to_string(),
            Value::Null => "NULL".to_owned(),
        }
    }
}

impl From<Value> for String {
    fn from(v: Value) -> String {
        match v {
            Value::Str(value) => value,
            _ => String::from(&v),
        }
    }
}

impl TryFrom<&Value> for bool {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<Self> {
        macro_rules! int_to_bool {
            ($num: ident) => {
                match $num {
                    1 => true,
                    0 => false,
                    _ => {
                        return Err(ConvertError {
                            value: v.clone(),
                            data_type: DataType::Boolean,
                        })
                    }
                }
            };
        }

        Ok(match v {
            Value::Bool(value) => *value,
            Value::I8(value) => int_to_bool!(value),
            Value::I16(value) => int_to_bool!(value),
            Value::I32(value) => int_to_bool!(value),
            Value::I64(value) => int_to_bool!(value),
            Value::I128(value) => int_to_bool!(value),
            Value::U8(value) => int_to_bool!(value),
            Value::U16(value) => int_to_bool!(value),
            Value::U32(value) => int_to_bool!(value),
            Value::U64(value) => int_to_bool!(value),
            Value::U128(value) => int_to_bool!(value),
            Value::Str(value) => match value.to_uppercase().as_str() {
                "TRUE" => true,
                "FALSE" => false,
                _ => {
                    return Err(ConvertError {
                        value: v.clone(),
                        data_type: DataType::Boolean,
                    })
                }
            },

            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Boolean,
                })
            }
        })
    }
}

impl TryFrom<&Value> for i8 {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<i8> {
        macro_rules! num_to_i8 {
            ($num: ident) => {
                $num.to_i8().ok_or_else(|| ConvertError {
                    value: v.clone(),
                    data_type: DataType::Int8,
                })?
            };
        }

        Ok(match v {
            Value::Bool(value) => i8::from(*value),
            Value::I8(value) => *value,
            Value::I16(value) => num_to_i8!(value),
            Value::I32(value) => num_to_i8!(value),
            Value::I64(value) => num_to_i8!(value),
            Value::I128(value) => num_to_i8!(value),
            Value::U8(value) => num_to_i8!(value),
            Value::U16(value) => num_to_i8!(value),
            Value::U32(value) => num_to_i8!(value),
            Value::U64(value) => num_to_i8!(value),
            Value::U128(value) => num_to_i8!(value),
            Value::Str(value) => value.parse::<i8>().map_err(|_| ConvertError {
                value: v.clone(),
                data_type: DataType::Int8,
            })?,

            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Int8,
                })
            }
        })
    }
}

impl TryFrom<&Value> for i16 {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<i16> {
        macro_rules! num_to_i16 {
            ($num: ident) => {
                $num.to_i16().ok_or_else(|| ConvertError {
                    value: v.clone(),
                    data_type: DataType::Int16,
                })?
            };
        }

        Ok(match v {
            Value::Bool(value) => i16::from(*value),
            Value::I8(value) => *value as i16,
            Value::I16(value) => *value,
            Value::I32(value) => num_to_i16!(value),
            Value::I64(value) => num_to_i16!(value),
            Value::I128(value) => num_to_i16!(value),
            Value::U8(value) => num_to_i16!(value),
            Value::U16(value) => num_to_i16!(value),
            Value::U32(value) => num_to_i16!(value),
            Value::U64(value) => num_to_i16!(value),
            Value::U128(value) => num_to_i16!(value),
            Value::Str(value) => value.parse::<i16>().map_err(|_| ConvertError {
                value: v.clone(),
                data_type: DataType::Int16,
            })?,

            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Int16,
                })
            }
        })
    }
}

impl TryFrom<&Value> for i32 {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<i32> {
        macro_rules! num_to_i32 {
            ($num: ident) => {
                $num.to_i32().ok_or_else(|| ConvertError {
                    value: v.clone(),
                    data_type: DataType::Int32,
                })?
            };
        }

        Ok(match v {
            Value::Bool(value) => i32::from(*value),
            Value::I8(value) => *value as i32,
            Value::I16(value) => *value as i32,
            Value::I32(value) => *value,
            Value::I64(value) => num_to_i32!(value),
            Value::I128(value) => num_to_i32!(value),
            Value::U8(value) => num_to_i32!(value),
            Value::U16(value) => num_to_i32!(value),
            Value::U32(value) => num_to_i32!(value),
            Value::U64(value) => num_to_i32!(value),
            Value::U128(value) => num_to_i32!(value),
            Value::Str(value) => value.parse::<i32>().map_err(|_| ConvertError {
                value: v.clone(),
                data_type: DataType::Int32,
            })?,
            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Int32,
                })
            }
        })
    }
}

impl TryFrom<&Value> for i64 {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<i64> {
        macro_rules! num_to_i64 {
            ($num: ident) => {
                $num.to_i64().ok_or_else(|| ConvertError {
                    value: v.clone(),
                    data_type: DataType::Int,
                })?
            };
        }

        Ok(match v {
            Value::Bool(value) => i64::from(*value),
            Value::I8(value) => *value as i64,
            Value::I16(value) => *value as i64,
            Value::I32(value) => *value as i64,
            Value::I64(value) => *value,
            Value::I128(value) => num_to_i64!(value),
            Value::U8(value) => num_to_i64!(value),
            Value::U16(value) => num_to_i64!(value),
            Value::U32(value) => num_to_i64!(value),
            Value::U64(value) => num_to_i64!(value),
            Value::U128(value) => num_to_i64!(value),
            Value::Str(value) => value.parse::<i64>().map_err(|_| ConvertError {
                value: v.clone(),
                data_type: DataType::Int,
            })?,

            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Int,
                })
            }
        })
    }
}

impl TryFrom<&Value> for i128 {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<i128> {
        macro_rules! num_to_i128 {
            ($num: ident) => {
                $num.to_i128().ok_or_else(|| ConvertError {
                    value: v.clone(),
                    data_type: DataType::Int128,
                })?
            };
        }

        Ok(match v {
            Value::Bool(value) => i128::from(*value),
            Value::I8(value) => *value as i128,
            Value::I16(value) => *value as i128,
            Value::I32(value) => *value as i128,
            Value::I64(value) => *value as i128,
            Value::I128(value) => *value,
            Value::U8(value) => *value as i128,
            Value::U16(value) => *value as i128,
            Value::U32(value) => num_to_i128!(value),
            Value::U64(value) => num_to_i128!(value),
            Value::U128(value) => num_to_i128!(value),
            Value::Str(value) => value.parse::<i128>().map_err(|_| ConvertError {
                value: v.clone(),
                data_type: DataType::Int128,
            })?,

            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Int128,
                })
            }
        })
    }
}

impl TryFrom<&Value> for u8 {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<u8> {
        macro_rules! num_to_u8 {
            ($num: ident) => {
                $num.to_u8().ok_or_else(|| ConvertError {
                    value: v.clone(),
                    data_type: DataType::Uint8,
                })?
            };
        }

        Ok(match v {
            Value::Bool(value) => u8::from(*value),
            Value::I8(value) => num_to_u8!(value),
            Value::I16(value) => num_to_u8!(value),
            Value::I32(value) => num_to_u8!(value),
            Value::I64(value) => num_to_u8!(value),
            Value::I128(value) => num_to_u8!(value),
            Value::U8(value) => *value,
            Value::U16(value) => num_to_u8!(value),
            Value::U32(value) => num_to_u8!(value),
            Value::U64(value) => num_to_u8!(value),
            Value::U128(value) => num_to_u8!(value),
            Value::Str(value) => value.parse::<u8>().map_err(|_| ConvertError {
                value: v.clone(),
                data_type: DataType::Uint8,
            })?,

            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Uint8,
                })
            }
        })
    }
}
impl TryFrom<&Value> for u16 {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<u16> {
        macro_rules! num_to_u16 {
            ($num: ident) => {
                $num.to_u16().ok_or_else(|| ConvertError {
                    value: v.clone(),
                    data_type: DataType::Uint16,
                })?
            };
        }

        Ok(match v {
            Value::Bool(value) => u16::from(*value),
            Value::I8(value) => num_to_u16!(value),
            Value::I16(value) => num_to_u16!(value),
            Value::I32(value) => num_to_u16!(value),
            Value::I64(value) => num_to_u16!(value),
            Value::I128(value) => num_to_u16!(value),
            Value::U8(value) => u16::from(*value),
            Value::U16(value) => *value,
            Value::U32(value) => num_to_u16!(value),
            Value::U64(value) => num_to_u16!(value),
            Value::U128(value) => num_to_u16!(value),
            Value::Str(value) => value.parse::<u16>().map_err(|_| ConvertError {
                value: v.clone(),
                data_type: DataType::Uint16,
            })?,

            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Uint16,
                })
            }
        })
    }
}

impl TryFrom<&Value> for u32 {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<u32> {
        macro_rules! num_to_u32 {
            ($num: ident) => {
                $num.to_u32().ok_or_else(|| ConvertError {
                    value: v.clone(),
                    data_type: DataType::Uint32,
                })?
            };
        }

        Ok(match v {
            Value::Bool(value) => u32::from(*value),
            Value::I8(value) => num_to_u32!(value),
            Value::I16(value) => num_to_u32!(value),
            Value::I32(value) => num_to_u32!(value),
            Value::I64(value) => num_to_u32!(value),
            Value::I128(value) => num_to_u32!(value),
            Value::U8(value) => u32::from(*value),
            Value::U16(value) => u32::from(*value),
            Value::U32(value) => *value,
            Value::U64(value) => num_to_u32!(value),
            Value::U128(value) => num_to_u32!(value),
            Value::Str(value) => value.parse::<u32>().map_err(|_| ConvertError {
                value: v.clone(),
                data_type: DataType::Uint32,
            })?,

            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Uint32,
                })
            }
        })
    }
}

impl TryFrom<&Value> for u64 {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<u64> {
        macro_rules! num_to_u64 {
            ($num: ident) => {
                $num.to_u64().ok_or_else(|| ConvertError {
                    value: v.clone(),
                    data_type: DataType::Uint64,
                })?
            };
        }

        Ok(match v {
            Value::Bool(value) => u64::from(*value),
            Value::I8(value) => num_to_u64!(value),
            Value::I16(value) => num_to_u64!(value),
            Value::I32(value) => num_to_u64!(value),
            Value::I64(value) => num_to_u64!(value),
            Value::I128(value) => num_to_u64!(value),
            Value::U8(value) => u64::from(*value),
            Value::U16(value) => u64::from(*value),
            Value::U32(value) => u64::from(*value),
            Value::U64(value) => *value,
            Value::U128(value) => num_to_u64!(value),
            Value::Str(value) => value.parse::<u64>().map_err(|_| ConvertError {
                value: v.clone(),
                data_type: DataType::Uint64,
            })?,

            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Uint64,
                })
            }
        })
    }
}

impl TryFrom<&Value> for u128 {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<u128> {
        macro_rules! num_to_u128 {
            ($num: ident) => {
                $num.to_u128().ok_or_else(|| ConvertError {
                    value: v.clone(),
                    data_type: DataType::Uint128,
                })?
            };
        }

        Ok(match v {
            Value::Bool(value) => u128::from(*value),
            Value::I8(value) => num_to_u128!(value),
            Value::I16(value) => num_to_u128!(value),
            Value::I32(value) => num_to_u128!(value),
            Value::I64(value) => num_to_u128!(value),
            Value::I128(value) => num_to_u128!(value),
            Value::U8(value) => u128::from(*value),
            Value::U16(value) => u128::from(*value),
            Value::U32(value) => u128::from(*value),
            Value::U64(value) => u128::from(*value),
            Value::U128(value) => *value,
            Value::Str(value) => value.parse::<u128>().map_err(|_| ConvertError {
                value: v.clone(),
                data_type: DataType::Uint128,
            })?,
            Value::Uuid(value) => *value,
            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Uint128,
                })
            }
        })
    }
}


impl TryFrom<&Value> for usize {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<usize> {
        let err = || ConvertError {
            value: v.clone(),
            #[cfg(target_pointer_width = "64")]
            data_type: DataType::Uint64,
            #[cfg(target_pointer_width = "32")]
            data_type: DataType::Uint32,
        };

        macro_rules! num_to_usize {
            ($num: ident) => {
                $num.to_usize().ok_or_else(err)?
            };
        }

        Ok(match v {
            Value::Bool(value) => usize::from(*value),
            Value::I8(value) => num_to_usize!(value),
            Value::I16(value) => num_to_usize!(value),
            Value::I32(value) => num_to_usize!(value),
            Value::I64(value) => num_to_usize!(value),
            Value::I128(value) => num_to_usize!(value),
            Value::U8(value) => num_to_usize!(value),
            Value::U16(value) => num_to_usize!(value),
            Value::U32(value) => num_to_usize!(value),
            Value::U64(value) => num_to_usize!(value),
            Value::U128(value) => num_to_usize!(value),
            Value::Str(value) => value.parse::<usize>().map_err(|_| err())?,

            Value::Date(_)
            | Value::Timestamp(_)
            | Value::Time(_)
            | Value::Interval(_)
            | Value::Uuid(_)
            | Value::Map(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Null => return Err(err()),
        })
    }
}


impl TryFrom<&Value> for NaiveDate {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<NaiveDate> {
        Ok(match v {
            Value::Date(value) => *value,
            Value::Timestamp(value) => value.date(),
            Value::Str(value) => parse_date(value).ok_or_else(|| ConvertError {
                value: v.clone(),
                data_type: DataType::Date,
            })?,

            _ => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Date,
                })
            }
        })
    }
}

impl TryFrom<&Value> for NaiveTime {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<NaiveTime> {
        Ok(match v {
            Value::Time(value) => *value,
            Value::Str(value) => parse_time(value).ok_or_else(|| ConvertError {
                value: v.clone(),
                data_type: DataType::Time,
            })?,

            _ => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Time,
                })
            }
        })
    }
}

impl TryFrom<&Value> for NaiveDateTime {
    type Error = ConvertError;

    fn try_from(v: &Value) -> Result<NaiveDateTime> {
        Ok(match v {
            Value::Date(value) => value.and_hms_opt(0, 0, 0).ok_or_else(|| ConvertError {
                value: v.clone(),
                data_type: DataType::Timestamp,
            })?,
            Value::Str(value) => parse_timestamp(value).ok_or_else(|| ConvertError {
                value: v.clone(),
                data_type: DataType::Timestamp,
            })?,
            Value::Timestamp(value) => *value,

            _ => {
                return Err(ConvertError {
                    value: v.clone(),
                    data_type: DataType::Timestamp,
                })
            }
        })
    }
}


#[cfg(test)]
mod tests {
    use {
        super::{ConvertError, Result, Value},
        crate::{ast::DataType, data::Interval as I},
        chrono::{self, NaiveDate, NaiveDateTime, NaiveTime},
        std::collections::HashMap,
    };

    fn timestamp(y: i32, m: u32, d: u32, hh: u32, mm: u32, ss: u32, ms: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_milli_opt(hh, mm, ss, ms)
            .unwrap()
    }

    fn time(hour: u32, min: u32, sec: u32, milli: u32) -> NaiveTime {
        NaiveTime::from_hms_milli_opt(hour, min, sec, milli).unwrap()
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn from() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!(String::from($from), $to.to_owned())
            };
        }

        test!(Value::Str("text".to_owned()), "text");
        test!(Value::Bytes(hex::decode("1234").unwrap()), "1234");
        test!(Value::Bool(true), "TRUE");
        test!(Value::I8(122), "122");
        test!(Value::I16(122), "122");
        test!(Value::I32(122), "122");
        test!(Value::I64(1234567890), "1234567890");
        test!(Value::I128(1234567890), "1234567890");
        test!(Value::U8(122), "122");
        test!(Value::U16(122), "122");
        test!(Value::U32(122), "122");
        test!(Value::U64(122), "122");
        test!(Value::U128(122), "122");
        test!(Value::Date(date(2021, 11, 20)), "2021-11-20");
        test!(
            Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)),
            "2021-11-20 10:00:00"
        );
        test!(Value::Time(time(10, 0, 0, 0)), "10:00:00");
        test!(Value::Interval(I::Month(1)), I::Month(1).to_sql_str());
        test!(
            Value::Uuid(195965723427462096757863453463987888808),
            "936da01f-9abd-4d9d-80c7-02af85c822a8"
        );
        test!(Value::Map(HashMap::new()), "{}");
        test!(Value::List(Vec::new()), "[]");

        let mut map = HashMap::new();
        map.insert("abc".to_owned(), Value::I32(123));
        test!(Value::Map(map), "{\"abc\":123}");
        test!(Value::List(vec![Value::I32(1), Value::I32(2)]), "[1,2]");
        test!(Value::Null, "NULL");
    }

    #[test]
    fn try_into_bool() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<bool>, $to);
                assert_eq!(bool::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Boolean,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(true));
        test!(Value::I8(1), Ok(true));
        test!(Value::I8(0), Ok(false));
        test!(Value::I16(1), Ok(true));
        test!(Value::I16(0), Ok(false));
        test!(Value::I32(1), Ok(true));
        test!(Value::I32(0), Ok(false));
        test!(Value::I64(1), Ok(true));
        test!(Value::I64(0), Ok(false));
        test!(Value::I128(1), Ok(true));
        test!(Value::I128(0), Ok(false));
        test!(Value::U8(1), Ok(true));
        test!(Value::U8(0), Ok(false));

        test!(Value::U16(1), Ok(true));
        test!(Value::U16(0), Ok(false));
        test!(Value::U32(1), Ok(true));
        test!(Value::U32(0), Ok(false));
        test!(Value::U64(1), Ok(true));
        test!(Value::U64(0), Ok(false));
        test!(Value::U128(1), Ok(true));
        test!(Value::U128(0), Ok(false));

        test!(Value::Str("true".to_owned()), Ok(true));
        test!(Value::Str("false".to_owned()), Ok(false));

        err!(Value::I8(3));
        err!(Value::I16(3));
        err!(Value::I32(3));
        err!(Value::I64(3));
        err!(Value::I128(3));
        err!(Value::U8(3));
        err!(Value::U16(3));
        err!(Value::U32(3));
        err!(Value::U64(3));
        err!(Value::U128(3));
        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));
        err!(Value::Null);
    }

    #[test]
    fn try_into_i8() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<i8>, $to);
                assert_eq!(i8::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Int8,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(1));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::Str("122".to_owned()), Ok(122));

        err!(Value::I16(128));
        err!(Value::I32(128));
        err!(Value::I64(128));
        err!(Value::I128(128));
        err!(Value::U8(128));
        err!(Value::U16(128));
        err!(Value::U32(128));
        err!(Value::U64(128));
        err!(Value::U128(128));
        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));

        err!(Value::Null);
    }

    #[test]
    fn try_into_i16() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<i16>, $to);
                assert_eq!(i16::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Int16,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(1));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::Str("122".to_owned()), Ok(122));

        err!(Value::I32(i32::MAX));
        err!(Value::I64(i64::MAX));
        err!(Value::I128(i128::MAX));

        err!(Value::U16(u16::MAX));
        err!(Value::U32(u32::MAX));
        err!(Value::U64(u64::MAX));
        err!(Value::U128(u128::MAX));

        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));

        err!(Value::Null);
    }

    #[test]
    fn try_into_i32() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<i32>, $to);
                assert_eq!(i32::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Int32,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(1));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::I64(1234567890), Ok(1234567890));
        test!(Value::Str("1234567890".to_owned()), Ok(1234567890));

        err!(Value::I64(i64::MAX));
        err!(Value::I128(i128::MAX));

        err!(Value::U32(u32::MAX));
        err!(Value::U64(u64::MAX));
        err!(Value::U128(u128::MAX));

        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));

        err!(Value::Null);
    }

    #[test]
    fn try_into_i64() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<i64>, $to);
                assert_eq!(i64::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Int,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(1));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::I64(1234567890), Ok(1234567890));
        test!(Value::Str("1234567890".to_owned()), Ok(1234567890));

        err!(Value::I128(i128::MAX));

        err!(Value::U64(u64::MAX));
        err!(Value::U128(u128::MAX));

        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));
        err!(Value::Null);
    }

    #[test]
    fn try_into_i128() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<i128>, $to);
                assert_eq!(i128::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Int128,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(1));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::I64(1234567890), Ok(1234567890));
        test!(Value::Str("1234567890".to_owned()), Ok(1234567890));

        err!(Value::U128(u128::MAX));

        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));
        err!(Value::Null);
    }

    #[test]
    fn try_into_u8() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<u8>, $to);
                assert_eq!(u8::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Uint8,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(1));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::Str("122".to_owned()), Ok(122));

        // impossible casts to u8
        err!(Value::I16(256));
        err!(Value::I32(256));
        err!(Value::I64(256));
        err!(Value::I128(256));

        err!(Value::U16(256));
        err!(Value::U32(256));
        err!(Value::U64(256));
        err!(Value::U128(256));

    
        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::List(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));
        err!(Value::Null);
    }

    #[test]
    fn try_into_u16() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<u16>, $to);
                assert_eq!(u16::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Uint16,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(1));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::Str("122".to_owned()), Ok(122));

        err!(Value::I32(65536));
        err!(Value::I64(65536));
        err!(Value::I128(65536));

        err!(Value::U32(65536));
        err!(Value::U64(65536));
        err!(Value::U128(65536));

        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));
        err!(Value::Null);
    }

    #[test]
    fn try_into_u32() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<u32>, $to);
                assert_eq!(u32::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Uint32,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(1));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::Str("122".to_owned()), Ok(122));

        err!(Value::I64(i64::MAX));
        err!(Value::I128(i128::MAX));

        err!(Value::U64(u64::MAX));
        err!(Value::U128(u128::MAX));


        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));
        err!(Value::Null);

    }

    #[test]
    fn try_into_u64() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<u64>, $to);
                assert_eq!(u64::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Uint64,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(1));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::Str("122".to_owned()), Ok(122));

        err!(Value::I128(i128::MIN));

        err!(Value::U128(u128::MAX));

        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));

        err!(Value::Null);
    }

    #[test]
    fn try_into_u128() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<u128>, $to);
                assert_eq!(u128::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Uint128,
                    })
                )
            };
        }

        test!(Value::Bool(true), Ok(1));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::Str("122".to_owned()), Ok(122));

        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));

        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));
        err!(Value::Null);

        let uuid = 195965723427462096757863453463987888808;
        assert_eq!((&Value::Uuid(uuid)).try_into() as Result<u128>, Ok(uuid));
        assert_eq!(u128::try_from(&Value::Uuid(uuid)), Ok(uuid));

    }

    #[test]
    fn try_into_usize() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<usize>, $to);
                assert_eq!(usize::try_from(&$from), $to);
            };
        }

        let err = |value: Value| ConvertError {
            value,
            #[cfg(target_pointer_width = "64")]
            data_type: DataType::Uint64,
            #[cfg(target_pointer_width = "32")]
            data_type: DataType::Uint32,
        };
        macro_rules! err {
            ($from: expr) => {
                test!($from, Err(err($from.clone())))
            };
        }

        test!(Value::Bool(true), Ok(1usize));
        test!(Value::Bool(false), Ok(0));
        test!(Value::I8(122), Ok(122));
        test!(Value::I16(122), Ok(122));
        test!(Value::I32(122), Ok(122));
        test!(Value::I64(122), Ok(122));
        test!(Value::I128(122), Ok(122));
        test!(Value::U8(122), Ok(122));
        test!(Value::U16(122), Ok(122));
        test!(Value::U32(122), Ok(122));
        test!(Value::U64(122), Ok(122));
        test!(Value::U128(122), Ok(122));
        test!(Value::I64(1234567890), Ok(1234567890));
        test!(Value::Str("1234567890".to_owned()), Ok(1234567890));

        err!(Value::I128(i128::MIN));

        err!(Value::U128(u128::MAX));

        err!(Value::Str("text".to_owned()));
        err!(Value::Bytes(Vec::new()));
        err!(Value::Date(date(2021, 11, 20)));
        err!(Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)));
        err!(Value::Time(time(10, 0, 0, 0)));
        err!(Value::Interval(I::Month(1)));
        err!(Value::Uuid(195965723427462096757863453463987888808));
        err!(Value::Map(HashMap::new()));
        err!(Value::List(Vec::new()));

        err!(Value::Null);
    }

    #[test]
    fn try_into_naive_date() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<chrono::NaiveDate>, $to);
                assert_eq!(chrono::NaiveDate::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Date,
                    })
                )
            };
        }

        test!(Value::Date(date(2021, 11, 20)), Ok(date(2021, 11, 20)));
        test!(
            Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)),
            Ok(date(2021, 11, 20))
        );
        test!(Value::Str("2021-11-20".to_owned()), Ok(date(2021, 11, 20)));
    }

    #[test]
    fn try_into_naive_time() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<chrono::NaiveTime>, $to);
                assert_eq!(chrono::NaiveTime::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Time,
                    })
                )
            };
        }

        test!(Value::Time(time(10, 0, 0, 0)), Ok(time(10, 0, 0, 0)));
        test!(Value::Str("10:00:00".to_owned()), Ok(time(10, 0, 0, 0)));

    }

    #[test]
    fn try_into_naive_date_time() {
        macro_rules! test {
            ($from: expr, $to: expr) => {
                assert_eq!((&$from).try_into() as Result<chrono::NaiveDateTime>, $to);
                assert_eq!(chrono::NaiveDateTime::try_from(&$from), $to);
            };
        }

        macro_rules! err {
            ($from: expr) => {
                test!(
                    $from,
                    Err(ConvertError {
                        value: $from.clone(),
                        data_type: DataType::Timestamp,
                    })
                )
            };
        }

        let datetime = chrono::NaiveDateTime::new;
        test!(
            Value::Date(date(2021, 11, 20)),
            Ok(datetime(date(2021, 11, 20), time(0, 0, 0, 0)))
        );
        test!(
            Value::Timestamp(timestamp(2021, 11, 20, 10, 0, 0, 0)),
            Ok(datetime(date(2021, 11, 20), time(10, 0, 0, 0)))
        );
        test!(
            Value::Str("2021-11-20".to_owned()),
            Ok(datetime(date(2021, 11, 20), time(0, 0, 0, 0)))
        );

    }

}
