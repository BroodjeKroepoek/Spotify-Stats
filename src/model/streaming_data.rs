use std::{collections::BTreeMap, fs, ops::AddAssign, path::Path};

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct TimeIdent {
    #[serde(
        deserialize_with = "vec_of_naive_date_time_deserialization",
        serialize_with = "vec_of_naive_date_time_serialization"
    )]
    pub end_times: Vec<NaiveDateTime>,
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub ms_played: Duration,
}

/// <ARTIST> : { <TRACK> : { <PLATFORM> : { ms_played: ..., end_times: [ ... ] } } }
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct FoldedStreamingData(pub BTreeMap<String, BTreeMap<String, BTreeMap<String, TimeIdent>>>);

#[macro_export]
macro_rules! insert_nested_map {
    ($map:expr, $k1:expr, $k2:expr, $k3:expr, $v:expr) => {{
        $map.0
            .entry($k1)
            .or_insert_with(|| BTreeMap::new())
            .entry($k2)
            .or_insert_with(|| BTreeMap::new())
            .entry($k3)
            .or_insert_with(|| $v);
    }};
}

#[macro_export]
macro_rules! iterate_nested_map {
    ($map:expr, $key1:ident, $key2:ident, $key3:ident, $val:ident, $body:block) => {
        for ($key1, inner_map) in $map.0 {
            for ($key2, nested_map) in inner_map {
                for ($key3, $val) in nested_map {
                    $body
                }
            }
        }
    };
}

impl From<&SpotifyEntry> for TimeIdent {
    fn from(value: &SpotifyEntry) -> Self {
        Self {
            end_times: vec![value.ts],
            ms_played: value.ms_played,
        }
    }
}

impl AddAssign for TimeIdent {
    fn add_assign(&mut self, rhs: Self) {
        self.end_times.extend(rhs.end_times);
        self.ms_played = self.ms_played + rhs.ms_played;
    }
}

/// Everthing as `SpotifyEntry`, but combining all the stats on a per-track basis
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CleanedSpotifyEntry {
    pub artist: String,
    pub track: String,
    pub platform: String,
    pub ms_played: Duration,
    pub end_times: Vec<NaiveDateTime>,
}

#[derive(Debug)]
pub struct CleanedStreamingData(pub Vec<CleanedSpotifyEntry>);

impl From<FoldedStreamingData> for CleanedStreamingData {
    fn from(value: FoldedStreamingData) -> Self {
        let mut accumulator = Vec::new();
        iterate_nested_map!(value, artist, track, platform, time, {
            accumulator.push(CleanedSpotifyEntry {
                artist: artist.clone(),
                track: track.clone(),
                platform,
                ms_played: time.ms_played,
                end_times: time.end_times,
            });
        });
        CleanedStreamingData(accumulator)
    }
}

impl From<RawStreamingData> for FoldedStreamingData {
    fn from(value: RawStreamingData) -> Self {
        let mut accumulator = FoldedStreamingData(BTreeMap::new());
        for thing in value.0.into_iter() {
            let time_ident = TimeIdent::from(&thing);
            let artist_name = thing.master_metadata_album_artist_name;
            let track_name = thing.master_metadata_track_name;
            let platform = thing.platform;
            if let Some(artist_name) = artist_name {
                if let Some(track_name) = track_name {
                    insert_nested_map!(
                        &mut accumulator,
                        artist_name,
                        track_name,
                        platform,
                        time_ident
                    );
                }
            };
        }
        accumulator
    }
}

impl Persist for FoldedStreamingData {
    type Error = std::io::Error;

    fn save<P>(&self, key: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path>,
    {
        Ok(fs::write(key, serde_json::to_string(&self)?)?)
    }

    fn load<P>(key: P) -> Result<Self, Self::Error>
    where
        Self: Sized,
        P: AsRef<Path>,
    {
        Ok(serde_json::from_str(&fs::read_to_string(key)?)?)
    }
}
