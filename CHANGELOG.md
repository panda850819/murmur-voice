# Changelog

All notable changes to Murmur Voice are documented in this file.

## [Unreleased]

### Added
- Background engine initialization — Whisper model loads in a background thread during startup, no longer blocks the UI
- Engine readiness signaling via `Condvar` — recording waits for engine if triggered before init completes
- Retry logic — if background engine init fails, retries on first recording attempt
- Dynamic thread count — uses `std::thread::available_parallelism()` instead of hardcoded 4 threads
- `engine-lifecycle` capability spec

### Changed
- All paths now resolved via Tauri's `app.path().app_data_dir()` — removed all `#[cfg]` path assembly and env var lookups from `model.rs` and `settings.rs`
- `model_dir()`, `model_path()`, `is_model_ready()`, `download_model()` now accept `base: &Path`
- `settings_path()`, `load_settings()`, `save_settings()` now accept `base: &Path`
- `MurmurState` created inside `setup()` closure with resolved `app_data_dir`
- LLM post-processing prompt rewritten to prevent model from answering/expanding dictated text
- User message prefixed with `[Raw transcription to clean up]` to reinforce cleaning-only behavior

### Fixed
- LLM post-processor was treating short dictation as questions and generating explanatory responses
- Windows model migration now runs at most once per process via `std::sync::Once`

## [0.2.0] - 2026-02-14

### Added
- Windows support with CI/Release workflows
- Transcription preview window with app badge
- Groq Whisper API as alternative transcription engine
- LLM post-processing (filler word removal, punctuation, Traditional Chinese conversion)
- Toggle recording mode (in addition to hold mode)
- App-aware style detection (formal/casual/technical based on foreground app)
- Personal dictionary for Whisper initial prompt
- Cursor added to app detection list

### Fixed
- CUDA kernel warmup on Windows
- Windows GPU support and cross-platform model path
- `stop_recording` deadlock
- macOS i8mm and Windows PWSTR CI build errors
- Toggle mode auto-stop sync issue

## [0.1.0] - 2026-02-13

### Added
- Initial release: privacy-first voice-to-text for macOS
- Local Whisper transcription with Metal GPU acceleration
- Push-to-talk with configurable modifier key hotkey
- System tray with recording status
- Settings window (language, hotkey, opacity, auto-start)
- First-run onboarding flow
- Clipboard insertion via Cmd+V simulation
