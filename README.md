# MIX_SORTER

Reorder Spotify playlists for DJ-friendly harmonic mixing using your own local BPM and Camelot data.

MIX_SORTER is a Rust CLI for one specific job:

1. read a Spotify playlist
2. match each track against `local_db.json` and any optional Exportify CSV overlays
3. enrich tracks with BPM and key from your local metadata
4. sort matched tracks by harmonic bucket and tempo
5. write the new order back to Spotify

It exists because Spotify's old audio-feature-driven workflow is no longer a solid foundation for this use case. `local_db.json` is the source of truth, and Exportify can act as an explicit CSV import layer when you want faster metadata coverage without hand-editing JSON.

## Quick Start

1. Create a Spotify developer app and set `RSPOTIFY_REDIRECT_URI` to `http://127.0.0.1:8888/callback`
2. Put your Spotify credentials in `.env`
3. Keep your curated metadata in `local_db.json`
4. Optionally export playlist metadata from Exportify and pass it with `--exportify-csv`
5. Run MIX_SORTER against a playlist URL, ID, or URI

Minimal example:

```bash
cargo run -- --exportify-csv exports/my_playlist.csv "https://open.spotify.com/playlist/2nOsiUa2nlXBGuDMjDIbDb"
```

## Why This Exists

If you already maintain your own track metadata, Spotify should be a delivery surface, not the authority.

MIX_SORTER gives you:

- local control over BPM and key
- predictable sorting behavior
- matching that tolerates title noise and artist formatting differences
- a practical way to prep Spotify playlists for mixing without depending on deprecated or unreliable Spotify metadata

## What It Actually Does

For a given playlist, MIX_SORTER:

- accepts a playlist ID, Spotify playlist URL, or Spotify playlist URI
- loads `local_db.json`
- optionally imports one or more Exportify CSV files passed with `--exportify-csv`
- authenticates with Spotify
- fetches every track in the target playlist
- attempts to match each track against your local metadata
- converts Camelot notation like `5A` or `10B` into sortable values
- sorts enriched tracks first
- appends unmatched or invalid-key tracks after the matched block
- updates the playlist order in place on Spotify

Important:

- this changes the order of the target playlist on Spotify
- unmatched tracks are kept, not dropped
- unmatched tracks remain at the end in their original relative order
- invalid Camelot values are reported and treated as unsorted tracks
- when `local_db.json` and Exportify are equally good matches, `local_db.json` wins

If you want to preserve the current order, duplicate the playlist first.

## Metadata Strategy

MIX_SORTER intentionally separates playlist transport from metadata authority.

- Spotify is used to authenticate, read the target playlist, and write the new order back
- `local_db.json` is the canonical metadata source when you already know the correct BPM/key values
- Exportify CSVs are optional overlays that help bootstrap coverage quickly from exported playlist metadata

Precedence:

1. `local_db.json`
2. Exportify CSV overlays passed on the command line
3. no fallback metadata source beyond what you explicitly provide

That priority is deliberate. Curated local data should beat imported convenience data when both match equally well.

## Requirements

- Rust toolchain
- a Spotify developer app
- a `.env` file with valid Spotify credentials
- a `local_db.json` file in the project root

## Spotify Setup

Create a Spotify developer app and configure a redirect URI.

Use the same value in both places:

- Spotify app dashboard
- your local `.env`

Recommended for this repo:

```text
http://127.0.0.1:8888/callback
```

Example `.env`:

```ini
RSPOTIFY_CLIENT_ID=your_client_id_here
RSPOTIFY_CLIENT_SECRET=your_client_secret_here
RSPOTIFY_REDIRECT_URI=http://127.0.0.1:8888/callback
```

## WSL / Windows Auth Note

This repo uses a manual paste auth flow on purpose.

On WSL, your Windows browser may fail to load the final callback page at `127.0.0.1:8888`. That is acceptable. The CLI does not require the browser to successfully connect to a local callback server.

The flow is:

1. MIX_SORTER prints a Spotify authorization URL.
2. You open it in your Windows browser.
3. Spotify redirects to a URL that looks like:

```text
http://127.0.0.1:8888/callback?code=...&state=...
```

4. Even if the browser shows "Unable to connect", copy the full redirected URL from the address bar.
5. Paste that full URL back into the CLI.
6. MIX_SORTER extracts the auth code and continues.

If the redirect URI in the Spotify dashboard does not exactly match `RSPOTIFY_REDIRECT_URI`, auth will fail.

## Local Database Format

`local_db.json` must be a JSON array of track records.

Example:

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
- `bpm`: BPM used for sorting
- `key_camelot`: Camelot key such as `5A`, `8B`, `10A`
- `duration_ms`: optional, but strongly recommended when multiple versions exist

## Exportify Integration

[Exportify](https://exportify.net/) can export Spotify playlists as CSV with fields including `Track Name`, `Artist Name(s)`, `Duration (ms)`, `Key`, `Mode`, and `Tempo`.

MIX_SORTER can import those CSVs directly at runtime:

```bash
cargo run -- --exportify-csv exports/my_playlist.csv "https://open.spotify.com/playlist/2nOsiUa2nlXBGuDMjDIbDb"
```

You can pass multiple CSVs:

```bash
cargo run -- --exportify-csv exports/a.csv --exportify-csv exports/b.csv 2nOsiUa2nlXBGuDMjDIbDb
```

Import behavior:

- Exportify rows are converted into MIX_SORTER's internal metadata shape
- Spotify numeric `Key` + `Mode` values are converted to Camelot automatically
- only rows with usable `Tempo`, `Key`, and `Mode` are imported
- the first listed artist from `Artist Name(s)` is used for matching
- `local_db.json` stays primary when there is a tie

Recommended workflow:

1. Export a playlist from Exportify
2. Run MIX_SORTER with `--exportify-csv` to get immediate coverage
3. Promote any corrected or trusted metadata back into `local_db.json`
4. Treat `local_db.json` as the long-term source of truth

## Evaluated Sources

Two external resources were evaluated explicitly for this project:

- `spicetify-dj-info`: useful as research, not as a direct dependency. It gets BPM/key through Spicetify desktop internals and Spotify client-internal endpoints, which are brittle and do not fit MIX_SORTER's Rust CLI architecture.
- `audioFlux`: useful if the product ever grows a real local-audio analysis pipeline, but not a fit for the current playlist-sorting workflow because it expects local audio files or audio arrays and adds a Python/C runtime stack.

This is why Exportify was integrated while the other two were rejected for direct use.

## Matching Logic

The matcher is intentionally simple and inspectable.

- titles are normalized before comparison
- exact artist matches score highest
- partial artist containment is allowed for common featuring variations
- exact normalized title matches get extra weight
- `duration_ms`, when present, helps distinguish originals from edits, remasters, or alternate versions
- if no exact-title bucket match works, the tool falls back to a fuzzy title+artist search over the whole local DB

Terminal markers:

- `[MATCH:LOCAL_DB]`: metadata from `local_db.json` was applied
- `[MATCH:EXPORTIFY]`: metadata imported from Exportify CSV was applied
- `[MISSING]`: no local metadata match found
- `[KEY ERROR]`: a matched entry had an invalid Camelot value

## Sorting Logic

Tracks with valid local metadata are sorted by:

1. Camelot wheel position
2. BPM ascending within the same harmonic bucket

Tracks without usable metadata are appended after the matched block.

This is a practical deterministic sort, not a full transition-planning engine.

## Usage

Run with a playlist URL:

```bash
cargo run -- "https://open.spotify.com/playlist/2nOsiUa2nlXBGuDMjDIbDb"
```

Run with a raw playlist ID:

```bash
cargo run -- 2nOsiUa2nlXBGuDMjDIbDb
```

Run with a Spotify URI:

```bash
cargo run -- "spotify:playlist:2nOsiUa2nlXBGuDMjDIbDb"
```

Run with Exportify CSV support:

```bash
cargo run -- --exportify-csv exports/my_playlist.csv "spotify:playlist:2nOsiUa2nlXBGuDMjDIbDb"
```

If you prefer the compiled binary:

```bash
cargo build --release
./target/release/spotify-key-bpm-sorter "https://open.spotify.com/playlist/2nOsiUa2nlXBGuDMjDIbDb"
```

Useful variants:

- local DB only:

```bash
cargo run -- 2nOsiUa2nlXBGuDMjDIbDb
```

- multiple Exportify overlays:

```bash
cargo run -- --exportify-csv exports/a.csv --exportify-csv exports/b.csv 2nOsiUa2nlXBGuDMjDIbDb
```

## Example Run

```text
$ cargo run -- "https://open.spotify.com/playlist/..."
Loading local_db.json...
Loaded 146 local metadata entries.
Importing Exportify CSV: exports/my_playlist.csv
Imported 52 Exportify metadata rows from exports/my_playlist.csv.
Metadata pool ready with 198 total entries.
Open this Spotify authorization URL in your browser:
https://accounts.spotify.com/authorize?...
Paste the full redirect URL after approving access:
Fetching playlist tracks...
Found 199 tracks in playlist.
Enriching data...
[MATCH:EXPORTIFY] Pop Smoke - Dior (metadata: Dior)
[MISSING] Drake - Jumpman
Enrichment summary: matched=173, unmatched=26, invalid_key=0
Updating Spotify playlist order...
Done! Check the playlist custom order in Spotify.
```

## Behavior and Safety

- playlist order is updated in place
- tracks are not removed from the playlist
- unmatched tracks are preserved
- the tool only sorts using data available in `local_db.json` plus any explicitly supplied Exportify CSVs
- better local metadata gives better results

If you care about keeping the current order, clone the playlist before running the tool.

## Troubleshooting

### Browser says "Unable to connect" on `127.0.0.1:8888`

That is usually fine on WSL.

Copy the full redirected URL from the browser address bar and paste it into the CLI anyway.

### Spotify auth keeps failing

Check all three:

- `RSPOTIFY_CLIENT_ID`
- `RSPOTIFY_CLIENT_SECRET`
- `RSPOTIFY_REDIRECT_URI`

Then confirm the redirect URI in the Spotify developer dashboard exactly matches the one in `.env`.

### Tracks are not matching

Check `local_db.json` and any Exportify CSV inputs for:

- title differences
- artist naming differences
- missing `duration_ms` on tracks with multiple versions
- invalid `key_camelot` values

If you are using Exportify CSVs, rows missing usable `Tempo`, `Key`, or `Mode` values are skipped and reported before authentication starts.

### The sort works, but the results are not musically ideal

That is expected if the underlying metadata is incomplete, inconsistent, or too coarse. MIX_SORTER is only as good as the local DB you feed it.

## Development

Recommended verification commands:

```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features
```

## Project Shape

- `src/main.rs`: CLI entrypoint, Spotify auth, matching, sorting, playlist update, tests
- `local_db.json`: local BPM/key metadata source of truth
- `Cargo.toml`: Rust crate definition and dependencies
- `orchestrator/decisions.md`: documented architecture and source-selection decisions

## Limits

- there is no UI
- matching is heuristic, not probabilistic
- sorting is based on Camelot bucket plus BPM, not advanced transition analysis
- the tool does not depend on unsupported Spotify desktop-internal endpoints such as the ones used by Spicetify extensions
- the tool does not analyze raw audio directly; libraries like `audioFlux` only become relevant if a future workflow introduces local audio files
- the current implementation is a single-file CLI, optimized for practical use rather than architecture ceremony

## License

MIT. See [LICENSE](LICENSE).
