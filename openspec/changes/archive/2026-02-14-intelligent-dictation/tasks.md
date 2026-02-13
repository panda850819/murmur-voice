## 1. LLM Post-Processing Module

- [x] 1.1 Create `llm.rs` module with Groq LLM API client (reqwest POST to chat completions endpoint)
- [x] 1.2 Build system prompt template with instructions: remove fillers, fix repetitions, auto-format, Simplified→Traditional Chinese
- [x] 1.3 Add `llm_enabled` and `llm_model` fields to Settings with serde defaults
- [x] 1.4 Integrate LLM call into `do_stop_recording()` between Whisper output and clipboard insert
- [x] 1.5 Add fallback: if API call fails, use raw Whisper output and emit warning event
- [x] 1.6 Add "processing" state to RecordingState enum and emit during LLM call

## 2. Toggle Recording Mode

- [x] 2.1 Add `recording_mode` field to Settings ("hold" | "toggle", default "hold")
- [x] 2.2 Modify hotkey event handler in `lib.rs` to support toggle logic (press toggles start/stop)
- [x] 2.3 Add 5-minute max duration auto-stop for toggle mode (spawn timeout thread)
- [x] 2.4 Add recording mode selector to settings UI (segmented control: Hold / Toggle)

## 3. Personal Dictionary

- [x] 3.1 Add `dictionary` field to Settings (String, comma-separated)
- [x] 3.2 Modify `whisper.rs` transcribe to accept and inject `initial_prompt` parameter
- [x] 3.3 Pass dictionary terms from Settings to Whisper transcribe call in `lib.rs`
- [x] 3.4 Add dictionary text input to settings UI

## 4. Extended Language Support

- [x] 4.1 Expand language options in settings UI (add ja, ko, fr, de, es, pt, ru, ar, hi, th, vi, id)
- [x] 4.2 Update `whisper_language()` in `settings.rs` to handle all new language codes

## 5. App-Aware Style

- [x] 5.1 Create `frontapp.rs` module: detect foreground app bundle ID via NSWorkspace FFI
- [x] 5.2 Define app category mapping (bundle ID → style: formal/casual/technical/default)
- [x] 5.3 Pass detected style to LLM prompt, adjusting tone accordingly
- [x] 5.4 Add `app_aware_style` toggle to Settings (default true)
- [x] 5.5 Add app-aware style toggle to settings UI

## 6. Settings UI Updates

- [x] 6.1 Add LLM section to settings: enable toggle, model selector
- [x] 6.2 Add recording mode selector (Hold / Toggle)
- [x] 6.3 Add dictionary text field
- [x] 6.4 Expand language dropdown with all supported languages
- [x] 6.5 Add app-aware style toggle

## 7. Integration & Testing

- [x] 7.1 Verify LLM pipeline end-to-end: speak → Whisper → LLM → Traditional Chinese clipboard output
- [x] 7.2 Verify toggle mode: press once to start, press again to stop, auto-stop at 5 min
- [x] 7.3 Verify dictionary injection improves recognition of custom terms
- [x] 7.4 Verify fallback when Groq API is unavailable
- [x] 7.5 Run `cargo check` and `cargo clippy` with zero warnings
