# Spotify-Stats

Command Line Interface that can process your Spotify Streaming Data.

## Obtaining your own Spotify Streaming Data

Go to the website <https://www.spotify.com/us/account/privacy/> and click on "Request Data", make sure to include streaming data.

## Quick Start

*First time running*: supply the '--data \<DATA\>' argument. This will print everything to stdout.

```ps
spotify-stats --data .\data\
```

After that try searching via '--artist \<ARTIST\>' or '--track \<TRACK\>'.

```ps
spotify-stats --artist "Lizzo"
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
