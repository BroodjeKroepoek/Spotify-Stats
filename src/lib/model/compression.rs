//! This module describes the format that Spotify uses.
//!
//! This is so we can interact with the Spotify streaming data.

use std::{collections::BTreeMap, net::IpAddr, ops::AddAssign};

use chrono::{Duration, NaiveDateTime};

use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::{
    model::end_stream::EndStreamKind,
    serde::{deserialization::duration_deserialization, serialization::duration_serialization},
};

use super::end_stream::{
    EndStreamWithKind, EndStreamWithKindContainer, FromFolderJson, INITIAL_VEC_CAP,
};

/// Represents a log entry for a streaming event, including play duration and reasons.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EndStreamLogEntry {
    pub platform: String,
    /// Duration of the music track played.
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub ms_played: Duration, // ms_played
    /// The reason for starting playback (e.g., "trackdone").
    pub reason_start: Option<String>, // reason_start
    /// The reason for ending playback (e.g., "trackdone").
    pub reason_end: Option<String>, // reason_end
    pub shuffle: Option<bool>,
    pub skipped: Option<bool>,
    pub offline: Option<bool>,
    pub ip_addr_decrypted: Option<IpAddr>,
    pub user_agent_decrypted: Option<String>,
    pub offline_timestamp: Option<u128>,
    pub incognito_mode: Option<bool>,
}

/// Represents a log of streaming events indexed by timestamp.
///
/// # Examples
///
/// ```rust
/// use spotify_stats::model::compression::{EndStreamLog, EndStreamLogEntry};
/// use chrono::{NaiveDateTime, Duration};
/// use std::collections::BTreeMap;
///
/// let mut log_map = BTreeMap::new();
/// log_map.insert(
///     NaiveDateTime::parse_from_str("2013-05-03T16:35:29Z", "%Y-%m-%dT%H:%M:%SZ").unwrap(),
///     EndStreamLogEntry{
///         total_ms_played: Duration::seconds(180),
///         reason_start: Some("trackstart".to_string()),
///         reason_end: Some("trackdone".to_string()),
///     },
/// );
/// let log = EndStreamLog(log_map);
/// ```
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EndStreamLog(
    /// A BTreeMap where the key is the timestamp and the value is a `LogEntry`.
    pub BTreeMap<NaiveDateTime, EndStreamLogEntry>,
);

impl EndStreamLog {
    fn new() -> Self {
        Self(BTreeMap::new())
    }

    fn bind(key: NaiveDateTime, value: EndStreamLogEntry) -> Self {
        let mut out = Self::new();
        out.insert(key, value);
        out
    }

    fn insert(&mut self, key: NaiveDateTime, value: EndStreamLogEntry) {
        assert!(self.0.insert(key, value).is_none())
    }
}

impl IntoIterator for EndStreamLog {
    type Item = (NaiveDateTime, EndStreamLogEntry);

    type IntoIter = std::collections::btree_map::IntoIter<NaiveDateTime, EndStreamLogEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(NaiveDateTime, EndStreamLogEntry)> for EndStreamLog {
    fn from_iter<T: IntoIterator<Item = (NaiveDateTime, EndStreamLogEntry)>>(iter: T) -> Self {
        EndStreamLog(BTreeMap::from_iter(iter))
    }
}

/// Represents information related to streaming data, including a log and total playtime.
///
/// # Examples
///
/// ```rust
/// use spotify_stats::model::compression::{EndStreamLog, AssocInfo};
/// use std::collections::BTreeMap;
/// use chrono::Duration;
///
/// let information = AssocInfo{
///     end_stream_log: EndStreamLog(BTreeMap::new()), // Empty log for illustration purposes
///     total_ms_played: Duration::milliseconds(180),
///     spotify_track_uri: Some("spotify:track:example_uri".to_string()),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct AssocInfo {
    /// Total duration of all music tracks played.
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub total_ms_played: Duration, // total_ms_played
    /// Spotify track URI associated with the streaming data.
    pub spotify_track_uri: Option<String>, // spotify_track_uri
    pub spotify_episode_uri: Option<String>,
    // TODO: Add the rest....
    /// Log of streaming events indexed by timestamp.
    pub end_stream_log: EndStreamLog, // log
}

/// Represents streaming data in a nested structure, grouped by artist, album, and track.
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct CompressedEndStreamWithKindContainer(
    // TODO: maybe add `platform` and `spotify_track_uri` and `spotify_episode_uri` (but this is none often) to the keys for increased searchability and compression
    /// A BTreeMap with nested structures representing artist,  album, track,   and track info...
    ///                                                podcast, show,  episode, and episode info...
    ///                                                ?,       ?,     ?,       and video info...
    pub  BTreeMap<
        String,
        BTreeMap<
            String,
            BTreeMap<
                EndStreamKind,
                BTreeMap<String, BTreeMap<String, BTreeMap<String, AssocInfo>>>,
            >,
        >,
    >,
);

impl CompressedEndStreamWithKindContainer {
    fn new() -> Self {
        Self(BTreeMap::new())
    }

    fn insert(
        &mut self,
        username: String,
        conn_country: String,
        kind: EndStreamKind,
        artist: String,
        album: String,
        track: String,
        info: AssocInfo,
    ) {
        self.0
            .entry(username)
            .or_default()
            .entry(conn_country)
            .or_default()
            .entry(kind)
            .or_default()
            .entry(artist)
            .or_default()
            .entry(album)
            .or_default()
            .entry(track)
            .and_modify(|x| *x += info.clone())
            .or_insert(info);
    }
}

impl FromFolderJson for CompressedEndStreamWithKindContainer {
    fn from_folder_of_json<P>(folder: P) -> Result<Self>
    where
        Self: Sized,
        P: AsRef<std::path::Path>,
    {
        let x = EndStreamWithKindContainer::from_folder_of_json(folder)?;
        Ok(Self::from(x))
    }
}

// TODO: this is an important function!
impl From<EndStreamWithKindContainer> for CompressedEndStreamWithKindContainer {
    fn from(value: EndStreamWithKindContainer) -> Self {
        let mut out = Self::new();
        for x in value {
            let (key1, key2, key3) = match x.kind {
                EndStreamKind::EndSong | EndStreamKind::EndVideoOrElse => (
                    &x.end_stream.master_metadata_album_artist_name,
                    &x.end_stream.master_metadata_album_album_name,
                    &x.end_stream.master_metadata_track_name,
                ),
                EndStreamKind::EndEpisode => (
                    &x.end_stream.master_metadata_album_artist_name,
                    &x.end_stream.episode_show_name,
                    &x.end_stream.episode_name,
                ),
            };
            let info = AssocInfo::from(&x);
            if let (Some(key1), Some(key2), Some(key3)) = (key1, key2, key3) {
                out.insert(
                    x.end_stream.username,
                    x.end_stream.conn_country,
                    x.kind,
                    key1.clone(),
                    key2.clone(),
                    key3.clone(),
                    info,
                )
            }
        }
        out
    }
}

impl IntoIterator for CompressedEndStreamWithKindContainer {
    type Item = (
        String,
        String,
        EndStreamKind,
        String,
        String,
        String,
        AssocInfo,
    );

    type IntoIter = <Vec<(
        String,
        String,
        EndStreamKind,
        String,
        String,
        String,
        AssocInfo,
    )> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let mut acc = Vec::with_capacity(INITIAL_VEC_CAP);
        for u in self.0 {
            for v in u.1 {
                for w in v.1 {
                    for x in w.1 {
                        for y in x.1 {
                            for z in y.1 {
                                acc.push((
                                    u.0.clone(),
                                    v.0.clone(),
                                    w.0.clone(),
                                    x.0.clone(),
                                    y.0.clone(),
                                    z.0,
                                    z.1,
                                ));
                            }
                        }
                    }
                }
            }
        }
        acc.into_iter()
    }
}

impl
    FromIterator<(
        String,
        String,
        EndStreamKind,
        String,
        String,
        String,
        AssocInfo,
    )> for CompressedEndStreamWithKindContainer
{
    fn from_iter<
        T: IntoIterator<
            Item = (
                String,
                String,
                EndStreamKind,
                String,
                String,
                String,
                AssocInfo,
            ),
        >,
    >(
        iter: T,
    ) -> Self {
        let mut out = CompressedEndStreamWithKindContainer::new();
        for (username, conn_country, kind, artist, album, track, info) in iter {
            out.insert(username, conn_country, kind, artist, album, track, info);
        }
        out
    }
}

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct CompressedEndStreamContainer(
    /// A BTreeMap with nested structures representing artist,  album, track,   and track info...
    ///                                                podcast, show,  episode, and episode info...
    ///                                                ?,       ?,     ?,       and video info...
    pub BTreeMap<String, BTreeMap<String, BTreeMap<String, AssocInfo>>>,
);

impl CompressedEndStreamContainer {
    fn new() -> Self {
        Self(BTreeMap::new())
    }

    fn insert(&mut self, artist: String, album: String, track: String, info: AssocInfo) {
        self.0
            .entry(artist)
            .or_default()
            .entry(album)
            .or_default()
            .entry(track)
            .and_modify(|x| *x += info.clone())
            .or_insert(info);
    }
}

impl IntoIterator for CompressedEndStreamContainer {
    type Item = (String, String, String, AssocInfo);

    type IntoIter = <Vec<(String, String, String, AssocInfo)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let mut acc = Vec::with_capacity(INITIAL_VEC_CAP);
        for x in self.0 {
            for y in x.1 {
                for z in y.1 {
                    acc.push((x.0.clone(), y.0.clone(), z.0, z.1))
                }
            }
        }
        acc.into_iter()
    }
}

impl FromIterator<(String, String, String, AssocInfo)> for CompressedEndStreamContainer {
    fn from_iter<T: IntoIterator<Item = (String, String, String, AssocInfo)>>(iter: T) -> Self {
        let mut out = Self::new();
        for (artist, album, track, info) in iter {
            out.insert(artist, album, track, info);
        }
        out
    }
}

impl From<&EndStreamWithKind> for EndStreamLog {
    fn from(value: &EndStreamWithKind) -> Self {
        Self::bind(value.end_stream.ts, EndStreamLogEntry::from(value))
    }
}

impl From<&EndStreamWithKind> for EndStreamLogEntry {
    fn from(value: &EndStreamWithKind) -> Self {
        EndStreamLogEntry {
            platform: value.end_stream.platform.clone(),
            ms_played: value.end_stream.ms_played,
            reason_start: value.end_stream.reason_start.clone(),
            reason_end: value.end_stream.reason_end.clone(),
            shuffle: value.end_stream.shuffle,
            skipped: value.end_stream.skipped,
            offline: value.end_stream.offline,
            ip_addr_decrypted: value.end_stream.ip_addr_decrypted,
            user_agent_decrypted: value.end_stream.user_agent_decrypted.clone(),
            offline_timestamp: value.end_stream.offline_timestamp,
            incognito_mode: value.end_stream.incognito_mode,
        }
    }
}

/// Convert a `SpotifyEntry` into `Information`.
impl From<&EndStreamWithKind> for AssocInfo {
    fn from(value: &EndStreamWithKind) -> Self {
        AssocInfo {
            total_ms_played: value.end_stream.ms_played,
            spotify_track_uri: value.end_stream.spotify_track_uri.clone(),
            spotify_episode_uri: value.end_stream.spotify_episode_uri.clone(),
            end_stream_log: EndStreamLog::from(value),
        }
    }
}

/// Implement addition and assignment for `Information`.
impl AddAssign for AssocInfo {
    fn add_assign(&mut self, rhs: Self) {
        self.total_ms_played = self.total_ms_played + rhs.total_ms_played;
        self.end_stream_log.0.extend(rhs.end_stream_log.0);
    }
}

/// Represents cleaned Spotify entry data, including artist, album, track, playtime, and log.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct EndStreamKindCompressedLog {
    /// Artist name associated with the played track.
    pub artist_or_podcast: String,
    /// Album name associated with the played track.
    pub album_or_show: String,
    /// Track name.
    pub track_or_episode: String,
    /// Total duration of the music track played.
    #[serde(
        deserialize_with = "duration_deserialization",
        serialize_with = "duration_serialization"
    )]
    pub total_ms_played: Duration,
    /// Log of streaming events indexed by timestamp.
    pub log: EndStreamLog,
}

/// Represents a collection of cleaned Spotify entries.
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct EndStreamKindCompressedLogContainer(
    /// A vector containing cleaned Spotify entry objects.
    pub Vec<EndStreamKindCompressedLog>,
);

/// Convert `FoldedStreamingData` into `CleanedStreamingData`.
impl From<CompressedEndStreamWithKindContainer> for EndStreamKindCompressedLogContainer {
    fn from(value: CompressedEndStreamWithKindContainer) -> Self {
        for _x in value {
            todo!()
        }
        todo!()
    }
}

// Convert `RawStreamingData` into `FoldedStreamingData`.
// impl From<EndStreamKindContainer> for CompressedEndStreamWithKindContainer {
//     fn from(value: EndStreamKindContainer) -> Self {
//         let mut accumulator = CompressedEndStreamWithKindContainer(BTreeMap::new());
//         for thing in value.0.into_iter() {
//             match thing {
//                 EndStreamKind::EndSong {
//                     ts,
//                     username,
//                     platform,
//                     ms_played,
//                     conn_country,
//                     ip_addr_decrypted,
//                     user_agent_decrypted,
//                     master_metadata_track_name,
//                     master_metadata_album_artist_name,
//                     master_metadata_album_album_name,
//                     spotify_track_uri,
//                     reason_start,
//                     reason_end,
//                     shuffle,
//                     skipped,
//                     offline,
//                     offline_timestamp,
//                     incognito_mode,
//                 } => {
//                     insert_nested_map!(
//                         accumulator.0,
//                         "music".to_string(),
//                         master_metadata_album_artist_name,
//                         master_metadata_album_album_name,
//                         master_metadata_track_name,
//                         AssocInfo::from(&thing.clone())
//                     );
//                 }
//                 EndStreamKind::EndEpisode {
//                     ts,
//                     username,
//                     platform,
//                     ms_played,
//                     conn_country,
//                     ip_addr_decrypted,
//                     user_agent_decrypted,
//                     master_metadata_album_artist_name,
//                     episode_name,
//                     episode_show_name,
//                     spotify_episode_uri,
//                     reason_start,
//                     reason_end,
//                     shuffle,
//                     skipped,
//                     offline,
//                     offline_timestamp,
//                     incognito_mode,
//                 } => {
//                     insert_nested_map!(
//                         accumulator.0,
//                         "podcast".to_string(),
//                         master_metadata_album_artist_name,
//                         episode_show_name,
//                         episode_name,
//                         AssocInfo::from(&thing)
//                     );
//                 }
//                 EndStreamKind::EndVideo {
//                     ts,
//                     username,
//                     platform,
//                     ms_played,
//                     conn_country,
//                     ip_addr_decrypted,
//                     user_agent_decrypted,
//                     master_metadata_track_name,
//                     master_metadata_album_artist_name,
//                     master_metadata_album_album_name,
//                     spotify_track_uri,
//                     episode_name,
//                     episode_show_name,
//                     spotify_episode_uri,
//                     reason_start,
//                     reason_end,
//                     shuffle,
//                     skipped,
//                     offline,
//                     offline_timestamp,
//                     incognito_mode,
//                 } => accumulator.insert("video".to_string(), artist, album, track, info),
//             };
//         }
//         accumulator
//     }
// }
