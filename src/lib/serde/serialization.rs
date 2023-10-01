use chrono::{Duration, NaiveDateTime};
use serde::{Serialize, Serializer};

pub fn vec_of_naive_date_time_serialization<S>(
    naive_date_times: &[NaiveDateTime],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s: Vec<String> = naive_date_times
        .iter()
        .map(|x| x.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .collect();
    Serialize::serialize(&s, serializer)
}

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
