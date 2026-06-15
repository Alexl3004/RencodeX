use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StreamInfo {
    pub index: u32,
    pub codec_type: String,
    pub codec_name: String,
    pub language: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileAnalysis {
    pub path: String,
    pub filename: String,
    pub size_mb: f64,
    pub duration_secs: f64,
    pub fps: f64,
    pub streams: Vec<StreamInfo>,
    pub audio_langs: Vec<String>,
    pub sub_langs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncodeJob {
    pub input_path: String,
    pub output_path: String,
    pub audio_langs: Vec<String>,
    pub sub_langs: Vec<String>,
    pub audio_overrides: HashMap<String, String>,
    pub sub_overrides: HashMap<String, String>,
    pub duration_secs: f64,
    pub fps: f64,
    pub crf:             u32,
    pub preset:          String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProgressEvent {
    pub file_index: usize,
    pub file_total: usize,
    pub file_name: String,
    pub percent: f64,
    pub speed: f64,
    pub remaining_file: f64,
    pub remaining_total: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileResult {
    pub path: String,
    pub name: String,
    pub status: String,
    pub original_mb: f64,
    pub encoded_mb: f64,
    pub duration_secs: f64,
    pub error_msg: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncodeSummary {
    pub files: Vec<FileResult>,
    pub total_original_mb: f64,
    pub total_encoded_mb: f64,
    pub total_secs: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub ffmpeg_path: String,
    pub dark_mode: bool,
    pub send_email: bool,
    pub email_to: String,
    pub discord_bot_token: String,
    pub discord_log_channel_id: String,
    pub discord_cmd_channel_id: String,
    pub discord_enabled: bool,
    pub discord_notify_start:    bool,
    pub discord_notify_file_done: bool,
    pub discord_notify_summary:   bool,
    pub discord_notify_error:     bool,
    pub discord_notify_progress:  bool,
    pub discord_progress_interval: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ffmpeg_path: r"C:\Outil\ffmpeg\bin\ffmpeg.exe".to_string(),
            dark_mode: true,
            send_email: false,
            email_to: String::new(),
            discord_bot_token: String::new(),
            discord_log_channel_id: String::new(),
            discord_cmd_channel_id: String::new(),
            discord_enabled: false,
            discord_notify_start:    true,
            discord_notify_file_done: true,
            discord_notify_summary:   true,
            discord_notify_error:     true,
            discord_notify_progress:  false,
            discord_progress_interval: 30,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CleanedName {
    pub title: String,
    pub year: String,
    pub season_episode: String,
    pub resolution: String,
    pub source: String,
    pub provider: String,
    pub hdr: String,
    pub edition: String,
    pub audio_tags: String,
    pub suggested: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub to: String,
}