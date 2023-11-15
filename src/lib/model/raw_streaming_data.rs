use std::{
    error::Error,
    fs::{self, read_dir},
    path::Path,
};

use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::serde::{
    deserialization::{duration_deserialization, naive_date_time_deserialization},
    serialization::{duration_serialization, naive_date_time_serialization},
};

use super::Persist;

/// # Spotify Entry
///
/// Represents an individual entry in Spotify streaming data.
///
/// A Spotify entry contains details about a user's streaming activity, such as the timestamp, username,
/// platform, and other relevant information.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SpotifyEntry {
    /// Timestamp of the streaming activity.
    #[serde(
        deserialize_with = "naive_date_time_deserialization",
        serialize_with = "naive_date_time_serialization"
    )]
    pub ts: NaiveDateTime,

    /// Username of the Spotify user.
    pub username: Option<String>,

    /// The platform on which the streaming activity occurred (e.g., "android").
    pub platform: Option<String>,
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]

    /// Duration of the music track played.
    pub ms_played: Duration,

    /// The country code representing the user's connection country (e.g., "NL").
    pub conn_country: Option<String>,

    /// Decrypted IP address of the user.
    pub ip_addr_decrypted: Option<String>,

    /// Decrypted user agent information.
    pub user_agent_decrypted: Option<String>,

    /// The name of the music track being played.
    pub master_metadata_track_name: Option<String>,

    /// The artist's name associated with the album containing the track.
    pub master_metadata_album_artist_name: Option<String>,

    /// The name of the album containing the track.
    pub master_metadata_album_album_name: Option<String>,

    /// Spotify URI of the track.
    pub spotify_track_uri: Option<String>,

    /// Name of the episode (if applicable).
    pub episode_name: Option<String>,

    /// Name of the show containing the episode (if applicable).
    pub episode_show_name: Option<String>,

    /// Spotify URI of the episode (if applicable).
    pub spotify_episode_uri: Option<String>,

    /// The reason for starting playback (e.g., "trackdone").
    pub reason_start: Option<String>,

    /// The reason for ending playback (e.g., "trackdone").
    pub reason_end: Option<String>,

    /// Indicates whether shuffle mode was active.
    pub shuffle: Option<bool>,

    /// Indicates whether the track was skipped.
    pub skipped: Option<bool>,

    /// Indicates whether the playback was in offline mode.
    pub offline: Option<bool>,

    /// Timestamp of the offline playback event.
    pub offline_timestamp: Option<u128>,

    /// Indicates whether the user was in incognito mode during the streaming activity.
    pub incognito_mode: Option<bool>,
}

/// # Raw Streaming Data
///
/// Represents a collection of Spotify streaming entries.
///
/// Raw streaming data is a collection of `SpotifyEntry` objects, typically obtained from
/// parsing and accumulating data from multiple JSON files.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct RawStreamingData(pub Vec<SpotifyEntry>);

impl RawStreamingData {
    /// Create a `RawStreamingData` object from a folder containing JSON files.
    ///
    /// This function reads all JSON files in the specified folder, parses their contents,
    /// and accumulates the streaming entries into a `RawStreamingData` object.
    ///
    /// # Arguments
    ///
    /// - `path`: A path to the folder containing the JSON files to be read.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `RawStreamingData` object if successful, or an error if any
    /// file reading or parsing fails.
    pub fn from_folder_of_json<P>(path: P) -> Result<RawStreamingData, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut accumulator = RawStreamingData(Vec::new());
        for file in read_dir(path)? {
            let content = fs::read_to_string(file?.path())?;
            let entries: RawStreamingData = serde_json::from_str(&content)?;
            accumulator.0.extend(entries.0);
        }
        Ok(accumulator)
    }
}

impl Persist for RawStreamingData {}
