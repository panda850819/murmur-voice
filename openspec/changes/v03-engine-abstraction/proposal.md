## Why

Murmur currently hardcodes Groq as the sole LLM provider for text enhancement. This means users who want fully offline operation (local Whisper + local LLM) cannot achieve it, and users with self-hosted or alternative OpenAI-compatible endpoints are locked out. Multi-provider LLM support is the core differentiator for v0.3: it enables full-chain offline voice-to-text with AI enhancement, which no other free, open-source tool offers.

## What Changes

- Add a `TextEnhancer` trait in Rust for pluggable LLM post-processing providers
- Implement `OpenAICompatibleEnhancer` struct covering Groq, Ollama, and custom endpoints (all three use the OpenAI chat completions API format)
- Add factory function `create_enhancer()` that reads settings and returns the appropriate provider
- Refactor `do_stop_recording()` to use trait dispatch instead of hardcoded `process_text()` call
- Remove the old `process_text()` function (replaced by `TextEnhancer::enhance()`)
- Add new settings fields: `llm_provider`, `ollama_url`, `ollama_model`, `custom_llm_url`, `custom_llm_key`, `custom_llm_model`
- Update settings UI with provider dropdown (Groq/Ollama/Custom) and conditional config sections
- Add data flow indicator (Local/Cloud badge) in preview window
- Maintain backward compatibility with existing settings files via serde defaults

## Capabilities

### New Capabilities
- `llm-provider-abstraction`: TextEnhancer trait system and OpenAICompatibleEnhancer for multi-provider LLM support (Groq, Ollama, Custom)
- `data-flow-indicator`: UI badge showing whether the active LLM provider is local or cloud-based

### Modified Capabilities
- `llm-post-processing`: Requirements change from hardcoded Groq to any OpenAI-compatible provider. The core processing behavior (filler removal, formatting, zh-TW conversion) is preserved, but the provider is now configurable.

## Impact

- **Backend**: `src-tauri/src/llm.rs` (trait + struct + factory, remove `process_text`), `src-tauri/src/lib.rs` (refactor `do_stop_recording` LLM block), `src-tauri/src/settings.rs` (new fields)
- **Frontend**: `src/settings.html` (provider dropdown UI), `src/settings.js` (visibility logic, save/load), `src/preview.js` + `src/preview.css` (data flow badge)
- **Dependencies**: No new crate dependencies. Ollama and custom endpoints use existing `reqwest`.
- **Backward compatibility**: Existing `settings.json` without `llm_provider` defaults to `"groq"`, preserving current behavior.
