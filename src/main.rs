// Module definitions
pub mod model;
pub mod serde;
#[cfg(test)]
pub mod tests;

// Std imports
use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    io::{stdout, Stdout},
    path::PathBuf,
};

use std::io::Write;

// Dependency imports
use clap::{Parser, ValueEnum};
use comfy_table::{presets::ASCII_MARKDOWN, Table};

// Modular imports
use crate::model::{
    raw_streaming_data::RawStreamingData,
    streaming_data::{CleanedStreamingData, FoldedStreamingData},
    Persist,
};

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
    /// `Table` formatting, but sorted, for now
    Sorted,
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
    #[arg(short, long, value_enum, default_value_t)]
    output: Output,
}

const JSON_DATA_PATH: &str = "spot_stats.json";
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
                panic!("the '--data <DATA>' argument was not provided, this is needed the first time only")
            }
        },
    };
    let output_stream = match args.output {
        Output::Stdout => OutputRuntime::Stdout(stdout()),
        Output::File => OutputRuntime::File(File::create(OUTPUT_PATH).unwrap()),
    };
    match args.format {
        Format::Json => {
            let json = match (args.artist, args.track) {
                (None, None) => serde_json::to_string_pretty(&streaming_data)?,
                (None, Some(args_track)) => {
                    let mut accumulator = FoldedStreamingData(BTreeMap::new());
                    iterate_nested_map!(streaming_data, artist, track, platform, time, {
                        if track == args_track {
                            insert_nested_map!(
                                accumulator,
                                artist.clone(),
                                track.clone(),
                                platform,
                                time
                            )
                        }
                    });
                    serde_json::to_string_pretty(&accumulator)?
                }
                (Some(args_artist), None) => {
                    serde_json::to_string_pretty(&streaming_data.0[&args_artist])?
                }
                (Some(args_artist), Some(args_track)) => {
                    serde_json::to_string_pretty(&streaming_data.0[&args_artist][&args_track])?
                }
            };
            match output_stream {
                OutputRuntime::Stdout(mut x) => x.write_all(json.as_bytes())?,
                OutputRuntime::File(mut x) => x.write_all(json.as_bytes())?,
            };
        }
        Format::Table => {
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            table.set_header(["Artist", "Track", "Platform", "Time Played (ms)"]);
            iterate_nested_map!(streaming_data, artist, track, platform, time, {
                if (Some(&artist) == args.artist.as_ref() || Some(&track) == args.track.as_ref())
                    ^ (args.artist.is_none() && args.track.is_none())
                {
                    table.add_row([
                        &artist,
                        &track,
                        &platform,
                        &time.ms_played.num_milliseconds().to_string(),
                    ]);
                }
            });
            match output_stream {
                OutputRuntime::Stdout(mut x) => x.write_all(table.to_string().as_bytes())?,
                OutputRuntime::File(mut x) => x.write_all(table.to_string().as_bytes())?,
            }
        }
        Format::Sorted => {
            let mut cleaned_entries = CleanedStreamingData::from(streaming_data);
            cleaned_entries
                .0
                .sort_by(|a, b| a.ms_played.cmp(&b.ms_played));
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            table.set_header(["Artist", "Track", "Platform", "Time Played (ms)"]);
            for cleaned_entry in cleaned_entries.0 {
                if (Some(&cleaned_entry.artist) == args.artist.as_ref()
                    || Some(&cleaned_entry.track) == args.track.as_ref())
                    ^ (args.artist.is_none() && args.track.is_none())
                {
                    table.add_row([
                        &cleaned_entry.artist,
                        &cleaned_entry.track,
                        &cleaned_entry.platform,
                        &cleaned_entry.ms_played.num_milliseconds().to_string(),
                    ]);
                }
            }
            match output_stream {
                OutputRuntime::Stdout(mut x) => x.write_all(table.to_string().as_bytes())?,
                OutputRuntime::File(mut x) => x.write_all(table.to_string().as_bytes())?,
            }
        }
    }
    Ok(())
}
