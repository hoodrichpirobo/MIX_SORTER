use serde_json::Value;
use reqwest::Client;
use rspotify::{
    prelude::*,
    scopes,
    AuthCodeSpotify,
    Credentials,
    OAuth,
    model::{PlaylistId, PlayableItem, TrackId, PlayableId},
};
use std::env;

use rspotify::model::Modality;

#[derive(Debug, Clone)]
struct TrackInfo {
    id: String,      // Spotify track ID
    name: String,    // track title
    artist: String,  // main artist name
    key: i32,        // 0–11
    mode: Modality,  // Major / Minor
    tempo: f32,      // BPM
}

fn parse_getsong_key(key: &str) -> Option<(i32, Modality)> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Normalize Unicode sharps/flats to ASCII # / b
    let mut ascii = String::new();
    for ch in trimmed.chars() {
        match ch {
            '♯' => ascii.push('#'),
            '♭' => ascii.push('b'),
            _ => ascii.push(ch),
        }
    }

    let lower = ascii.to_ascii_lowercase();
    // Very simple: if it contains 'm' and not 'maj', call it minor
    let is_minor = lower.contains('m') && !lower.contains("maj");

    // Root = first letter + optional #/b
    let mut chars = ascii.chars();
    let first = chars.next()?; // note the ?
    let second = chars.next();

    let mut root = String::new();
    root.push(first.to_ascii_uppercase());
    if let Some(c2) = second {
        if c2 == '#' || c2 == 'b' || c2 == 'B' {
            root.push(c2.to_ascii_uppercase());
        }
    }

    let pitch = match root.as_str() {
        "C"       => 0,
        "C#"|"DB" => 1,
        "D"       => 2,
        "D#"|"EB" => 3,
        "E"       => 4,
        "F"       => 5,
        "F#"|"GB" => 6,
        "G"       => 7,
        "G#"|"AB" => 8,
        "A"       => 9,
        "A#"|"BB" => 10,
        "B"       => 11,
        _ => return None,
    };

    let mode = if is_minor { Modality::Minor } else { Modality::Major };
    Some((pitch, mode))
}

async fn fetch_getsong_features(
    http: &Client,
    api_key: &str,
    title: &str,
    artist: &str,
) -> anyhow::Result<Option<(f32, i32, Modality)>> {
    // Use the format recommended in the docs:
    // type=both + lookup="song:<title> artist:<artist>"
    let lookup = format!("song:{} artist:{}", title, artist);

    let resp = http
        .get("https://api.getsong.co/search/")
        .query(&[
            ("api_key", api_key),
            ("type", "both"),
            ("lookup", &lookup),
            ("limit", "1"),
        ])
        .send()
        .await?;

    // Don’t crash on 4xx/5xx: just log and skip this track
    let status = resp.status();
    if !status.is_success() {
        eprintln!(
            "GetSongBPM HTTP {} for '{} - {}'",
            status, artist, title
        );
        return Ok(None);
    }

    let body = resp.text().await?;

    let v: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "GetSongBPM: JSON parse error for '{} - {}': {}. Body: {}",
                artist, title, e, body
            );
            return Ok(None);
        }
    };

    // Normal success case: { "search": [ { song1 }, { song2 }, ... ] }
    let search_val = match v.get("search") {
        Some(s) => s,
        None => {
            eprintln!(
                "GetSongBPM: no 'search' field for '{} - {}': {}",
                artist, title, v
            );
            return Ok(None);
        }
    };

    // Error case you are seeing: { "search": { "error": "no result" } }
    if let Some(obj) = search_val.as_object() {
        if obj.get("error").is_some() {
            // Explicit "no result" -> just skip quietly
            return Ok(None);
        }
    }

    let items = match search_val.as_array() {
        Some(arr) if !arr.is_empty() => arr,
        _ => return Ok(None),
    };

    let song = &items[0];

    // tempo can be number or string
    let tempo = match song.get("tempo") {
        Some(t) => {
            if let Some(n) = t.as_f64() {
                n as f32
            } else if let Some(s) = t.as_str() {
                match s.parse::<f32>() {
                    Ok(n) => n,
                    Err(_) => {
                        eprintln!(
                            "GetSongBPM: cannot parse tempo for '{} - {}': {:?}",
                            artist, title, t
                        );
                        return Ok(None);
                    }
                }
            } else {
                eprintln!(
                    "GetSongBPM: unsupported tempo type for '{} - {}': {:?}",
                    artist, title, t
                );
                return Ok(None);
            }
        }
        None => return Ok(None),
    };

    let key_str = match song.get("key_of").and_then(|k| k.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(None),
    };

    let (pitch, mode) = match parse_getsong_key(key_str) {
        Some(pm) => pm,
        None => {
            eprintln!(
                "GetSongBPM: cannot parse key_of '{}' for '{} - {}'",
                key_str, artist, title
            );
            return Ok(None);
        }
    };

    Ok(Some((tempo, pitch, mode)))
}

fn sort_tuple(key: i32, mode: Modality) -> (i32, i32) {
    let mode_val = match mode {
        Modality::Minor => 0,
        Modality::Major => 1,
        _ => 2,
    };
    (key, mode_val)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // 1. Read credentials & OAuth config from env/.env
    let creds = Credentials::from_env()
        .expect("RSPOTIFY_CLIENT_ID / SECRET missing");
    let oauth = OAuth::from_env(scopes!(
        "playlist-read-private",
        "playlist-modify-private",
        "playlist-modify-public"
    ))
    .expect("RSPOTIFY_REDIRECT_URI missing");

    let spotify = AuthCodeSpotify::new(creds, oauth);

    // 2. Run the Authorization Code flow in the CLI
    let authorize_url = spotify.get_authorize_url(false)?;
    spotify.prompt_for_token(&authorize_url).await?;

    // 3. Read playlist ID from CLI arg
    let playlist_arg = env::args().nth(1)
        .expect("Usage: spotify-key-bpm-sorter <playlist_url_or_id>");

    let playlist_id = PlaylistId::from_uri(&playlist_arg)
        .or_else(|_| PlaylistId::from_id(&playlist_arg))
        .expect("Could not parse playlist id / url");

    // 4. Collect all tracks in the playlist
    let mut all_tracks: Vec<TrackInfo> = Vec::new();

    // Using manual pagination. Limit 100 items per page.
    let mut offset: u32 = 0;
    loop {
        let page = spotify
            .playlist_items_manual(
                playlist_id.clone(),
                None,          // fields
                None,          // market
                Some(100),     // limit
                Some(offset),  // offset
            )
            .await?;

        if page.items.is_empty() {
            break;
        }

        for item in page.items {
            if let Some(PlayableItem::Track(track)) = item.track {
                if let Some(id) = track.id {
                    let artist_name = track
                        .artists
                        .get(0)
                        .map(|a| a.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());

                    all_tracks.push(TrackInfo {
                        id: id.id().to_string(),
                        name: track.name.clone(),
                        artist: artist_name,
                        key: -1,
                        mode: Modality::Major, // placeholder, will be overwritten
                        tempo: 0.0,
                    });
                }
            }
        }

        offset += 100;
        if page.next.is_none() {
            break;
        }
    }

    println!("Found {} tracks", all_tracks.len());

    let http = Client::new();
    let gs_api_key = std::env::var("GETSONGBPM_API_KEY")
        .expect("GETSONGBPM_API_KEY not set in env");

    for track in &mut all_tracks {
        // Try GetSongBPM; if it fails, leave defaults and skip
        match fetch_getsong_features(&http, &gs_api_key, &track.name, &track.artist).await? {
            Some((tempo, pitch, mode)) => {
                track.tempo = tempo;
                track.key = pitch;
                track.mode = mode;
            }
            None => {
                eprintln!("GetSongBPM: no match for '{}' - '{}'", track.artist, track.name);
                // leave tempo/key/mode as defaults
            }
        }
    }

    // 6. Split tracks: ones with features vs unknown, then sort only the known ones
    let mut with_features: Vec<TrackInfo> = Vec::new();
    let mut without_features: Vec<TrackInfo> = Vec::new();

    for t in all_tracks.into_iter() {
        if t.key >= 0 && t.tempo > 0.0 {
            with_features.push(t);
        } else {
            without_features.push(t);
        }
    }

    // Sort the tracks that actually have key/BPM
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

    // Put tracks without features at the end, preserving their relative order
    with_features.extend(without_features);
    let sorted_tracks = with_features;

    // OPTIONAL: print the new order before updating Spotify
    for t in &sorted_tracks {
        println!("{:?} – {:.1} BPM – {}", sort_tuple(t.key, t.mode), t.tempo, t.name);
    }

    // 7. Replace playlist items with the new order (Spotify max 100 items per request)
    let playable_items: Vec<PlayableId> = sorted_tracks
        .iter()
        .map(|t| {
            let tid = TrackId::from_id(&t.id).expect("bad track id");
            PlayableId::from(tid)
        })
        .collect();

    // First chunk: replace playlist contents
    let mut chunks = playable_items.chunks(100);

    if let Some(first_chunk) = chunks.next() {
        spotify
            .playlist_replace_items(playlist_id.clone(), first_chunk.to_vec())
            .await?;
    }

    // Remaining chunks: append
    for chunk in chunks {
        spotify
            .playlist_add_items(playlist_id.clone(), chunk.to_vec(), None)
            .await?;
    }

    println!("Playlist reordered successfully.");
    Ok(())
}
