# MIX_SORTER 🎚️🎧

*Reorder your Spotify playlists by musical key and BPM — in one CLI command.*

> A small Rust tool that reshuffles a Spotify playlist into a smooth, DJ-friendly key & tempo progression, using the Spotify Web API plus external audio-feature data from [GetSongBPM.com](https://getsongbpm.com).

---

## ✨ What this does

* Connects to your **Spotify account** via OAuth.
* Reads all tracks from a given **playlist** (with pagination, so big playlists are fine).
* Looks up **key (major/minor)** and **tempo (BPM)** for each track via the GetSongBPM API.
* Sorts tracks by:

  1. **Key** (C→B, including sharps/flats)
  2. **Mode** (minor before major)
  3. **Tempo** (ascending BPM)
* Writes the **reordered playlist back to Spotify**, using official playlist endpoints (replace + add items, max 100 tracks per request).
* Tracks where no key/BPM could be found are kept, but pushed to the **end**, preserving their original order.

Perfect for:

* DJs who want playlists that **flow harmonically**
* People who like their playlists a bit more **musically organized**
* Rust enjoyers who want a concrete example of **Spotify Web API + external API + CLI** in one repo

---

## 🧠 Why not use Spotify’s own audio features?

Spotify used to expose key, mode, and tempo directly via its **Audio Features** endpoints (`/v1/audio-features`). However, as of late 2024 those endpoints are **no longer available to new apps** unless they had “extended access” granted beforehand.

Because of that, MIX_SORTER does **not** call `audio-features` at all. Instead, it:

* Pulls basic metadata (track name + main artist) from Spotify.
* Sends those to **GetSongBPM**.
* Uses GetSongBPM’s response to fill in `key`, `mode`, and `tempo`, then sorts.

This keeps MIX_SORTER working even after Spotify’s policy changes.

---

## 🏗 Tech stack

* **Language:** Rust
* **Spotify client:** [`rspotify`](https://crates.io/crates/rspotify)
* **HTTP client:** [`reqwest`](https://crates.io/crates/reqwest)
* **Config:** `.env` via [`dotenvy`](https://crates.io/crates/dotenvy)
* **Audio features:** [GetSongBPM.com](https://getsongbpm.com)

---

## ⚙️ How the sorting works

Each track is represented internally as:

```rust
struct TrackInfo {
    id: String,      // Spotify track ID
    name: String,    // track title
    artist: String,  // main artist
    key: i32,        // 0–11 (C=0, C#/Db=1, ..., B=11; -1 => unknown)
    mode: Modality,  // Major / Minor (Spotify-like enum)
    tempo: f32,      // BPM (0.0 => unknown)
}
```

1. **Key parsing**

   * GetSongBPM might return things like `"D#m"`, `"A♯"`, `"F♯m"`, etc.
   * These are normalized:

     * Unicode sharps/flats → `#` / `b`
     * Root note extracted (`C`, `C#`, `Db`, …)
     * Mapped to `0–11` (C=0, C#/Db=1, …, B=11)
   * Mode is inferred:

     * If the string contains `m` and doesn’t say `maj`, it’s treated as **minor**, otherwise **major**.

2. **Tempo**

   * `tempo` can come back as a number or a string; both are supported.
   * If parsing fails, tempo is left at `0.0` (unknown).

3. **Sorting**

   * Tracks with **known key & tempo** are sorted by:

     ```text
     1. (key, mode)  // key as 0–11, mode = minor(0) then major(1)
     2. tempo (ascending BPM)
     ```
   * Tracks with **missing features** (`key < 0` or `tempo <= 0.0`) are:

     * Not used in the sort
     * Appended to the **end** of the playlist
     * Kept in their **original order** so nothing feels totally random

---

## 🔐 What this app does *not* do

* It **does not delete** any playlists.
* It **does not modify tracks** themselves — it only changes **order** in a single playlist.
* It **does not store your tokens or credentials** in the repo; everything is read from environment variables at runtime.

Still, you should always treat it like any script that can change playlists:
**Test on a throwaway playlist first.**

---

## 🧩 Prerequisites

You’ll need:

1. **Rust toolchain** (via [`rustup`](https://www.rust-lang.org/))
2. A **Spotify Developer** account and a registered app (for client ID/secret).
3. A **GetSongBPM API key** from [GetSongBPM.com](https://getsongbpm.com) / [api.getsong.co](https://api.getsong.co).

---

## 🗂 Environment variables

Create a `.env` file in the project root (it is *not* committed):

```env
RSPOTIFY_CLIENT_ID=your_spotify_client_id
RSPOTIFY_CLIENT_SECRET=your_spotify_client_secret
RSPOTIFY_REDIRECT_URI=http://127.0.0.1:8888/callback

GETSONGBPM_API_KEY=your_getsongbpm_api_key
```

Notes:

* The redirect URI must match what you configure in the **Spotify Developer Dashboard**.
* The `GETSONGBPM_API_KEY` is passed to `https://api.getsong.co/search/` to retrieve tempo & key.

---

## 🚀 Installation

```bash
git clone https://github.com/<your-username>/MIX_SORTER.git
cd MIX_SORTER

# Build once to fetch dependencies
cargo build --release
```

Make sure your `.env` is in place before running.

---

## ▶️ Usage

Basic pattern:

```bash
cargo run -- <playlist_url_or_id>
```

Both of these work:

```bash
# Using a full URL:
cargo run -- https://open.spotify.com/playlist/14GYKMTN0v1zxqLMufanlE

# Using just the playlist ID:
cargo run -- 14GYKMTN0v1zxqLMufanlE
```

What happens when you run it:

1. A browser window opens asking you to **authorize** the app for:

   * `playlist-read-private`
   * `playlist-modify-private`
   * `playlist-modify-public`
2. MIX_SORTER:

   * Fetches all playlist tracks (100 at a time, until there are no more).
   * Queries GetSongBPM for each track’s tempo & key.
   * Sorts tracks according to the rules above.
   * Writes them back to your playlist using Spotify’s playlist update endpoints.
3. You’ll see a summary printed like:

   ```text
   (0, 0) – 73.0 BPM – Space Song
   (5, 1) – 112.0 BPM – Love/Paranoia
   (11, 0) – 104.0 BPM – Breathe Deeper
   ...
   Playlist reordered successfully.
   ```

If something fails (e.g., GetSongBPM has no data for a track, network hiccup, etc.), the tool logs a message and carries on, keeping that track in the unknown section at the end.

---

## 🧪 Development & hacking

A few pointers if you want to extend this:

* **Change the sorting logic**

  The core sort is currently:

  ```rust
  with_features.sort_by(|a, b| {
      let a_key = sort_tuple(a.key, a.mode);
      let b_key = sort_tuple(b.key, b.mode);

      a_key
          .cmp(&b_key)
          .then(
              a.tempo
                  .partial_cmp(&b.tempo)
                  .unwrap_or(std::cmp::Ordering::Equal),
          )
  });
  ```

  You could replace this with:

  * Camelot wheel ordering
  * Grouping by BPM ranges (e.g. warm-up vs peak-time)
  * Custom “energy level” rules

* **Swap out the audio-feature source**

  All the GetSongBPM logic lives in a single `fetch_getsong_features` function that uses `reqwest`. You can point that at any other service that returns key/BPM from track metadata.

* **Alternative modes**

  Ideas:

  * `--by-duration` – sort by track length instead of key/BPM.
  * `--dry-run` – show the new order without updating Spotify.
  * `--output-json` – dump the sorted order as JSON for inspection.

---

## 🧱 Project structure (high level)

* `main.rs`

  * Auth setup (`Credentials::from_env`, `OAuth::from_env`, Authorization Code flow).
  * Playlist reading with `playlist_items_manual` (handles pagination).
  * Track collection into `Vec<TrackInfo>`.
  * GetSongBPM feature lookup via `reqwest`.
  * Sorting logic & split into “with features” vs “without features”.
  * Playlist update (`playlist_replace_items` + `playlist_add_items` in chunks of 100).

---

## ⚠️ Disclaimer

* This is a **personal tool**, not affiliated with or endorsed by Spotify or GetSongBPM.
* APIs and policies can change; always check the latest Spotify Web API and GetSongBPM documentation if something stops working.

Use at your own risk — and always keep a backup or duplicate of playlists you really care about.

---

## 🙏 Acknowledgements

* [Spotify Web API](https://developer.spotify.com/documentation/web-api) for playlist access.
* [GetSongBPM.com](https://getsongbpm.com) / `api.getsong.co` for tempo & key data.
* [`rspotify`](https://crates.io/crates/rspotify) for making Spotify + Rust pleasant.
* Everyone sharing tricks & alternatives after Spotify’s audio-feature changes.

---

## 📜 License

MIT License – see LICENSE file for details.

