# Tech Stack

## Application Type

- Rust command-line application

## Core Language And Runtime

- Rust 2021 edition
- Tokio for async runtime

## Libraries

- `rspotify` for Spotify authentication and playlist operations
- `dotenvy` for environment loading
- `serde` and `serde_json` for local database parsing
- `reqwest` for HTTP support via dependencies
- `anyhow` for application-level error handling

## Data And Storage

- Local JSON database in `local_db.json`
- Environment-based configuration in `.env`
- No dedicated external database

## External Services

- Spotify Web API via OAuth
- Local browser-based auth callback flow

## Development Tooling

- Cargo for build, run, and dependency management
- Git for source control
- Existing Orchestrator command definitions under `.claude/commands`

## Current Architecture Notes

- Playlist tracks are fetched from Spotify
- Tracks are enriched from local metadata rather than Spotify audio features
- Matching relies on normalized titles, artist checks, and duration tolerance
- Sorting is based on Camelot wheel order and BPM
