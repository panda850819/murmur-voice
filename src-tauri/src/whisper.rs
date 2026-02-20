use thiserror::Error;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const MIN_SAMPLES: usize = 16_000; // 1s at 16kHz — shorter clips produce hallucinations

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
        engine.warmup();
        Ok(engine)
    }

    /// Run a short dummy inference to warm up CUDA/Metal kernels.
    /// Without this, the first real transcription is very slow due to JIT compilation.
    fn warmup(&self) {
        let Ok(mut state) = self.ctx.create_state() else {
            log::warn!("warmup: failed to create state, first transcription may be slow");
            return;
        };
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
        let _ = state.full(params, &dummy);
    }

    pub(crate) fn transcribe(&self, samples: &[f32], language: &str, initial_prompt: &str) -> Result<String, WhisperError> {
        if samples.len() < MIN_SAMPLES {
            return Ok(String::new());
        }

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| WhisperError::StateCreate(e.to_string()))?;

        // Check if audio has enough energy (not just silence)
        let energy: f32 = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
        if energy < 1e-6 {
            return Ok(String::new());
        }

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(optimal_threads());
        params.set_language(Some(language));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);

        // Anti-hallucination settings
        params.set_suppress_blank(true);
        params.set_no_speech_thold(0.6);
        params.set_temperature_inc(0.0); // disable temperature fallback — it just produces more hallucinations
        params.set_entropy_thold(2.4);   // reject segments with high entropy (uncertain/hallucinated)

        if !initial_prompt.is_empty() {
            params.set_initial_prompt(initial_prompt);
        }

        state
            .full(params, samples)
            .map_err(|e| WhisperError::Transcription(e.to_string()))?;

        let n_segments = state.full_n_segments();
        let mut text = String::new();
        for i in 0..n_segments {
            if let Some(segment) = state.get_segment(i) {
                let segment_text = segment
                    .to_str()
                    .map_err(|e: whisper_rs::WhisperError| {
                        WhisperError::Transcription(e.to_string())
                    })?;
                text.push_str(segment_text);
            }
        }

        Ok(text.trim().to_string())
    }
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
}
