use std::{
    collections::{
        btree_map::{Entry, IntoIter},
        BTreeMap,
    },
    fs,
    ops::AddAssign,
    path::Path,
};

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct StreamingData(pub BTreeMap<String, BTreeMap<String, TimeIdent>>);

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CleanedSpotifyEntry {
    pub artist: String,
    pub track: String,
    pub ms_played: Duration,
    pub end_times: Vec<NaiveDateTime>,
}

#[derive(Debug)]
pub struct CleanedStreamingData(pub Vec<CleanedSpotifyEntry>);

impl From<StreamingData> for CleanedStreamingData {
    fn from(value: StreamingData) -> Self {
        let mut accumulator = Vec::new();
        for (artist, rest) in value.0 {
            for (track, time) in rest {
                accumulator.push(CleanedSpotifyEntry {
                    artist: artist.clone(),
                    track,
                    ms_played: time.ms_played,
                    end_times: time.end_times,
                })
            }
        }
        CleanedStreamingData(accumulator)
    }
}

impl From<RawStreamingData> for StreamingData {
    fn from(value: RawStreamingData) -> Self {
        let mut accumulator = StreamingData(BTreeMap::new());
        for thing in value.0.into_iter() {
            let time_ident = TimeIdent::from(&thing);
            let artist_name = thing.master_metadata_album_artist_name;
            let track_name = thing.master_metadata_track_name;
            if let Some(artist_name) = artist_name {
                if let Some(track_name) = track_name {
                    match accumulator.0.entry(artist_name) {
                        Entry::Vacant(vacant_entry) => {
                            let mut new = BTreeMap::new();
                            new.insert(track_name, time_ident);
                            vacant_entry.insert(new);
                        }
                        Entry::Occupied(mut occupied_entry) => {
                            let mutable = occupied_entry.get_mut();
                            match mutable.entry(track_name) {
                                Entry::Vacant(vacant_entry) => {
                                    vacant_entry.insert(time_ident);
                                }
                                Entry::Occupied(mut occupied_entry) => {
                                    let mutable = occupied_entry.get_mut();
                                    *mutable += time_ident;
                                }
                            }
                        }
                    }
                }
            };
        }
        accumulator
    }
}

impl Persist for StreamingData {
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
