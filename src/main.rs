// std imports
use std::{
    collections::{btree_map, BTreeMap},
    error::Error,
    fs::{self, read_to_string},
    ops::AddAssign,
    path::{Path, PathBuf},
};

// dependency imports
use chrono::{Duration, NaiveDateTime};
use clap::{Parser, ValueEnum};
use comfy_table::{presets::ASCII_MARKDOWN, Table};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

// modular imports
use deserialization::{
    duration_deserialization, naive_date_time_deserialization,
    vec_of_naive_date_time_deserialization,
};
use serialization::{
    duration_serialization, naive_date_time_serialization, vec_of_naive_date_time_serialization,
};

/// # Deserialization
///
/// Here we will define our own deserialization functions for the foreign types.
pub mod deserialization {
    use crate::{de, Deserialize, Deserializer, Duration, NaiveDateTime};

    pub fn vec_of_naive_date_time_deserialization<'de, D>(
        deserializer: D,
    ) -> Result<Vec<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Vec<String> = Deserialize::deserialize(deserializer)?;
        let s = s
            .into_iter()
            .map(|x| NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M").unwrap())
            .collect();
        Ok(s)
    }

    pub fn naive_date_time_deserialization<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M").map_err(de::Error::custom)
    }

    pub fn duration_deserialization<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: i64 = Deserialize::deserialize(deserializer)?;
        Ok(Duration::milliseconds(s))
    }
}

/// # Deserialization
///
/// Here we will define our own serialization functions for the foreign types.
pub mod serialization {
    use crate::NaiveDateTime;

    use crate::{Duration, Serialize, Serializer};

    pub fn vec_of_naive_date_time_serialization<S>(
        naive_date_times: &[NaiveDateTime],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s: Vec<String> = naive_date_times
            .iter()
            .map(|x| x.format("%Y-%m-%d %H:%M").to_string())
            .collect();
        Serialize::serialize(&s, serializer)
    }

    pub fn duration_serialization<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&duration.num_milliseconds(), serializer)
    }

    pub fn naive_date_time_serialization<S>(
        naive_date_time: &NaiveDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(
            &naive_date_time.format("%Y-%m-%d %H:%M").to_string(),
            serializer,
        )
    }
}

/// # Entry
///
/// This is what an entry looks like in the Spotify JSON streamingdata. The data consists of multiple entries.
///
/// ```json
/// {
///     "endTime": "2022-08-18 22:31",
///     "artistName": "Lizzo",
///     "trackName": "About Damn Time",
///     "msPlayed": 62676
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
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

pub type RawStreamingHistory = Vec<Entry>;

#[derive(Debug, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize, Clone)]
pub struct TimeIdentifier {
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

impl From<&Entry> for TimeIdentifier {
    fn from(value: &Entry) -> Self {
        Self {
            end_times: vec![value.end_time],
            ms_played: value.ms_played,
        }
    }
}

impl AddAssign for TimeIdentifier {
    fn add_assign(&mut self, rhs: Self) {
        self.end_times.extend(rhs.end_times);
        self.ms_played = self.ms_played + rhs.ms_played;
    }
}

type StreamingHistory = BTreeMap<String, BTreeMap<String, TimeIdentifier>>;

fn combined_raw_streaming_history_from_folder<P: AsRef<Path>>(
    folder: P,
) -> Result<RawStreamingHistory, Box<dyn Error>> {
    let mut accumulator: RawStreamingHistory = Vec::new();
    for thing in fs::read_dir(folder)? {
        let content = read_to_string(thing?.path())?;
        let entries: RawStreamingHistory = serde_json::from_str(&content)?;
        accumulator.extend(entries);
    }
    Ok(accumulator)
}

fn clean_raw_streaming_history(raw_streaming_history: RawStreamingHistory) -> StreamingHistory {
    let mut accumulator = BTreeMap::new();
    for thing in raw_streaming_history.into_iter() {
        let time_ident = TimeIdentifier::from(&thing);
        let artist_name = thing.artist_name;
        let track_name = thing.track_name;

        match accumulator.entry(artist_name) {
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

#[derive(Debug, Clone, ValueEnum, Default)]
enum Format {
    /// Use JSON formatting
    ///
    /// The JSON looks like:
    ///
    /// ```json
    /// {
    ///     "Lizzo": {
    ///         "About Damn Time": {
    ///             "end_times": [
    ///                 ...
    ///             ],
    ///             "ms_played": 6520330
    ///         }
    ///     }
    /// }
    /// ```
    Json,
    /// Use table formatting
    ///
    /// The table looks like:
    ///
    /// ```markdown
    /// | Artist | Track           | Time Played (ms) |
    /// |--------|-----------------|------------------|
    /// | Lizzo  | About Damn Time | 6520330          |
    /// | ...    | ...             | ...              |
    /// ```
    #[default]
    Table,
}

/// Command Line Interface that can process your Spotify Streaming Data
#[derive(Parser, Debug)]
#[clap(version, author)]
struct MyCLI {
    /// The name of the artist to search for
    #[arg(short, long)]
    artist: Option<String>,
    /// The name of the track to search for
    #[arg(short, long)]
    track: Option<String>,
    /// The formatting to use when printing the results to stdout
    #[arg(short, long, value_enum, default_value_t)]
    format: Format,
    /// The folder to extract the data from
    ///
    /// After one time running this executable it will create a summarized file, that contains everything from this folder!
    /// So after one run it is no longer necessary to supply this argument, because it won't do anything if the summarized file is detected.
    #[arg(short, long)]
    data: Option<PathBuf>,
}

const JSON_DATA: &str = "spotify-stats.json";

fn main() -> Result<(), Box<dyn Error>> {
    let args = MyCLI::parse();
    let streaming_history: StreamingHistory = match read_to_string(JSON_DATA) {
        Ok(string) => serde_json::from_str(&string)?,
        Err(_) => match args.data {
            Some(path) => {
                eprintln!("[INFO] `{}` didn't exist yet, creating...", JSON_DATA);
                let raw_streaming_history: RawStreamingHistory =
                    combined_raw_streaming_history_from_folder(&path)?;
                let streaming_history = clean_raw_streaming_history(raw_streaming_history);
                let json = serde_json::to_string(&streaming_history)?;
                fs::write(JSON_DATA, json)?;
                eprintln!("[INFO] Finished creating `combined.json`");
                streaming_history
            }
            None => {
                panic!("the '--data <DATA>' argument was not provided, this is needed the first time only")
            }
        },
    };
    match args.format {
        Format::Json => {
            let json = match (args.artist, args.track) {
                (None, None) => serde_json::to_string_pretty(&streaming_history)?,
                (None, Some(args_track)) => {
                    let mut accumulator: StreamingHistory = BTreeMap::new();
                    for (artist, rest) in streaming_history {
                        for (track, time) in rest {
                            if track == args_track {
                                let mut new = BTreeMap::new();
                                new.insert(track, time);
                                accumulator.insert(artist.clone(), new);
                            }
                        }
                    }
                    serde_json::to_string_pretty(&accumulator)?
                }
                (Some(args_artist), None) => {
                    serde_json::to_string_pretty(&streaming_history[&args_artist])?
                }
                (Some(args_artist), Some(args_track)) => {
                    serde_json::to_string_pretty(&streaming_history[&args_artist][&args_track])?
                }
            };
            println!("{}", json);
        }
        Format::Table => {
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            table.set_header(["Artist", "Track", "Time Played (ms)"]);
            for (artist, rest) in &streaming_history {
                for (track, time) in rest {
                    if (Some(artist) == args.artist.as_ref() || Some(track) == args.track.as_ref())
                        ^ (args.artist.is_none() && args.track.is_none())
                    {
                        table.add_row([
                            artist,
                            track,
                            &time.ms_played.num_milliseconds().to_string(),
                        ]);
                    }
                }
            }
            println!("{}", table);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_isomorphism_raw_internal_streaming_history() -> Result<(), Box<dyn Error>> {
        let initial_entries = combined_raw_streaming_history_from_folder("data")?;
        let initial_json_representation = serde_json::to_string(&initial_entries)?;
        let secondary_entries: RawStreamingHistory =
            serde_json::from_str(&initial_json_representation)?;
        assert_eq!(initial_entries, secondary_entries);
        Ok(())
    }

    #[test]
    fn test_isomorphism_raw_external_streaming_history() -> Result<(), Box<dyn Error>> {
        let initial_entries = combined_raw_streaming_history_from_folder("data")?;
        let initial_json_representation = serde_json::to_string(&initial_entries)?;
        let secondary_entries: RawStreamingHistory =
            serde_json::from_str(&initial_json_representation)?;
        let secondary_json_representation = serde_json::to_string(&secondary_entries)?;
        assert_eq!(initial_json_representation, secondary_json_representation);
        Ok(())
    }

    #[test]
    fn test_isomorphism_internal_streaming_history() -> Result<(), Box<dyn Error>> {
        let initial_entries = combined_raw_streaming_history_from_folder("data")?;
        let initial_cleaned: StreamingHistory = clean_raw_streaming_history(initial_entries);
        let initial_json_cleaned_representation = serde_json::to_string(&initial_cleaned)?;
        let secondary_cleaned: StreamingHistory =
            serde_json::from_str(&initial_json_cleaned_representation)?;
        assert_eq!(initial_cleaned, secondary_cleaned);
        Ok(())
    }

    #[test]
    fn test_isomorphism_external_streaming_history() -> Result<(), Box<dyn Error>> {
        let initial_entries = combined_raw_streaming_history_from_folder("data")?;
        let initial_cleaned: StreamingHistory = clean_raw_streaming_history(initial_entries);
        let initial_json_cleaned_representation = serde_json::to_string(&initial_cleaned)?;
        let secondary_cleaned: StreamingHistory =
            serde_json::from_str(&initial_json_cleaned_representation)?;
        let secondary_json_cleaned_representation = serde_json::to_string(&secondary_cleaned)?;
        assert_eq!(
            initial_json_cleaned_representation,
            secondary_json_cleaned_representation
        );
        Ok(())
    }
}
