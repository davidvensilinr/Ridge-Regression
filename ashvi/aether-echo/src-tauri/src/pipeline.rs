// AetherEcho — Audio Processing Pipeline
// Connects: AudioCapture → VAD → Groq Whisper API → Groq LLM → UI events
// Transcription is done via Groq's Whisper API (sub-second, no local CPU).

use anyhow::Result;
use crossbeam_channel::bounded;
use tauri::{AppHandle, Emitter};
use tokio::task;

use crate::{
    audio,
    groq_client::GroqClient,
    state::{AppState, AppStatus, Settings},
    vad::{Vad, VadConfig},
};

/// Start the full audio → VAD → Groq Whisper → Groq LLM pipeline.
pub async fn start_pipeline(
    app: AppHandle,
    state: AppState,
    api_key: String,
    _model_path: String,   // kept for API compatibility; local model no longer used
    settings: Settings,
) -> Result<()> {
    // Channel: audio capture → VAD (bounded to ~2 seconds of audio)
    let (audio_tx, audio_rx) = bounded::<crate::state::AudioChunk>(50);
    // Stop signal channel
    let (stop_tx, stop_rx) = bounded::<()>(1);

    // Store the stop sender so commands can stop the pipeline
    {
        let mut lock = state.0.lock().unwrap();
        lock.audio_stop_tx = Some(stop_tx);
        lock.is_listening = true;
        lock.status = AppStatus::Listening;
        lock.current_transcript = String::new();
        lock.current_answer = String::new();
    }

    // Emit status update to frontend
    let _ = app.emit("status-changed", serde_json::json!({
        "is_listening": true,
        "status": "listening"
    }));

    // ── Thread 1: WASAPI loopback capture (OS thread, not tokio) ──────────────
    let _audio_thread = audio::start_loopback_capture(audio_tx, stop_rx.clone())
        .map_err(|e| anyhow::anyhow!("Failed to start audio capture: {e}"))?;

    // ── Task: VAD + Groq Whisper + Groq LLM (all async, no blocking threads) ──
    let app_clone = app.clone();
    let state_clone = state.clone();
    let vad_config = VadConfig {
        threshold: settings.vad_sensitivity,
        silence_chunks_to_end: (settings.vad_silence_timeout as usize / 100).max(1),
        ..Default::default()
    };

    task::spawn(async move {
        let groq = match GroqClient::new(api_key) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to create Groq client: {e}");
                emit_error(&app_clone, &format!("Groq client error: {e}"));
                set_stopped(&state_clone, &app_clone);
                return;
            }
        };

        log::info!("Pipeline ready — listening for speech (using Groq Whisper API)");
        let mut vad = Vad::new(vad_config);

        // Process audio chunks until the channel is closed or stop signal
        loop {
            // Check stop flag
            let still_listening = state_clone.0.lock().map(|l| l.is_listening).unwrap_or(false);
            if !still_listening {
                break;
            }

            // Non-blocking receive with timeout — yields back to tokio between chunks
            let chunk = match task::spawn_blocking({
                let audio_rx = audio_rx.clone();
                move || audio_rx.recv_timeout(std::time::Duration::from_millis(200))
            }).await {
                Ok(Ok(c)) => c,
                Ok(Err(crossbeam_channel::RecvTimeoutError::Timeout)) => continue,
                Ok(Err(crossbeam_channel::RecvTimeoutError::Disconnected)) => {
                    log::info!("Audio channel disconnected — stopping pipeline");
                    break;
                }
                Err(_) => break,
            };

            // VAD: accumulate until a speech segment is complete
            if let Some(segment) = vad.process_chunk(chunk) {
                // Update status to transcribing
                {
                    if let Ok(mut lock) = state_clone.0.lock() {
                        lock.status = AppStatus::Transcribing;
                    }
                }
                let _ = app_clone.emit("status-changed", serde_json::json!({
                    "status": "transcribing"
                }));

                // Transcribe via Groq Whisper API — fast, async, no CPU blocking
                let transcript = match groq.transcribe_audio(&segment.pcm_16khz).await {
                    Ok(text) => text,
                    Err(e) => {
                        log::warn!("Whisper API error: {e}");
                        emit_error(&app_clone, &format!("Transcription error: {e}"));
                        if let Ok(mut lock) = state_clone.0.lock() {
                            lock.status = AppStatus::Listening;
                        }
                        continue;
                    }
                };

                if transcript.is_empty() || transcript.len() < 5 {
                    // Too short — likely silence or noise
                    if let Ok(mut lock) = state_clone.0.lock() {
                        lock.status = AppStatus::Listening;
                    }
                    continue;
                }

                // Emit transcript to UI
                {
                    if let Ok(mut lock) = state_clone.0.lock() {
                        lock.current_transcript = transcript.clone();
                        lock.status = AppStatus::Thinking;
                    }
                }
                let _ = app_clone.emit("transcript-updated", &transcript);
                let _ = app_clone.emit("status-changed", serde_json::json!({
                    "status": "thinking",
                    "transcript": &transcript
                }));

                // Query Groq LLM for an answer
                let (model, system_prompt) = {
                    let lock = state_clone.0.lock().unwrap();
                    (lock.settings.model.clone(), lock.settings.system_prompt.clone())
                };

                match groq.get_answer(&transcript, &model, &system_prompt).await {
                    Ok(answer) => {
                        {
                            if let Ok(mut lock) = state_clone.0.lock() {
                                lock.current_answer = answer.clone();
                                lock.status = AppStatus::Listening;
                            }
                        }
                        let _ = app_clone.emit("answer-ready", serde_json::json!({
                            "transcript": &transcript,
                            "answer": &answer
                        }));
                        let _ = app_clone.emit("status-changed", serde_json::json!({
                            "status": "listening"
                        }));
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        log::warn!("Groq LLM error: {msg}");
                        emit_error(&app_clone, &msg);
                        if let Ok(mut lock) = state_clone.0.lock() {
                            lock.status = AppStatus::Listening;
                        }
                    }
                }
            }
        }

        set_stopped(&state_clone, &app_clone);
        log::info!("Pipeline stopped");
    });

    Ok(())
}

/// Stop the pipeline by sending a stop signal and clearing state.
pub fn stop_pipeline(state: AppState) {
    let mut lock = state.0.lock().unwrap();
    // Send stop signal to audio thread
    if let Some(tx) = lock.audio_stop_tx.take() {
        let _ = tx.try_send(());
    }
    lock.is_listening = false;
    lock.status = AppStatus::Idle;
}

fn emit_error(app: &AppHandle, msg: &str) {
    let _ = app.emit("pipeline-error", msg);
    let _ = app.emit("status-changed", serde_json::json!({
        "status": "error",
        "error": msg
    }));
}

fn set_stopped(state: &AppState, app: &AppHandle) {
    if let Ok(mut lock) = state.0.lock() {
        lock.is_listening = false;
        lock.status = AppStatus::Idle;
    }
    let _ = app.emit("status-changed", serde_json::json!({
        "is_listening": false,
        "status": "idle"
    }));
}
