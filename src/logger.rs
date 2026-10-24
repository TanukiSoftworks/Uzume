use crate::i18n::{detect_system_language, get_strings};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_ICONERROR, MB_YESNO, SW_SHOW,
};

pub const REPO_URL: &str = "https://github.com/TanukiSoftworks/Uzume";
pub const REPO_ISSUES_URL: &str = "https://github.com/TanukiSoftworks/Uzume/issues";

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

pub fn init() -> PathBuf {
    let appdata = std::env::var_os("APPDATA").unwrap_or_default();
    let dir = PathBuf::from(appdata).join("Uzume");
    let _ = fs::create_dir_all(&dir);
    let log_path = dir.join("presence.log");

    if let Ok(file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let mut guard = LOG_FILE.lock().unwrap();
        *guard = Some(file);
    }

    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Panic occurred without message".to_string()
        };

        handle_crash("RUST_PANIC", &format!("{} (at {})", payload, location));
    }));

    log_path
}

pub fn handle_crash(kind: &str, details: &str) {
    let now = SystemTime::now();
    let timestamp = match now.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => {
            let s = d.as_secs();
            format!("{:02}:{:02}:{:02}", (s / 3600) % 24, (s / 60) % 60, s % 60)
        }
        Err(_) => "00:00:00".to_string(),
    };

    let appdata = std::env::var_os("APPDATA").unwrap_or_default();
    let dir = PathBuf::from(appdata).join("Uzume");
    let crash_path = dir.join("crash.log");

    let crash_content = format!(
        "=======================================================\r\n\
         UZUME CRITICAL ERROR REPORT\r\n\
         Repo: {}\r\n\
         Issues: {}\r\n\
         =======================================================\r\n\
         Timestamp: {}\r\n\
         Version: {}\r\n\
         Type: [{}]\r\n\
         Details: {}\r\n\
         =======================================================\r\n",
        REPO_URL,
        REPO_ISSUES_URL,
        timestamp,
        env!("CARGO_PKG_VERSION"),
        kind,
        details
    );

    let _ = fs::write(&crash_path, crash_content);
    log_raw("CRITICAL", &format!("CRASH [{}] {}", kind, details));

    let lang = detect_system_language();
    let strings = get_strings(lang);

    let prompt = strings
        .crash_prompt
        .replacen("{}", &crash_path.to_string_lossy(), 1)
        .replacen("{}", REPO_ISSUES_URL, 1);

    let wide_title: Vec<u16> = strings.crash_title.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_prompt: Vec<u16> = prompt.encode_utf16().chain(std::iter::once(0)).collect();

    let choice = unsafe {
        MessageBoxW(
            HWND::default(),
            PCWSTR(wide_prompt.as_ptr()),
            PCWSTR(wide_title.as_ptr()),
            MB_ICONERROR | MB_YESNO,
        )
    };

    if choice == IDYES {
        unsafe {
            let wide_crash_path: Vec<u16> = crash_path.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
            let _ = ShellExecuteW(
                HWND::default(),
                w!("open"),
                PCWSTR(wide_crash_path.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOW,
            );

            let wide_issues: Vec<u16> = REPO_ISSUES_URL.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = ShellExecuteW(
                HWND::default(),
                w!("open"),
                PCWSTR(wide_issues.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOW,
            );
        }
    }
}

pub fn log(level: &str, msg: &str) {
    let upper = level.to_uppercase();
    if upper == "INFO" || upper == "DEBUG" {
        return;
    }
    log_raw(&upper, msg);
}

fn log_raw(level: &str, msg: &str) {
    let now = SystemTime::now();
    let timestamp = match now.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => {
            let s = d.as_secs();
            format!("{:02}:{:02}:{:02}", (s / 3600) % 24, (s / 60) % 60, s % 60)
        }
        Err(_) => "00:00:00".to_string(),
    };

    let line = format!("[{}] [{}] {}\n", timestamp, level, msg);

    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(file) = guard.as_mut() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }
}
