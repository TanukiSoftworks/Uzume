#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod cover;
mod discord;
mod i18n;
mod logger;
mod smtc;
mod ui;

use config::{Config, StatusDisplayMode};
use discord::{Activity, Assets, DiscordIpc, Timestamps};
use smtc::{PlaybackState, SmtcListener};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;
use ui::SHOULD_EXIT;

const DEFAULT_MUSIC_ICON: &str =
    "https://cdn.jsdelivr.net/gh/twitter/twemoji@14.0.2/assets/72x72/1f3b5.png";
const SPOTIFY_ICON: &str =
    "https://storage.googleapis.com/pr-newsroom-wp/1/2023/05/Spotify_Primary_Logo_RGB_Green.png";
const DEEZER_ICON: &str =
    "https://e-cdns-images.dzcdn.net/images/common/favicon/favicon-240x240.png";

fn identify_player(app_id: &str) -> (&'static str, &'static str) {
    let lower = app_id.to_lowercase();
    if lower.contains("spotify") {
        ("Spotify", SPOTIFY_ICON)
    } else if lower.contains("deezer") {
        ("Deezer", DEEZER_ICON)
    } else if lower.contains("tidal") {
        ("Tidal", DEFAULT_MUSIC_ICON)
    } else if lower.contains("apple") {
        ("Apple Music", DEFAULT_MUSIC_ICON)
    } else if lower.contains("amazon") {
        ("Amazon Music", DEFAULT_MUSIC_ICON)
    } else if lower.contains("youtube") {
        ("YouTube Music", DEFAULT_MUSIC_ICON)
    } else if lower.contains("foobar") {
        ("foobar2000", DEFAULT_MUSIC_ICON)
    } else if lower.contains("musicbee") {
        ("MusicBee", DEFAULT_MUSIC_ICON)
    } else if lower.contains("aimp") {
        ("AIMP", DEFAULT_MUSIC_ICON)
    } else if lower.contains("vlc") {
        ("VLC", DEFAULT_MUSIC_ICON)
    } else {
        ("Music", DEFAULT_MUSIC_ICON)
    }
}

fn get_app_name(app_id: &str) -> String {
    let (name, _) = identify_player(app_id);
    if name != "Music" {
        return name.to_string();
    }
    let file = app_id.rsplit('\\').next().unwrap_or(app_id);
    file.strip_suffix(".exe").unwrap_or(file).to_string()
}

fn main() {
    let _log_path = logger::init();

    let config = Arc::new(Mutex::new(Config::load()));
    let current_status = Arc::new(Mutex::new("Iniciando servicio de medios...".to_string()));

    let worker_config = config.clone();
    let worker_status = current_status.clone();

    thread::spawn(move || {
        worker_loop(worker_config, worker_status);
    });

    ui::run_ui(config, current_status);
}

fn cache_cover_async(cover_url: &str) {
    if cover_url.is_empty() || !cover_url.starts_with("http") {
        return;
    }
    let url = cover_url.to_string();
    thread::spawn(move || {
        let appdata = std::env::var_os("APPDATA").unwrap_or_default();
        let cover_path = std::path::PathBuf::from(appdata).join("Uzume").join("cover.jpg");
        let thumb_url = url.replace("1000x1000", "250x250");
        if let Ok(resp) = ureq::get(&thumb_url).header("User-Agent", "Uzume/1.0").call() {
            if let Ok(bytes) = resp.into_body().read_to_vec() {
                let _ = std::fs::write(cover_path, bytes);
            }
        }
    });
}

fn worker_loop(config: Arc<Mutex<Config>>, status: Arc<Mutex<String>>) {
    let smtc = match SmtcListener::new() {
        Ok(s) => s,
        Err(e) => {
            let err_msg = format!("Error al inicializar Windows SMTC: {}", e);
            logger::log("ERROR", &err_msg);
            let mut st = status.lock().unwrap();
            *st = err_msg;
            return;
        }
    };

    let mut discord_client: Option<DiscordIpc> = None;
    let mut current_connected_id = String::new();

    let mut last_title = String::new();
    let mut last_state = PlaybackState::Other;
    let mut last_status_mode = StatusDisplayMode::ListeningSong;
    let mut last_custom_text = String::new();

    while !SHOULD_EXIT.load(Ordering::SeqCst) {
        let (desired_client_id, current_cfg) = {
            let cfg = config.lock().unwrap();
            (cfg.get_active_client_id(), cfg.clone())
        };

        if discord_client.is_none() || desired_client_id != current_connected_id {
            match DiscordIpc::connect(&desired_client_id) {
                Ok(client) => {
                    discord_client = Some(client);
                    current_connected_id = desired_client_id.clone();
                    let mut st = status.lock().unwrap();
                    *st = "Conectado a Discord. Esperando reproduccion...".to_string();
                }
                Err(e) => {
                    logger::log("WARN", &format!("Reintentando conexion con Discord: {}", e));
                    let mut st = status.lock().unwrap();
                    *st = "Buscando cliente de Discord...".to_string();
                    sleep(Duration::from_secs(2));
                    continue;
                }
            }
        }

        if let Some(track) = smtc.get_current_track() {
            let config_changed = current_cfg.status_mode != last_status_mode
                || current_cfg.custom_text != last_custom_text;
            let state_changed = track.title != last_title
                || track.playback_state != last_state
                || config_changed;

            if state_changed {
                let app_name = get_app_name(&track.app_id);
                let (_, player_icon) = identify_player(&track.app_id);

                match track.playback_state {
                    PlaybackState::Playing => {
                        if let Some(bytes) = &track.thumbnail_bytes {
                            let appdata = std::env::var_os("APPDATA").unwrap_or_default();
                            let cover_path = std::path::PathBuf::from(appdata).join("Uzume").join("cover.jpg");
                            let _ = std::fs::write(cover_path, bytes);
                        }

                        let meta = cover::fetch_metadata(
                            &track.title,
                            &track.artist,
                            &track.album,
                            track.duration.as_secs(),
                        );

                        let (artist_name, album_name, cover_url) = if let Some(m) = meta {
                            let artist = if !track.artist.is_empty() {
                                track.artist.clone()
                            } else {
                                m.artist
                            };
                            let album = if !track.album.is_empty() {
                                track.album.clone()
                            } else {
                                m.album
                            };
                            (artist, album, m.cover_url)
                        } else {
                            let artist = if !track.artist.is_empty() {
                                track.artist.clone()
                            } else {
                                app_name.clone()
                            };
                            let album = if !track.album.is_empty() {
                                track.album.clone()
                            } else {
                                track.title.clone()
                            };
                            (artist, album, player_icon.to_string())
                        };

                        cache_cover_async(&cover_url);

                        let timestamps = if let (Some(start), Some(end)) = (track.start_timestamp, track.end_timestamp) {
                            Some(Timestamps {
                                start: Some(start * 1000),
                                end: Some(end * 1000),
                            })
                        } else {
                            None
                        };

                        let large_tooltip = if !album_name.is_empty() {
                            format!("{} - {}", track.title, album_name)
                        } else {
                            track.title.clone()
                        };

                        let (activity_type, status_display_type, activity_state, status_preview) = match current_cfg.status_mode {
                            StatusDisplayMode::ListeningSong => {
                                (2, Some(2), Some(artist_name.clone()), format!("Listening to {}", track.title))
                            }
                            StatusDisplayMode::CustomText => {
                                let text = if !current_cfg.custom_text.trim().is_empty() {
                                    current_cfg.custom_text.trim().to_string()
                                } else {
                                    artist_name.clone()
                                };
                                (2, Some(1), Some(text.clone()), format!("Listening to {}", text))
                            }
                            StatusDisplayMode::AppDefault => {
                                (2, Some(0), Some(artist_name.clone()), format!("Listening to {}", app_name))
                            }
                        };

                        let activity = Activity {
                            activity_type,
                            status_display_type,
                            details: Some(track.title.clone()),
                            state: activity_state,
                            timestamps,
                            assets: Some(Assets {
                                large_image: Some(cover_url.clone()),
                                large_text: Some(large_tooltip),
                                small_image: Some(player_icon.to_string()),
                                small_text: Some(app_name.clone()),
                            }),
                        };

                        if let Some(client) = discord_client.as_mut() {
                            if let Err(e) = client.set_activity(activity) {
                                logger::log("ERROR", &format!("Error al enviar presencia a Discord: {}", e));
                                discord_client = None;
                            }
                        }

                        let mut st = status.lock().unwrap();
                        *st = format!("{}|En reproducción|{}|{}|{}|{}", app_name, track.title, artist_name, status_preview, cover_url);
                    }
                    PlaybackState::Paused => {
                        let artist_text = if !track.artist.is_empty() {
                            track.artist.clone()
                        } else {
                            app_name.clone()
                        };

                        let (activity_type, status_display_type, activity_state, status_preview) = match current_cfg.status_mode {
                            StatusDisplayMode::ListeningSong => {
                                (2, Some(2), Some(format!("Pausado - {}", artist_text)), format!("Listening to {}", track.title))
                            }
                            StatusDisplayMode::CustomText => {
                                let text = if !current_cfg.custom_text.trim().is_empty() {
                                    format!("Pausado - {}", current_cfg.custom_text.trim())
                                } else {
                                    format!("Pausado - {}", artist_text)
                                };
                                (2, Some(1), Some(text.clone()), format!("Listening to {}", text))
                            }
                            StatusDisplayMode::AppDefault => {
                                (2, Some(0), Some(format!("Pausado - {}", artist_text)), format!("Listening to {}", app_name))
                            }
                        };

                        let activity = Activity {
                            activity_type,
                            status_display_type,
                            details: Some(track.title.clone()),
                            state: activity_state,
                            timestamps: None,
                            assets: Some(Assets {
                                large_image: Some(player_icon.to_string()),
                                large_text: Some("Pausado".to_string()),
                                small_image: None,
                                small_text: None,
                            }),
                        };

                        if let Some(client) = discord_client.as_mut() {
                            let _ = client.set_activity(activity);
                        }

                        let mut st = status.lock().unwrap();
                        *st = format!("{}|Pausado|{}|{}|{}|", app_name, track.title, artist_text, status_preview);
                    }
                    _ => {
                        if let Some(client) = discord_client.as_mut() {
                            let _ = client.clear_activity();
                        }
                        let mut st = status.lock().unwrap();
                        *st = format!("{}|Inactivo|Sin música activa||Listening to {}|", app_name, app_name);
                    }
                }

                last_title = track.title;
                last_state = track.playback_state;
                last_status_mode = current_cfg.status_mode;
                last_custom_text = current_cfg.custom_text;
            }
        } else if !last_title.is_empty() {
            if let Some(client) = discord_client.as_mut() {
                let _ = client.clear_activity();
            }
            last_title.clear();
            last_state = PlaybackState::Other;
            let mut st = status.lock().unwrap();
            *st = "Uzume|Inactivo|Sin música activa||Listening to Uzume|".to_string();
        }

        sleep(Duration::from_millis(1000));
    }

    if let Some(client) = discord_client.as_mut() {
        let _ = client.clear_activity();
    }
}
