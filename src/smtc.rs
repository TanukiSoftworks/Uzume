use std::time::{Duration, SystemTime, UNIX_EPOCH};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;

const WINDOWS_TICK_OFFSET: i64 = 116_444_736_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackInfo {
    pub app_id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub playback_state: PlaybackState,
    pub position: Duration,
    pub duration: Duration,
    pub start_timestamp: Option<i64>,
    pub end_timestamp: Option<i64>,
    pub thumbnail_bytes: Option<Vec<u8>>,
}

pub struct SmtcListener {
    manager: GlobalSystemMediaTransportControlsSessionManager,
}

impl SmtcListener {
    pub fn new() -> windows::core::Result<Self> {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()?;
        Ok(Self { manager })
    }

    fn is_browser(app_id: &str) -> bool {
        let lower = app_id.to_lowercase();
        const BROWSERS: &[&str] = &[
            "chrome", "msedge", "edge", "firefox", "brave", "opera", "vivaldi", "arc", "safari",
            "browser", "chromium",
        ];

        if BROWSERS.iter().any(|k| lower.contains(k)) {
            return true;
        }

        app_id.len() == 16 && app_id.chars().all(|c| c.is_ascii_hexdigit())
    }

    pub fn get_current_track(&self) -> Option<TrackInfo> {
        let mut playing = Vec::new();
        let mut paused = Vec::new();

        let current_app_id = self
            .manager
            .GetCurrentSession()
            .ok()
            .and_then(|s| s.SourceAppUserModelId().ok())
            .map(|id| id.to_string());

        if let Ok(sessions) = self.manager.GetSessions() {
            if let Ok(count) = sessions.Size() {
                for i in 0..count {
                    if let Ok(session) = sessions.GetAt(i) {
                        if let Some(track) = Self::extract_track_info(&session) {
                            match track.playback_state {
                                PlaybackState::Playing => playing.push(track),
                                PlaybackState::Paused => paused.push(track),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        if !playing.is_empty() {
            if let Some(ref curr_id) = current_app_id {
                if let Some(pos) = playing.iter().position(|t| t.app_id.eq_ignore_ascii_case(curr_id)) {
                    return Some(playing.remove(pos));
                }
            }
            return Some(playing.remove(0));
        }

        if !paused.is_empty() {
            if let Some(ref curr_id) = current_app_id {
                if let Some(pos) = paused.iter().position(|t| t.app_id.eq_ignore_ascii_case(curr_id)) {
                    return Some(paused.remove(pos));
                }
            }
            return Some(paused.remove(0));
        }

        None
    }

    fn extract_track_info(session: &GlobalSystemMediaTransportControlsSession) -> Option<TrackInfo> {
        let app_id = session.SourceAppUserModelId().ok()?.to_string();
        if Self::is_browser(&app_id) {
            return None;
        }

        let playback_info = session.GetPlaybackInfo().ok()?;
        let status = playback_info.PlaybackStatus().ok()?;
        let playback_state = match status {
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => PlaybackState::Playing,
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => PlaybackState::Paused,
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => PlaybackState::Stopped,
            _ => PlaybackState::Other,
        };

        let media_props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
        let title = media_props.Title().unwrap_or_default().to_string();
        let artist = media_props.Artist().unwrap_or_default().to_string();
        let album = media_props.AlbumTitle().unwrap_or_default().to_string();

        if title.is_empty() && artist.is_empty() {
            return None;
        }

        let mut position = Duration::ZERO;
        let mut duration = Duration::ZERO;
        let mut start_timestamp = None;
        let mut end_timestamp = None;

        if let Ok(timeline) = session.GetTimelineProperties() {
            if let Ok(pos) = timeline.Position() {
                position = Duration::from_millis((pos.Duration / 10_000).max(0) as u64);
            }
            if let Ok(dur) = timeline.EndTime() {
                duration = Duration::from_millis((dur.Duration / 10_000).max(0) as u64);
            }

            if playback_state == PlaybackState::Playing && duration.as_secs() > 0 {
                let now_unix = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;

                let last_updated = timeline.LastUpdatedTime().map(|t| t.UniversalTime).unwrap_or(0);
                let update_unix = if last_updated > WINDOWS_TICK_OFFSET {
                    (last_updated - WINDOWS_TICK_OFFSET) / 10_000_000
                } else {
                    now_unix
                };

                let start = (update_unix - position.as_secs() as i64).max(0);
                let end = start + duration.as_secs() as i64;

                start_timestamp = Some(start);
                end_timestamp = Some(end);
            }
        }

        let mut thumbnail_bytes = None;
        if let Ok(thumb_ref) = media_props.Thumbnail() {
            if let Ok(stream_op) = thumb_ref.OpenReadAsync() {
                if let Ok(stream) = stream_op.get() {
                    if let Ok(size) = stream.Size() {
                        if size > 0 && size < 10_000_000 {
                            if let Ok(reader) = DataReader::CreateDataReader(&stream) {
                                if reader.LoadAsync(size as u32).and_then(|op| op.get()).is_ok() {
                                    let mut buf = vec![0u8; size as usize];
                                    if reader.ReadBytes(&mut buf).is_ok() {
                                        thumbnail_bytes = Some(buf);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Some(TrackInfo {
            app_id,
            title,
            artist,
            album,
            playback_state,
            position,
            duration,
            start_timestamp,
            end_timestamp,
            thumbnail_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_browser() {
        assert!(SmtcListener::is_browser("chrome.exe"));
        assert!(SmtcListener::is_browser("msedge.exe"));
        assert!(SmtcListener::is_browser("firefox.exe"));
        assert!(SmtcListener::is_browser("brave.exe"));
        assert!(SmtcListener::is_browser("308046B0AF4A39CB"));
        assert!(!SmtcListener::is_browser("Spotify.exe"));
        assert!(!SmtcListener::is_browser("SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify"));
        assert!(!SmtcListener::is_browser("Deezer.62021768415AF_q7m17pa7q8kj0!Deezer.Music"));
        assert!(!SmtcListener::is_browser("TIDAL.exe"));
        assert!(!SmtcListener::is_browser("foobar2000.exe"));
    }

    #[test]
    fn test_get_current_track_detects_active_player() {
        if let Ok(listener) = SmtcListener::new() {
            if let Some(track) = listener.get_current_track() {
                assert!(!track.title.is_empty() || !track.artist.is_empty());
                assert!(!SmtcListener::is_browser(&track.app_id));
            }
        }
    }
}
