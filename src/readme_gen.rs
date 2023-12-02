//! Use clap derive macro to automatically also generate the overview section of the README.md
//! 
//! I want to extend this module to add more functionally for generating README automatically.
//! For example finding all the semver version in the git tags of the repo and link them in a `history` or `releases` section... etc.

use std::{error::Error, fs::File, io::Write as IOWrite, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = File::create("README.md")?;
    let header = r#"# Spotify-Stats

Command Line Interface that can process your Spotify Streaming Data.

## How to obtain your own Spotify Streaming Data

To obtain your own streaming data, go to the website [Spotify Privacy Settings](https://www.spotify.com/us/account/privacy/) and click on `Request Data` and make sure to include the actual streaming data.

## Overview

```txt
"#;
    let output = Command::new("cargo")
        .arg("run")
        .arg("-r")
        .arg("--bin")
        .arg("spotify_stats")
        .arg("--")
        .arg("-h")
        .current_dir(".")
        .env("RUSTFLAGS", "-Z threads=12")
        .output()
        .unwrap();

    let overview_content = String::from_utf8_lossy(&output.stdout);
    let after_overview_content = r#"```
"#;
    IOWrite::write_all(&mut file, header.as_bytes())?;
    IOWrite::write_all(&mut file, overview_content.as_bytes())?;
    IOWrite::write_all(&mut file, after_overview_content.as_bytes())?;
    Ok(())
}
