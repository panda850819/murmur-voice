## 1. TextEnhancer Trait + OpenAICompatibleEnhancer

- [x] 1.1 Add `TextEnhancer` trait and `OpenAICompatibleEnhancer` struct with `groq()`, `ollama()`, `custom()` presets to `src-tauri/src/llm.rs`
- [x] 1.2 Add unit tests: trait Send+Sync bounds, groq preset, ollama preset, custom preset
- [x] 1.3 Run `cargo test` — all tests pass

## 2. Settings Fields + Factory Function

- [x] 2.1 Add new fields to `Settings` struct in `src-tauri/src/settings.rs`: `llm_provider`, `ollama_url`, `ollama_model`, `custom_llm_url`, `custom_llm_key`, `custom_llm_model` with serde defaults
- [x] 2.2 Add `create_enhancer()` factory function to `src-tauri/src/llm.rs`
- [x] 2.3 Add unit tests: factory returns None when disabled, returns Groq/Ollama/Custom correctly, returns None for Groq without key
- [x] 2.4 Run `cargo test` — all tests pass

## 3. Refactor do_stop_recording

- [x] 3.1 Replace hardcoded `process_text()` call in `src-tauri/src/lib.rs` with `create_enhancer()` + trait dispatch
- [x] 3.2 Add `enhancer_info` event emission for data flow indicator
- [x] 3.3 Remove old `process_text()` function from `src-tauri/src/llm.rs`
- [x] 3.4 Run `cargo check` and `cargo clippy --all-targets -- -D warnings` — zero errors/warnings

## 4. Frontend Settings UI

- [x] 4.1 Update `src/settings.html` AI Processing section: add provider dropdown (Groq/Ollama/Custom) with conditional config sections
- [x] 4.2 Move Groq API key input to AI Processing section; replace old transcription groq-section with hint text
- [x] 4.3 Add `updateLlmProviderVisibility()` to `src/settings.js` and wire up event listeners
- [x] 4.4 Update settings load/save in `src/settings.js` to include new fields
- [x] 4.5 Add `.row-hint` CSS to `src/settings.css`

## 5. Data Flow Indicator

- [x] 5.1 Add `enhancer_info` event listener in `src/preview.js` to show Local/Cloud badge
- [x] 5.2 Add badge styles (`.badge-local`, `.badge-cloud`) to `src/preview.css`
- [x] 5.3 Add `transcription_engine_info` event emission in `src-tauri/src/lib.rs`

## 6. Backward Compatibility Tests

- [x] 6.1 Add test: deserialize legacy settings JSON (no `llm_provider` field) defaults to "groq"
- [x] 6.2 Add test: deserialize new settings JSON with ollama config
- [x] 6.3 Run `cargo test` — all tests pass (existing 6 state + 10 new = 16 total)

## 7. Final Verification

- [x] 7.1 Run `cargo clippy --all-targets -- -D warnings` — zero warnings
- [x] 7.2 Run `cargo test` — all 16 tests pass
- [x] 7.3 Clean up any dead code or unused imports
