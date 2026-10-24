use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct TrackMeta {
    pub artist: String,
    pub album: String,
    pub cover_url: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    data: Vec<SearchItem>,
}

#[derive(Deserialize)]
struct SearchItem {
    #[serde(default)]
    duration: i64,
    artist: ArtistItem,
    album: AlbumItem,
}

#[derive(Deserialize)]
struct ArtistItem {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct AlbumItem {
    #[serde(default)]
    title: String,
    cover_xl: Option<String>,
    cover_big: Option<String>,
    cover_medium: Option<String>,
}

static CACHE: Mutex<Option<HashMap<String, Option<TrackMeta>>>> = Mutex::new(None);

fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

fn artist_matches(candidate: &str, target: &str) -> bool {
    let c = normalize(candidate);
    let t = normalize(target);
    if c.is_empty() || t.is_empty() {
        return true;
    }
    c == t || c.contains(&t) || t.contains(&c)
}

fn pick_cover(xl: Option<&String>, big: Option<&String>, med: Option<&String>) -> Option<String> {
    xl.or(big).or(med).cloned()
}

pub fn fetch_metadata(title: &str, artist: &str, album: &str, duration_s: u64) -> Option<TrackMeta> {
    let clean_title = title.trim();
    if clean_title.is_empty() {
        return None;
    }

    let cache_key = format!("{}::{}::{}", normalize(artist), normalize(clean_title), normalize(album));

    {
        let mut guard = CACHE.lock().unwrap();
        let map = guard.get_or_insert_with(HashMap::new);
        if let Some(hit) = map.get(&cache_key) {
            return hit.clone();
        }
    }

    let query = if !artist.trim().is_empty() {
        format!("{} {}", clean_title, artist.trim())
    } else {
        clean_title.to_string()
    };

    let url = format!(
        "https://api.deezer.com/search?q={}&limit=10",
        urlencoding::encode(&query)
    );

    let mut result: Option<TrackMeta> = None;

    if let Ok(resp) = ureq::get(&url).header("User-Agent", "Uzume/1.0").call() {
        if let Ok(payload) = resp.into_body().read_json::<SearchResponse>() {
            let matches: Vec<&SearchItem> = payload
                .data
                .iter()
                .filter(|item| artist.trim().is_empty() || artist_matches(&item.artist.name, artist))
                .collect();

            let candidate = if duration_s > 0 && !matches.is_empty() {
                matches
                    .iter()
                    .find(|i| (i.duration - duration_s as i64).abs() <= 4)
                    .copied()
                    .or_else(|| matches.first().copied())
            } else {
                matches.first().copied().or_else(|| payload.data.first())
            };

            if let Some(item) = candidate {
                if let Some(cover_url) = pick_cover(
                    item.album.cover_xl.as_ref(),
                    item.album.cover_big.as_ref(),
                    item.album.cover_medium.as_ref(),
                ) {
                    result = Some(TrackMeta {
                        artist: item.artist.name.clone(),
                        album: item.album.title.clone(),
                        cover_url,
                    });
                }
            }
        }
    }

    {
        let mut guard = CACHE.lock().unwrap();
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(cache_key, result.clone());
    }

    result
}
