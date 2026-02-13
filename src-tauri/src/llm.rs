use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum LlmError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("unexpected response format")]
    Format,
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
        "formal" => "Use formal, professional tone. Proper punctuation and complete sentences.",
        "casual" => "Use casual, conversational tone. Keep it natural and concise.",
        "technical" => "Use precise technical language. Preserve code terms and technical jargon exactly.",
        _ => "Use a natural, clear tone.",
    };

    format!(
        r#"You are a dictation post-processor. Clean up the raw speech-to-text output and return ONLY the cleaned text with no explanations.

Rules:
1. Remove filler words: um, uh, er, like, you know, I mean, so, well, hmm, 嗯, 啊, 呃, 那個, 就是, 然後, 對, 齁, 蛤, 喔
2. Remove false starts, repetitions, and self-corrections (keep only the final intended version)
3. Add proper punctuation and formatting (paragraphs, lists where appropriate)
4. Convert any Simplified Chinese characters to Traditional Chinese (zh-TW)
5. {tone_instruction}
6. Do NOT add any content that wasn't in the original speech
7. Do NOT wrap the output in quotes or add commentary
8. Preserve the original language(s) used by the speaker"#
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
        "temperature": 0.3,
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
