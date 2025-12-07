use rspotify::{
    model::{Modality, PlayableId, PlayableItem, PlaylistId, TrackId},
    prelude::*,
    scopes, AuthCodeSpotify, Credentials, OAuth,
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::cmp::Ordering;

// --- Structs ---

#[derive(Debug, Clone)]
struct TrackInfo {
    id: String,
    name: String,
    artist: String,
    key: i32,
    mode: Modality,
    tempo: f32,
    duration_ms: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct LocalTrackData {
    name: String,
    artist: String,
    bpm: f32,
    key_camelot: String,
    #[serde(default)]
    duration_ms: Option<u32>, 
    #[serde(default)]
    album: Option<String>,
}

// --- Helper Functions ---

fn normalize(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .replace("’", "'")
        .replace("`", "'")
        .replace("“", "\"")
        .replace("”", "\"")
        .replace("-", " ") 
        .replace("  ", " ")
}

fn camelot_to_spotify(camelot: &str) -> Option<(i32, Modality)> {
    let clean = camelot.trim().to_uppercase();
    let mode = if clean.contains('A') { Modality::Minor } else { Modality::Major };
    let num_str: String = clean.chars().filter(|c| c.is_numeric()).collect();
    let num: i32 = num_str.parse().ok()?;

    let pitch = match (num, mode) {
        (1, Modality::Major) => 11, (1, Modality::Minor) => 8,
        (2, Modality::Major) => 6,  (2, Modality::Minor) => 3,
        (3, Modality::Major) => 1,  (3, Modality::Minor) => 10,
        (4, Modality::Major) => 8,  (4, Modality::Minor) => 5,
        (5, Modality::Major) => 3,  (5, Modality::Minor) => 0,
        (6, Modality::Major) => 10, (6, Modality::Minor) => 7,
        (7, Modality::Major) => 5,  (7, Modality::Minor) => 2,
        (8, Modality::Major) => 0,  (8, Modality::Minor) => 9,
        (9, Modality::Major) => 7,  (9, Modality::Minor) => 4,
        (10, Modality::Major) => 2, (10, Modality::Minor) => 11,
        (11, Modality::Major) => 9, (11, Modality::Minor) => 6,
        (12, Modality::Major) => 4, (12, Modality::Minor) => 1,
        _ => return None,
    };
    Some((pitch, mode))
}

fn get_sort_weight(pitch: i32, mode: Modality) -> i32 {
    let (num, is_major) = match (pitch, mode) {
        (11, Modality::Major) => (1, true), (8, Modality::Minor) => (1, false),
        (6, Modality::Major) => (2, true), (3, Modality::Minor) => (2, false),
        (1, Modality::Major) => (3, true), (10, Modality::Minor) => (3, false),
        (8, Modality::Major) => (4, true), (5, Modality::Minor) => (4, false),
        (3, Modality::Major) => (5, true), (0, Modality::Minor) => (5, false),
        (10, Modality::Major) => (6, true), (7, Modality::Minor) => (6, false),
        (5, Modality::Major) => (7, true), (2, Modality::Minor) => (7, false),
        (0, Modality::Major) => (8, true), (9, Modality::Minor) => (8, false),
        (7, Modality::Major) => (9, true), (4, Modality::Minor) => (9, false),
        (2, Modality::Major) => (10, true), (11, Modality::Minor) => (10, false),
        (9, Modality::Major) => (11, true), (6, Modality::Minor) => (11, false),
        (4, Modality::Major) => (12, true), (1, Modality::Minor) => (12, false),
        _ => (99, false),
    };
    (num * 10) + if is_major { 1 } else { 0 }
}

/// Updated: Returns an OWNED `LocalTrackData` (cloned) to fix lifetime issues.
fn find_best_match(
    spotify_track: &TrackInfo, 
    candidates: &[LocalTrackData]
) -> Option<LocalTrackData> {
    if candidates.is_empty() { return None; }

    let mut best_candidate: Option<&LocalTrackData> = None;
    let mut highest_score = 0;

    let spot_artist_norm = normalize(&spotify_track.artist);
    
    for candidate in candidates {
        let mut score = 0;
        let db_artist_norm = normalize(&candidate.artist);

        // Artist Check
        if db_artist_norm == spot_artist_norm {
            score += 100;
        } else if db_artist_norm.contains(&spot_artist_norm) || spot_artist_norm.contains(&db_artist_norm) {
            score += 80;
        } else {
            // Check strict artist mismatch but allow "feat" differences
            // If completely different strings, skip unless it's a very strong title match context?
            // For now, small penalty so we can still match "Song (Live)" against "Song"
            continue; 
        }

        // Duration Check
        if let Some(db_dur) = candidate.duration_ms {
            let diff = (db_dur as i64 - spotify_track.duration_ms as i64).abs();
            if diff < 5000 { score += 50; } else { score -= 50; }
        }

        // Exact Name Boost
        if normalize(&candidate.name) == normalize(&spotify_track.name) {
            score += 20;
        }

        if score > highest_score {
            highest_score = score;
            best_candidate = Some(candidate);
        }
    }

    // Return a clone so the data is owned and not tied to the `candidates` lifetime
    best_candidate.cloned()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    println!("Loading local_db.json...");
    let file_content = fs::read_to_string("local_db.json").unwrap_or_else(|_| "[]".to_string());
    let local_entries: Vec<LocalTrackData> = serde_json::from_str(&file_content).unwrap_or_default();
    
    // Key: Normalized Title | Value: List of Candidates
    let mut local_db: HashMap<String, Vec<LocalTrackData>> = HashMap::new();
    
    for entry in local_entries {
        let key = normalize(&entry.name);
        local_db.entry(key).or_default().push(entry);
    }
    
    // Flattened list for fuzzy fallback
    let all_db_entries: Vec<LocalTrackData> = local_db.values().flatten().cloned().collect();

    println!("Loaded {} total entries.", all_db_entries.len());

    // Spotify Auth
    let creds = Credentials::from_env().expect("RSPOTIFY_CLIENT_ID / SECRET missing");
    let oauth = OAuth::from_env(scopes!("playlist-modify-private", "playlist-modify-public"))
        .expect("RSPOTIFY_REDIRECT_URI missing");
    let spotify = AuthCodeSpotify::new(creds, oauth);
    let authorize_url = spotify.get_authorize_url(false)?;
    spotify.prompt_for_token(&authorize_url).await?;

    let playlist_arg = env::args().nth(1).expect("Usage: cargo run -- <playlist_id>");
    let playlist_id = PlaylistId::from_id(&playlist_arg).expect("Invalid Playlist ID");

    // Fetch Tracks
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
                    
                    // Uses num_milliseconds for rspotify's Duration (chrono::TimeDelta)
                    let duration = track.duration.num_milliseconds() as u32;

                    all_tracks.push(TrackInfo {
                        id: id.id().to_string(),
                        name: track.name.clone(),
                        artist: artist_name,
                        key: -1,
                        mode: Modality::Major,
                        tempo: 0.0,
                        duration_ms: duration,
                    });
                }
            }
        }
        offset += 100;
        if page.next.is_none() { break; }
    }
    println!("Found {} tracks in playlist.", all_tracks.len());

    // Enrich Data
    println!("Enriching data...");
    let mut enriched_count = 0;
    
    for track in &mut all_tracks {
        let title_key = normalize(&track.name);
        
        // Changed to owned Option<LocalTrackData> to fix lifetime issues
        let mut best_match: Option<LocalTrackData> = None;

        // 1. Try Direct Lookup
        if let Some(candidates) = local_db.get(&title_key) {
            best_match = find_best_match(track, candidates);
        }

        // 2. Fallback: Fuzzy Search
        if best_match.is_none() {
            // Filter a temporary list of candidates based on fuzzy name matching
            let fuzzy_candidates: Vec<LocalTrackData> = all_db_entries.iter()
                .filter(|db_item| {
                    let db_norm = normalize(&db_item.name);
                    let spot_norm = normalize(&track.name);
                    
                    // Title Check: "Song - Live" contains "Song"
                    let title_match = db_norm.contains(&spot_norm) || spot_norm.contains(&db_norm);
                    
                    // Artist Check: Pre-filter to avoid checking thousands of irrelevant tracks
                    let db_artist = normalize(&db_item.artist);
                    let spot_artist = normalize(&track.artist);
                    let artist_match = db_artist.contains(&spot_artist) || spot_artist.contains(&db_artist);

                    title_match && artist_match
                })
                .cloned()
                .collect();
            
            // Now pass this temp vector. find_best_match returns a CLONE, so we are safe.
            best_match = find_best_match(track, &fuzzy_candidates);
        }

        if let Some(match_data) = best_match {
            if let Some((pitch, mode)) = camelot_to_spotify(&match_data.key_camelot) {
                track.tempo = match_data.bpm;
                track.key = pitch;
                track.mode = mode;
                enriched_count += 1;
                println!("[\u{2713} MATCH] {} - {} (DB: {})", track.artist, track.name, match_data.name);
            } else {
                eprintln!("[! KEY ERR] Invalid Camelot '{}' for {}", match_data.key_camelot, track.name);
            }
        } else {
            eprintln!("[X MISSING] {} - {}", track.artist, track.name);
        }
    }
    println!("Successfully enriched {}/{} tracks.", enriched_count, all_tracks.len());

    // Sort
    let mut with_features: Vec<TrackInfo> = Vec::new();
    let mut without_features: Vec<TrackInfo> = Vec::new();

    for t in all_tracks.into_iter() {
        if t.key >= 0 && t.tempo > 0.0 { with_features.push(t); } else { without_features.push(t); }
    }

    with_features.sort_by(|a, b| {
        let a_weight = get_sort_weight(a.key, a.mode);
        let b_weight = get_sort_weight(b.key, b.mode);
        a_weight.cmp(&b_weight).then(a.tempo.partial_cmp(&b.tempo).unwrap_or(Ordering::Equal))
    });

    with_features.extend(without_features);
    let sorted_tracks = with_features;

    // Update Spotify
    let playable_items: Vec<PlayableId> = sorted_tracks.iter()
        .map(|t| PlayableId::from(TrackId::from_id(&t.id).unwrap()))
        .collect();

    if !playable_items.is_empty() {
        println!("Updating Spotify playlist order...");
        let mut chunks = playable_items.chunks(100);
        if let Some(first) = chunks.next() {
            spotify.playlist_replace_items(playlist_id.clone(), first.to_vec()).await?;
        }
        for chunk in chunks {
            spotify.playlist_add_items(playlist_id.clone(), chunk.to_vec(), None).await?;
        }
        println!("Done! Check 'Custom Order' in Spotify.");
    } else {
        println!("No tracks to update.");
    }

    Ok(())
}
