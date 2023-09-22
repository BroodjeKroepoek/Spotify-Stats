# Spotify-Stats

Command Line Interface that can process your Spotify Streaming Data.

## Obtaining your own Spotify Streaming Data

Go to the website <https://www.spotify.com/us/account/privacy/> and click on "Request Data", make sure to include streaming data.

## Quick Start

*First time running*: supply the '--data \<DATA\>' argument. This will print everything to stdout.

```ps
cargo run -r -- --data .\data\
```

After that try searching via '--artist \<ARTIST\>' or '--track \<TRACK\>'.

```ps
cargo run -r -- --artist "Lizzo"
```

See help or the long help, by using '-h' or '--help' respectively.

```ps
cargo run -r -- --help
```
