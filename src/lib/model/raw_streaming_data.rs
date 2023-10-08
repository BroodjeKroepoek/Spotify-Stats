use std::{
    error::Error,
    fs::{read_dir, read_to_string},
    path::Path,
};

use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::serde::{
    deserialization::{duration_deserialization, naive_date_time_deserialization}, serialization::{duration_serialization, naive_date_time_serialization},
};

use super::Persist;

/// # Entry
///
/// This is what an one singular entry looks like in the Spotify JSON extended streaming data, the full data consists of multiple entries.
///
/// ```json
/// {
///     "ts": "2023-02-22T07:01:41Z",
///     "username": "arnoarie",
///     "platform": "android",
///     "ms_played": 131538,
///     "conn_country": "NL",
///     "ip_addr_decrypted": "84.241.202.31",
///     "user_agent_decrypted": "unknown",
///     "master_metadata_track_name": "Moan",
///     "master_metadata_album_artist_name": "Sabrina Claudio",
///     "master_metadata_album_album_name": "Archives & Lullabies",
///     "spotify_track_uri": "spotify:track:3xRue8c0zDkTWZ9fdDdz0u",
///     "episode_name": null,
///     "episode_show_name": null,
///     "spotify_episode_uri": null,
///     "reason_start": "trackdone",
///     "reason_end": "trackdone",
///     "shuffle": false,
///     "skipped": false,
///     "offline": false,
///     "offline_timestamp": 1677049169,
///     "incognito_mode": false
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SpotifyEntry {
    #[serde(
        deserialize_with = "naive_date_time_deserialization",
        serialize_with = "naive_date_time_serialization"
    )]
    pub ts: NaiveDateTime,
    pub username: Option<String>,
    pub platform: Option<String>,
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub ms_played: Duration,
    pub conn_country: Option<String>,
    pub ip_addr_decrypted: Option<String>,
    pub user_agent_decrypted: Option<String>,
    pub master_metadata_track_name: Option<String>,
    pub master_metadata_album_artist_name: Option<String>,
    pub master_metadata_album_album_name: Option<String>,
    pub spotify_track_uri: Option<String>,
    pub episode_name: Option<String>,
    pub episode_show_name: Option<String>,
    pub spotify_episode_uri: Option<String>,
    pub reason_start: Option<String>,
    pub reason_end: Option<String>,
    pub shuffle: Option<bool>,
    pub skipped: Option<bool>,
    pub offline: Option<bool>,
    pub offline_timestamp: Option<u128>,
    pub incognito_mode: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct RawStreamingData(pub Vec<SpotifyEntry>);

impl RawStreamingData {
    pub fn from_path<P>(path: P) -> Result<RawStreamingData, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut accumulator = RawStreamingData(Vec::new());
        for file in read_dir(path)? {
            let file_content = read_to_string(file?.path())?;
            let entries: RawStreamingData = postcard::from_bytes(&file_content.as_bytes())?;
            accumulator.0.extend(entries.0);
        }
        Ok(accumulator)
    }
}

impl Persist for RawStreamingData {}
