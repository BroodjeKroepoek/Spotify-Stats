//! Here we describe the main control flow of the CLI, and all the commands and arguments.

#[cfg(test)]
pub mod tests;

use std::{
    fmt::{Debug, Display},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use clap::{command, Parser, Subcommand};

use comfy_table::{presets::ASCII_MARKDOWN, Table};
use eyre::{Ok, Result};
use spotify_stats::model::{
    compression::{CompressedEndStreamWithKindContainer, EndStreamKindCompressedLogContainer},
    end_stream::FromFolderJson,
    Persist,
};

#[derive(Debug, Clone, Subcommand)]
enum RawFormat {
    /// Displays the debug formatting of the internal data used by this executable.
    Rust,
    /// Displays the internal data used by this executable in JSON format.
    Json,
    /// Displays the internal data used in raw binary format.
    Bin {
        /// Apply compression
        #[arg(short, long)]
        compression: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum Format {
    /// Only the top most played, or when passing the `reversed` flag the top least played.
    Sort {
        /// Display the `top <COUNT>` entries.
        #[arg(short, long)]
        count: Option<usize>,
        /// Reverse the sorting, i.e. displaying the `top least` played songs.
        #[arg(short, long)]
        reversed: bool,
    },
    /// Don't sort use default lexicographical ordering.
    Lex,
}

#[derive(Debug, Clone, Subcommand)]
enum SpotifyStatsCommand {
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
    /// Display the streaming data using the raw internal data format.
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
        mode: RawFormat,
    },
}

/// Command Line Interface that can process your Spotify Streaming Data.
///
/// In the commands section you'll find the different formatting options.
#[derive(Parser, Debug)]
#[clap(version, author)]
struct SpotifyStats {
    /// FIRST RUN: The folder to extract the streaming data from.
    ///
    /// After first run: a persistent binary file `.\spotify_stats.bin` is created, relative to this executable, that contains all the relevant data compressed.
    /// This executable first tries to find this file, and if it is not present only then will an error be displayed, asking you to provide this folder.
    #[arg(short, long)]
    data: Option<PathBuf>,
    /// The format to use when presenting the results to the user.
    #[command(subcommand)]
    command: SpotifyStatsCommand,
}

fn deligate_output_debug<P, T>(file: Option<P>, output: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: Debug,
{
    if let Some(path) = file {
        let mut handle = File::create(path)?;
        writeln!(handle, "{output:?}")?;
    } else {
        println!("{output:?}")
    }
    Ok(())
}

fn deligate_output_debug_pretty<P, T>(file: Option<P>, output: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: Debug,
{
    if let Some(path) = file {
        let mut handle = File::create(path)?;
        writeln!(handle, "{output:#?}")?;
    } else {
        println!("{output:#?}")
    }
    Ok(())
}

fn deligate_output_display<P, T>(file: Option<P>, output: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: Display,
{
    if let Some(path) = file {
        let mut handle = File::create(path)?;
        writeln!(handle, "{output}")?;
    } else {
        println!("{output}")
    }
    Ok(())
}

fn init_data(
    data_path: Option<PathBuf>,
    compress: bool,
) -> Result<CompressedEndStreamWithKindContainer> {
    let streaming_data = if let Some(path) = data_path {
        if let Result::Ok(data) = CompressedEndStreamWithKindContainer::load_from_file(BIN_PATH) {
            Ok(data)?
        } else {
            CompressedEndStreamWithKindContainer::from_folder_of_json(path)?
        }
    } else {
        panic!("TODO")
    };
    streaming_data.save_to_file(BIN_PATH, compress)?;
    Ok(streaming_data)
}

pub const INITIAL_VEC_CAP: usize = 128;

pub const BIN_PATH: &str = "spotify_stats.bin";

fn main() -> Result<()> {
    let args = SpotifyStats::parse();
    let streaming_data = init_data(args.data, true)?;
    match args.command {
        SpotifyStatsCommand::Raw { file, mode } => match mode {
            RawFormat::Rust => deligate_output_debug_pretty(file, &streaming_data)?,
            RawFormat::Json => {
                deligate_output_display(file, &serde_json::to_string(&streaming_data)?)?
            }
            RawFormat::Bin { compression } => {
                let bytes = streaming_data.to_bytes(compression)?;
                deligate_output_display(file, &bytes.escape_ascii())?;
            }
        },
        SpotifyStatsCommand::Table {
            file,
            format,
            artist: _,
            album: _,
            track: _,
        } => {
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            match format {
                Format::Sort { count, reversed } => {
                    let mut counter = 1;
                    let mut cleaned_entries =
                        EndStreamKindCompressedLogContainer::from(streaming_data);
                    if reversed {
                        cleaned_entries
                            .0
                            .sort_by(|a, b| a.total_ms_played.cmp(&b.total_ms_played))
                    } else {
                        cleaned_entries
                            .0
                            .sort_by(|a, b| a.total_ms_played.cmp(&b.total_ms_played).reverse())
                    };
                    table.set_header(["Rank", "Artist", "Album", "Track", "Duration (ms)"]);
                    for cleaned_entry in cleaned_entries
                        .0
                        .iter()
                        .take(count.unwrap_or(cleaned_entries.0.len()))
                    {
                        table.add_row([
                            counter.to_string(),
                            cleaned_entry.artist_or_podcast.clone(),
                            cleaned_entry.album_or_show.clone(),
                            cleaned_entry.track_or_episode.clone(),
                            cleaned_entry.total_ms_played.num_milliseconds().to_string(),
                        ]);
                        counter += 1;
                    }
                }
                Format::Lex => {
                    table.set_header(["Artist", "Album", "Track", "Duration (ms)"]);
                    todo!()
                }
            };
            deligate_output_display(file, &table)?;
        }
    }
    Ok(())
}
