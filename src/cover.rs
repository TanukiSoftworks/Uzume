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
struct DeezerSearchResponse {
    #[serde(default)]
    data: Vec<DeezerSearchItem>,
}

#[derive(Deserialize)]
struct DeezerSearchItem {
    #[serde(default)]
    duration: i64,
    artist: DeezerArtistItem,
    album: DeezerAlbumItem,
}

#[derive(Deserialize)]
struct DeezerArtistItem {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct DeezerAlbumItem {
    #[serde(default)]
    title: String,
    cover_xl: Option<String>,
    cover_big: Option<String>,
    cover_medium: Option<String>,
}

#[derive(Deserialize)]
struct ItunesSearchResponse {
    #[serde(default)]
    results: Vec<ItunesSearchItem>,
}

#[derive(Deserialize)]
struct ItunesSearchItem {
    #[serde(default, rename = "artistName")]
    artist_name: String,
    #[serde(default, rename = "collectionName")]
    collection_name: String,
    #[serde(rename = "artworkUrl100")]
    artwork_url_100: Option<String>,
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

fn pick_deezer_cover(xl: Option<&String>, big: Option<&String>, med: Option<&String>) -> Option<String> {
    xl.or(big).or(med).cloned()
}

fn clean_title_noise(title: &str) -> String {
    let mut cleaned = title.to_string();
    // Remove (feat. ...), [feat. ...], (Remastered...), etc.
    if let Some(idx) = cleaned.find(" (feat.") {
        cleaned.truncate(idx);
    } else if let Some(idx) = cleaned.find(" (Feat.") {
        cleaned.truncate(idx);
    } else if let Some(idx) = cleaned.find(" [feat.") {
        cleaned.truncate(idx);
    } else if let Some(idx) = cleaned.find(" - Remaster") {
        cleaned.truncate(idx);
    } else if let Some(idx) = cleaned.find(" (Remaster") {
        cleaned.truncate(idx);
    }
    cleaned.trim().to_string()
}

fn primary_artist(artist: &str) -> &str {
    let delimiters = [",", "&", "feat.", "ft.", "Feat.", "Ft."];
    let mut result = artist;
    for delim in delimiters {
        if let Some(idx) = result.find(delim) {
            result = &result[..idx];
        }
    }
    result.trim()
}

fn search_deezer(title: &str, artist: &str, duration_s: u64) -> Option<TrackMeta> {
    let query = if !artist.is_empty() {
        format!("{} {}", title, artist)
    } else {
        title.to_string()
    };

    let url = format!(
        "https://api.deezer.com/search?q={}&limit=10",
        urlencoding::encode(&query)
    );

    let resp = ureq::get(&url)
        .header("User-Agent", "Uzume/1.0")
        .call()
        .ok()?;

    let payload = resp.into_body().read_json::<DeezerSearchResponse>().ok()?;

    let matches: Vec<&DeezerSearchItem> = payload
        .data
        .iter()
        .filter(|item| artist.is_empty() || artist_matches(&item.artist.name, artist))
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
        if let Some(cover_url) = pick_deezer_cover(
            item.album.cover_xl.as_ref(),
            item.album.cover_big.as_ref(),
            item.album.cover_medium.as_ref(),
        ) {
            return Some(TrackMeta {
                artist: item.artist.name.clone(),
                album: item.album.title.clone(),
                cover_url,
            });
        }
    }

    None
}

fn search_itunes(title: &str, artist: &str) -> Option<TrackMeta> {
    let query = if !artist.is_empty() {
        format!("{} {}", title, artist)
    } else {
        title.to_string()
    };

    let url = format!(
        "https://itunes.apple.com/search?term={}&entity=song&limit=5",
        urlencoding::encode(&query)
    );

    let resp = ureq::get(&url)
        .header("User-Agent", "Uzume/1.0")
        .call()
        .ok()?;

    let payload = resp.into_body().read_json::<ItunesSearchResponse>().ok()?;

    for item in payload.results {
        if let Some(art100) = item.artwork_url_100 {
            // Upgrade artwork to high resolution 1000x1000
            let cover_url = art100.replace("100x100bb.jpg", "1000x1000bb.jpg");
            return Some(TrackMeta {
                artist: item.artist_name,
                album: item.collection_name,
                cover_url,
            });
        }
    }

    None
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

    let stripped_title = clean_title_noise(clean_title);
    let p_artist = primary_artist(artist.trim());

    // 1. Primary search on Deezer (with exact artist if available)
    let mut result = search_deezer(&stripped_title, p_artist, duration_s);

    // 2. If no match and artist was noisy, try Deezer with just cleaned title
    if result.is_none() && !p_artist.is_empty() {
        result = search_deezer(&stripped_title, "", duration_s);
    }

    // 3. Fallback to iTunes / Apple Music catalog
    if result.is_none() {
        result = search_itunes(&stripped_title, p_artist);
    }

    // 4. Fallback to iTunes with title only
    if result.is_none() && !p_artist.is_empty() {
        result = search_itunes(&stripped_title, "");
    }

    {
        let mut guard = CACHE.lock().unwrap();
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(cache_key, result.clone());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title_noise() {
        assert_eq!(clean_title_noise("Song (feat. Drake)"), "Song");
        assert_eq!(clean_title_noise("Track [feat. Artist]"), "Track");
        assert_eq!(clean_title_noise("Classic (Remastered 2020)"), "Classic");
        assert_eq!(clean_title_noise("Simple Title"), "Simple Title");
    }

    #[test]
    fn test_primary_artist() {
        assert_eq!(primary_artist("Artist One, Artist Two"), "Artist One");
        assert_eq!(primary_artist("Artist One & Artist Two"), "Artist One");
        assert_eq!(primary_artist("Artist One feat. Guest"), "Artist One");
        assert_eq!(primary_artist("Solo Artist"), "Solo Artist");
    }

    #[test]
    fn test_fetch_metadata_live() {
        // Deezer primary lookup
        let meta = fetch_metadata("Aftershock", "Macky Gee", "", 0);
        assert!(meta.is_some());
        let meta = meta.unwrap();
        assert!(meta.cover_url.starts_with("http"));

        // Fallback lookup
        let meta_fb = fetch_metadata("The Spaceship Heart", "aku.", "", 0);
        assert!(meta_fb.is_some());
        let meta_fb = meta_fb.unwrap();
        assert!(meta_fb.cover_url.starts_with("http"));
    }
}
