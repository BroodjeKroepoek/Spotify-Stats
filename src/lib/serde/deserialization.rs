use chrono::{Duration, NaiveDateTime};
use serde::{de, Deserialize, Deserializer};

pub fn vec_of_naive_date_time_deserialization<'de, D>(
    deserializer: D,
) -> Result<Vec<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Vec<String> = Deserialize::deserialize(deserializer)?;
    let s = s
        .into_iter()
        .map(|x| NaiveDateTime::parse_from_str(&x, "%Y-%m-%dT%H:%M:%SZ").unwrap())
        .collect();
    Ok(s)
}

pub fn naive_date_time_deserialization<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    // "2013-05-03T16:34:58Z"
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%SZ").map_err(de::Error::custom)
}

pub fn duration_deserialization<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: i64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::milliseconds(s))
}
