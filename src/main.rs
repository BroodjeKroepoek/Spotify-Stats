#[cfg(test)]
pub mod tests;

use std::{
    error::Error,
    fs::File,
    io::Write,
    io::{stdout, Stdout},
    path::PathBuf,
};

use bevy::{
    prelude::{App, Startup, Update},
    DefaultPlugins,
};
use clap::{Parser, Subcommand, ValueEnum};
use comfy_table::{presets::ASCII_MARKDOWN, Table};

use spot_stats::{
    gui::{animate_rotation, animate_scale, animate_translation, setup},
    iterate_nested_map,
    model::{
        raw_streaming_data::RawStreamingData,
        streaming_data::{CleanedStreamingData, FoldedStreamingData},
        Persist,
    },
};

#[derive(Debug, Clone, Default, Subcommand)]
enum Format {
    /// Use table formatting.
    #[default]
    Table,
    /// Use JSON formatting.
    Json,
    /// Use table formatting, but displaying your `top x`.
    Sorted {
        /// Amount of track to display in sorted order.
        ///
        /// If no extra info is given, sort everything.
        #[arg(default_value_t)]
        amount_tracks: usize,
    },
    /// GUI using bevy-engine!
    Bevy,
}

#[derive(Debug, Clone, ValueEnum, Default)]
enum Output {
    #[default]
    Stdout,
    File,
}

enum OutputRuntime {
    Stdout(Stdout),
    File(File),
}

/// Command Line Interface that can process your Spotify Streaming Data
#[derive(Parser, Debug)]
#[clap(version, author)]
struct MyCLI {
    /// The name of the artist to search for.
    #[arg(long)]
    artist: Option<String>,
    /// The name of the album to search for.
    #[arg(long)]
    album: Option<String>,
    /// The name of the track to search for.
    #[arg(long)]
    track: Option<String>,
    /// The formatting to use when printing the results to stdout.
    #[command(subcommand)]
    format: Format,
    /// The folder to extract the data from, required on first run.
    #[arg(short, long)]
    data: Option<PathBuf>,
    #[arg(short, long, value_enum, default_value_t)]
    output: Output,
}

const JSON_DATA_PATH: &str = "spot_stats_folded.json";
const OUTPUT_PATH: &str = "spot_stats_output.txt";

fn main() -> Result<(), Box<dyn Error>> {
    let args = MyCLI::parse();
    let streaming_data = match FoldedStreamingData::load(JSON_DATA_PATH) {
        Ok(streaming_data) => streaming_data,
        Err(_) => match args.data {
            Some(path) => {
                let raw_streaming_data: RawStreamingData = RawStreamingData::from_path(&path)?;
                let streaming_data = FoldedStreamingData::from(raw_streaming_data);
                streaming_data.save(JSON_DATA_PATH)?;
                streaming_data
            }
            None => {
                panic!("the '--data <DATA>' argument was not provided, required on first run.")
            }
        },
    };
    let output_stream = match args.output {
        Output::Stdout => OutputRuntime::Stdout(stdout()),
        Output::File => OutputRuntime::File(File::create(OUTPUT_PATH).unwrap()),
    };
    match args.format {
        Format::Json => {
            // TODO: Redo this
        }
        Format::Table => {
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            table.set_header(["Artist", "Album", "Track", "Duration (ms)"]);
            iterate_nested_map!(streaming_data, artist, album, track, info, {
                if (Some(artist) == args.artist.as_ref()
                    || Some(album) == args.album.as_ref()
                    || Some(track) == args.track.as_ref())
                    ^ (args.artist.is_none() && args.album.is_none() && args.track.is_none())
                {
                    table.add_row([
                        artist,
                        album,
                        track,
                        &info.total_ms_played.num_milliseconds().to_string(),
                    ]);
                }
            });
            match output_stream {
                OutputRuntime::Stdout(mut x) => x.write_all(table.to_string().as_bytes())?,
                OutputRuntime::File(mut x) => x.write_all(table.to_string().as_bytes())?,
            }
        }
        Format::Sorted { amount_tracks } => {
            let mut counter = 1;
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
                    if counter <= amount_tracks || amount_tracks == 0 {
                        table.add_row([
                            counter.to_string(),
                            cleaned_entry.artist.clone(),
                            cleaned_entry.album.clone(),
                            cleaned_entry.track.clone(),
                            cleaned_entry.total_ms_played.num_milliseconds().to_string(),
                        ]);
                    } else {
                        break;
                    }
                }
                counter += 1;
            }
            match output_stream {
                OutputRuntime::Stdout(mut x) => x.write_all(table.to_string().as_bytes())?,
                OutputRuntime::File(mut x) => x.write_all(table.to_string().as_bytes())?,
            }
        }
        Format::Bevy => {
            App::new()
                .add_plugins(DefaultPlugins)
                .add_systems(Startup, setup)
                .add_systems(
                    Update,
                    (animate_translation, animate_rotation, animate_scale),
                )
                .run();
        }
    }
    Ok(())
}
