use std::{
    fs::{read_dir, read_to_string},
    net::IpAddr,
    path::Path,
};

use chrono::{Duration, NaiveDateTime};

use crate::serde::{
    deserialization::{duration_deserialization, naive_date_time_deserialization},
    serialization::{duration_serialization, naive_date_time_serialization},
};
use eyre::Result;
use serde::{Deserialize, Serialize};

pub const INITIAL_VEC_CAP: usize = 128;

pub trait FromFolderJson {
    fn from_folder_of_json<P>(folder: P) -> Result<Self>
    where
        Self: Sized,
        P: AsRef<Path>;
}

/// Represents a singular EndStream: [EndTrack: [EndSong || EndEpisode] || EndVideo].
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EndStream {
    /// This field is a timestamp indicating when the track stopped
    /// playing in UTC (Coordinated Universal Time). The order is
    /// year, month and day followed by a timestamp in military time.
    #[serde(
        deserialize_with = "naive_date_time_deserialization",
        serialize_with = "naive_date_time_serialization"
    )]
    pub ts: NaiveDateTime,

    /// This field is your Spotify username.
    pub username: String,

    /// This field is the platform used when streaming the track (e.g.
    /// Android OS, Google Chromecast).
    pub platform: String,

    /// This field is the number of milliseconds the stream was
    /// played.
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub ms_played: Duration,

    /// This field is the country code of the country where the stream
    /// was played (e.g. SE - Sweden).
    pub conn_country: String,

    /// This field contains the IP address logged when streaming the
    /// track.
    pub ip_addr_decrypted: Option<IpAddr>,

    /// This field contains the user agent used when streaming the
    /// track (e.g. a browser, like Mozilla Firefox, or Safari).
    pub user_agent_decrypted: Option<String>,

    /// This field is the name of the track.
    pub master_metadata_track_name: Option<String>,

    /// This field is the name of the artist, band or podcast
    pub master_metadata_album_artist_name: Option<String>,

    /// This field is the name of the album of the track.
    pub master_metadata_album_album_name: Option<String>,

    /// A Spotify URI, uniquely identifying the track in the form of
    /// “spotify:track:\<base-62 string\>”
    /// A Spotify URI is a resource identifier that you can enter, for
    /// example, in the Spotify Desktop client’s search box to locate
    /// an artist, album, or track.    
    pub spotify_track_uri: Option<String>,

    /// This field contains the name of the episode of the podcast.
    pub episode_name: Option<String>,

    /// This field contains the name of the show of the podcast.
    pub episode_show_name: Option<String>,

    /// A Spotify Episode URI, uniquely identifying the podcast
    /// episode in the form of “spotify:episode:\<base-62 string\>”
    /// A Spotify Episode URI is a resource identifier that you can
    /// enter, for example, in the Spotify Desktop client’s search box
    /// to locate an episode of a podcast.    
    pub spotify_episode_uri: Option<String>,

    /// This field is a value telling why the track started (e.g.
    /// “trackdone”).        
    pub reason_start: Option<String>,

    /// This field is a value telling why the track ended (e.g.
    /// “endplay”).        
    pub reason_end: Option<String>,

    /// This field has the value True or False depending on if shuffle
    /// mode was used when playing the track.
    pub shuffle: Option<bool>,

    /// This field indicates if the user skipped to the next song.
    pub skipped: Option<bool>,

    /// This field indicates whether the track was played in offline
    /// mode (“True”) or not (“False”).    
    pub offline: Option<bool>,

    /// This field is a timestamp of when offline mode was used, if
    /// used.    
    pub offline_timestamp: Option<u128>,

    /// This field indicates whether the track was played in incognito
    /// mode (“True”) or not (“False”).
    pub incognito_mode: Option<bool>,
}

/// Represents a collection of EndStream.
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EndStreamContainer(pub Vec<EndStream>);

impl EndStreamContainer {
    fn new() -> Self {
        Self(Vec::with_capacity(INITIAL_VEC_CAP))
    }
}

impl FromFolderJson for EndStreamContainer {
    fn from_folder_of_json<P>(folder: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut out = Self::new();
        for maybe_file in read_dir(&folder)? {
            let file = maybe_file?;
            let content = read_to_string(file.path())?;
            let raw = serde_json::from_str::<Self>(&content)?;
            out.0.extend(raw.into_iter())
        }
        Ok(out)
    }
}

impl IntoIterator for EndStreamContainer {
    type Item = EndStream;

    type IntoIter = <Vec<EndStream> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<EndStream> for EndStreamContainer {
    fn from_iter<T: IntoIterator<Item = EndStream>>(iter: T) -> Self {
        EndStreamContainer(Vec::from_iter(iter))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum EndStreamKind {
    EndSong,
    EndEpisode,
    EndVideoOrElse,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EndStreamWithKind {
    pub kind: EndStreamKind,
    pub end_stream: EndStream,
}

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EndStreamWithKindContainer(pub Vec<EndStreamWithKind>);

impl EndStreamWithKindContainer {
    fn new() -> Self {
        Self(Vec::with_capacity(INITIAL_VEC_CAP))
    }
}

impl FromFolderJson for EndStreamWithKindContainer {
    fn from_folder_of_json<P>(folder: P) -> Result<Self>
    where
        Self: Sized,
        P: AsRef<Path>,
    {
        Ok(Self::from(EndStreamContainer::from_folder_of_json(folder)?))
    }
}

impl IntoIterator for EndStreamWithKindContainer {
    type Item = EndStreamWithKind;

    type IntoIter = <Vec<EndStreamWithKind> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<EndStreamWithKind> for EndStreamWithKindContainer {
    fn from_iter<T: IntoIterator<Item = EndStreamWithKind>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

// TODO: rewrite this and keep an eye an which field contribute to which kind
impl From<EndStream> for EndStreamWithKind {
    fn from(value: EndStream) -> Self {
        let x = if let (Some(ref _artist), Some(ref _album)) = (
            &value.master_metadata_album_artist_name,
            &value.master_metadata_album_album_name,
        ) {
            EndStreamKind::EndSong
        } else if let Some(ref _episode) = value.episode_name {
            EndStreamKind::EndEpisode
        } else {
            EndStreamKind::EndVideoOrElse
        };
        Self {
            end_stream: value,
            kind: x,
        }
    }
}

impl From<EndStreamContainer> for EndStreamWithKindContainer {
    fn from(value: EndStreamContainer) -> Self {
        Self(value.into_iter().map(EndStreamWithKind::from).collect())
    }
}
