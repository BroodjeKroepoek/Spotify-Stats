#[cfg(test)]
pub mod tests;

use std::{error::Error, fs::File, io::Write, path::PathBuf};

use clap::{Parser, Subcommand};
use comfy_table::{presets::ASCII_MARKDOWN, Table};

use spotify_stats::{
    iterate_nested_map,
    model::{
        raw_streaming_data::RawStreamingData,
        streaming_data::{CleanedStreamingData, FoldedStreamingData},
        Persist,
    },
};

#[derive(Debug, Clone, Subcommand)]
enum MyCliCommand {
    /// Use tabular format.
    ///
    /// It's possible to only show entries that match your search query.
    /// Everything is in lexicographical ordering.
    Table {
        /// Redirect output to a file.
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Use JSON format.
    ///
    /// It's possible to only show data that matches your search query.
    /// The JSON is a nested map: `artist->album->track->[INFO]`.
    Json {
        /// Redirect output to a file.
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Use tabular format but in sorted order according to `Duration` played.
    ///
    /// It's possible to only show entries that match your search query.
    /// Everything is in sorted order, according to the last column `Duration`.
    Sorted {
        /// Display only `top <N>` entries.
        n: Option<usize>,
        /// Redirect output to a file.
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

/// Command Line Interface that can process your Spotify Streaming Data.
#[derive(Parser, Debug)]
#[clap(version, author)]
struct MyCLI {
    /// REQUIRED ON FIRST RUN: The folder to extract the Spotify streaming data from.
    ///
    /// After first run: A binary file is created relative to this cli, that contains all the data summarized.
    #[arg(short, long)]
    data: Option<PathBuf>,
    /// Show only entries with this artist, or matching other queries provided.
    #[arg(long)]
    artist: Option<String>,
    /// Show only entries from this album, or matching other queries provided.
    #[arg(long)]
    album: Option<String>,
    /// Use compression for the database.
    #[arg(short, long)]
    compression: bool,
    /// Show only entries of this track, or matching other queries provided.
    #[arg(long)]
    track: Option<String>,
    /// The format to use when presenting the results to the user.
    #[command(subcommand)]
    command: MyCliCommand,
}

const JSON_DATA_PATH: &str = "spotify_stats.bin";

fn main() -> Result<(), Box<dyn Error>> {
    let args = MyCLI::parse();
    let streaming_data = match FoldedStreamingData::load_from_file(JSON_DATA_PATH) {
        Ok(streaming_data) => streaming_data,
        Err(err) => match args.data {
            Some(path) => {
                let raw_streaming_data: RawStreamingData =
                    RawStreamingData::from_folder_of_json(&path)?;
                let streaming_data = FoldedStreamingData::from(raw_streaming_data);
                streaming_data.save_to_file(JSON_DATA_PATH, args.compression)?;
                streaming_data
            }
            None => {
                panic!(
                    "the '--data <DATA>' argument was not provided, required on first run: {}",
                    err
                );
            }
        },
    };
    match args.command {
        MyCliCommand::Json { file: _ } => {
            // TODO: Implement this
            unimplemented!()
        }
        MyCliCommand::Table { file } => {
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            table.set_header(["Artist", "Album", "Track", "Duration (ms)"]);
            iterate_nested_map!(streaming_data, artist, album, track, info, {
                if (Some(artist) == args.artist.as_ref()
                    || Some(album) == args.album.as_ref()
                    || Some(track) == args.track.as_ref())
                    ^ (args.artist.is_none() && args.album.is_none() && args.track.is_none())
                {
                    table.add_row([artist, album, track, &info.1.num_milliseconds().to_string()]);
                }
            });
            if let Some(path) = file {
                let mut handle = File::create(path)?;
                write!(handle, "{}", table)?;
            } else {
                println!("{}", table)
            }
        }
        MyCliCommand::Sorted { n, file } => {
            let mut counter = 1;
            // TODO: Is CleanedStreamingData really needed? It is nice to have, but maybe unnecessary for us.
            let mut cleaned_entries = CleanedStreamingData::from(streaming_data);
            cleaned_entries
                .0
                .sort_by(|a, b| a.total_ms_played.cmp(&b.total_ms_played).reverse());
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            table.set_header(["Rank", "Artist", "Album", "Track", "Duration (ms)"]);
            for cleaned_entry in cleaned_entries.0 {
                if (Some(&cleaned_entry.artist) == args.artist.as_ref()
                    || Some(&cleaned_entry.album) == args.album.as_ref()
                    || Some(&cleaned_entry.track) == args.track.as_ref())
                    ^ (args.artist.is_none() && args.album.is_none() && args.track.is_none())
                {
                    if counter <= n.unwrap_or_default() || n.is_none() {
                        table.add_row([
                            counter.to_string(),
                            cleaned_entry.artist.clone(),
                            cleaned_entry.album.clone(),
                            cleaned_entry.track.clone(),
                            cleaned_entry.total_ms_played.num_milliseconds().to_string(),
                        ]);
                        counter += 1;
                    }
                }
            }
            if let Some(path) = file {
                let mut handle = File::create(path)?;
                write!(handle, "{}", table)?;
            } else {
                println!("{}", table)
            }
        }
    }
    Ok(())
}
