use serde::{de, ser};
use serde::{Deserialize, Deserializer, Serializer};

// JavaScript cannot precisely represent integers above 2^53 - 1.
pub const MAX_JSON_SAFE_INTEGER_U64: u64 = 9_007_199_254_740_991;

pub fn serialize_u64<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *value > MAX_JSON_SAFE_INTEGER_U64 {
        return Err(ser::Error::custom(format!(
            "u64 out of JSON-safe range (> 2^53 - 1): {value}"
        )));
    }
    serializer.serialize_u64(*value)
}

pub fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u64::deserialize(deserializer)?;
    if value > MAX_JSON_SAFE_INTEGER_U64 {
        return Err(de::Error::custom(format!(
            "u64 out of JSON-safe range (> 2^53 - 1): {value}"
        )));
    }
    Ok(value)
}

pub fn serialize_option_u64<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serialize_u64(value, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_option_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<u64>::deserialize(deserializer)?;
    if let Some(value) = value {
        if value > MAX_JSON_SAFE_INTEGER_U64 {
            return Err(de::Error::custom(format!(
                "u64 out of JSON-safe range (> 2^53 - 1): {value}"
            )));
        }
    }
    Ok(value)
}
