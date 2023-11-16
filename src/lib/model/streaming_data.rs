//! This module describes the format that Spotify uses.
//!
//! This is so we can interact with the Spotify streaming data.

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

/// Represents a log entry for a streaming event, including play duration and reasons.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct LogEntry(
    /// Duration of the music track played.
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub Duration, // ms_played
    /// The reason for starting playback (e.g., "trackdone").
    pub Option<String>, // reason_start
    /// The reason for ending playback (e.g., "trackdone").
    pub Option<String>, // reason_end
);

/// Represents a log of streaming events indexed by timestamp.
///
/// # Examples
///
/// ```rust
/// use spotify_stats::model::streaming_data::{Log, LogEntry};
/// use chrono::{NaiveDateTime, Duration};
/// use std::collections::BTreeMap;
///
/// let mut log_map = BTreeMap::new();
/// log_map.insert(
///     NaiveDateTime::parse_from_str("2013-05-03T16:35:29Z", "%Y-%m-%dT%H:%M:%SZ").unwrap(),
///     LogEntry(
///         Duration::seconds(180),
///         Some("trackstart".to_string()),
///         Some("trackdone".to_string()),
///     ),
/// );
/// let log = Log(log_map);
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Log(
    /// A BTreeMap where the key is the timestamp and the value is a `LogEntry`.
    pub BTreeMap<NaiveDateTime, LogEntry>,
);

/// Represents information related to streaming data, including a log and total playtime.
///
/// # Examples
///
/// ```rust
/// use spotify_stats::model::streaming_data::{Log, Information};
/// use std::collections::BTreeMap;
/// use chrono::Duration;
///
/// let information = Information(
///     Log(BTreeMap::new()), // Empty log for illustration purposes
///     Duration::milliseconds(180),
///     Some("spotify:track:example_uri".to_string()),
/// );
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Information(
    /// Log of streaming events indexed by timestamp.
    pub Log, // log
    /// Total duration of all music tracks played.
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub Duration, // total_ms_played
    /// Spotify track URL associated with the streaming data.
    pub Option<String>, // spotify_track_url
);

/// Represents streaming data in a nested structure, grouped by artist, album, and track.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct FoldedStreamingData(
    /// A BTreeMap with nested structures representing artist, album, track, and information.
    pub BTreeMap<String, BTreeMap<String, BTreeMap<String, Information>>>,
);

/// A macro to insert data into nested BTreeMaps efficiently.
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

/// A macro to iterate over nested BTreeMaps efficiently.
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

/// Convert a `SpotifyEntry` into `Information`.
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

/// Implement addition and assignment for `Information`.
impl AddAssign for Information {
    fn add_assign(&mut self, rhs: Self) {
        self.1 = self.1 + rhs.1;
        self.0 .0.extend(rhs.0 .0);
    }
}

/// Represents cleaned Spotify entry data, including artist, album, track, playtime, and log.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CleanedSpotifyEntry {
    /// Artist name associated with the played track.
    pub artist: String,
    /// Album name associated with the played track.
    pub album: String,
    /// Track name.
    pub track: String,
    /// Total duration of the music track played.
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub total_ms_played: Duration,
    /// Log of streaming events indexed by timestamp.
    pub log: Log,
}

/// Represents a collection of cleaned Spotify entries.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CleanedStreamingData(
    /// A vector containing cleaned Spotify entry objects.
    pub Vec<CleanedSpotifyEntry>,
);

impl Persist for CleanedStreamingData {}

/// Convert `FoldedStreamingData` into `CleanedStreamingData`.
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

/// Convert `RawStreamingData` into `FoldedStreamingData`.
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

impl Persist for FoldedStreamingData {}
