// AetherEcho — Voice Activity Detection (VAD)
// Energy-based RMS VAD with configurable silence threshold.
// Accumulates audio chunks and emits speech segments when speech ends.

use crate::state::{AudioChunk, SpeechSegment};
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

/// VAD configuration
pub struct VadConfig {
    /// RMS energy threshold to detect speech (tune per environment)
    pub threshold: f32,
    /// Number of consecutive silent chunks before declaring speech ended (~200ms at 100ms chunks)
    pub silence_chunks_to_end: usize,
    /// Minimum chunks with speech before we start recording (~100ms)
    pub min_speech_chunks: usize,
    /// Maximum speech buffer duration in seconds before forcing flush
    pub max_duration_secs: f32,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            threshold: 0.015,
            silence_chunks_to_end: 4,   // ~400ms of silence ends a segment (was 800ms)
            min_speech_chunks: 2,        // ~200ms minimum speech
            max_duration_secs: 8.0,      // Force flush after 8s (was 30s)
        }
    }
}

pub struct Vad {
    config: VadConfig,
    speech_buffer: Vec<f32>,      // Raw samples accumulated during speech
    silent_chunk_count: usize,
    speech_chunk_count: usize,
    is_speaking: bool,
    source_sample_rate: u32,
    source_channels: u16,
    /// Accumulator for partial silence (so we don't cut trailing audio)
    trailing_buffer: Vec<f32>,
}

impl Vad {
    pub fn new(config: VadConfig) -> Self {
        Self {
            config,
            speech_buffer: Vec::new(),
            silent_chunk_count: 0,
            speech_chunk_count: 0,
            is_speaking: false,
            source_sample_rate: 48000,
            source_channels: 2,
            trailing_buffer: Vec::new(),
        }
    }

    /// Process one audio chunk. Returns a completed SpeechSegment if one is ready.
    pub fn process_chunk(&mut self, chunk: AudioChunk) -> Option<SpeechSegment> {
        self.source_sample_rate = chunk.sample_rate;
        self.source_channels = chunk.channels;

        let energy = rms_energy(&chunk.samples);
        let is_speech = energy > self.config.threshold;

        if is_speech {
            self.silent_chunk_count = 0;
            self.speech_chunk_count += 1;
            self.is_speaking = true;
            // Add any buffered trailing audio
            self.speech_buffer.extend_from_slice(&self.trailing_buffer);
            self.trailing_buffer.clear();
            self.speech_buffer.extend_from_slice(&chunk.samples);

            // Force flush if too long
            let samples_max = (self.config.max_duration_secs
                * self.source_sample_rate as f32
                * self.source_channels as f32) as usize;
            if self.speech_buffer.len() >= samples_max {
                return self.flush_segment();
            }
        } else if self.is_speaking {
            self.silent_chunk_count += 1;
            // Keep trailing audio in case speech resumes
            self.trailing_buffer.extend_from_slice(&chunk.samples);

            if self.silent_chunk_count >= self.config.silence_chunks_to_end {
                if self.speech_chunk_count >= self.config.min_speech_chunks {
                    return self.flush_segment();
                } else {
                    // Too short — discard
                    self.reset();
                }
            }
        }

        None
    }

    fn flush_segment(&mut self) -> Option<SpeechSegment> {
        let samples = std::mem::take(&mut self.speech_buffer);
        self.reset();

        if samples.is_empty() {
            return None;
        }

        // Convert to mono and resample to 16kHz for Whisper
        match to_mono_16khz(samples, self.source_sample_rate, self.source_channels) {
            Ok(pcm_16khz) => Some(SpeechSegment { pcm_16khz }),
            Err(e) => {
                log::warn!("Audio resampling failed: {e}");
                None
            }
        }
    }

    fn reset(&mut self) {
        self.speech_buffer.clear();
        self.trailing_buffer.clear();
        self.silent_chunk_count = 0;
        self.speech_chunk_count = 0;
        self.is_speaking = false;
    }
}

/// Compute RMS (root-mean-square) energy of a PCM buffer.
fn rms_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Convert interleaved multi-channel PCM to mono f32 at 16 kHz.
/// Whisper expects mono 16 kHz f32 PCM.
fn to_mono_16khz(
    samples: Vec<f32>,
    source_rate: u32,
    source_channels: u16,
) -> anyhow::Result<Vec<f32>> {
    // Step 1: Downmix to mono
    let mono: Vec<f32> = if source_channels == 1 {
        samples
    } else {
        let ch = source_channels as usize;
        samples
            .chunks_exact(ch)
            .map(|frame| frame.iter().sum::<f32>() / ch as f32)
            .collect()
    };

    // Step 2: Resample to 16 kHz if needed
    if source_rate == 16000 {
        return Ok(mono);
    }

    let params = SincInterpolationParameters {
        sinc_len: 64,                          // was 256 — 64 is fine for speech
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 128,              // was 256 — 128 is plenty for 16kHz speech
        window: WindowFunction::BlackmanHarris2,
    };

    let resample_ratio = 16000.0 / source_rate as f64;
    let chunk_size = 1024;

    let mut resampler = SincFixedIn::<f32>::new(
        resample_ratio,
        2.0,
        params,
        chunk_size,
        1, // mono
    )?;

    // Pad input to a multiple of chunk_size
    let mut padded = mono.clone();
    let rem = padded.len() % chunk_size;
    if rem != 0 {
        padded.resize(padded.len() + (chunk_size - rem), 0.0);
    }

    let mut output = Vec::new();
    for chunk in padded.chunks(chunk_size) {
        let wave_in = vec![chunk.to_vec()];
        let wave_out = resampler.process(&wave_in, None)?;
        output.extend_from_slice(&wave_out[0]);
    }

    // Trim trailing silence added by padding
    let expected_len = (mono.len() as f64 * resample_ratio) as usize;
    output.truncate(expected_len + 512);

    Ok(output)
}
