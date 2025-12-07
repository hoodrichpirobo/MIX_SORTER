Here is the **ultimate README.md** for `MIX_SORTER`.

I have updated it to reflect the **current architecture**: it now uses a **Local Database** (JSON) instead of external APIs, features **Advanced Fuzzy Matching**, and sorts by **Camelot Wheel**.

-----

# MIX\_SORTER 🎛️✨

**Harmonic mixing for Spotify playlists — powered by Rust & Local Data.**

> A high-performance CLI tool that reorders Spotify playlists by **Camelot Key** and **BPM**. Built to bypass Spotify's API restrictions by using a local database with advanced fuzzy matching logic.

-----

## 🔮 Why this exists

Spotify recently restricted access to their **Audio Features** API (Key, BPM, Energy), breaking many DJ tools.

**MIX\_SORTER** solves this by decoupling the data from the platform. instead of relying on a flaky or restricted API, it uses a **`local_db.json`** file as a source of truth. It matches your Spotify tracks against this database using a robust scoring algorithm to ensure the right metadata is applied, even if the titles differ slightly (e.g., *"Remastered 2009"* vs *"Original Mix"*).

-----

## ✨ Features

  * **⚡ Blazing Fast:** Written in Rust using `rspotify` and async/await.
  * **🧠 Smart Matching:** Uses normalization (accent removal, case insensitivity) and fuzzy logic to match Spotify tracks to your local database.
  * **🎚️ Harmonic Sorting:** Sorts tracks primarily by **Camelot Key** (1A → 1B → 2A...) for perfect harmonic mixing, and secondarily by **BPM**.
  * **🛡️ Conflict Resolution:** Verifies matches using **Track Duration** (±5s tolerance) and **Artist Name** containment to distinguish between remixes and originals.
  * **💾 Local Control:** You own your data. No API rate limits. No subscription fees.

-----

## 🛠️ Installation

### 1\. Prerequisites

  * **Rust Toolchain:** [Install Rust](https://www.rust-lang.org/tools/install)
  * **Spotify Developer App:** Create one at [developer.spotify.com](https://developer.spotify.com/dashboard).
      * Set Redirect URI to: `http://localhost:8888/callback`

### 2\. Clone & Configure

```bash
git clone https://github.com/your-username/MIX_SORTER.git
cd MIX_SORTER
```

Create a `.env` file in the project root:

```ini
RSPOTIFY_CLIENT_ID=your_client_id_here
RSPOTIFY_CLIENT_SECRET=your_client_secret_here
RSPOTIFY_REDIRECT_URI=http://localhost:8888/callback
```

### 3\. Prepare the Database

Ensure you have a `local_db.json` file in the root directory. It should look like this:

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

-----

## 🚀 Usage

Run the tool by passing a **Spotify Playlist ID** or **URL**:

```bash
# Using ID
cargo run -- 2nOsiUa2nlXBGuDMjDIbDb

# Using URL
cargo run -- "https://open.spotify.com/playlist/2nOsiUa2nlXBGuDMjDIbDb"
```

### What happens next?

1.  **Authentication:** A browser window opens. Log in to Spotify to authorize the app.
2.  **Fetching:** The app downloads your playlist tracks.
3.  **Enriching:** It scans your `local_db.json` to find metadata for every track using the fuzzy matcher.
4.  **Sorting:** Tracks are reordered:
      * **Primary:** Camelot Key (1A, 1B, 2A, 2B...)
      * **Secondary:** BPM (Ascending)
5.  **Updating:** The playlist on Spotify is instantly updated with the new order.

-----

## 🧠 How the Matching Works

Since Spotify titles often contain "fluff" (e.g., *"- 2011 Remaster"*, *"feat. X"*), strict string matching fails. MIX\_SORTER uses a **Weighted Scoring System**:

1.  **Normalization:** Converts "Maldición" -\> "maldicion", removes curly quotes, trims whitespace.
2.  **Lookup:** Finds potential candidates in the DB by normalized title.
3.  **Scoring:**
      * **+100 points:** Exact Artist match.
      * **+80 points:** Partial Artist match (e.g. "Drake" in "Drake, Future").
      * **+50 points:** Duration match (within 5 seconds).
      * **-50 points:** Duration mismatch (likely a different remix).
      * **+20 points:** Exact Title match (case-sensitive tie-breaker).

Only the candidate with the highest positive score is selected.

-----

## 📦 Project Structure

  * `src/main.rs`: The core logic.
      * `LocalTrackData`: Struct for JSON parsing.
      * `find_best_match`: The fuzzy matching engine.
      * `get_sort_weight`: Converts Camelot keys to sortable integers.
      * `camelot_to_spotify`: Maps Camelot strings (e.g., "5A") to Spotify Pitch/Modality.

-----

## 🤝 Contributing

Got a better sorting algorithm? Want to add a CLI flag to reverse the sort order?

1.  Fork it.
2.  Create your feature branch (`git checkout -b feature/AmazingFeature`).
3.  Commit your changes (`git commit -m 'Add AmazingFeature'`).
4.  Push to the branch (`git push origin feature/AmazingFeature`).
5.  Open a Pull Request.

-----

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.

-----

*“Music gives a soul to the universe, wings to the mind, flight to the imagination and life to everything.”*
