use anyhow::{anyhow, bail, Context, Result};
use csv::StringRecord;
use rspotify::{
    model::{Modality, PlayableId, PlayableItem, PlaylistId, TrackId},
    prelude::*,
    scopes, AuthCodeSpotify, Credentials, OAuth,
};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};

const LOCAL_DB_PATH: &str = "local_db.json";
const DURATION_MATCH_TOLERANCE_MS: i64 = 5_000;
const CAMELOT_KEYS: [&str; 24] = [
    "1A", "1B", "2A", "2B", "3A", "3B", "4A", "4B", "5A", "5B", "6A", "6B", "7A", "7B", "8A", "8B",
    "9A", "9B", "10A", "10B", "11A", "11B", "12A", "12B",
];

#[derive(Debug, Clone, PartialEq)]
struct TrackInfo {
    id: String,
    name: String,
    artist: String,
    key: Option<i32>,
    mode: Option<Modality>,
    tempo: Option<f32>,
    duration_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetadataSource {
    LocalDb,
    Exportify,
}

impl MetadataSource {
    fn label(self) -> &'static str {
        match self {
            Self::LocalDb => "LOCAL_DB",
            Self::Exportify => "EXPORTIFY",
        }
    }

    fn priority(self) -> u8 {
        match self {
            Self::LocalDb => 2,
            Self::Exportify => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MetadataEntry {
    name: String,
    artist: String,
    bpm: f32,
    key_camelot: String,
    duration_ms: Option<u32>,
    source: MetadataSource,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
struct LocalTrackData {
    name: String,
    artist: String,
    bpm: f32,
    key_camelot: String,
    #[serde(default)]
    duration_ms: Option<u32>,
}

#[derive(Debug, Clone)]
struct MetadataIndex {
    by_title: HashMap<String, Vec<MetadataEntry>>,
    all_entries: Vec<MetadataEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliArgs {
    binary_name: String,
    playlist_input: String,
    exportify_csv_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct ExportifyColumns {
    track_name: usize,
    artist_names: usize,
    duration_ms: usize,
    key: usize,
    mode: usize,
    tempo: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ExportifySkipStats {
    missing_tempo: usize,
    missing_key_mode: usize,
    invalid_tempo: usize,
    invalid_key_mode: usize,
}

impl ExportifySkipStats {
    fn total(self) -> usize {
        self.missing_tempo + self.missing_key_mode + self.invalid_tempo + self.invalid_key_mode
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ExportifyImport {
    entries: Vec<MetadataEntry>,
    skipped: ExportifySkipStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExportifySkipReason {
    MissingTempo,
    MissingKeyMode,
    InvalidTempo,
    InvalidKeyMode,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct EnrichmentStats {
    matched: usize,
    unmatched: usize,
    invalid_key: usize,
}

fn normalize(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .replace(['’', '`'], "'")
        .replace(['“', '”'], "\"")
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn camelot_to_spotify(camelot: &str) -> Option<(i32, Modality)> {
    let clean = camelot.trim().to_uppercase();
    let suffix = clean.chars().last()?;
    let mode = match suffix {
        'A' => Modality::Minor,
        'B' => Modality::Major,
        _ => return None,
    };

    let num: i32 = clean[..clean.len().saturating_sub(1)].parse().ok()?;
    let pitch = match (num, mode) {
        (1, Modality::Major) => 11,
        (1, Modality::Minor) => 8,
        (2, Modality::Major) => 6,
        (2, Modality::Minor) => 3,
        (3, Modality::Major) => 1,
        (3, Modality::Minor) => 10,
        (4, Modality::Major) => 8,
        (4, Modality::Minor) => 5,
        (5, Modality::Major) => 3,
        (5, Modality::Minor) => 0,
        (6, Modality::Major) => 10,
        (6, Modality::Minor) => 7,
        (7, Modality::Major) => 5,
        (7, Modality::Minor) => 2,
        (8, Modality::Major) => 0,
        (8, Modality::Minor) => 9,
        (9, Modality::Major) => 7,
        (9, Modality::Minor) => 4,
        (10, Modality::Major) => 2,
        (10, Modality::Minor) => 11,
        (11, Modality::Major) => 9,
        (11, Modality::Minor) => 6,
        (12, Modality::Major) => 4,
        (12, Modality::Minor) => 1,
        _ => return None,
    };

    Some((pitch, mode))
}

fn spotify_to_camelot(pitch: i32, mode: Modality) -> Option<String> {
    CAMELOT_KEYS
        .iter()
        .find(|camelot| camelot_to_spotify(camelot) == Some((pitch, mode)))
        .map(|camelot| (*camelot).to_string())
}

fn get_sort_weight(pitch: i32, mode: Modality) -> i32 {
    let (num, is_major) = match (pitch, mode) {
        (11, Modality::Major) => (1, true),
        (8, Modality::Minor) => (1, false),
        (6, Modality::Major) => (2, true),
        (3, Modality::Minor) => (2, false),
        (1, Modality::Major) => (3, true),
        (10, Modality::Minor) => (3, false),
        (8, Modality::Major) => (4, true),
        (5, Modality::Minor) => (4, false),
        (3, Modality::Major) => (5, true),
        (0, Modality::Minor) => (5, false),
        (10, Modality::Major) => (6, true),
        (7, Modality::Minor) => (6, false),
        (5, Modality::Major) => (7, true),
        (2, Modality::Minor) => (7, false),
        (0, Modality::Major) => (8, true),
        (9, Modality::Minor) => (8, false),
        (7, Modality::Major) => (9, true),
        (4, Modality::Minor) => (9, false),
        (2, Modality::Major) => (10, true),
        (11, Modality::Minor) => (10, false),
        (9, Modality::Major) => (11, true),
        (6, Modality::Minor) => (11, false),
        (4, Modality::Major) => (12, true),
        (1, Modality::Minor) => (12, false),
        _ => (99, false),
    };

    (num * 10) + if is_major { 1 } else { 0 }
}

fn usage(binary_name: &str) -> String {
    format!(
        "Usage: {binary_name} [--exportify-csv <path>]... <playlist_id_or_url>\n\nExamples:\n  {binary_name} 2nOsiUa2nlXBGuDMjDIbDb\n  {binary_name} https://open.spotify.com/playlist/2nOsiUa2nlXBGuDMjDIbDb\n  {binary_name} --exportify-csv exports/playlist.csv spotify:playlist:2nOsiUa2nlXBGuDMjDIbDb"
    )
}

fn parse_args_from<I>(args: I) -> Result<CliArgs>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let binary_name = args
        .next()
        .unwrap_or_else(|| "spotify-key-bpm-sorter".to_string());
    let mut playlist_input = None;
    let mut exportify_csv_paths = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => bail!("{}", usage(&binary_name)),
            "--exportify-csv" => {
                let path = args.next().ok_or_else(|| {
                    anyhow!(
                        "missing value for --exportify-csv\n\n{}",
                        usage(&binary_name)
                    )
                })?;
                exportify_csv_paths.push(path);
            }
            _ => {
                if let Some(path) = arg.strip_prefix("--exportify-csv=") {
                    if path.is_empty() {
                        bail!(
                            "missing value for --exportify-csv\n\n{}",
                            usage(&binary_name)
                        );
                    }
                    exportify_csv_paths.push(path.to_string());
                } else if arg.starts_with('-') {
                    bail!("unknown option: {arg}\n\n{}", usage(&binary_name));
                } else if playlist_input.replace(arg).is_some() {
                    bail!("{}", usage(&binary_name));
                }
            }
        }
    }

    let playlist_input = playlist_input.ok_or_else(|| anyhow!(usage(&binary_name)))?;

    Ok(CliArgs {
        binary_name,
        playlist_input,
        exportify_csv_paths,
    })
}

fn extract_playlist_id(raw_input: &str) -> Result<String> {
    let trimmed = raw_input.trim();

    if trimmed.is_empty() {
        bail!("playlist input is empty");
    }

    if let Some(id) = trimmed.strip_prefix("spotify:playlist:") {
        return validate_playlist_id(id);
    }

    if let Some((_, rest)) = trimmed.split_once("open.spotify.com/playlist/") {
        let id = rest.split(['?', '/']).next().unwrap_or_default();
        return validate_playlist_id(id);
    }

    validate_playlist_id(trimmed)
}

fn validate_playlist_id(id: &str) -> Result<String> {
    PlaylistId::from_id(id)
        .map(|playlist_id| playlist_id.id().to_string())
        .map_err(|_| anyhow!("invalid Spotify playlist input: {id}"))
}

fn load_local_db_entries(path: &str) -> Result<Vec<MetadataEntry>> {
    let file_content =
        fs::read_to_string(path).with_context(|| format!("failed to read {path}"))?;
    let all_entries: Vec<LocalTrackData> =
        serde_json::from_str(&file_content).with_context(|| format!("failed to parse {path}"))?;

    Ok(all_entries
        .into_iter()
        .map(|entry| MetadataEntry {
            name: entry.name,
            artist: entry.artist,
            bpm: entry.bpm,
            key_camelot: entry.key_camelot,
            duration_ms: entry.duration_ms,
            source: MetadataSource::LocalDb,
        })
        .collect())
}

fn build_metadata_index(all_entries: Vec<MetadataEntry>) -> MetadataIndex {
    let mut by_title: HashMap<String, Vec<MetadataEntry>> = HashMap::new();
    for entry in &all_entries {
        by_title
            .entry(normalize(&entry.name))
            .or_default()
            .push(entry.clone());
    }

    MetadataIndex {
        by_title,
        all_entries,
    }
}

fn normalized_header(header: &str) -> String {
    header
        .trim()
        .trim_start_matches('\u{feff}')
        .to_ascii_lowercase()
}

fn find_header_index(headers: &StringRecord, expected: &[&str]) -> Option<usize> {
    headers.iter().position(|header| {
        let normalized = normalized_header(header);
        expected.iter().any(|candidate| normalized == *candidate)
    })
}

impl ExportifyColumns {
    fn from_headers(headers: &StringRecord) -> Result<Self> {
        let track_name = find_header_index(headers, &["track name"])
            .ok_or_else(|| anyhow!("Exportify CSV is missing the 'Track Name' column"))?;
        let artist_names = find_header_index(headers, &["artist name(s)"])
            .ok_or_else(|| anyhow!("Exportify CSV is missing the 'Artist Name(s)' column"))?;
        let duration_ms = find_header_index(headers, &["duration (ms)"])
            .ok_or_else(|| anyhow!("Exportify CSV is missing the 'Duration (ms)' column"))?;
        let key = find_header_index(headers, &["key"])
            .ok_or_else(|| anyhow!("Exportify CSV is missing the 'Key' column"))?;
        let mode = find_header_index(headers, &["mode", "mode (major or minor)"])
            .ok_or_else(|| anyhow!("Exportify CSV is missing the 'Mode' column"))?;
        let tempo = find_header_index(headers, &["tempo"])
            .ok_or_else(|| anyhow!("Exportify CSV is missing the 'Tempo' column"))?;

        Ok(Self {
            track_name,
            artist_names,
            duration_ms,
            key,
            mode,
            tempo,
        })
    }
}

fn parse_exportify_mode(raw_mode: &str) -> Option<Modality> {
    match raw_mode.trim().to_ascii_lowercase().as_str() {
        "1" | "major" => Some(Modality::Major),
        "0" | "minor" => Some(Modality::Minor),
        _ => None,
    }
}

fn first_exportify_artist(raw_artists: &str) -> String {
    raw_artists
        .split(';')
        .next()
        .unwrap_or(raw_artists)
        .trim()
        .to_string()
}

fn parse_exportify_record(
    record: &StringRecord,
    columns: ExportifyColumns,
) -> std::result::Result<MetadataEntry, ExportifySkipReason> {
    let tempo_raw = record.get(columns.tempo).unwrap_or_default().trim();
    if tempo_raw.is_empty() {
        return Err(ExportifySkipReason::MissingTempo);
    }

    let bpm = tempo_raw
        .parse::<f32>()
        .map_err(|_| ExportifySkipReason::InvalidTempo)?;

    let key_raw = record.get(columns.key).unwrap_or_default().trim();
    let mode_raw = record.get(columns.mode).unwrap_or_default().trim();
    if key_raw.is_empty() || mode_raw.is_empty() {
        return Err(ExportifySkipReason::MissingKeyMode);
    }

    let key = key_raw
        .parse::<i32>()
        .map_err(|_| ExportifySkipReason::InvalidKeyMode)?;
    let mode = parse_exportify_mode(mode_raw).ok_or(ExportifySkipReason::InvalidKeyMode)?;
    let key_camelot = spotify_to_camelot(key, mode).ok_or(ExportifySkipReason::InvalidKeyMode)?;

    let duration_ms = record
        .get(columns.duration_ms)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u32>().ok());

    Ok(MetadataEntry {
        name: record
            .get(columns.track_name)
            .unwrap_or_default()
            .trim()
            .to_string(),
        artist: first_exportify_artist(record.get(columns.artist_names).unwrap_or_default()),
        bpm,
        key_camelot,
        duration_ms,
        source: MetadataSource::Exportify,
    })
}

fn load_exportify_csv_reader<R: io::Read>(reader: R) -> Result<ExportifyImport> {
    let mut csv_reader = csv::ReaderBuilder::new().flexible(true).from_reader(reader);
    let headers = csv_reader.headers()?.clone();
    let columns = ExportifyColumns::from_headers(&headers)?;
    let mut import = ExportifyImport::default();

    for record in csv_reader.records() {
        let record = record?;
        match parse_exportify_record(&record, columns) {
            Ok(entry) => import.entries.push(entry),
            Err(ExportifySkipReason::MissingTempo) => import.skipped.missing_tempo += 1,
            Err(ExportifySkipReason::MissingKeyMode) => import.skipped.missing_key_mode += 1,
            Err(ExportifySkipReason::InvalidTempo) => import.skipped.invalid_tempo += 1,
            Err(ExportifySkipReason::InvalidKeyMode) => import.skipped.invalid_key_mode += 1,
        }
    }

    Ok(import)
}

fn load_exportify_csv(path: &str) -> Result<ExportifyImport> {
    let file = fs::File::open(path).with_context(|| format!("failed to read {path}"))?;
    load_exportify_csv_reader(file)
        .with_context(|| format!("failed to parse Exportify CSV at {path}"))
}

fn find_best_match(
    spotify_track: &TrackInfo,
    candidates: &[MetadataEntry],
) -> Option<MetadataEntry> {
    let mut best_candidate: Option<&MetadataEntry> = None;
    let mut best_rank = (i32::MIN, 0_u8);
    let spot_artist_norm = normalize(&spotify_track.artist);
    let spot_title_norm = normalize(&spotify_track.name);

    for candidate in candidates {
        let mut score = 0;
        let db_artist_norm = normalize(&candidate.artist);

        if db_artist_norm == spot_artist_norm {
            score += 100;
        } else if db_artist_norm.contains(&spot_artist_norm)
            || spot_artist_norm.contains(&db_artist_norm)
        {
            score += 80;
        } else {
            continue;
        }

        if let Some(db_dur) = candidate.duration_ms {
            let diff = (db_dur as i64 - spotify_track.duration_ms as i64).abs();
            if diff <= DURATION_MATCH_TOLERANCE_MS {
                score += 50;
            } else {
                score -= 50;
            }
        }

        if normalize(&candidate.name) == spot_title_norm {
            score += 20;
        }

        let candidate_rank = (score, candidate.source.priority());
        if candidate_rank > best_rank {
            best_rank = candidate_rank;
            best_candidate = Some(candidate);
        }
    }

    best_candidate.cloned()
}

fn find_match_for_track(
    track: &TrackInfo,
    metadata_index: &MetadataIndex,
) -> Option<MetadataEntry> {
    let title_key = normalize(&track.name);

    if let Some(candidates) = metadata_index.by_title.get(&title_key) {
        if let Some(best_match) = find_best_match(track, candidates) {
            return Some(best_match);
        }
    }

    let fuzzy_candidates: Vec<MetadataEntry> = metadata_index
        .all_entries
        .iter()
        .filter(|db_item| {
            let db_title = normalize(&db_item.name);
            let spot_title = normalize(&track.name);
            let title_match = db_title.contains(&spot_title) || spot_title.contains(&db_title);

            let db_artist = normalize(&db_item.artist);
            let spot_artist = normalize(&track.artist);
            let artist_match = db_artist.contains(&spot_artist) || spot_artist.contains(&db_artist);

            title_match && artist_match
        })
        .cloned()
        .collect();

    find_best_match(track, &fuzzy_candidates)
}

fn enrich_tracks(tracks: &mut [TrackInfo], metadata_index: &MetadataIndex) -> EnrichmentStats {
    let mut stats = EnrichmentStats::default();

    for track in tracks {
        match find_match_for_track(track, metadata_index) {
            Some(match_data) => match camelot_to_spotify(&match_data.key_camelot) {
                Some((pitch, mode)) => {
                    track.tempo = Some(match_data.bpm);
                    track.key = Some(pitch);
                    track.mode = Some(mode);
                    stats.matched += 1;
                    println!(
                        "[MATCH:{}] {} - {} (metadata: {})",
                        match_data.source.label(),
                        track.artist,
                        track.name,
                        match_data.name
                    );
                }
                None => {
                    stats.invalid_key += 1;
                    eprintln!(
                        "[KEY ERROR] Invalid Camelot '{}' for {} - {}",
                        match_data.key_camelot, track.artist, track.name
                    );
                }
            },
            None => {
                stats.unmatched += 1;
                eprintln!("[MISSING] {} - {}", track.artist, track.name);
            }
        }
    }

    stats
}

fn sort_tracks(tracks: Vec<TrackInfo>) -> Vec<TrackInfo> {
    let mut with_features = Vec::new();
    let mut without_features = Vec::new();

    for track in tracks {
        if let (Some(key), Some(mode), Some(tempo)) = (track.key, track.mode, track.tempo) {
            with_features.push((key, mode, tempo, track));
        } else {
            without_features.push(track);
        }
    }

    with_features.sort_by(|a, b| {
        let a_weight = get_sort_weight(a.0, a.1);
        let b_weight = get_sort_weight(b.0, b.1);
        a_weight
            .cmp(&b_weight)
            .then_with(|| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal))
    });

    let mut sorted_tracks: Vec<TrackInfo> = with_features
        .into_iter()
        .map(|(_, _, _, track)| track)
        .collect();
    sorted_tracks.extend(without_features);
    sorted_tracks
}

async fn authenticate_spotify() -> Result<AuthCodeSpotify> {
    let creds =
        Credentials::from_env().context("RSPOTIFY_CLIENT_ID / RSPOTIFY_CLIENT_SECRET missing")?;
    let oauth = OAuth::from_env(scopes!("playlist-modify-private", "playlist-modify-public"))
        .context("RSPOTIFY_REDIRECT_URI missing")?;
    let spotify = AuthCodeSpotify::new(creds, oauth);
    let authorize_url = spotify.get_authorize_url(false)?;
    println!("Open this Spotify authorization URL in your browser:");
    println!("{authorize_url}");
    print!("Paste the full redirect URL after approving access: ");
    io::stdout().flush().context("failed to flush stdout")?;

    let mut redirect_url = String::new();
    io::stdin()
        .read_line(&mut redirect_url)
        .context("failed to read redirect URL from stdin")?;

    let code = spotify
        .parse_response_code(redirect_url.trim())
        .ok_or_else(|| anyhow!("failed to parse Spotify authorization code from redirect URL"))?;
    spotify.request_token(&code).await?;
    Ok(spotify)
}

async fn fetch_playlist_tracks(
    spotify: &AuthCodeSpotify,
    playlist_id: &PlaylistId<'_>,
) -> Result<Vec<TrackInfo>> {
    let mut all_tracks = Vec::new();
    let mut offset: u32 = 0;

    println!("Fetching playlist tracks...");
    loop {
        let page = spotify
            .playlist_items_manual(playlist_id.clone(), None, None, Some(100), Some(offset))
            .await?;

        if page.items.is_empty() {
            break;
        }

        for item in page.items {
            if let Some(PlayableItem::Track(track)) = item.track {
                if let Some(id) = track.id {
                    let artist_name = track
                        .artists
                        .first()
                        .map(|artist| artist.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    let duration_ms = track.duration.num_milliseconds().max(0) as u32;

                    all_tracks.push(TrackInfo {
                        id: id.id().to_string(),
                        name: track.name.clone(),
                        artist: artist_name,
                        key: None,
                        mode: None,
                        tempo: None,
                        duration_ms,
                    });
                }
            }
        }

        offset += 100;
        if page.next.is_none() {
            break;
        }
    }

    Ok(all_tracks)
}

async fn update_playlist(
    spotify: &AuthCodeSpotify,
    playlist_id: &PlaylistId<'_>,
    tracks: &[TrackInfo],
) -> Result<()> {
    let playable_items: Vec<PlayableId<'_>> = tracks
        .iter()
        .map(|track| {
            let track_id = TrackId::from_id(&track.id)
                .map_err(|_| anyhow!("invalid track id returned by Spotify: {}", track.id))?;
            Ok(PlayableId::from(track_id))
        })
        .collect::<Result<_>>()?;

    if playable_items.is_empty() {
        println!("No tracks to update.");
        return Ok(());
    }

    println!("Updating Spotify playlist order...");
    let mut chunks = playable_items.chunks(100);

    if let Some(first_chunk) = chunks.next() {
        spotify
            .playlist_replace_items(playlist_id.clone(), first_chunk.to_vec())
            .await?;
    }

    for chunk in chunks {
        spotify
            .playlist_add_items(playlist_id.clone(), chunk.to_vec(), None)
            .await?;
    }

    println!("Done! Check the playlist custom order in Spotify.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli_args = parse_args_from(env::args())?;

    println!("Loading {LOCAL_DB_PATH}...");
    let mut metadata_entries = load_local_db_entries(LOCAL_DB_PATH)?;
    println!("Loaded {} local metadata entries.", metadata_entries.len());

    for exportify_csv_path in &cli_args.exportify_csv_paths {
        println!("Importing Exportify CSV: {exportify_csv_path}");
        let import = load_exportify_csv(exportify_csv_path)?;
        println!(
            "Imported {} Exportify metadata rows from {}.",
            import.entries.len(),
            exportify_csv_path
        );
        if import.skipped.total() > 0 {
            println!(
                "Skipped {} Exportify rows from {} (missing_tempo={}, missing_key_mode={}, invalid_tempo={}, invalid_key_mode={}).",
                import.skipped.total(),
                exportify_csv_path,
                import.skipped.missing_tempo,
                import.skipped.missing_key_mode,
                import.skipped.invalid_tempo,
                import.skipped.invalid_key_mode
            );
        }
        metadata_entries.extend(import.entries);
    }

    let metadata_index = build_metadata_index(metadata_entries);
    println!(
        "Metadata pool ready with {} total entries.",
        metadata_index.all_entries.len()
    );

    let playlist_id_value = extract_playlist_id(&cli_args.playlist_input)?;
    let playlist_id = PlaylistId::from_id(&playlist_id_value)
        .map_err(|_| anyhow!("failed to construct playlist id from {}", playlist_id_value))?;

    let spotify = authenticate_spotify().await?;
    let mut all_tracks = fetch_playlist_tracks(&spotify, &playlist_id).await?;
    println!("Found {} tracks in playlist.", all_tracks.len());

    println!("Enriching data...");
    let stats = enrich_tracks(&mut all_tracks, &metadata_index);
    println!(
        "Enrichment summary: matched={}, unmatched={}, invalid_key={}",
        stats.matched, stats.unmatched, stats.invalid_key
    );

    let sorted_tracks = sort_tracks(all_tracks);
    update_playlist(&spotify, &playlist_id, &sorted_tracks).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_track(name: &str, artist: &str, duration_ms: u32) -> TrackInfo {
        TrackInfo {
            id: "track-id".to_string(),
            name: name.to_string(),
            artist: artist.to_string(),
            key: None,
            mode: None,
            tempo: None,
            duration_ms,
        }
    }

    fn metadata_entry(
        name: &str,
        artist: &str,
        bpm: f32,
        key_camelot: &str,
        duration_ms: Option<u32>,
        source: MetadataSource,
    ) -> MetadataEntry {
        MetadataEntry {
            name: name.to_string(),
            artist: artist.to_string(),
            bpm,
            key_camelot: key_camelot.to_string(),
            duration_ms,
            source,
        }
    }

    #[test]
    fn normalize_collapses_spacing_and_quotes() {
        assert_eq!(normalize("  Don’t  Stop -  Now  "), "don't stop now");
    }

    #[test]
    fn camelot_to_spotify_parses_minor_and_major() {
        assert_eq!(camelot_to_spotify("5A"), Some((0, Modality::Minor)));
        assert_eq!(camelot_to_spotify("10B"), Some((2, Modality::Major)));
        assert_eq!(camelot_to_spotify("13B"), None);
        assert_eq!(camelot_to_spotify("5"), None);
    }

    #[test]
    fn spotify_to_camelot_round_trips_supported_keys() {
        assert_eq!(
            spotify_to_camelot(0, Modality::Minor),
            Some("5A".to_string())
        );
        assert_eq!(
            spotify_to_camelot(2, Modality::Major),
            Some("10B".to_string())
        );
        assert_eq!(spotify_to_camelot(-1, Modality::Major), None);
    }

    #[test]
    fn extract_playlist_id_accepts_plain_id_url_and_uri() {
        let id = "2nOsiUa2nlXBGuDMjDIbDb";
        assert_eq!(extract_playlist_id(id).unwrap(), id);
        assert_eq!(
            extract_playlist_id("https://open.spotify.com/playlist/2nOsiUa2nlXBGuDMjDIbDb?si=abc")
                .unwrap(),
            id
        );
        assert_eq!(
            extract_playlist_id("spotify:playlist:2nOsiUa2nlXBGuDMjDIbDb").unwrap(),
            id
        );
    }

    #[test]
    fn find_best_match_prefers_exact_artist_and_duration() {
        let track = sample_track("Losing It", "FISHER", 248_000);
        let candidates = vec![
            metadata_entry(
                "Losing It",
                "Random Artist",
                126.0,
                "10B",
                Some(248_000),
                MetadataSource::LocalDb,
            ),
            metadata_entry(
                "Losing It",
                "FISHER",
                125.0,
                "10B",
                Some(248_100),
                MetadataSource::LocalDb,
            ),
            metadata_entry(
                "Losing It",
                "FISHER",
                124.0,
                "10B",
                Some(270_000),
                MetadataSource::LocalDb,
            ),
        ];

        let best = find_best_match(&track, &candidates).unwrap();
        assert_eq!(best.artist, "FISHER");
        assert_eq!(best.bpm, 125.0);
    }

    #[test]
    fn find_match_for_track_uses_fuzzy_title_fallback() {
        let track = sample_track("Space Song - Remastered", "Beach House", 320_000);
        let metadata_index = MetadataIndex {
            by_title: HashMap::new(),
            all_entries: vec![metadata_entry(
                "Space Song",
                "Beach House",
                147.0,
                "5A",
                None,
                MetadataSource::LocalDb,
            )],
        };

        let matched = find_match_for_track(&track, &metadata_index).unwrap();
        assert_eq!(matched.name, "Space Song");
    }

    #[test]
    fn find_best_match_prefers_local_db_on_score_tie() {
        let track = sample_track("Losing It", "FISHER", 248_000);
        let local = metadata_entry(
            "Losing It",
            "FISHER",
            125.0,
            "10B",
            Some(248_000),
            MetadataSource::LocalDb,
        );
        let exportify = metadata_entry(
            "Losing It",
            "FISHER",
            125.0,
            "10B",
            Some(248_000),
            MetadataSource::Exportify,
        );

        let best = find_best_match(&track, &[exportify, local.clone()]).unwrap();
        assert_eq!(best.source, MetadataSource::LocalDb);
        assert_eq!(best, local);
    }

    #[test]
    fn sort_tracks_puts_unmatched_tracks_last() {
        let matched_later = TrackInfo {
            key: Some(11),
            mode: Some(Modality::Major),
            tempo: Some(128.0),
            ..sample_track("Later", "Artist", 200_000)
        };
        let matched_earlier = TrackInfo {
            key: Some(0),
            mode: Some(Modality::Minor),
            tempo: Some(120.0),
            ..sample_track("Earlier", "Artist", 200_000)
        };
        let unmatched = sample_track("Unknown", "Artist", 200_000);

        let sorted = sort_tracks(vec![matched_later, unmatched.clone(), matched_earlier]);
        assert_eq!(sorted[0].name, "Later");
        assert_eq!(sorted[1].name, "Earlier");
        assert_eq!(sorted[2], unmatched);
    }

    #[test]
    fn parse_args_accepts_repeated_exportify_flags() {
        let cli = parse_args_from([
            "mix_sorter".to_string(),
            "--exportify-csv".to_string(),
            "one.csv".to_string(),
            "--exportify-csv=two.csv".to_string(),
            "spotify:playlist:2nOsiUa2nlXBGuDMjDIbDb".to_string(),
        ])
        .unwrap();

        assert_eq!(
            cli,
            CliArgs {
                binary_name: "mix_sorter".to_string(),
                playlist_input: "spotify:playlist:2nOsiUa2nlXBGuDMjDIbDb".to_string(),
                exportify_csv_paths: vec!["one.csv".to_string(), "two.csv".to_string()],
            }
        );
    }

    #[test]
    fn load_exportify_csv_reader_imports_numeric_mode_with_bom_header() {
        let csv = "\u{feff}Track Name,Artist Name(s),Duration (ms),Key,Mode,Tempo\nLosing It,FISHER;Chris Lake,248000,2,1,125.0\n";
        let import = load_exportify_csv_reader(csv.as_bytes()).unwrap();

        assert_eq!(import.skipped.total(), 0);
        assert_eq!(import.entries.len(), 1);
        assert_eq!(import.entries[0].name, "Losing It");
        assert_eq!(import.entries[0].artist, "FISHER");
        assert_eq!(import.entries[0].duration_ms, Some(248_000));
        assert_eq!(import.entries[0].key_camelot, "10B");
        assert_eq!(import.entries[0].source, MetadataSource::Exportify);
    }

    #[test]
    fn load_exportify_csv_reader_tracks_skipped_rows() {
        let csv = "Track Name,Artist Name(s),Duration (ms),Key,Mode,Tempo\n\
                   Missing Tempo,FISHER,248000,2,1,\n\
                   Invalid Mode,FISHER,248000,2,weird,125.0\n\
                   Good Track,FISHER,248000,2,1,125.0\n";
        let import = load_exportify_csv_reader(csv.as_bytes()).unwrap();

        assert_eq!(import.entries.len(), 1);
        assert_eq!(import.skipped.missing_tempo, 1);
        assert_eq!(import.skipped.invalid_key_mode, 1);
        assert_eq!(import.skipped.total(), 2);
    }
}
