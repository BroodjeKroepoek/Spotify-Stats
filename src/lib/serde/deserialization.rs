use chrono::{Duration, NaiveDateTime};

use serde::{de, Deserialize};

pub fn naive_date_time_deserialization<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%SZ").map_err(|parse_error| {
        let custom_error_msg = format!("Failed to parse NaiveDateTime: {}", parse_error);
        de::Error::custom(custom_error_msg)
    })
}

pub fn duration_deserialization<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: i64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::milliseconds(s))
}
