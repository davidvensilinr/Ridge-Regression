// AetherEcho — Groq API Client
// Sends transcribed text to Groq's LLM API and returns AI answers.
// Also handles audio transcription via Groq's Whisper API endpoint.
// Uses OpenAI-compatible endpoints with reqwest.

use anyhow::{Context, Result};
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};

const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/chat/completions";
const GROQ_WHISPER_URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
}

// ── API Error types ───────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum GroqError {
    #[error("Invalid API key — check your Groq API key in Settings")]
    InvalidApiKey,
    #[error("Rate limit exceeded — please wait a moment before trying again")]
    RateLimited,
    #[error("Model not available: {0}")]
    ModelNotFound(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },
}

// ── Groq Client ───────────────────────────────────────────────────────────────

pub struct GroqClient {
    http: Client,
    api_key: String,
}

impl GroqClient {
    pub fn new(api_key: String) -> Result<Self> {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("AetherEcho/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { http, api_key })
    }

    /// Send a transcribed question to Groq and return the AI's answer.
    pub async fn get_answer(
        &self,
        question: &str,
        model: &str,
        system_prompt: &str,
    ) -> Result<String, GroqError> {
        let messages = vec![
            Message {
                role: "system".into(),
                content: system_prompt.into(),
            },
            Message {
                role: "user".into(),
                content: question.into(),
            },
        ];

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            max_tokens: 512,
            temperature: 0.7,
            stream: false,
        };

        let response = self
            .http
            .post(GROQ_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if status == 401 {
            return Err(GroqError::InvalidApiKey);
        }
        if status == 429 {
            return Err(GroqError::RateLimited);
        }
        if status == 404 {
            return Err(GroqError::ModelNotFound(model.to_string()));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GroqError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        let chat_resp: ChatResponse = response.json().await?;

        let answer = chat_resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(answer)
    }

    /// Send a Base64-encoded screen capture to Groq's Vision API and return the AI's answer/solution.
    pub async fn solve_vision_screenshot(&self, base64_image: &str) -> Result<String, GroqError> {
        let request_body = serde_json::json!({
            "model": "meta-llama/llama-4-scout-17b-16e-instruct",
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "Solve the problem or answer the question visible on this screen. Be direct and concise. Limit your answer to under 200 words."
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/jpeg;base64,{}", base64_image)
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 1024,
            "temperature": 0.2
        });

        let response = self
            .http
            .post(GROQ_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();

        if status == 401 {
            return Err(GroqError::InvalidApiKey);
        }
        if status == 429 {
            return Err(GroqError::RateLimited);
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GroqError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        let chat_resp: ChatResponse = response.json().await?;

        let answer = chat_resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(answer)
    }

    /// Test the API key by sending a minimal request to Groq.
    pub async fn test_connection(api_key: &str) -> Result<bool, GroqError> {
        let client = GroqClient::new(api_key.to_string())
            .map_err(|e| GroqError::Api { status: 0, message: e.to_string() })?;

        let result = client
            .get_answer("Say 'OK' in one word.", "llama-3.1-8b-instant", "You are a test assistant.")
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(GroqError::InvalidApiKey) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Transcribe audio using Groq's Whisper API (whisper-large-v3-turbo).
    /// Input: mono f32 PCM samples at 16 kHz.
    /// Much faster than local CPU inference — typically < 1 second.
    pub async fn transcribe_audio(&self, pcm_16khz: &[f32]) -> Result<String, GroqError> {
        // Encode as WAV in memory — no temp files needed
        let wav_bytes = encode_wav(pcm_16khz).map_err(|e| GroqError::Api {
            status: 0,
            message: format!("WAV encoding failed: {e}"),
        })?;

        let t0 = std::time::Instant::now();

        let file_part = multipart::Part::bytes(wav_bytes)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| GroqError::Network(e))?;

        let form = multipart::Form::new()
            .part("file", file_part)
            .text("model", "whisper-large-v3-turbo")
            .text("language", "en")
            .text("response_format", "json");

        let response = self
            .http
            .post(GROQ_WHISPER_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        let status = response.status();
        log::info!("Groq Whisper transcription returned in {}ms", t0.elapsed().as_millis());

        if status == 401 {
            return Err(GroqError::InvalidApiKey);
        }
        if status == 429 {
            return Err(GroqError::RateLimited);
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GroqError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        let resp: WhisperResponse = response.json().await?;
        Ok(resp.text.trim().to_string())
    }
}

// ── Available Models ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: u32,
}

pub fn available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "llama-3.3-70b-versatile".into(),
            name: "Llama 3.3 70B (Best Quality)".into(),
            context_window: 128000,
        },
        ModelInfo {
            id: "llama-3.1-8b-instant".into(),
            name: "Llama 3.1 8B (Fastest)".into(),
            context_window: 128000,
        },
        ModelInfo {
            id: "llama3-70b-8192".into(),
            name: "Llama 3 70B".into(),
            context_window: 8192,
        },
        ModelInfo {
            id: "mixtral-8x7b-32768".into(),
            name: "Mixtral 8x7B".into(),
            context_window: 32768,
        },
        ModelInfo {
            id: "gemma2-9b-it".into(),
            name: "Gemma 2 9B".into(),
            context_window: 8192,
        },
    ]
}

// ── WAV Encoder ───────────────────────────────────────────────────────────────

/// Encode mono f32 PCM samples (16 kHz) into a 16-bit PCM WAV byte buffer.
/// Groq's Whisper API accepts standard WAV — no extra dependencies needed.
fn encode_wav(samples: &[f32]) -> anyhow::Result<Vec<u8>> {
    let sample_rate: u32 = 16000;
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;
    let pcm_data_len = samples.len() * 2; // 2 bytes per i16 sample

    let mut buf = Vec::with_capacity(44 + pcm_data_len);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&((36 + pcm_data_len) as u32).to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());         // chunk size
    buf.extend_from_slice(&1u16.to_le_bytes());          // PCM format
    buf.extend_from_slice(&channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&(pcm_data_len as u32).to_le_bytes());

    // Convert f32 [-1.0, 1.0] to i16
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let sample_i16 = (clamped * i16::MAX as f32) as i16;
        buf.extend_from_slice(&sample_i16.to_le_bytes());
    }

    Ok(buf)
}
