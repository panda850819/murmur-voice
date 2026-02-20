## ADDED Requirements

### Requirement: TextEnhancer trait for pluggable LLM providers
The system SHALL define a `TextEnhancer` trait with `name()`, `is_local()`, and `enhance()` methods that any LLM provider can implement. Implementations MUST be `Send + Sync`.

#### Scenario: Trait is object-safe and thread-safe
- **WHEN** a `TextEnhancer` implementation is created
- **THEN** it can be stored as `Box<dyn TextEnhancer>` and shared across threads

### Requirement: OpenAI-compatible enhancer
The system SHALL provide an `OpenAICompatibleEnhancer` struct that implements `TextEnhancer` and works with any OpenAI chat completions API endpoint.

#### Scenario: Groq preset
- **WHEN** `OpenAICompatibleEnhancer::groq(api_key, model)` is called
- **THEN** the enhancer targets `https://api.groq.com/openai/v1/chat/completions` with Bearer auth, reports `name()` as "Groq", and `is_local()` as false

#### Scenario: Ollama preset
- **WHEN** `OpenAICompatibleEnhancer::ollama(base_url, model)` is called
- **THEN** the enhancer targets `{base_url}/v1/chat/completions` without auth, reports `name()` as "Ollama", and `is_local()` as true

#### Scenario: Custom endpoint preset
- **WHEN** `OpenAICompatibleEnhancer::custom(api_url, api_key, model)` is called
- **THEN** the enhancer targets the given URL with Bearer auth, reports `name()` as "Custom", and `is_local()` as false

### Requirement: Enhancer factory function
The system SHALL provide a `create_enhancer(settings)` function that returns the appropriate `TextEnhancer` based on current settings, or `None` if LLM is disabled or required config is missing.

#### Scenario: LLM disabled
- **WHEN** `llm_enabled` is false
- **THEN** `create_enhancer()` returns `None`

#### Scenario: Groq provider selected with valid key
- **WHEN** `llm_provider` is "groq" and `groq_api_key` is non-empty
- **THEN** `create_enhancer()` returns a Groq enhancer

#### Scenario: Groq provider selected without key
- **WHEN** `llm_provider` is "groq" and `groq_api_key` is empty
- **THEN** `create_enhancer()` returns `None`

#### Scenario: Ollama provider selected
- **WHEN** `llm_provider` is "ollama"
- **THEN** `create_enhancer()` returns an Ollama enhancer (no API key required)

#### Scenario: Custom provider selected with URL
- **WHEN** `llm_provider` is "custom" and `custom_llm_url` is non-empty
- **THEN** `create_enhancer()` returns a Custom enhancer

#### Scenario: Custom provider selected without URL
- **WHEN** `llm_provider` is "custom" and `custom_llm_url` is empty
- **THEN** `create_enhancer()` returns `None`

### Requirement: LLM provider settings fields
The system SHALL persist the following settings: `llm_provider` (default "groq"), `ollama_url` (default "http://localhost:11434"), `ollama_model` (default "llama3.2"), `custom_llm_url` (default ""), `custom_llm_key` (default ""), `custom_llm_model` (default "").

#### Scenario: New settings fields have serde defaults
- **WHEN** an existing settings file without `llm_provider` is loaded
- **THEN** the field defaults to "groq" and all other new fields get their defaults

#### Scenario: New settings fields persist
- **WHEN** the user configures Ollama URL and model and saves
- **THEN** the values are written to settings.json and restored on next load

### Requirement: Provider selection UI
The system SHALL display a provider dropdown (Groq/Ollama/Custom) in the settings AI Processing section. Each provider's config fields SHALL only appear when that provider is selected.

#### Scenario: Provider dropdown changes visible fields
- **WHEN** the user selects "Ollama" from the provider dropdown
- **THEN** the Ollama URL and model fields appear, and Groq API key / model fields are hidden

#### Scenario: Groq API key shared with transcription
- **WHEN** the user selects "Groq" as transcription engine
- **THEN** the transcription section shows a hint that the API key is in AI Processing
