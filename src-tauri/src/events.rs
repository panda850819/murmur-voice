// IPC event names and state strings — single source of truth for the Rust/JS bridge.
// Keep in sync with src/events.js.

// --- Event names ---
pub const MODEL_DOWNLOAD_PROGRESS: &str = "model_download_progress";
pub const MODEL_READY: &str = "model_ready";
pub const RECORDING_STATE_CHANGED: &str = "recording_state_changed";
pub const PARTIAL_TRANSCRIPTION: &str = "partial_transcription";
pub const TRANSCRIPTION_COMPLETE: &str = "transcription_complete";
pub const TRANSCRIPTION_ENGINE_INFO: &str = "transcription_engine_info";
pub const FOREGROUND_APP_INFO: &str = "foreground_app_info";
pub const OPACITY_CHANGED: &str = "opacity_changed";
pub const RECORDING_ERROR: &str = "recording_error";
pub const ACCESSIBILITY_ERROR: &str = "accessibility_error";
pub const ACCESSIBILITY_GRANTED: &str = "accessibility_granted";
pub const ENHANCER_INFO: &str = "enhancer_info";
pub const RECORDING_CANCELLED: &str = "recording_cancelled";

// --- Recording state strings ---
pub const STATE_IDLE: &str = "idle";
pub const STATE_STARTING: &str = "starting";
pub const STATE_RECORDING: &str = "recording";
pub const STATE_STOPPING: &str = "stopping";
pub const STATE_TRANSCRIBING: &str = "transcribing";
pub const STATE_PROCESSING: &str = "processing";
pub const STATE_TRANSLATING: &str = "translating";

// --- Transcription output modes ---
pub const MODE_PASTED: &str = "pasted";
pub const MODE_CLIPBOARD: &str = "clipboard";
pub const MODE_TRANSLATED: &str = "translated";
