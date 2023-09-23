use std::{
    collections::{btree_map, BTreeMap},
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
    raw_streaming_data::{Entry, RawStreamingData},
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

impl From<&Entry> for TimeIdent {
    fn from(value: &Entry) -> Self {
        Self {
            end_times: vec![value.end_time],
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

impl From<RawStreamingData> for StreamingData {
    fn from(value: RawStreamingData) -> Self {
        let mut accumulator = StreamingData(BTreeMap::new());
        for thing in value.0.into_iter() {
            let time_ident = TimeIdent::from(&thing);
            let artist_name = thing.artist_name;
            let track_name = thing.track_name;

            match accumulator.0.entry(artist_name) {
                btree_map::Entry::Vacant(vacant_entry) => {
                    let mut new = BTreeMap::new();
                    new.insert(track_name, time_ident);
                    vacant_entry.insert(new);
                }
                btree_map::Entry::Occupied(mut occupied_entry) => {
                    let mutable = occupied_entry.get_mut();
                    match mutable.entry(track_name) {
                        btree_map::Entry::Vacant(vacant_entry) => {
                            vacant_entry.insert(time_ident);
                        }
                        btree_map::Entry::Occupied(mut occupied_entry) => {
                            let mutable = occupied_entry.get_mut();
                            *mutable += time_ident;
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
