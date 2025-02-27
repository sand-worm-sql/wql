use {
    super::{Value, ValueError},
    crate::result::{Error, Result},
    chrono::{offset::Utc, TimeZone},
    core::str::FromStr,
    serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue},
    std::collections::HashMap,
    uuid::Uuid,
};

pub trait HashMapJsonExt {
    fn parse_json_object(value: &str) -> Result<HashMap<String, Value>>;

    fn try_from_json_map(json_map: JsonMap<String, JsonValue>) -> Result<HashMap<String, Value>>;
}

impl HashMapJsonExt for HashMap<String, Value> {
    fn parse_json_object(value: &str) -> Result<HashMap<String, Value>> {
        let value = serde_json::from_str(value)
            .map_err(|_| ValueError::InvalidJsonString(value.to_owned()))?;

        match value {
            JsonValue::Object(json_map) => HashMap::try_from_json_map(json_map),
            _ => Err(ValueError::JsonObjectTypeRequired.into()),
        }
    }

    fn try_from_json_map(json_map: JsonMap<String, JsonValue>) -> Result<HashMap<String, Value>> {
        json_map
            .into_iter()
            .map(|(key, value)| value.try_into().map(|value| (key, value)))
            .collect::<Result<HashMap<String, Value>>>()
    }
}

impl Value {
    pub fn parse_json_map(value: &str) -> Result<Value> {
        HashMap::parse_json_object(value).map(Value::Map)
    }

    pub fn parse_json_list(value: &str) -> Result<Value> {
        let value = serde_json::from_str(value)
            .map_err(|_| ValueError::InvalidJsonString(value.to_owned()))?;

        if !matches!(value, JsonValue::Array(_)) {
            return Err(ValueError::JsonArrayTypeRequired.into());
        }

        value.try_into()
    }
}

impl TryFrom<Value> for JsonValue {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Bool(v) => Ok(JsonValue::Bool(v)),
            Value::I8(v) => Ok(v.into()),
            Value::I16(v) => Ok(v.into()),
            Value::I32(v) => Ok(v.into()),
            Value::I64(v) => Ok(v.into()),
            Value::I128(v) => JsonNumber::from_str(&v.to_string())
                .map(JsonValue::Number)
                .map_err(|_| ValueError::UnreachableJsonNumberParseFailure(v.to_string()).into()),
            Value::U8(v) => Ok(v.into()),
            Value::U16(v) => Ok(v.into()),
            Value::U32(v) => Ok(v.into()),
            Value::U64(v) => Ok(v.into()),
            Value::U128(v) => JsonNumber::from_str(&v.to_string())
                .map(JsonValue::Number)
                .map_err(|_| ValueError::UnreachableJsonNumberParseFailure(v.to_string()).into()),
            Value::Str(v) => Ok(v.into()),
            Value::Bytes(v) => Ok(hex::encode(v).into()),
            Value::Date(v) => Ok(v.to_string().into()),
            Value::Timestamp(v) => Ok(Utc.from_utc_datetime(&v).to_string().into()),
            Value::Time(v) => Ok(v.to_string().into()),
            Value::Interval(v) => Ok(v.to_sql_str().into()),
            Value::Uuid(v) => Ok(Uuid::from_u128(v).hyphenated().to_string().into()),
            Value::Map(v) => v
                .into_iter()
                .map(|(key, value)| value.try_into().map(|value| (key, value)))
                .collect::<Result<Vec<(String, JsonValue)>>>()
                .map(|v| JsonMap::from_iter(v).into()),
            Value::List(v) => v
                .into_iter()
                .map(|value| value.try_into())
                .collect::<Result<Vec<JsonValue>>>()
                .map(|v| v.into()),
            Value::Null => Ok(JsonValue::Null),
        }
    }
}

impl TryFrom<JsonValue> for Value {
    type Error = Error;

    fn try_from(json_value: JsonValue) -> Result<Self> {
        match json_value {
            JsonValue::Null => Ok(Value::Null),
            JsonValue::Bool(v) => Ok(Value::Bool(v)),
            JsonValue::Number(v) => {
                if let Some(value) = v.as_i64().map(Value::I64) {
                    return Ok(value);
                }

                Err(ValueError::UnreachableJsonNumberParseFailure(v.to_string()).into())
            }
            JsonValue::String(v) => Ok(Value::Str(v)),
            JsonValue::Array(json_array) => json_array
                .into_iter()
                .map(Value::try_from)
                .collect::<Result<Vec<Value>>>()
                .map(Value::List),
            JsonValue::Object(json_map) => json_map
                .into_iter()
                .map(|(key, value)| value.try_into().map(|value| (key, value)))
                .collect::<Result<HashMap<String, Value>>>()
                .map(Value::Map),
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::data::{value::uuid::parse_uuid, Interval, Value, ValueError},
        chrono::{NaiveDate, NaiveTime},
        serde_json::{json, Value as JsonValue},
    };

    #[test]
    fn parse_json() {
        assert_eq!(
            Value::parse_json_map("[1, 2, 3]"),
            Err(ValueError::JsonObjectTypeRequired.into())
        );
        assert_eq!(
            Value::parse_json_list(r#"{ "a": 30 }"#),
            Err(ValueError::JsonArrayTypeRequired.into())
        );
    }

    #[test]
    fn value_to_json() {
        assert_eq!(Value::Bool(true).try_into(), Ok(JsonValue::Bool(true)));
        assert_eq!(Value::I8(16).try_into(), Ok(JsonValue::Number(16.into())));
        assert_eq!(
            Value::I16(100).try_into(),
            Ok(JsonValue::Number(100.into()))
        );
        assert_eq!(
            Value::I32(100).try_into(),
            Ok(JsonValue::Number(100.into()))
        );
        assert_eq!(
            Value::I64(100).try_into(),
            Ok(JsonValue::Number(100.into()))
        );
        assert_eq!(
            Value::I128(100).try_into(),
            Ok(JsonValue::Number(100.into()))
        );
        assert_eq!(Value::U8(100).try_into(), Ok(JsonValue::Number(100.into())));
        assert_eq!(
            Value::U16(100).try_into(),
            Ok(JsonValue::Number(100.into()))
        );
        assert_eq!(
            Value::U32(100).try_into(),
            Ok(JsonValue::Number(100.into()))
        );
        assert_eq!(
            Value::U64(100).try_into(),
            Ok(JsonValue::Number(100.into()))
        );
        assert_eq!(
            Value::U128(100).try_into(),
            Ok(JsonValue::Number(100.into()))
        );
        assert!(JsonValue::try_from(Value::I128(i128::MAX)).is_ok());

        assert_eq!(
            Value::Str("abc".to_owned()).try_into(),
            Ok(JsonValue::String("abc".to_owned()))
        );
        assert_eq!(
            Value::Bytes(hex::decode("a1b2").unwrap()).try_into(),
            Ok(JsonValue::String("a1b2".to_owned()))
        );
        assert_eq!(
            Value::Date(NaiveDate::from_ymd_opt(2020, 1, 3).unwrap()).try_into(),
            Ok(JsonValue::String("2020-01-03".to_owned()))
        );
        assert_eq!(
            Value::Timestamp(
                NaiveDate::from_ymd_opt(2022, 6, 11)
                    .unwrap()
                    .and_hms_opt(13, 30, 1)
                    .unwrap()
            )
            .try_into(),
            Ok(JsonValue::String("2022-06-11 13:30:01 UTC".to_owned()))
        );
        assert_eq!(
            Value::Time(NaiveTime::from_hms_opt(20, 11, 59).unwrap()).try_into(),
            Ok(JsonValue::String("20:11:59".to_owned()))
        );
        assert_eq!(
            Value::Interval(Interval::Month(17)).try_into(),
            Ok(JsonValue::String("'1-5' YEAR TO MONTH".to_owned()))
        );

        let uuid = "43185717-59af-4e2b-9cd3-3264bf3691a4";
        assert_eq!(
            Value::Uuid(parse_uuid(uuid).unwrap()).try_into(),
            Ok(JsonValue::String(uuid.to_owned()))
        );

        assert_eq!(
            Value::parse_json_map(r#"{ "a": 10, "b": { "c": true, "d": "hello" }}"#)
                .unwrap()
                .try_into(),
            Ok(json!({
                "a": 10,
                "b": {
                    "c": true,
                    "d": "hello",
                }
            }))
        );
        assert_eq!(
            Value::parse_json_list(r#"[1, 2, { "a": 3 }]"#)
                .unwrap()
                .try_into(),
            Ok(json!([1, 2, { "a": 3 }]))
        );
        assert_eq!(Value::Null.try_into(), Ok(JsonValue::Null));
    }

    #[test]
    fn json_to_value() {
        assert!(Value::try_from(JsonValue::Null).unwrap().is_null());
        assert!(Value::try_from(JsonValue::Bool(false))
            .unwrap()
            .evaluate_eq(&Value::Bool(false)));
        assert!(Value::try_from(JsonValue::Number(54321.into()))
            .unwrap()
            .evaluate_eq(&Value::I32(54321)));
        assert!(Value::try_from(JsonValue::Number(54321.into()))
            .unwrap()
            .evaluate_eq(&Value::I64(54321)));
        assert!(Value::try_from(JsonValue::Number(54321.into()))
            .unwrap()
            .evaluate_eq(&Value::I128(54321)));
        assert!(Value::try_from(JsonValue::String("world".to_owned()))
            .unwrap()
            .evaluate_eq(&Value::Str("world".to_owned())));
        assert!(
            Value::try_from(JsonValue::Array(vec![JsonValue::Bool(true)]))
                .unwrap()
                .evaluate_eq(&Value::List(vec![Value::Bool(true)]))
        );
        assert!(Value::try_from(json!({ "a": true }))
            .unwrap()
            .evaluate_eq(&Value::Map(
                [("a".to_owned(), Value::Bool(true))].into_iter().collect()
            )));
    }
}
