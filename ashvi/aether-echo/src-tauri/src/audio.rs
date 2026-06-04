// AetherEcho — WASAPI System Audio Loopback Capture (Windows)
// Captures what comes out of the speakers (interviewer's voice, meeting audio)
// using the Windows Audio Session API (WASAPI) in loopback mode.
//
// Unlike cpal, the `wasapi` crate exposes AUDCLNT_STREAMFLAGS_LOOPBACK which
// lets us record exactly what the system is playing, with no virtual cable needed.

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender};
use std::thread;
use wasapi::*;

use crate::state::AudioChunk;

/// Start the loopback capture thread.
///
/// Sends `AudioChunk` structs over `audio_tx` until a stop signal arrives on
/// `stop_rx`. The thread is spawned as a regular OS thread (not tokio) because
/// WASAPI requires COM to be initialized on its own STA thread.
pub fn start_loopback_capture(
    audio_tx: Sender<AudioChunk>,
    stop_rx: Receiver<()>,
) -> Result<thread::JoinHandle<()>> {
    let handle = thread::Builder::new()
        .name("aether-audio-capture".into())
        .spawn(move || {
            if let Err(e) = capture_loop(audio_tx, stop_rx) {
                log::error!("Audio capture error: {e:#}");
            }
        })
        .context("Failed to spawn audio capture thread")?;
    Ok(handle)
}

fn capture_loop(audio_tx: Sender<AudioChunk>, stop_rx: Receiver<()>) -> Result<()> {
    // Initialize COM on this thread (required for WASAPI)
    initialize_mta().ok().context("Failed to initialize COM MTA")?;

    // Get the default render (output/speakers) device
    let device = get_default_device(&Direction::Render)
        .map_err(|e| anyhow::anyhow!("Wasapi error: {e:?}"))
        .context("No default render device found — is a speaker/headphone connected?")?;

    log::info!(
        "Loopback capturing from: {}",
        device.get_friendlyname().unwrap_or_else(|_| "Unknown Device".into())
    );

    // Open the audio client
    let mut audio_client = device
        .get_iaudioclient()
        .map_err(|e| anyhow::anyhow!("Wasapi error: {e:?}"))
        .context("Failed to get IAudioClient")?;

    // Get the device's native mix format
    let wave_format = audio_client
        .get_mixformat()
        .map_err(|e| anyhow::anyhow!("Wasapi error: {e:?}"))
        .context("Failed to get mix format")?;

    let sample_rate = wave_format.get_samplespersec();
    let channels = wave_format.get_nchannels();

    log::info!(
        "Audio format: {}Hz, {} channel(s), {} bit",
        sample_rate,
        channels,
        wave_format.get_bitspersample()
    );

    // 100ms buffer duration in 100-nanosecond intervals
    let buffer_duration_100ns: i64 = 1_000_000;

    // Initialize in loopback mode (Capture direction + loopback=true)
    audio_client
        .initialize_client(
            &wave_format,
            buffer_duration_100ns,
            &Direction::Capture,
            &ShareMode::Shared,
            true, // loopback = true ← this is what captures speaker output
        )
        .map_err(|e| anyhow::anyhow!("Wasapi error: {e:?}"))
        .context("Failed to initialize audio client for loopback")?;

    let capture_client = audio_client
        .get_audiocaptureclient()
        .map_err(|e| anyhow::anyhow!("Wasapi error: {e:?}"))
        .context("Failed to get IAudioCaptureClient")?;

    // Create the event handle for buffer-ready notifications
    let h_event = audio_client
        .set_get_eventhandle()
        .map_err(|e| anyhow::anyhow!("Wasapi error: {e:?}"))
        .context("Failed to create WASAPI event handle")?;

    audio_client
        .start_stream()
        .map_err(|e| anyhow::anyhow!("Wasapi error: {e:?}"))
        .context("Failed to start audio stream")?;
    log::info!("✅ WASAPI loopback capture started — capturing speaker output");

    loop {
        // Check for stop signal (non-blocking)
        if stop_rx.try_recv().is_ok() {
            log::info!("Audio capture stop signal received");
            break;
        }

        // Wait up to 200ms for a buffer-ready event
        match h_event.wait_for_event(200) {
            Ok(_) => {}
            Err(e) => {
                if e.to_string().contains("Wait timed out") {
                    continue;
                }
                log::warn!("WASAPI wait error: {e}");
                continue;
            }
        }

        // Drain all available packets
        loop {
            let nbr_frames = match capture_client.get_next_nbr_frames() {
                Ok(Some(0)) | Ok(None) => break,
                Ok(Some(n)) => n,
                Err(e) => {
                    log::warn!("get_next_nbr_frames error: {e}");
                    break;
                }
            };

            let bytes_per_frame = wave_format.get_blockalign() as usize;
            let mut buffer = vec![0u8; nbr_frames as usize * bytes_per_frame];

            match capture_client.read_from_device(&mut buffer) {
                Ok((_frames, _flags)) => {
                    let samples_f32 = bytes_to_f32_samples(&buffer, wave_format.get_bitspersample());
                    if !samples_f32.is_empty() {
                        let chunk = AudioChunk {
                            samples: samples_f32,
                            sample_rate,
                            channels,
                        };
                        // Send to VAD/transcription pipeline (discard if full)
                        let _ = audio_tx.try_send(chunk);
                    }
                }
                Err(e) => {
                    log::warn!("read_from_device error: {e}");
                    break;
                }
            }
        }
    }

    audio_client.stop_stream().ok();
    log::info!("Audio capture stopped");
    Ok(())
}

fn bytes_to_f32_samples(bytes: &[u8], bits_per_sample: u16) -> Vec<f32> {
    match bits_per_sample {
        32 => {
            // 32-bit float
            bytes.chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect()
        }
        16 => {
            // 16-bit PCM integer
            bytes.chunks_exact(2)
                .map(|chunk| {
                    let val = i16::from_le_bytes([chunk[0], chunk[1]]);
                    val as f32 / 32768.0
                })
                .collect()
        }
        _ => {
            log::warn!("Unsupported bits per sample: {}", bits_per_sample);
            Vec::new()
        }
    }
}

/// List all available render (output) audio devices.
pub fn list_audio_output_devices() -> Vec<(String, String)> {
    if initialize_mta().is_err() {
        return vec![];
    }
    let Ok(collection) = DeviceCollection::new(&Direction::Render) else {
        return vec![];
    };
    let count = collection.get_nbr_devices().unwrap_or(0);
    let mut result = Vec::new();
    for i in 0..count {
        if let Ok(dev) = collection.get_device_at_index(i) {
            let name = dev
                .get_friendlyname()
                .unwrap_or_else(|_| format!("Device {i}"));
            result.push((format!("device_{i}"), name));
        }
    }
    result
}
