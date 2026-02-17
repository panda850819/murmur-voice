# Changelog

All notable changes to Murmur Voice are documented in this file.

## [Unreleased]

## [0.3.0] - 2026-02-17

### Added
- Multi-provider LLM support — Groq, Ollama (local), and Custom OpenAI-compatible endpoints
- `TextEnhancer` trait with `OpenAICompatibleEnhancer` struct for pluggable LLM providers
- Provider dropdown in Settings with conditional config sections (API key, URL, model per provider)
- Data flow indicator badges in preview window (Local/Cloud)
- Fully offline AI enhancement via Ollama (local transcription + local LLM)

### Changed
- LLM post-processing refactored from hardcoded Groq call to trait-based dispatch
- Groq API key input moved to AI Processing section; Transcription section shows hint text
- Settings schema extended with 6 new fields (`llm_provider`, `ollama_url`, `ollama_model`, `custom_llm_url`, `custom_llm_key`, `custom_llm_model`) — backward compatible via serde defaults

## [0.2.1] - 2026-02-15

### Added
- Smart clipboard — detects focused text input before pasting; falls back to clipboard-only mode on Desktop/Finder/Explorer
- Copy button in transcription preview window
- Editable preview text — click to edit transcription directly
- Dictionary delete undo — 4-second toast to restore accidentally deleted terms
- Mixed-language English word preservation — English words in CJK-English speech are protected from LLM translation via placeholder mechanism
- Toggle mode debounce — 500ms protection against accidental double-trigger
- Background engine initialization — Whisper model loads in a background thread during startup
- Engine readiness signaling via `Condvar` — recording waits for engine if triggered before init completes
- Retry logic — if background engine init fails, retries on first recording attempt
- Dynamic thread count — uses `std::thread::available_parallelism()` instead of hardcoded 4 threads

### Changed
- Transcription preview persists indefinitely in clipboard-only mode (10s auto-hide in paste mode)
- `transcription_complete` event payload changed from string to `{ text, mode }` object
- All paths now resolved via Tauri's `app.path().app_data_dir()` — removed `#[cfg]` path assembly
- LLM post-processing prompt rewritten to prevent model from answering/expanding dictated text

### Fixed
- LLM infinite repetition loop — capped `max_tokens` relative to input length, added `frequency_penalty`
- LLM translating English words to Chinese in mixed-language text (e.g. "Settings" → "設定")
- LLM outputting preamble prefixes like "最終輸出：" — now stripped automatically
- `has_focused_text_input()` crash on macOS — inverted logic to default auto-paste, only check Finder/Explorer
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
