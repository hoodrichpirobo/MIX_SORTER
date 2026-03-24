# MIX_SORTER

Sort Spotify playlists for smoother harmonic mixing using your own local track metadata.

MIX_SORTER is a Rust CLI that:

- reads a Spotify playlist
- matches tracks against `local_db.json`
- enriches them with BPM and Camelot key data
- reorders the playlist by harmonic compatibility and tempo
- writes the new order back to Spotify

This project exists because Spotify audio-feature workflows are no longer reliable enough for this use case. Instead of depending on deprecated or restricted metadata endpoints, MIX_SORTER treats your local database as the source of truth.

## Why It Is Useful

If you already know the BPM and key of tracks in your own library, this tool gives you a practical way to keep Spotify playlists usable for DJ preparation.

The value is simple:

- local control over metadata
- no dependence on Spotify key/BPM endpoints
- matching that tolerates title noise like remasters, edits, and formatting differences
- deterministic playlist ordering for harmonic mixing

## What It Does

Given a playlist ID or Spotify playlist URL, MIX_SORTER will:

1. authenticate with Spotify
2. fetch all playlist tracks
3. look up each track in `local_db.json`
4. score possible matches using artist, title, and optional duration
5. convert Camelot notation into sortable values
6. sort matched tracks first, then append unmatched tracks at the end
7. replace the playlist order on Spotify

## Matching Rules

The matcher is intentionally simple and inspectable.

- Track titles are normalized before comparison
- Exact artist matches score highest
- Partial artist containment is allowed for cases like featured artists
- Optional `duration_ms` helps separate originals from alternate edits or remixes
- Exact normalized title matches get an extra boost
- Tracks with invalid Camelot values are reported and skipped from enriched sorting
- Tracks with no usable match stay in the playlist, but are placed after matched tracks

Terminal output includes markers such as:

- `[MATCH]` for successful enrichment
- `[MISSING]` when no local metadata match is found
- `[KEY ERROR]` when a local entry has an invalid Camelot key

## Requirements

- Rust toolchain
- a Spotify developer app
- a `.env` file with Spotify credentials
- a local metadata file at `local_db.json`

## Spotify App Setup

Create a Spotify developer app and set the redirect URI to:

```text
http://localhost:8888/callback
```

Then create a `.env` file in the project root:

```ini
RSPOTIFY_CLIENT_ID=your_client_id_here
RSPOTIFY_CLIENT_SECRET=your_client_secret_here
RSPOTIFY_REDIRECT_URI=http://localhost:8888/callback
```

## Local Database Format

`local_db.json` should be a JSON array of track records like this:

```json
[
  {
    "name": "Losing It",
    "artist": "FISHER",
    "bpm": 125,
    "key_camelot": "10B",
    "duration_ms": 248000
  },
  {
    "name": "Space Song",
    "artist": "Beach House",
    "bpm": 147,
    "key_camelot": "5A"
  }
]
```

Fields:

- `name`: track title
- `artist`: artist name used for matching
- `bpm`: tempo used for sorting
- `key_camelot`: Camelot key like `5A` or `10B`
- `duration_ms`: optional but recommended for distinguishing different versions

## Usage

Run with a raw playlist ID:

```bash
cargo run -- 2nOsiUa2nlXBGuDMjDIbDb
```

Or with a Spotify playlist URL:

```bash
cargo run -- "https://open.spotify.com/playlist/2nOsiUa2nlXBGuDMjDIbDb"
```

The CLI also accepts a Spotify URI:

```bash
cargo run -- "spotify:playlist:2nOsiUa2nlXBGuDMjDIbDb"
```

On first run, your browser should open for Spotify authorization.

## What Gets Sorted

Tracks with valid local metadata are sorted by:

1. Camelot wheel position
2. BPM ascending within the same harmonic bucket

Tracks that cannot be enriched remain in the playlist and are appended after the matched tracks.

## Current Project Shape

- [src/main.rs](/home/hoodrichpirobo/projects/MIX_SORTER/src/main.rs): CLI entrypoint, matcher, sorting logic, Spotify integration, and regression tests
- [local_db.json](/home/hoodrichpirobo/projects/MIX_SORTER/local_db.json): local metadata source
- [orchestrator/](/home/hoodrichpirobo/projects/MIX_SORTER/orchestrator): project workflow and context documents
- [.claude/commands](/home/hoodrichpirobo/projects/MIX_SORTER/.claude/commands): Claude-oriented project commands included with the repo

## Development

Recommended verification commands:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

At the time of the latest update, all three pass on the current codebase.

## Known Limits

- Matching quality is only as good as `local_db.json`
- There is no dedicated UI yet; this is a CLI-first tool
- Sorting is deterministic, but not “intelligent” beyond the current matching and harmonic rules
- Large metadata libraries may eventually justify moving from JSON to a structured store, but JSON is still the right default for this project right now

## Roadmap Direction

Likely next improvements:

- split core logic out of `main.rs` into dedicated modules
- add import/export tooling for metadata maintenance
- improve match explainability and confidence reporting
- support richer metadata beyond BPM and Camelot key

## License

MIT. See [LICENSE](/home/hoodrichpirobo/projects/MIX_SORTER/LICENSE).
