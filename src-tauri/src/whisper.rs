use thiserror::Error;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::audio;

/// Segment no-speech probability above this → skip segment (also passed to whisper params).
const NO_SPEECH_THRESHOLD: f32 = 0.6;
/// Minimum average token probability to accept a transcription result.
const CONFIDENCE_THRESHOLD: f64 = 0.4;

fn calculate_threads(available: usize) -> i32 {
    if available <= 4 {
        // Use all available threads if count is low (but at least 1)
        std::cmp::max(1, available as i32)
    } else {
        // Reserve 2 threads for system/UI to keep app responsive
        // Cap at 8 to avoid diminishing returns/overhead
        let threads = available.saturating_sub(2);
        std::cmp::min(threads, 8) as i32
    }
}

fn optimal_threads() -> i32 {
    let available = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    calculate_threads(available)
}

#[derive(Debug, Error)]
pub(crate) enum WhisperError {
    #[error("failed to load whisper model: {0}")]
    ModelLoad(String),
    #[error("failed to create whisper state: {0}")]
    StateCreate(String),
    #[error("transcription failed: {0}")]
    Transcription(String),
}

impl serde::Serialize for WhisperError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub(crate) struct TranscriptionEngine {
    ctx: WhisperContext,
}

// WhisperContext is Send+Sync via its Arc<WhisperInnerContext>
unsafe impl Send for TranscriptionEngine {}
unsafe impl Sync for TranscriptionEngine {}

impl TranscriptionEngine {
    pub(crate) fn new(model_path: &str) -> Result<Self, WhisperError> {
        let mut params = WhisperContextParameters::new();
        params.use_gpu(true); // Metal (macOS) or CUDA (Windows)
        let ctx = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| WhisperError::ModelLoad(e.to_string()))?;
        let engine = Self { ctx };
        engine.warmup()?;
        Ok(engine)
    }

    /// Run a short dummy inference to warm up CUDA/Metal kernels.
    /// Without this, the first real transcription is very slow due to JIT compilation.
    fn warmup(&self) -> Result<(), WhisperError> {
        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| WhisperError::StateCreate(e.to_string()))?;
        // 1 second of silence at 16kHz
        let dummy = vec![0.0f32; 16_000];
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(optimal_threads());
        params.set_language(Some("en"));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        state
            .full(params, &dummy)
            .map_err(|e| WhisperError::Transcription(e.to_string()))?;
        Ok(())
    }

    pub(crate) fn transcribe(
        &self,
        samples: &[f32],
        language: &str,
        initial_prompt: &str,
    ) -> Result<String, WhisperError> {
        if !audio::is_audio_usable(samples) {
            return Ok(String::new());
        }

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| WhisperError::StateCreate(e.to_string()))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(optimal_threads());
        params.set_language(Some(language));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);

        // Anti-hallucination settings
        params.set_suppress_blank(true);
        params.set_no_speech_thold(NO_SPEECH_THRESHOLD);
        params.set_temperature_inc(0.0); // disable temperature fallback — it just produces more hallucinations
        params.set_entropy_thold(2.4); // reject segments with high entropy (uncertain/hallucinated)

        if !initial_prompt.is_empty() {
            params.set_initial_prompt(initial_prompt);
        }

        state
            .full(params, samples)
            .map_err(|e| WhisperError::Transcription(e.to_string()))?;

        let n_segments = state.full_n_segments();
        let mut text = String::new();
        let mut total_token_prob = 0.0f64;
        let mut total_tokens = 0usize;

        for i in 0..n_segments {
            if let Some(segment) = state.get_segment(i) {
                let no_speech = segment.no_speech_probability();
                if no_speech > NO_SPEECH_THRESHOLD {
                    log::info!("skipping segment {i} with high no_speech_prob ({no_speech:.3})");
                    continue;
                }

                // Accumulate token probabilities for confidence scoring
                let n_tokens = segment.n_tokens();
                for t in 0..n_tokens {
                    if let Some(token) = segment.get_token(t as std::ffi::c_int) {
                        total_token_prob += token.token_probability() as f64;
                        total_tokens += 1;
                    }
                }

                let segment_text = segment.to_str().map_err(|e: whisper_rs::WhisperError| {
                    WhisperError::Transcription(e.to_string())
                })?;
                text.push_str(segment_text);
            }
        }

        let trimmed = text.trim().to_string();

        // Confidence gate: reject if average token probability is too low
        if total_tokens > 0 {
            let avg_prob = total_token_prob / total_tokens as f64;
            log::info!("transcription confidence: avg_token_prob={avg_prob:.4}, tokens={total_tokens}, text={trimmed:?}");
            if avg_prob < CONFIDENCE_THRESHOLD {
                log::info!("rejected low-confidence transcription (avg_prob={avg_prob:.4})");
                return Ok(String::new());
            }
        }

        // Filter known Whisper hallucination patterns (common when no speech is present)
        if is_hallucination(&trimmed) {
            log::info!("filtered hallucinated text: {trimmed:?}");
            return Ok(String::new());
        }

        Ok(trimmed)
    }
}

/// Common Whisper hallucination phrases that appear when no real speech is present.
const HALLUCINATION_PATTERNS: &[&str] = &[
    "thank you for watching",
    "thanks for watching",
    "please subscribe",
    "like and subscribe",
    "see you next time",
    "see you in the next",
    "goodbye",
    "thank you for listening",
    "thanks for listening",
    "subtitles by",
    "translated by",
    "amara.org",
    "www.",
    "http",
    // CJK common hallucinations
    "謝謝觀看",
    "感謝觀看",
    "感謝收看",
    "請訂閱",
    "字幕",
    "谢谢观看",
    "感谢观看",
    "请订阅",
    "ご視聴ありがとうございました",
];

fn is_hallucination(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let lower = text.to_lowercase();
    let trimmed =
        lower.trim_matches(|c: char| c.is_whitespace() || c == '.' || c == '!' || c == ',');
    // Reject if only punctuation/whitespace remains after trimming
    if trimmed.is_empty() || trimmed.chars().all(|c| !c.is_alphanumeric()) {
        return true;
    }
    HALLUCINATION_PATTERNS
        .iter()
        .any(|pat| trimmed.contains(pat))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_threads() {
        let cases = vec![
            (1, 1),
            (2, 2),
            (3, 3),
            (4, 4),
            (5, 3),
            (6, 4),
            (8, 6),
            (10, 8),
            (12, 8),
            (16, 8),
            (32, 8),
        ];

        for (input, expected) in cases {
            assert_eq!(
                calculate_threads(input),
                expected,
                "failed for input {}",
                input
            );
        }
    }

    #[test]
    fn test_hallucination_filter() {
        assert!(is_hallucination("Thank you for watching."));
        assert!(is_hallucination("謝謝觀看"));
        assert!(is_hallucination("  thanks for watching!  "));
        assert!(is_hallucination("Subtitles by Amara.org"));
        assert!(is_hallucination("...")); // only punctuation after trim
        assert!(!is_hallucination("好")); // valid CJK single char
        assert!(!is_hallucination("OK")); // valid short word
        assert!(!is_hallucination("Hello, how are you today?"));
        assert!(!is_hallucination("The meeting is at 3pm"));
        assert!(!is_hallucination("")); // empty is not hallucination (handled elsewhere)
    }
}
