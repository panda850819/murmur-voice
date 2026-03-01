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
  RECORDING_CANCELLED: "recording_cancelled",
  TRANSCRIPTION_ENGINE_INFO: "transcription_engine_info",
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

const COMMANDS = {
  GET_SETTINGS: "get_settings",
  SAVE_SETTINGS: "save_settings",
  OPEN_SETTINGS: "open_settings",
  IS_MODEL_READY: "is_model_ready",
  DOWNLOAD_MODEL_CMD: "download_model_cmd",
  CHECK_ACCESSIBILITY: "check_accessibility",
  CHECK_MICROPHONE: "check_microphone",
  REQUEST_MICROPHONE: "request_microphone",
  PAUSE_HOTKEY_LISTENER: "pause_hotkey_listener",
  RESUME_HOTKEY_LISTENER: "resume_hotkey_listener",
  CHECK_FOR_UPDATES: "check_for_updates",
  OPEN_URL: "open_url",
  ADD_DICTIONARY_TERM: "add_dictionary_term",
  ADD_DICTIONARY_TERMS: "add_dictionary_terms",
  COPY_TO_CLIPBOARD: "copy_to_clipboard",
  HIDE_PREVIEW: "hide_preview",
  HIDE_OVERLAY_WINDOWS: "hide_overlay_windows",
  COMPLETE_ONBOARDING: "complete_onboarding",
};
