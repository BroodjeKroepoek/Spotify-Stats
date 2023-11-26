# Spotify-Stats

Command Line Interface that can process your Spotify Streaming Data.

## Obtaining your own Spotify Streaming Data

Go to the website <https://www.spotify.com/us/account/privacy/> and click on "Request Data", make sure to include streaming data.

## Quick Start

*First time running*: supply the '--data \<DATA\>' argument.
The executable expects of folder of one or more JSON files, that is expected to use the Spotify format described in `SpotifyEntry` and `RawStreamingData` in [raw_streaming_data.rs](src/lib/model/raw_streaming_data.rs)

```ps
spotify-stats --data .\data_folder
```

After that try searching via '--artist \<ARTIST\>' or '--track \<TRACK\>'.

```ps
spotify-stats --artist "Lizzo" pretty
```

See help or the long help, by using '-h' or '--help' respectively.

```ps
spotify-stats --help
```

## Milestones

- [x] Implement a 'top x' feature, showing your top *x* most listened to songs / artist, in sorted order.
- [x] Maybe change the 'StreamingHistory' struct to again include durations per entry of the original Spotify data, instead of summing it over each track.
- [ ] Do something with the end timings, we don't use that data now.
  - [ ] Maybe the first date you ever listened to the song / artist, and the last date you did.
  - [ ] Or show the day that you 'binge listened', to a specific song / artist.
- [ ] Using the Spotify API to create a playlist of your 'top x' favourite songs.
  - [ ] This will need some sort of persistent storage and credentials.
- [ ] *ms* listened to, is not very easy to read.
- [x] Can API calls also supply us with streaming data? -> Not really.
- [ ] Add case insensitivity to searching, and maybe Levenshtein distance, so making small spelling errors is allowed.
- [ ] Add benchmarks for persistence?
- [x] Add compression for our database. (Data went down from 62 mb to ~5 mb with our folded data and with compression down to 1.4 mb!)
- [ ] Separate our persistence-related functionality to a separate crate.

## Keep in Mind

- [ ] <https://crates.io/crates/mdbook-autosummary>
- [ ] <https://crates.io/crates/rubedo>
