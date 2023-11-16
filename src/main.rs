//! Here we describe the main control flow of the CLI, and all the commands and arguments.

#[cfg(test)]
pub mod tests;

use std::{error::Error, fmt::Debug, fs::File, io::Write, path::PathBuf};

use clap::{command, Parser, Subcommand};
use comfy_table::{presets::ASCII_MARKDOWN, Table};

use spotify_stats::model::{
    raw_streaming_data::RawStreamingData,
    streaming_data::{CleanedStreamingData, FoldedStreamingData},
    Persist,
};

#[derive(Debug, Clone, Subcommand)]
enum Mode {
    Rust,
    JSON,
}

#[derive(Debug, Clone, Subcommand)]
enum Sorted {
    /// Only the top most played, or when passing the `reversed` flag the top least played.
    Sorted {
        #[arg(short, long)]
        count: Option<usize>,
        #[arg(short, long)]
        reversed: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum MyCliCommand {
    /// Use a pretty and readable format in a table.
    ///
    /// It's possible to only show entries that match your search query, i.e. by specific `artist`, `album` or `track` name.
    /// Everything is in lexicographical ordering.
    Pretty {
        /// Redirect output to a file, with the given path.
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Use a pretty and readable format but in sorted order according to `Duration` played.
        ///
        /// It's possible to only show entries that match your search query.
        /// Everything is in sorted order, according to the last column `Duration`.
        ///
        /// NOTE: If you pass in 0, it will give back everything in sorted order, not only the `top <SORTED>` entries.
        #[command(subcommand)]
        sorted: Option<Sorted>,
    },
    /// Use raw format.
    ///
    /// Either using the internal Rust representation or formatting as JSON data.
    ///
    /// It's possible to only show data that matches your search query.
    /// The raw data is a nested map: `artist->album->track->[info entries]*`.
    Raw {
        /// Redirect output to a file.
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Mode
        #[command(subcommand)]
        mode: Mode,
    },
}

/// Command Line Interface that can process your Spotify Streaming Data.
///
/// In the commands section you'll find the different formatting options.
#[derive(Parser, Debug)]
#[clap(version, author)]
struct MyCLI {
    /// REQUIRED ON FIRST RUN: The folder to extract the Spotify streaming data from.
    ///
    /// After first run: A binary file is created relative to this executable, that contains all the relevant data summarized.
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

fn deligate_output<T>(file: Option<PathBuf>, output: T) -> Result<(), Box<dyn Error>>
where
    T: Debug,
{
    if let Some(path) = file {
        let mut handle = File::create(path)?;
        writeln!(handle, "{:#?}", output)?;
    } else {
        println!("{:#?}", output)
    }
    Ok(())
}

const JSON_DATA_PATH: &str = "spotify_stats.bin";

fn main() -> Result<(), Box<dyn Error>> {
    // Step 1: Parse command line.
    let args = MyCLI::parse();

    // Step 2: Always open the persistent storage.
    let streaming_data = match FoldedStreamingData::load_from_file(JSON_DATA_PATH) {
        Ok(streaming_data) => streaming_data,
        Err(err) => match args.data {
            Some(path) => {
                let raw_streaming_data: RawStreamingData =
                    RawStreamingData::from_folder_of_json(&path)?;
                let streaming_data = FoldedStreamingData::from(raw_streaming_data);
                streaming_data.save_to_file(JSON_DATA_PATH)?;
                streaming_data
            }
            None => {
                panic!(
                    "the '--data <DATA>' argument was not provided, which is required on first run, exited with this error: {}",
                    err
                );
            }
        },
    };

    // Step 3: Match on parsed command line arguments and subcommands.
    match args.command {
        MyCliCommand::Raw { file, mode: _ } => deligate_output(file, streaming_data)?,
        MyCliCommand::Pretty { file: _, sorted } => {
            let mut table = Table::new();
            if let Some(sorted) = sorted {
                match sorted {
                    Sorted::Sorted { count, reversed } => {
                        let mut counter = 1;
                        // TODO: Is CleanedStreamingData really needed? It is nice to have, but maybe unnecessary for us.
                        let mut cleaned_entries = CleanedStreamingData::from(streaming_data);
                        if !reversed {
                            cleaned_entries.0.sort_by(|a, b| {
                                a.total_ms_played.cmp(&b.total_ms_played).reverse()
                            });
                        } else {
                            cleaned_entries
                                .0
                                .sort_by(|a, b| a.total_ms_played.cmp(&b.total_ms_played));
                        }
                        table.load_preset(ASCII_MARKDOWN);
                        table.set_header(["Rank", "Artist", "Album", "Track", "Duration (ms)"]);
                        for cleaned_entry in cleaned_entries.0 {
                            if (Some(&cleaned_entry.artist) == args.artist.as_ref()
                                || Some(&cleaned_entry.album) == args.album.as_ref()
                                || Some(&cleaned_entry.track) == args.track.as_ref())
                                ^ (args.artist.is_none()
                                    && args.album.is_none()
                                    && args.track.is_none())
                            {
                                if counter <= count.unwrap_or_default() {
                                    table.add_row([
                                        counter.to_string(),
                                        cleaned_entry.artist.clone(),
                                        cleaned_entry.album.clone(),
                                        cleaned_entry.track.clone(),
                                        cleaned_entry
                                            .total_ms_played
                                            .num_milliseconds()
                                            .to_string(),
                                    ]);
                                    counter += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
