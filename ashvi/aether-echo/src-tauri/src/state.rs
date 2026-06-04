// AetherEcho — Shared Application State
// All state is wrapped in Arc<Mutex> for safe cross-thread access.

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crossbeam_channel::Sender;

// ── Settings ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Selected Groq model identifier
    pub model: String,
    /// Overlay window opacity (0.1 – 1.0)
    pub overlay_opacity: f32,
    /// Hotkey to toggle overlay visibility
    pub hotkey_toggle_overlay: String,
    /// Hotkey to toggle listening
    pub hotkey_toggle_listening: String,
    /// Hotkey to toggle overlay click-through
    #[serde(default = "default_click_through_hotkey")]
    pub hotkey_toggle_click_through: String,
    /// Hotkey to trigger screenshot solve
    #[serde(default = "default_screenshot_solve_hotkey")]
    pub hotkey_screenshot_solve: String,
    /// Minimum RMS energy to detect speech (0.001 – 0.1)
    pub vad_sensitivity: f32,
    /// Silence duration threshold in milliseconds (e.g. 300 – 3000)
    #[serde(default = "default_silence_timeout")]
    pub vad_silence_timeout: u32,
    /// Whether to start listening automatically on launch
    pub auto_start_listening: bool,
    /// Whisper model size: "tiny" | "base" | "small"
    pub whisper_model: String,
    /// System prompt sent to Groq before the transcribed question
    pub system_prompt: String,
    /// Whether overlay is click-through
    pub overlay_click_through: bool,
    /// Dark/light theme
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            model: "llama-3.3-70b-versatile".into(),
            overlay_opacity: 0.88,
            hotkey_toggle_overlay: "CommandOrControl+Alt+Shift+O".into(),
            hotkey_toggle_listening: "CommandOrControl+Alt+Shift+L".into(),
            hotkey_toggle_click_through: "CommandOrControl+Alt+Shift+K".into(),
            hotkey_screenshot_solve: "CommandOrControl+Alt+Shift+S".into(),
            vad_sensitivity: 0.015,
            vad_silence_timeout: 1000,
            auto_start_listening: false,
            whisper_model: "tiny".into(),
            system_prompt: "You are an expert AI assistant helping someone in a technical interview. \
                Provide concise, accurate, and direct answers. \
                Focus on key points and structure your response clearly. \
                Keep answers under 200 words unless the question requires more depth.".into(),
            overlay_click_through: true,
            theme: "dark".into(),
        }
    }
}

// ── Pipeline message types ────────────────────────────────────────────────────

/// Raw audio chunk from WASAPI loopback capture (f32 PCM, interleaved)
pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Decoded speech segment ready for transcription
pub struct SpeechSegment {
    /// Mono f32 PCM at 16 kHz
    pub pcm_16khz: Vec<f32>,
}

// ── Application Status ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AppStatus {
    Idle,
    Listening,
    Transcribing,
    Thinking,
    Error(String),
}

// ── Shared App State ──────────────────────────────────────────────────────────

pub struct AppStateInner {
    pub status: AppStatus,
    pub is_listening: bool,
    pub current_transcript: String,
    pub current_answer: String,
    pub settings: Settings,
    /// Channel sender to signal the audio thread to stop
    pub audio_stop_tx: Option<Sender<()>>,
    /// Whether the overlay is currently visible
    pub overlay_visible: bool,
    /// Path to the whisper model file
    pub model_path: Option<String>,
}

impl Default for AppStateInner {
    fn default() -> Self {
        Self {
            status: AppStatus::Idle,
            is_listening: false,
            current_transcript: String::new(),
            current_answer: String::new(),
            settings: Settings::default(),
            audio_stop_tx: None,
            overlay_visible: false,
            model_path: None,
        }
    }
}

#[derive(Clone)]
pub struct AppState(pub Arc<Mutex<AppStateInner>>);

impl AppState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(AppStateInner::default())))
    }
}

fn default_silence_timeout() -> u32 {
    1000
}

fn default_click_through_hotkey() -> String {
    "CommandOrControl+Alt+Shift+K".to_string()
}

fn default_screenshot_solve_hotkey() -> String {
    "CommandOrControl+Alt+Shift+S".to_string()
}
