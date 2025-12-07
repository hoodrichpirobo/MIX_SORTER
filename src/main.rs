use reqwest::Client;
use rspotify::{
    model::{Modality, PlayableId, PlayableItem, PlaylistId, TrackId},
    prelude::*,
    scopes, AuthCodeSpotify, Credentials, OAuth,
};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;

// --- Structs ---

#[derive(Debug, Clone)]
struct TrackInfo {
    id: String,
    name: String,
    artist: String,
    key: i32,        // Spotify Pitch Class (0-11)
    mode: Modality,  // Major / Minor
    tempo: f32,
}

#[derive(Debug, serde::Deserialize)]
struct LocalTrackData {
    name: String,
    artist: String,
    bpm: f32,
    key_camelot: String,
}

// --- Helper Functions ---

fn camelot_to_spotify(camelot: &str) -> Option<(i32, Modality)> {
    let clean = camelot.trim().to_uppercase();
    let mode = if clean.contains('A') { Modality::Minor } else { Modality::Major };
    let num_str: String = clean.chars().filter(|c| c.is_numeric()).collect();
    let num: i32 = num_str.parse().ok()?;

    let pitch = match (num, mode) {
        (1, Modality::Major) => 11, // B
        (1, Modality::Minor) => 8,  // G#
        (2, Modality::Major) => 6,  // F#
        (2, Modality::Minor) => 3,  // Eb
        (3, Modality::Major) => 1,  // Db
        (3, Modality::Minor) => 10, // Bb
        (4, Modality::Major) => 8,  // Ab
        (4, Modality::Minor) => 5,  // F
        (5, Modality::Major) => 3,  // Eb
        (5, Modality::Minor) => 0,  // C
        (6, Modality::Major) => 10, // Bb
        (6, Modality::Minor) => 7,  // G
        (7, Modality::Major) => 5,  // F
        (7, Modality::Minor) => 2,  // D
        (8, Modality::Major) => 0,  // C
        (8, Modality::Minor) => 9,  // A
        (9, Modality::Major) => 7,  // G
        (9, Modality::Minor) => 4,  // E
        (10, Modality::Major) => 2, // D
        (10, Modality::Minor) => 11,// B
        (11, Modality::Major) => 9, // A
        (11, Modality::Minor) => 6, // F#
        (12, Modality::Major) => 4, // E
        (12, Modality::Minor) => 1, // Db
        _ => return None,
    };
    Some((pitch, mode))
}

fn parse_api_key_string(key: &str) -> Option<(i32, Modality)> {
    let trimmed = key.trim();
    if trimmed.is_empty() { return None; }
    
    // Normalize logic omitted for brevity, assuming standard API responses or local DB
    // Simple parser for standard pitch notation
    let is_minor = key.to_ascii_lowercase().contains("minor") || key.contains('m');
    let mode = if is_minor { Modality::Minor } else { Modality::Major };
    
    // This is a simplified fallback. Ideally relying on local_db mostly.
    Some((0, mode)) // Placeholder if API parsing is needed strictly
}

async fn fetch_getsong_features(
    http: &Client,
    api_key: &str,
    title: &str,
    artist: &str,
) -> anyhow::Result<Option<(f32, i32, Modality)>> {
    let lookup = format!("song:{} artist:{}", title, artist);
    let resp = http.get("https://api.getsong.co/search/")
        .query(&[("api_key", api_key), ("type", "both"), ("lookup", &lookup), ("limit", "1")])
        .send().await?;

    if !resp.status().is_success() { return Ok(None); }
    let body = resp.text().await?;
    let v: Value = serde_json::from_str(&body).unwrap_or(Value::Null);

    // Simplified extraction for brevity
    let song = v.get("search").and_then(|s| s.as_array()).and_then(|arr| arr.get(0));
    if let Some(song) = song {
        let tempo = song.get("tempo").and_then(|t| t.as_f64()).map(|f| f as f32).unwrap_or(0.0);
        let key_str = song.get("key_of").and_then(|k| k.as_str()).unwrap_or("");
        
        // Very basic parsing for fallback
        if tempo > 0.0 && !key_str.is_empty() {
             if let Some((p, m)) = camelot_to_spotify(key_str) { // try treating as camelot first
                 return Ok(Some((tempo, p, m)));
             }
             // Fallback logic for "C Major" strings would go here
        }
    }
    Ok(None)
}

/// Reverse Lookup: Converts Spotify Pitch back to a sortable "Camelot Weight"
/// 1A=10, 1B=11, 2A=20, 2B=21 ... 12B=121.
fn get_sort_weight(pitch: i32, mode: Modality) -> i32 {
    // (CamelotNumber, is_major)
    let (num, is_major) = match (pitch, mode) {
        (11, Modality::Major) => (1, true),  // 1B
        (8, Modality::Minor)  => (1, false), // 1A
        (6, Modality::Major)  => (2, true),  // 2B
        (3, Modality::Minor)  => (2, false), // 2A
        (1, Modality::Major)  => (3, true),  // 3B
        (10, Modality::Minor) => (3, false), // 3A
        (8, Modality::Major)  => (4, true),  // 4B
        (5, Modality::Minor)  => (4, false), // 4A
        (3, Modality::Major)  => (5, true),  // 5B
        (0, Modality::Minor)  => (5, false), // 5A
        (10, Modality::Major) => (6, true),  // 6B
        (7, Modality::Minor)  => (6, false), // 6A
        (5, Modality::Major)  => (7, true),  // 7B
        (2, Modality::Minor)  => (7, false), // 7A
        (0, Modality::Major)  => (8, true),  // 8B
        (9, Modality::Minor)  => (8, false), // 8A
        (7, Modality::Major)  => (9, true),  // 9B
        (4, Modality::Minor)  => (9, false), // 9A
        (2, Modality::Major)  => (10, true), // 10B
        (11, Modality::Minor) => (10, false),// 10A
        (9, Modality::Major)  => (11, true), // 11B
        (6, Modality::Minor)  => (11, false),// 11A
        (4, Modality::Major)  => (12, true), // 12B
        (1, Modality::Minor)  => (12, false),// 12A
        _ => (99, false), // Unknown
    };

    // Calculate weight: 1A -> 10, 1B -> 11
    (num * 10) + if is_major { 1 } else { 0 }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // 1. Load Local DB
    println!("Loading local_db.json...");
    let file_content = fs::read_to_string("local_db.json").unwrap_or_else(|_| "[]".to_string());
    let local_entries: Vec<LocalTrackData> = serde_json::from_str(&file_content).unwrap_or_default();
    let mut local_db: HashMap<String, (f32, String)> = HashMap::new();
    for entry in local_entries {
        let key = format!("{}-{}", entry.artist.to_lowercase(), entry.name.to_lowercase());
        local_db.insert(key, (entry.bpm, entry.key_camelot));
    }
    println!("Loaded {} entries from local DB.", local_db.len());

    // 2. Auth
    let creds = Credentials::from_env().expect("RSPOTIFY_CLIENT_ID / SECRET missing");
    let oauth = OAuth::from_env(scopes!("playlist-modify-private", "playlist-modify-public"))
        .expect("RSPOTIFY_REDIRECT_URI missing");
    let spotify = AuthCodeSpotify::new(creds, oauth);
    let authorize_url = spotify.get_authorize_url(false)?;
    spotify.prompt_for_token(&authorize_url).await?;

    // 3. Playlist ID
    let playlist_arg = env::args().nth(1).expect("Usage: cargo run -- <playlist_id>");
    let playlist_id = PlaylistId::from_id(&playlist_arg).expect("Invalid Playlist ID");

    // 4. Fetch Tracks
    let mut all_tracks: Vec<TrackInfo> = Vec::new();
    let mut offset: u32 = 0;
    println!("Fetching playlist tracks...");
    loop {
        let page = spotify.playlist_items_manual(playlist_id.clone(), None, None, Some(100), Some(offset)).await?;
        if page.items.is_empty() { break; }

        for item in page.items {
            if let Some(PlayableItem::Track(track)) = item.track {
                if let Some(id) = track.id {
                    let artist_name = track.artists.get(0).map(|a| a.name.clone()).unwrap_or("Unknown".to_string());
                    all_tracks.push(TrackInfo {
                        id: id.id().to_string(),
                        name: track.name.clone(),
                        artist: artist_name,
                        key: -1,
                        mode: Modality::Major,
                        tempo: 0.0,
                    });
                }
            }
        }
        offset += 100;
        if page.next.is_none() { break; }
    }
    println!("Found {} tracks.", all_tracks.len());

    // 5. Enrich Data
    let http = Client::new();
    let gs_api_key = env::var("GETSONGBPM_API_KEY").unwrap_or_default();

    for track in &mut all_tracks {
        let lookup_key = format!("{}-{}", track.artist.to_lowercase(), track.name.to_lowercase());

        if let Some((bpm, camelot)) = local_db.get(&lookup_key) {
            if let Some((pitch, mode)) = camelot_to_spotify(camelot) {
                track.tempo = *bpm;
                track.key = pitch;
                track.mode = mode;
                println!("[\u{2713} LOCAL] {} - {}", track.artist, track.name);
                continue;
            }
        }
        
        // Skip API if missing key
        if !gs_api_key.is_empty() {
             // API call logic here (simplified for brevity)
        } else {
             eprintln!("[ X MISS ] {} - {}", track.artist, track.name);
        }
    }

    // 6. Sort
    let mut with_features: Vec<TrackInfo> = Vec::new();
    let mut without_features: Vec<TrackInfo> = Vec::new();

    for t in all_tracks.into_iter() {
        if t.key >= 0 && t.tempo > 0.0 { with_features.push(t); } else { without_features.push(t); }
    }

    // --- KEY CHANGE: SORT BY CAMELOT WEIGHT ---
    with_features.sort_by(|a, b| {
        let a_weight = get_sort_weight(a.key, a.mode);
        let b_weight = get_sort_weight(b.key, b.mode);
        
        a_weight.cmp(&b_weight)
            .then(a.tempo.partial_cmp(&b.tempo).unwrap_or(std::cmp::Ordering::Equal))
    });

    with_features.extend(without_features);
    let sorted_tracks = with_features;

    // 7. Update Spotify
    let playable_items: Vec<PlayableId> = sorted_tracks.iter()
        .map(|t| PlayableId::from(TrackId::from_id(&t.id).unwrap()))
        .collect();

    println!("Updating Spotify playlist order...");
    let mut chunks = playable_items.chunks(100);
    if let Some(first) = chunks.next() {
        spotify.playlist_replace_items(playlist_id.clone(), first.to_vec()).await?;
    }
    for chunk in chunks {
        spotify.playlist_add_items(playlist_id.clone(), chunk.to_vec(), None).await?;
    }

    println!("Done! Check 'Custom Order' in Spotify.");
    Ok(())
}
