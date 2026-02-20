## MODIFIED Requirements

### Requirement: LLM post-processing pipeline
The system SHALL pass Whisper transcription output through a configurable LLM provider (selected via `llm_provider` setting) for text refinement before inserting into clipboard, when LLM processing is enabled in settings. The system SHALL use the `TextEnhancer` trait for provider dispatch.

#### Scenario: LLM enabled with valid provider config
- **WHEN** llm_enabled is true and the selected provider has valid configuration
- **THEN** the system creates the appropriate TextEnhancer via `create_enhancer()`, sends transcribed text through `enhance()`, and uses the refined output

#### Scenario: LLM disabled
- **WHEN** llm_enabled is false
- **THEN** the system inserts raw Whisper output directly

#### Scenario: Provider API call fails
- **WHEN** the LLM enhance call fails (network error, provider down, invalid key)
- **THEN** the system falls back to raw Whisper output and emits a `recording_error` event with details

#### Scenario: Provider config incomplete
- **WHEN** llm_enabled is true but required config is missing (e.g., Groq without API key)
- **THEN** `create_enhancer()` returns None and the system uses raw Whisper output without error
