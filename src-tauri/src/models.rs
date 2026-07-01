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
    pub audio_mode:      String,
    pub audio_bitrate:   u32,
    pub spatial_aq:      bool,
    pub temporal_aq:     bool,
    pub aq_strength:     u8,
    pub multipass:       String,
    pub container:       String,
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
    #[serde(default)]
    pub discord_fields: HashMap<String, Vec<String>>,
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
            discord_fields: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncodingPrefs {
    #[serde(default = "default_crf")]
    pub crf: u32,
    #[serde(default = "default_preset")]
    pub preset: String,
    #[serde(default = "default_se_format")]
    pub se_format: String,
    #[serde(default = "default_tag_order")]
    pub tag_order: Vec<String>,
    #[serde(default)]
    pub team: String,
    #[serde(default = "default_audio_mode")]
    pub audio_mode: String,
    #[serde(default = "default_audio_bitrate")]
    pub audio_bitrate: u32,
    #[serde(default)]
    pub spatial_aq: bool,
    #[serde(default)]
    pub temporal_aq: bool,
    #[serde(default = "default_aq_strength")]
    pub aq_strength: u32,
    #[serde(default = "default_multipass")]
    pub multipass: String,
    #[serde(default = "default_container")]
    pub container: String,
    #[serde(default = "default_sub_extract_format")]
    pub sub_extract_format: String,
    #[serde(default = "default_sub_extract_naming")]
    pub sub_extract_naming: String,
    #[serde(default = "default_sub_extract_path_mode")]
    pub sub_extract_path_mode: String,
    #[serde(default)]
    pub sub_extract_custom_path: String,
    #[serde(default = "default_show_extract_button")]
    pub show_extract_button: bool,
}

fn default_crf() -> u32 { 28 }
fn default_preset() -> String { "p5".to_string() }
fn default_se_format() -> String { "S01E01".to_string() }
fn default_tag_order() -> Vec<String> {
    vec![
        "title", "se", "audio", "resolution", "provider", "source",
        "codec", "bitdepth", "audioCodec", "team",
    ].into_iter().map(String::from).collect()
}
fn default_audio_mode() -> String { "reencode".to_string() }
fn default_audio_bitrate() -> u32 { 192 }
fn default_aq_strength() -> u32 { 8 }
fn default_multipass() -> String { "disabled".to_string() }
fn default_container() -> String { "mkv".to_string() }
fn default_sub_extract_format() -> String { "srt".to_string() }
fn default_sub_extract_naming() -> String { "source".to_string() }
fn default_sub_extract_path_mode() -> String { "source".to_string() }
fn default_show_extract_button() -> bool { true }

impl Default for EncodingPrefs {
    fn default() -> Self {
        Self {
            crf: default_crf(),
            preset: default_preset(),
            se_format: default_se_format(),
            tag_order: default_tag_order(),
            team: String::new(),
            audio_mode: default_audio_mode(),
            audio_bitrate: default_audio_bitrate(),
            spatial_aq: false,
            temporal_aq: false,
            aq_strength: default_aq_strength(),
            multipass: default_multipass(),
            container: default_container(),
            sub_extract_format: default_sub_extract_format(),
            sub_extract_naming: default_sub_extract_naming(),
            sub_extract_path_mode: default_sub_extract_path_mode(),
            sub_extract_custom_path: String::new(),
            show_extract_button: default_show_extract_button(),
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Stats {
    pub total_files: u32,
    pub total_original_mb: f64,
    pub total_encoded_mb: f64,
    pub sum_ratio_pct: f64,
    pub last_updated: Option<String>,
}