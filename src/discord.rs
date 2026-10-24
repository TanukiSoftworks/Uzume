use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Timestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Assets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Activity {
    #[serde(rename = "type")]
    pub activity_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_display_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<Timestamps>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<Assets>,
}

pub struct DiscordIpc {
    pipe: File,
}

impl DiscordIpc {
    pub fn connect(client_id: &str) -> std::io::Result<Self> {
        for i in 0..10 {
            let path = format!(r"\\.\pipe\discord-ipc-{}", i);
            if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(&path) {
                let payload = serde_json::json!({
                    "v": 1,
                    "client_id": client_id
                });
                let json_bytes = serde_json::to_vec(&payload)?;
                let len = json_bytes.len() as u32;

                file.write_all(&0u32.to_le_bytes())?;
                file.write_all(&len.to_le_bytes())?;
                file.write_all(&json_bytes)?;

                let mut header = [0u8; 8];
                file.read_exact(&mut header)?;
                let resp_len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
                let mut resp_buf = vec![0u8; resp_len];
                file.read_exact(&mut resp_buf)?;

                return Ok(Self { pipe: file });
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Discord named pipe not found",
        ))
    }

    pub fn set_activity(&mut self, activity: Activity) -> std::io::Result<()> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .to_string();

        let payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": activity
            },
            "nonce": nonce
        });

        self.send_frame(&payload)
    }

    pub fn clear_activity(&mut self) -> std::io::Result<()> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .to_string();

        let payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": serde_json::Value::Null
            },
            "nonce": nonce
        });

        self.send_frame(&payload)
    }

    fn send_frame(&mut self, payload: &serde_json::Value) -> std::io::Result<()> {
        let json_bytes = serde_json::to_vec(payload)?;
        let len = json_bytes.len() as u32;

        self.pipe.write_all(&1u32.to_le_bytes())?;
        self.pipe.write_all(&len.to_le_bytes())?;
        self.pipe.write_all(&json_bytes)?;

        let mut header = [0u8; 8];
        self.pipe.read_exact(&mut header)?;
        let resp_len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
        let mut resp_buf = vec![0u8; resp_len];
        self.pipe.read_exact(&mut resp_buf)?;

        Ok(())
    }
}
