use thiserror::Error;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const MIN_SAMPLES: usize = 3_200; // 0.2s at 16kHz

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
        params.use_gpu(true); // Metal on macOS
        let ctx = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| WhisperError::ModelLoad(e.to_string()))?;
        Ok(Self { ctx })
    }

    pub(crate) fn transcribe(&self, samples: &[f32], language: &str, initial_prompt: &str) -> Result<String, WhisperError> {
        if samples.len() < MIN_SAMPLES {
            return Ok(String::new());
        }

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| WhisperError::StateCreate(e.to_string()))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(4);
        params.set_language(Some(language));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);

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
