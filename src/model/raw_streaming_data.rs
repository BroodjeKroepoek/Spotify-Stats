use std::{
    fs::{read_dir, read_to_string},
    path::Path,
};

use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::serde::{
    deserialization::{duration_deserialization, naive_date_time_deserialization},
    serialization::{duration_serialization, naive_date_time_serialization},
};

/// # Entry
///
/// This is what an one singular entry looks like in the Spotify JSON streaming data, the full data consists of multiple entries.
///
/// ```json
/// {
///     "endTime": "2022-08-18 22:31",
///     "artistName": "Lizzo",
///     "trackName": "About Damn Time",
///     "msPlayed": 62676
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Entry {
    #[serde(
        alias = "endTime",
        deserialize_with = "naive_date_time_deserialization",
        serialize_with = "naive_date_time_serialization"
    )]
    pub end_time: NaiveDateTime,
    #[serde(alias = "artistName")]
    pub artist_name: String,
    #[serde(alias = "trackName")]
    pub track_name: String,
    #[serde(
        alias = "msPlayed",
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub ms_played: Duration,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct RawStreamingData(pub Vec<Entry>);

impl RawStreamingData {
    pub fn from_path<P>(path: P) -> Result<RawStreamingData, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let mut accumulator = RawStreamingData(Vec::new());
        for file in read_dir(path)? {
            let file_content = read_to_string(file?.path())?;
            let entries: RawStreamingData = serde_json::from_str(&file_content)?;
            accumulator.0.extend(entries.0);
        }
        Ok(accumulator)
    }
}
