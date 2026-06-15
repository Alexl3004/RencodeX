//! État global de l'encodeur

use std::sync::{Arc, Mutex};
use std::time::Instant;
use once_cell::sync::Lazy;

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