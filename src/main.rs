//! Here we describe the main control flow of the CLI, and all the commands and arguments.

#[cfg(test)]
pub mod tests;

use std::{
    error::Error,
    fmt::{Debug, Display},
    fs::File,
    io::Write,
    path::PathBuf,
};

use clap::{command, Parser, Subcommand};
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
enum Mode {
    Rust,
    JSON,
}

#[derive(Debug, Clone, Subcommand)]
enum Format {
    /// Only the top most played, or when passing the `reversed` flag the top least played.
    Sorted {
        /// Display the `top <COUNT>` entries.
        #[arg(short, long)]
        count: Option<usize>,
        /// Reverse the sorting, i.e. displaying the `top least` played songs.
        #[arg(short, long)]
        reversed: bool,
    },
    /// Don't sort use default lexicographical ordering.
    Lexicographical,
}

#[derive(Debug, Clone, Subcommand)]
enum MyCliCommand {
    /// Display the streaming data using a pretty and readable format in a table.
    ///
    /// It's possible to only show entries that match your search query, i.e. by specific `artist`, `album` or `track` name.
    /// Everything is in lexicographical ordering.
    Table {
        /// Redirect output to a file, with the given path.
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Only show entries that match this artist name, and/or matching other search queries.
        #[arg(long)]
        artist: Option<String>,
        /// Only show entries that match this album name, and/or matching other search queries.
        #[arg(long)]
        album: Option<String>,
        /// Only show entries that match this track name, and/or matching other search queries.
        #[arg(long)]
        track: Option<String>,
        /// Specify a specify format to use.
        #[command(subcommand)]
        format: Format,
    },
    /// Use raw format.
    ///
    /// Either using the internal Rust representation or formatting as JSON data.
    ///
    /// It's possible to only show data that matches your search query.
    /// The raw data is a nested map: `artist->album->track->[info entries]*`.
    Raw {
        /// Redirect output to a file, with the given path.
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Which raw mode to use: `Rust` or `JSON`.
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
    /// After first run: a persistent binary file is created relative to this executable, that contains all the relevant data compressed.
    /// This executable first tries to find this file, and if it is not present only then will an error be displayed, asking you to provide this folder.
    #[arg(short, long)]
    data: Option<PathBuf>,
    /// The format to use when presenting the results to the user.
    #[command(subcommand)]
    command: MyCliCommand,
}

fn deligate_output_debug<T>(file: Option<PathBuf>, output: T) -> Result<(), Box<dyn Error>>
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

fn deligate_output_display<T>(file: Option<PathBuf>, output: T) -> Result<(), Box<dyn Error>>
where
    T: Display,
{
    if let Some(path) = file {
        let mut handle = File::create(path)?;
        writeln!(handle, "{}", output)?;
    } else {
        println!("{}", output)
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
        MyCliCommand::Raw { file, mode } => match mode {
            Mode::Rust => deligate_output_debug(file, streaming_data)?,
            Mode::JSON => {
                let string = serde_json::to_string_pretty(&streaming_data)?;
                deligate_output_display(file, string)?;
            }
        },
        MyCliCommand::Table {
            file,
            format,
            artist,
            album,
            track,
        } => {
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            match format {
                Format::Sorted { count, reversed } => {
                    let mut counter = 1;
                    let mut cleaned_entries = CleanedStreamingData::from(streaming_data);
                    if !reversed {
                        cleaned_entries
                            .0
                            .sort_by(|a, b| a.total_ms_played.cmp(&b.total_ms_played).reverse());
                    } else {
                        cleaned_entries
                            .0
                            .sort_by(|a, b| a.total_ms_played.cmp(&b.total_ms_played));
                    }
                    table.set_header(["Rank", "Artist", "Album", "Track", "Duration (ms)"]);
                    for cleaned_entry in cleaned_entries.0 {
                        if (Some(&cleaned_entry.artist) == artist.as_ref()
                            || Some(&cleaned_entry.album) == album.as_ref()
                            || Some(&cleaned_entry.track) == track.as_ref())
                            ^ (artist.is_none() && album.is_none() && track.is_none())
                        {
                            if counter <= count.unwrap_or_default() || count.is_none() {
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
                }
                Format::Lexicographical => {
                    table.set_header(["Artist", "Album", "Track", "Duration (ms)"]);
                    iterate_nested_map!(streaming_data, artist, album, track, info, {
                        let duration = info.1;
                        table.add_row([
                            format!("{artist}"),
                            format!("{album}"),
                            format!("{track}"),
                            format!("{duration}"),
                        ]);
                    });
                }
            }
            deligate_output_display(file, table)?;
        }
    }
    Ok(())
}
