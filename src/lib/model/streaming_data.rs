use std::{collections::BTreeMap, ops::AddAssign};

use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::serde::{
    deserialization::duration_deserialization, serialization::duration_serialization,
};

use super::{
    raw_streaming_data::{RawStreamingData, SpotifyEntry},
    Persist,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct LogEntry(
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    Duration, // ms_played
    Option<String>, // reason_start
    Option<String>, // reason_end
);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Log(pub BTreeMap<NaiveDateTime, LogEntry>);

// This should ideally be in 3rd normal form, or 5th normal form.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Information(
    pub Log, // log
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub Duration, // total_ms_played
    pub Option<String>, // spotify_track_url
);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct FoldedStreamingData(
    //           Artist           Album            Track,  Info
    pub BTreeMap<String, BTreeMap<String, BTreeMap<String, Information>>>,
);

#[macro_export]
macro_rules! insert_nested_map {
    ($map:expr, $k1:expr, $k2:expr, $k3:expr, $v:expr) => {{
        $map.0
            .entry($k1)
            .or_insert(BTreeMap::new())
            .entry($k2)
            .or_insert(BTreeMap::new())
            .entry($k3)
            .and_modify(|x| *x += $v.clone())
            .or_insert($v);
    }};
}

#[macro_export]
macro_rules! iterate_nested_map {
    ($map:expr, $key1:ident, $key2:ident, $key3:ident, $val:ident, $body:block) => {
        for ($key1, inner_map1) in &$map.0 {
            for ($key2, inner_map2) in inner_map1 {
                for ($key3, $val) in inner_map2 {
                    $body
                }
            }
        }
    };
}

impl From<&SpotifyEntry> for Information {
    fn from(value: &SpotifyEntry) -> Self {
        let mut log = BTreeMap::new();
        log.insert(
            value.ts,
            LogEntry(
                value.ms_played,
                value.reason_start.clone(),
                value.reason_end.clone(),
            ),
        );
        Self(Log(log), value.ms_played, value.spotify_track_uri.clone())
    }
}

impl AddAssign for Information {
    fn add_assign(&mut self, rhs: Self) {
        self.1 = self.1 + rhs.1;
        self.0 .0.extend(rhs.0 .0);
    }
}

/// Everything as `SpotifyEntry`, but combining all the stats on a per-track basis
///
/// The full uncleaned version, in `raw_streaming_data.rs` looks like:
///
/// pub struct SpotifyEntry {
///     #[serde(
///         deserialize_with = "naive_date_time_deserialization",
///         serialize_with = "naive_date_time_serialization"
///     )]
///     pub ts: NaiveDateTime,
///     pub username: Option<String>,
///     pub platform: Option<String>,
///     #[serde(
///         deserialize_with = "duration_deserialization",
///         serialize_with = "duration_serialization"
///     )]
///     pub ms_played: Duration,
///     pub conn_country: Option<String>,
///     pub ip_addr_decrypted: Option<String>,
///     pub user_agent_decrypted: Option<String>,
///     pub master_metadata_track_name: Option<String>,
///     pub master_metadata_album_artist_name: Option<String>,
///     pub master_metadata_album_album_name: Option<String>,
///     pub spotify_track_uri: Option<String>,
///     pub episode_name: Option<String>,
///     pub episode_show_name: Option<String>,
///     pub spotify_episode_uri: Option<String>,
///     pub reason_start: Option<String>,
///     pub reason_end: Option<String>,
///     pub shuffle: Option<bool>,
///     pub skipped: Option<bool>,
///     pub offline: Option<bool>,
///     pub offline_timestamp: Option<u64>,
///     pub incognito_mode: Option<bool>,
/// }
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CleanedSpotifyEntry {
    pub artist: String,
    pub album: String,
    pub track: String,
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub total_ms_played: Duration,
    pub log: Log,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CleanedStreamingData(pub Vec<CleanedSpotifyEntry>);

impl Persist for CleanedStreamingData {
    type Error = std::io::Error;
}

impl From<FoldedStreamingData> for CleanedStreamingData {
    fn from(value: FoldedStreamingData) -> Self {
        let mut accumulator = Vec::new();
        iterate_nested_map!(value, artist, album, track, info, {
            accumulator.push(CleanedSpotifyEntry {
                artist: artist.clone(),
                album: album.clone(),
                track: track.clone(),
                total_ms_played: info.1,
                log: info.0.clone(),
            });
        });
        CleanedStreamingData(accumulator)
    }
}

impl From<RawStreamingData> for FoldedStreamingData {
    fn from(value: RawStreamingData) -> Self {
        let mut accumulator = FoldedStreamingData(BTreeMap::new());
        for thing in value.0.into_iter() {
            let info = Information::from(&thing);
            if let Some(artist_name) = thing.master_metadata_album_artist_name {
                if let Some(track_name) = thing.master_metadata_track_name {
                    if let Some(album) = thing.master_metadata_album_album_name {
                        insert_nested_map!(&mut accumulator, artist_name, album, track_name, info);
                    }
                }
            }
        }
        accumulator
    }
}

impl Persist for FoldedStreamingData {
    type Error = std::io::Error;
}
