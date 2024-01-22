//! This module describes the serialization process, i.e. saving to persistent files.

use chrono::{Duration, NaiveDateTime};
use eyre::Result;
use serde::{Serialize, Serializer};

pub fn duration_serialization<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Serialize::serialize(&duration.num_milliseconds(), serializer)
}

// "2023-02-22T07:01:41Z"
pub fn naive_date_time_serialization<S>(
    naive_date_time: &NaiveDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Serialize::serialize(
        &naive_date_time.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        serializer,
    )
}
