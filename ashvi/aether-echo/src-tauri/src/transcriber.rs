// AetherEcho — Whisper Transcription Engine
// Uses whisper-rs to run local transcription via whisper.cpp.
// Runs on a dedicated blocking thread to avoid blocking the async runtime.

use anyhow::{Context, Result};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState};

use crate::state::SpeechSegment;

#[allow(dead_code)]
pub struct Transcriber {
    _ctx: WhisperContext,
    /// Cached state — avoids re-allocating ~200 MB of GGML compute buffers
    /// on every transcription call (the biggest single source of latency on CPU).
    state: WhisperState,
}

// SAFETY: WhisperContext and WhisperState are only accessed from the single
// pipeline blocking thread. They are stored together in the struct.
unsafe impl Send for Transcriber {}
unsafe impl Sync for Transcriber {}

impl Transcriber {
    /// Load a whisper model from the given path and pre-allocate inference state.
    pub fn new(model_path: &str) -> Result<Self> {
        log::info!("Loading whisper model from: {model_path}");

        if !Path::new(model_path).exists() {
            return Err(anyhow::anyhow!(
                "Whisper model not found at: {model_path}\n\
                 Please download a model from https://huggingface.co/ggerganov/whisper.cpp"
            ));
        }

        let ctx_params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(model_path, ctx_params)
            .context("Failed to load whisper model")?;

        // Pre-allocate the state once — this allocates the ~200 MB of GGML
        // compute buffers. By caching it we avoid repeating this work every call.
        let state = ctx.create_state().context("Failed to create whisper state")?;

        log::info!("✅ Whisper model loaded (state pre-allocated, ready for fast inference)");
        Ok(Self { _ctx: ctx, state })
    }

    /// Transcribe a speech segment. Returns the detected text.
    /// Input must be mono f32 PCM at 16 kHz.
    pub fn transcribe(&mut self, segment: &SpeechSegment) -> Result<String> {
        let samples = &segment.pcm_16khz;

        if samples.len() < 1600 {
            // Less than 100ms of audio — too short, skip
            return Ok(String::new());
        }

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // whisper.cpp performs best with exactly 4 threads on CPU.
        // More threads causes GGML memory bandwidth saturation and slows decoding.
        params.set_n_threads(4);

        // Language & transcription settings
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_token_timestamps(false);

        // Reuse the cached state — no buffer re-allocation, runs immediately
        let t0 = std::time::Instant::now();
        self.state
            .full(params, samples)
            .context("Whisper transcription failed")?;
        let whisper_ms = t0.elapsed().as_millis();

        let num_segments = self.state.full_n_segments().context("Failed to get segment count")?;

        let mut text = String::new();
        for i in 0..num_segments {
            // full_get_segment_text returns Result<String> in whisper-rs 0.13
            match self.state.full_get_segment_text(i) {
                Ok(s) => {
                    text.push_str(&s);
                    text.push(' ');
                }
                Err(e) => log::warn!("Segment {i} text error: {e}"),
            }
        }

        let trimmed = text.trim().to_string();
        let audio_secs = samples.len() as f32 / 16000.0;
        log::info!(
            "Transcribed ({:.1}s audio in {}ms, {:.1}x realtime): {:?}",
            audio_secs, whisper_ms,
            audio_secs / (whisper_ms as f32 / 1000.0).max(0.001),
            trimmed
        );
        Ok(trimmed)
    }
}

// ── Model Management ──────────────────────────────────────────────────────────

/// Returns the expected path for the whisper model in the app data directory.
pub fn model_path(app_data_dir: &str, model_name: &str) -> String {
    format!("{app_data_dir}\\models\\ggml-{model_name}.bin")
}

/// Check whether a given model file exists.
pub fn model_exists(app_data_dir: &str, model_name: &str) -> bool {
    Path::new(&model_path(app_data_dir, model_name)).exists()
}

/// Download a whisper model from Hugging Face.
/// Sends progress (0.0 – 1.0) via the progress callback.
pub async fn download_model(
    app_data_dir: &str,
    model_name: &str,
    on_progress: impl Fn(f32) + Send + 'static,
) -> Result<String> {
    use tokio::io::AsyncWriteExt;

    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{model_name}.bin"
    );

    let model_dir = format!("{app_data_dir}\\models");
    tokio::fs::create_dir_all(&model_dir)
        .await
        .context("Failed to create models directory")?;

    let dest_path = model_path(app_data_dir, model_name);

    log::info!("Downloading whisper model '{model_name}' from {url}");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?;

    let resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to start model download")?;

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Model download failed with status: {}",
            resp.status()
        ));
    }

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = tokio::fs::File::create(&dest_path)
        .await
        .context("Failed to create model file")?;

    let mut stream = resp.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Download stream error")?;
        file.write_all(&chunk).await.context("Write error")?;
        downloaded += chunk.len() as u64;

        if total > 0 {
            let pct = downloaded as f32 / total as f32;
            on_progress(pct);
        }
    }

    file.flush().await?;
    on_progress(1.0);
    log::info!("✅ Model downloaded to {dest_path}");
    Ok(dest_path)
}
