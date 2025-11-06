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
    id: String,      // Spotify track ID (not URI)
    name: String,
    key: i32,        // 0–11 (Spotify pitch class)
    mode: Modality,       // 1 = major, 0 = minor
    tempo: f32,      // BPM
}

fn sort_tuple(key: i32, mode: Modality) -> (i32, i32) {
    let mode_val = match mode {
        Modality::Minor => 0,
        Modality::Major => 1,
        _ => 2, // just in case Spotify adds something new
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
                    all_tracks.push(TrackInfo {
                        id: id.id().to_string(), // store plain track ID
                        name: track.name.clone(),
                        key: -1,                         // placeholder until features
                        mode: Modality::Major,           // ✅ placeholder enum value
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

    // 5. Fetch audio features in batches of 100
    for chunk in all_tracks.chunks_mut(100) {
        let ids: Vec<TrackId> = chunk
            .iter()
            .map(|t| TrackId::from_id(&t.id).expect("bad track id"))
            .collect();

        // tracks_features returns Option<Vec<AudioFeatures>>
        let features_opt_vec = spotify.tracks_features(ids).await?;
        let features_vec = match features_opt_vec {
            Some(v) => v,
            None => Vec::new(),
        };

        for (track_info, f) in chunk.iter_mut().zip(features_vec.into_iter()) {
            track_info.key = f.key;
            track_info.mode = f.mode;
            track_info.tempo = f.tempo;
        }
    }

    // 6. Sort by key (and mode) then BPM
    all_tracks.sort_by(|a, b| {
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

    // OPTIONAL: print the new order before updating Spotify
    for t in &all_tracks {
        println!("{:?} – {:.1} BPM – {}", sort_tuple(t.key, t.mode), t.tempo, t.name);
    }

    // 7. Replace playlist items with the new order
    let playable_items: Vec<PlayableId> = all_tracks
        .iter()
        .map(|t| {
            let tid = TrackId::from_id(&t.id).expect("bad track id");
            PlayableId::from(tid)
        })
        .collect();

    spotify
        .playlist_replace_items(playlist_id, playable_items)
        .await?;

    println!("Playlist reordered successfully.");
    Ok(())
}
