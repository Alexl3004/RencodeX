//! État global de l'encodeur

use std::sync::{Arc, Mutex};
use std::time::Instant;
use once_cell::sync::Lazy;
use std::os::windows::process::CommandExt;

pub struct EncoderState {
    pub cancel: bool,
    pub pause: bool,
    pub child_pid: Option<u32>,
    pub current_out: Option<String>,
    pub encoding: bool,
    pub current_file: String,
    pub file_index: usize,
    pub file_total: usize,
    pub percent: f64,
    pub speed: f64,
    pub remaining: f64,
    pub queue_names: Vec<String>,
    pub start_time: Option<Instant>,
}

impl Default for EncoderState {
    fn default() -> Self {
        Self {
            cancel: false,
            pause: false,
            child_pid: None,
            current_out: None,
            encoding: false,
            current_file: String::new(),
            file_index: 0,
            file_total: 0,
            percent: 0.0,
            speed: 0.0,
            remaining: 0.0,
            queue_names: Vec::new(),
            start_time: None,
        }
    }
}

pub static ENCODER: Lazy<Arc<Mutex<EncoderState>>> = Lazy::new(|| {
    Arc::new(Mutex::new(EncoderState::default()))
});

pub fn lock_encoder() -> std::sync::MutexGuard<'static, EncoderState> {
    ENCODER.lock().unwrap_or_else(|e| e.into_inner())
}

/// Tue proprement le process ffmpeg en cours (si existant) et marque l'encodeur
/// comme annulé. Appelé depuis le handler on-close-requested ET depuis cancel_encoding.
/// Retourne le chemin du fichier de sortie partiel à supprimer, si applicable.
pub fn kill_ffmpeg_process() -> Option<String> {
    let (pid_opt, out_opt) = {
        let mut s = lock_encoder();
        if !s.encoding {
            return None;
        }
        s.cancel = true;
        (s.child_pid.take(), s.current_out.clone())
    };

    if let Some(pid) = pid_opt {
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F", "/T"])
            .creation_flags(0x08000000)
            .spawn();
    }

    out_opt
}