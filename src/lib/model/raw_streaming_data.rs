//! This module describes our own format that we use in for example persistent storage of data.
//!
//! This is so we can filter out irrelevant data, and decrease the footprint of the data, i.e. by using nested maps and DEFLATE compression.

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

/// Represents an individual entry in Spotify streaming data.
///
/// A Spotify entry contains details about a user's streaming activity, such as the timestamp, username,
/// platform, and other relevant information.
///
/// # Examples
///
/// ```rust
/// use spotify_stats::model::raw_streaming_data::SpotifyEntry;
/// use chrono::{Duration, NaiveDateTime};
///
/// let entry = SpotifyEntry {
///     ts: NaiveDateTime::parse_from_str("2013-05-03T16:35:29Z", "%Y-%m-%dT%H:%M:%SZ").unwrap(),
///     username: Some("example_user".to_string()),
///     platform: Some("android".to_string()),
///     ms_played: Duration::milliseconds(180),
///     conn_country: Some("NL".to_string()),
///     episode_name: None,
///     episode_show_name: None,
///     ip_addr_decrypted: None,
///     master_metadata_album_album_name: Some("album_name".to_string()),
///     master_metadata_album_artist_name: Some("artist_name".to_string()),
///     master_metadata_track_name: Some("track_name".to_string()),
///     offline: Some(false),
///     offline_timestamp: None,
///     reason_end: None,
///     reason_start: None,
///     shuffle: Some(false),
///     skipped: Some(false),
///     spotify_episode_uri: None,
///     spotify_track_uri: None,
///     user_agent_decrypted: None,
///     incognito_mode: Some(true),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SpotifyEntry {
    /// Timestamp of the streaming activity.
    ///
    /// This field represents the date and time when the streaming activity occurred.
    #[serde(
        deserialize_with = "naive_date_time_deserialization",
        serialize_with = "naive_date_time_serialization"
    )]
    pub ts: NaiveDateTime,

    /// Username of the Spotify user.
    ///
    /// This field contains the username of the Spotify user associated with the streaming activity.
    pub username: Option<String>,

    /// The platform on which the streaming activity occurred (e.g., "android").
    ///
    /// This field indicates the platform (e.g., device or application) on which the user
    /// streamed the content.
    pub platform: Option<String>,

    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    /// Duration of the music track played.
    ///
    /// This field represents the duration of the music track played during the streaming activity.
    pub ms_played: Duration,

    /// The country code representing the user's connection country (e.g., "NL").
    ///
    /// This field contains the country code representing the user's location or connection country.
    pub conn_country: Option<String>,

    /// Decrypted IP address of the user.
    ///
    /// This field contains the decrypted IP address of the user during the streaming activity.
    pub ip_addr_decrypted: Option<String>,

    /// Decrypted user agent information.
    ///
    /// This field contains decrypted user agent information associated with the streaming activity.
    pub user_agent_decrypted: Option<String>,

    /// The name of the music track being played.
    ///
    /// This field contains the name of the music track that the user played.
    pub master_metadata_track_name: Option<String>,

    /// The artist's name associated with the album containing the track.
    ///
    /// This field contains the name of the artist associated with the album containing the track.
    pub master_metadata_album_artist_name: Option<String>,

    /// The name of the album containing the track.
    ///
    /// This field contains the name of the album that includes the played track.
    pub master_metadata_album_album_name: Option<String>,

    /// Spotify URI of the track.
    ///
    /// This field contains the Spotify URI (Uniform Resource Identifier) of the played track.
    pub spotify_track_uri: Option<String>,

    /// Name of the episode (if applicable).
    ///
    /// This field contains the name of the episode if the streaming activity involves an episode.
    pub episode_name: Option<String>,

    /// Name of the show containing the episode (if applicable).
    ///
    /// This field contains the name of the show if the streaming activity involves an episode.
    pub episode_show_name: Option<String>,

    /// Spotify URI of the episode (if applicable).
    ///
    /// This field contains the Spotify URI of the episode if the streaming activity involves an episode.
    pub spotify_episode_uri: Option<String>,

    /// The reason for starting playback (e.g., "trackdone").
    ///
    /// This field contains the reason for starting playback, providing context for the streaming activity.
    pub reason_start: Option<String>,

    /// The reason for ending playback (e.g., "trackdone").
    ///
    /// This field contains the reason for ending playback, providing context for the streaming activity.
    pub reason_end: Option<String>,

    /// Indicates whether shuffle mode was active.
    ///
    /// This field is a boolean indicating whether shuffle mode was active during the streaming activity.
    pub shuffle: Option<bool>,

    /// Indicates whether the track was skipped.
    ///
    /// This field is a boolean indicating whether the user skipped the track during the streaming activity.
    pub skipped: Option<bool>,

    /// Indicates whether the playback was in offline mode.
    ///
    /// This field is a boolean indicating whether the streaming activity occurred in offline mode.
    pub offline: Option<bool>,

    /// Timestamp of the offline playback event.
    ///
    /// This field contains the timestamp of the offline playback event if the streaming activity occurred offline.
    pub offline_timestamp: Option<u128>,

    /// Indicates whether the user was in incognito mode during the streaming activity.
    ///
    /// This field is a boolean indicating whether the user was in incognito mode during the streaming activity.
    pub incognito_mode: Option<bool>,
}

/// Represents a collection of Spotify streaming entries.
///
/// Raw streaming data is a collection of `SpotifyEntry` objects, typically obtained from
/// parsing and accumulating data from multiple JSON files.
///
/// # Examples
///
/// ```rust
/// use spotify_stats::model::raw_streaming_data::RawStreamingData;
///
/// let raw_data = RawStreamingData(vec![/*... list of SpotifyEntry objects ...*/]);
/// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spotify_stats::{model::{Persist, raw_streaming_data::RawStreamingData}};
    ///
    /// // Specify the path to the folder containing JSON files
    /// let folder_path = "full_data";
    ///
    /// // Attempt to create a RawStreamingData object from the specified folder
    /// match RawStreamingData::from_folder_of_json(folder_path) {
    ///     Ok(raw_data) => {
    ///         // Successfully created RawStreamingData object
    ///         println!("Raw streaming data loaded successfully: {:?}", raw_data);
    ///     }
    ///     Err(err) => {
    ///         // Handle the error if reading or parsing fails
    ///         eprintln!("Error loading raw streaming data: {}", err);
    ///         panic!();
    ///     }
    /// }
    /// ```
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
