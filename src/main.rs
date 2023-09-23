pub mod model;
pub mod serde;
#[cfg(test)]
pub mod tests;

use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, read_to_string},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};
use comfy_table::{presets::ASCII_MARKDOWN, Table};

use crate::model::{raw_streaming_data::RawStreamingData, streaming_data::StreamingData};

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
}

const JSON_DATA: &str = "spotify-stats.json";

fn main() -> Result<(), Box<dyn Error>> {
    let args = MyCLI::parse();
    let streaming_history: StreamingData = match read_to_string(JSON_DATA) {
        Ok(string) => serde_json::from_str(&string)?,
        Err(_) => match args.data {
            Some(path) => {
                eprintln!("[INFO] `{}` didn't exist yet, creating...", JSON_DATA);
                let raw_streaming_history: RawStreamingData = RawStreamingData::from_path(&path)?;
                let streaming_history = StreamingData::from(raw_streaming_history);
                let json = serde_json::to_string(&streaming_history)?;
                fs::write(JSON_DATA, json)?;
                eprintln!("[INFO] Finished creating `combined.json`");
                streaming_history
            }
            None => {
                panic!("the '--data <DATA>' argument was not provided, this is needed the first time only")
            }
        },
    };
    match args.format {
        Format::Json => {
            let json = match (args.artist, args.track) {
                (None, None) => serde_json::to_string_pretty(&streaming_history)?,
                (None, Some(args_track)) => {
                    let mut accumulator: StreamingData = StreamingData(BTreeMap::new());
                    for (artist, rest) in streaming_history.0 {
                        for (track, time) in rest {
                            if track == args_track {
                                let mut new = BTreeMap::new();
                                new.insert(track, time);
                                accumulator.0.insert(artist.clone(), new);
                            }
                        }
                    }
                    serde_json::to_string_pretty(&accumulator)?
                }
                (Some(args_artist), None) => {
                    serde_json::to_string_pretty(&streaming_history.0[&args_artist])?
                }
                (Some(args_artist), Some(args_track)) => {
                    serde_json::to_string_pretty(&streaming_history.0[&args_artist][&args_track])?
                }
            };
            println!("{}", json);
        }
        Format::Table => {
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            table.set_header(["Artist", "Track", "Time Played (ms)"]);
            for (artist, rest) in &streaming_history.0 {
                for (track, time) in rest {
                    if (Some(artist) == args.artist.as_ref() || Some(track) == args.track.as_ref())
                        ^ (args.artist.is_none() && args.track.is_none())
                    {
                        table.add_row([
                            artist,
                            track,
                            &time.ms_played.num_milliseconds().to_string(),
                        ]);
                    }
                }
            }
            println!("{}", table);
        }
    }
    Ok(())
}
