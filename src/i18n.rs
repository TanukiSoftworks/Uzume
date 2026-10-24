use crate::config::Language;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Strings {
    pub section_format: String,
    pub opt_song_title: String,
    pub opt_song_desc: String,
    pub opt_custom_title: String,
    pub opt_custom_desc: String,
    pub opt_custom_placeholder: String,
    pub opt_app_title_fmt: String,
    pub opt_app_desc: String,
    pub active_player_header: String,
    pub in_profile_label: String,
    pub status_listening_fmt: String,
    pub state_playing: String,
    pub state_paused: String,
    pub state_inactive: String,
    pub no_music_active: String,
    pub btn_save: String,
    pub save_success_title: String,
    pub save_success_body: String,
    pub tray_open: String,
    pub tray_logs: String,
    pub tray_exit: String,
    pub crash_title: String,
    pub crash_prompt: String,
    pub tray_ai_notice: String,
    pub ai_translation_notice: String,
    pub tray_check_updates: String,
    pub update_available_title: String,
    pub update_available_msg: String,
    pub update_latest_title: String,
    pub update_latest_msg: String,
    pub update_error_msg: String,
}

static JSON_ES: &str = include_str!("../locales/es.json");
static JSON_EN: &str = include_str!("../locales/en.json");
static JSON_ZH_HANS: &str = include_str!("../locales/zh-hans.json");
static JSON_ZH_HANT: &str = include_str!("../locales/zh-hant.json");
static JSON_JA: &str = include_str!("../locales/ja.json");
static JSON_KO: &str = include_str!("../locales/ko.json");

static STRINGS_ES: OnceLock<Strings> = OnceLock::new();
static STRINGS_EN: OnceLock<Strings> = OnceLock::new();
static STRINGS_ZH_HANS: OnceLock<Strings> = OnceLock::new();
static STRINGS_ZH_HANT: OnceLock<Strings> = OnceLock::new();
static STRINGS_JA: OnceLock<Strings> = OnceLock::new();
static STRINGS_KO: OnceLock<Strings> = OnceLock::new();

extern "system" {
    fn GetUserDefaultUILanguage() -> u16;
}

pub fn detect_system_language() -> Language {
    let lang_id = unsafe { GetUserDefaultUILanguage() };
    let primary = lang_id & 0x03FF;
    let sub = (lang_id >> 10) & 0x003F;

    match primary {
        0x0A => Language::Es,     // Spanish
        0x11 => Language::Ja,     // Japanese
        0x12 => Language::Ko,     // Korean
        0x04 => {                 // Chinese
            if sub == 0x01 || sub == 0x03 || sub == 0x05 {
                Language::ZhHant // Traditional (Taiwan, Hong Kong, Macau)
            } else {
                Language::ZhHans // Simplified (PRC, Singapore)
            }
        }
        0x09 => Language::En,     // English
        _ => Language::En,        // Default fallback to English
    }
}

pub fn get_strings(lang: Language) -> &'static Strings {
    let effective = match lang {
        Language::Auto => detect_system_language(),
        other => other,
    };

    match effective {
        Language::Es => STRINGS_ES.get_or_init(|| serde_json::from_str(JSON_ES).expect("JSON es.json válido")),
        Language::En => STRINGS_EN.get_or_init(|| serde_json::from_str(JSON_EN).expect("JSON en.json válido")),
        Language::ZhHans => STRINGS_ZH_HANS.get_or_init(|| serde_json::from_str(JSON_ZH_HANS).expect("JSON zh-hans.json válido")),
        Language::ZhHant => STRINGS_ZH_HANT.get_or_init(|| serde_json::from_str(JSON_ZH_HANT).expect("JSON zh-hant.json válido")),
        Language::Ja => STRINGS_JA.get_or_init(|| serde_json::from_str(JSON_JA).expect("JSON ja.json válido")),
        Language::Ko => STRINGS_KO.get_or_init(|| serde_json::from_str(JSON_KO).expect("JSON ko.json válido")),
        Language::Auto => STRINGS_EN.get_or_init(|| serde_json::from_str(JSON_EN).expect("JSON en.json válido")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_languages_provide_strings() {
        let languages = [
            Language::Es,
            Language::En,
            Language::ZhHans,
            Language::ZhHant,
            Language::Ja,
            Language::Ko,
        ];

        for &lang in &languages {
            let s = get_strings(lang);
            assert!(!s.section_format.is_empty());
            assert!(!s.opt_song_title.is_empty());
            assert!(!s.opt_song_desc.is_empty());
            assert!(!s.opt_custom_title.is_empty());
            assert!(!s.opt_custom_desc.is_empty());
            assert!(!s.opt_app_title_fmt.is_empty());
            assert!(!s.opt_app_desc.is_empty());
            assert!(!s.active_player_header.is_empty());
            assert!(!s.btn_save.is_empty());
            assert!(!s.crash_title.is_empty());
            assert!(!s.crash_prompt.is_empty());
            assert!(!s.tray_check_updates.is_empty());
            assert!(!s.update_available_title.is_empty());
            assert!(!s.update_available_msg.is_empty());
            assert!(!s.update_latest_title.is_empty());
            assert!(!s.update_latest_msg.is_empty());
            assert!(!s.update_error_msg.is_empty());
        }
    }

    #[test]
    fn test_system_language_detection() {
        let detected = detect_system_language();
        println!("Detected language: {:?}", detected);
        assert!(matches!(detected, Language::Es | Language::En));
    }
}
