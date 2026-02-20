## Context

Murmur v0.2.1 has a single LLM provider (Groq) hardcoded in `llm.rs::process_text()`. The function builds a request body, sends it to `api.groq.com`, and parses the response. The `do_stop_recording()` function in `lib.rs` calls `process_text()` directly with Groq-specific parameters.

Users wanting fully offline operation (local Whisper + local LLM) or self-hosted endpoints are currently locked out. The competitive differentiator for Murmur is verifiable privacy through local-first operation — this requires abstracting LLM providers.

## Goals / Non-Goals

**Goals:**
- Enable multiple LLM providers via a trait-based abstraction
- Support Groq (cloud), Ollama (local), and custom OpenAI-compatible endpoints
- Preserve existing text enhancement behavior (filler removal, formatting, zh-TW)
- Maintain backward compatibility with existing settings files
- Show users whether their data stays local or goes to cloud

**Non-Goals:**
- Transcription engine abstraction (deferred to v0.4)
- Streaming LLM responses
- Non-OpenAI-compatible LLM APIs (e.g., Anthropic, Google)
- Model download/management for Ollama (user manages Ollama separately)

## Decisions

### Decision 1: Single struct, not three implementations

**Choice:** One `OpenAICompatibleEnhancer` struct with factory presets (`groq()`, `ollama()`, `custom()`) instead of separate structs per provider.

**Rationale:** All three target providers (Groq, Ollama, custom) use the OpenAI chat completions API format. The only differences are URL, auth header, model name, and a minor parameter tweak (Ollama doesn't support `frequency_penalty`). A single struct with preset constructors eliminates code duplication.

**Alternative considered:** Separate `GroqEnhancer`, `OllamaEnhancer`, `CustomEnhancer` structs. Rejected because 95% of the code would be identical.

### Decision 2: Sync trait with internal tokio runtime

**Choice:** `TextEnhancer::enhance()` is a synchronous method. The `OpenAICompatibleEnhancer` creates a `tokio::runtime::Runtime` internally for the HTTP call.

**Rationale:** `do_stop_recording()` runs in a `tauri::async_runtime::spawn` block but the LLM call is a single blocking operation in the pipeline. Creating a short-lived runtime matches the existing `process_text()` pattern and avoids propagating `async` through the trait (which would require `async_trait` crate and complicate the factory). The runtime creation cost (~0.1ms) is negligible vs the LLM API latency (~200-2000ms).

**Alternative considered:** `async fn enhance()` with `#[async_trait]`. Rejected because it adds a crate dependency and complexity for no measurable benefit in this use case.

### Decision 3: Settings backward compatibility via serde defaults

**Choice:** New fields use `#[serde(default = "...")]` so existing settings files without `llm_provider` deserialize correctly, defaulting to `"groq"`.

**Rationale:** Users upgrading from v0.2.x should not lose their configuration. Serde's default attribute handles this transparently with zero migration code.

### Decision 4: Groq API key shared between transcription and LLM

**Choice:** Keep `groq_api_key` as a single field used by both Groq Whisper transcription and Groq LLM enhancement. The key input moves to the AI Processing section; the Transcription section shows a hint.

**Rationale:** One Groq account = one API key for all endpoints. Duplicating the field would confuse users. The hint approach avoids a breaking UI change while clarifying the shared usage.

## Risks / Trade-offs

- **Ollama availability:** Users may not have Ollama running → `enhance()` returns network error → system falls back to raw text with warning. Acceptable degradation.
- **Ollama parameter differences:** Ollama may not support all OpenAI parameters (e.g., `frequency_penalty`). → Mitigation: skip unsupported params for local providers via `is_local()` check.
- **Short-lived tokio runtime:** Creating a runtime per `enhance()` call is wasteful if called frequently. → Mitigation: transcription happens at most every few seconds; the overhead is negligible. If it becomes a problem, the runtime can be cached in the struct later.
- **No connection test:** Settings UI doesn't validate Ollama connectivity before save. → Acceptable for v0.3; a "Test Connection" button can be added later.
