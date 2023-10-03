use std::{collections::BTreeMap, ops::AddAssign};

use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::serde::{
    deserialization::{duration_deserialization, vec_of_naive_date_time_deserialization},
    serialization::{duration_serialization, vec_of_naive_date_time_serialization},
};

use super::{
    raw_streaming_data::{RawStreamingData, SpotifyEntry},
    Persist,
};

// This should ideally be in 3rd normal form, or 5th normal form.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Information {
    #[serde(
        deserialize_with = "vec_of_naive_date_time_deserialization",
        serialize_with = "vec_of_naive_date_time_serialization"
    )]
    pub timestamps: Vec<NaiveDateTime>,
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub total_ms_played: Duration,
    pub reasons_start: Vec<String>,
    pub reasons_end: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct FoldedStreamingData(
    //           Artist           Album            Track,  Ident
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
        Self {
            timestamps: vec![value.ts],
            total_ms_played: value.ms_played,
            reasons_start: vec![value
                .reason_start
                .clone()
                .unwrap_or_else(|| "None".to_string())],
            reasons_end: vec![value
                .reason_end
                .clone()
                .unwrap_or_else(|| "None".to_string())],
        }
    }
}

impl AddAssign for Information {
    fn add_assign(&mut self, rhs: Self) {
        self.timestamps.extend(rhs.timestamps);
        self.total_ms_played = self.total_ms_played + rhs.total_ms_played;
        self.reasons_start.extend(rhs.reasons_start);
        self.reasons_end.extend(rhs.reasons_end);
    }
}

/// Everthing as `SpotifyEntry`, but combining all the stats on a per-track basis
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
    #[serde(
        deserialize_with = "vec_of_naive_date_time_deserialization",
        serialize_with = "vec_of_naive_date_time_serialization"
    )]
    pub timestamps: Vec<NaiveDateTime>,
    pub reasons_start: Vec<String>,
    pub reasons_end: Vec<String>,
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
                total_ms_played: info.total_ms_played,
                timestamps: info.timestamps.clone(),
                reasons_start: info.reasons_start.clone(),
                reasons_end: info.reasons_end.clone(),
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
