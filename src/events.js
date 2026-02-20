const EVENTS = {
  MODEL_DOWNLOAD_PROGRESS: "model_download_progress",
  MODEL_READY: "model_ready",
  RECORDING_STATE_CHANGED: "recording_state_changed",
  PARTIAL_TRANSCRIPTION: "partial_transcription",
  TRANSCRIPTION_COMPLETE: "transcription_complete",
  FOREGROUND_APP_INFO: "foreground_app_info",
  OPACITY_CHANGED: "opacity_changed",
  RECORDING_ERROR: "recording_error",
  ACCESSIBILITY_ERROR: "accessibility_error",
  ACCESSIBILITY_GRANTED: "accessibility_granted",
  ENHANCER_INFO: "enhancer_info",
};

const RECORDING_STATES = {
  STARTING: "starting",
  RECORDING: "recording",
  STOPPING: "stopping",
  TRANSCRIBING: "transcribing",
  PROCESSING: "processing",
  IDLE: "idle",
};

const TRANSCRIPTION_MODES = {
  PASTED: "pasted",
  CLIPBOARD: "clipboard",
};
