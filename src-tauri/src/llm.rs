use serde::{Deserialize, Serialize};
use std::io::Cursor;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum LlmError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("unexpected response format")]
    Format,
    #[error("audio encoding failed: {0}")]
    AudioEncode(String),
}

impl Serialize for LlmError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

fn build_system_prompt(style: &str) -> String {
    let tone_instruction = match style {
        "formal" => "Tone: formal and professional. Use complete sentences with proper structure.",
        "casual" => "Tone: casual and conversational. Keep it natural, concise, and friendly.",
        "technical" => "Tone: precise and technical. Preserve all code terms, variable names, CLI commands, and technical jargon exactly as spoken. Do not rephrase technical content.",
        _ => "Tone: natural and clear.",
    };

    format!(
        r#"You are a speech-to-text post-processor for a dictation app. Your job is to clean up raw transcription output. Return ONLY the cleaned text. No explanations, no quotes, no commentary.

## Core Rules

1. REMOVE filler words and verbal tics:
   - English: um, uh, er, erm, like, you know, I mean, so, well, hmm, right, okay so, basically
   - Chinese: 嗯、啊、呃、那個、就是、然後、對、齁、蛤、喔、欸、好、就是說、怎麼說、反正就是

2. REMOVE false starts, repetitions, and self-corrections. Keep only the speaker's final intended version.
   Example: "我想要去台北 不對 我想要去台中" → "我想要去台中"

3. FIX punctuation:
   - Chinese text uses full-width punctuation: ，、。！？：；（）「」
   - English text uses half-width punctuation: , . ! ? : ; ( ) " "
   - Add sentence-ending punctuation where missing
   - Insert commas at natural clause boundaries

4. CONVERT all Simplified Chinese to Traditional Chinese (zh-TW):
   - 机器 → 機器, 学习 → 學習, 运行 → 運行, 设置 → 設定, 视频 → 影片, 软件 → 軟體
   - Use Taiwan-standard vocabulary, not just character conversion (e.g. 信息 → 資訊, 內存 → 記憶體, 服務器 → 伺服器)

5. MIXED Chinese-English handling:
   - Add a space between Chinese characters and English words/numbers (e.g. "使用React框架" → "使用 React 框架")
   - Preserve English words, brand names, and technical terms exactly as spoken
   - Do not translate English words that the speaker intentionally used in English

6. FORMAT when appropriate:
   - If the speaker lists items ("第一...第二...第三..."), format as a numbered list
   - Break long monologues into paragraphs at natural topic boundaries
   - Keep short utterances as a single line

## Constraints

- Do NOT add content that wasn't in the original speech
- Do NOT summarize or paraphrase — preserve the speaker's wording
- Do NOT translate between languages — if the speaker used English, keep it in English
- Minimal editing: fix only what's clearly wrong, preserve the speaker's voice
- {tone_instruction}"#
    )
}

pub(crate) async fn process_text(
    api_key: &str,
    model: &str,
    text: &str,
    style: &str,
) -> Result<String, LlmError> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": build_system_prompt(style)
            },
            {
                "role": "user",
                "content": text
            }
        ],
        "temperature": 0.1,
        "max_tokens": 4096,
    });

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        return Err(LlmError::Api(format!("{status}: {body_text}")));
    }

    let chat: ChatResponse = response.json().await?;
    let content = chat
        .choices
        .into_iter()
        .next()
        .ok_or(LlmError::Format)?
        .message
        .content;

    Ok(content.trim().to_string())
}

// --- Groq Whisper API transcription ---

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: String,
}

/// Encodes f32 PCM samples (16kHz mono) into a WAV byte buffer.
fn encode_wav(samples: &[f32]) -> Result<Vec<u8>, LlmError> {
    let mut buf = Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer =
        hound::WavWriter::new(&mut buf, spec).map_err(|e| LlmError::AudioEncode(e.to_string()))?;
    for &s in samples {
        let val = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer
            .write_sample(val)
            .map_err(|e| LlmError::AudioEncode(e.to_string()))?;
    }
    writer
        .finalize()
        .map_err(|e| LlmError::AudioEncode(e.to_string()))?;
    Ok(buf.into_inner())
}

/// Transcribes audio via Groq Whisper API.
pub(crate) async fn transcribe_groq(
    api_key: &str,
    samples: &[f32],
    language: &str,
    initial_prompt: &str,
) -> Result<String, LlmError> {
    let wav_bytes = encode_wav(samples)?;

    let file_part = reqwest::multipart::Part::bytes(wav_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| LlmError::Api(e.to_string()))?;

    let mut form = reqwest::multipart::Form::new()
        .text("model", "whisper-large-v3-turbo".to_string())
        .text("response_format", "json".to_string())
        .part("file", file_part);

    if language != "auto" {
        form = form.text("language", language.to_string());
    }

    if !initial_prompt.is_empty() {
        form = form.text("prompt", initial_prompt.to_string());
    }

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        return Err(LlmError::Api(format!("{status}: {body_text}")));
    }

    let result: TranscriptionResponse = response.json().await?;
    Ok(result.text.trim().to_string())
}
