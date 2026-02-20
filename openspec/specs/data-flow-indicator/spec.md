## ADDED Requirements

### Requirement: LLM provider data flow badge
The system SHALL display a badge in the preview window indicating the active LLM provider name and whether it is local or cloud-based, after enhancement completes.

#### Scenario: Cloud provider badge
- **WHEN** text is enhanced by a cloud provider (e.g., Groq)
- **THEN** the preview window shows a badge with "Groq (Cloud)" styled with cloud colors

#### Scenario: Local provider badge
- **WHEN** text is enhanced by a local provider (e.g., Ollama)
- **THEN** the preview window shows a badge with "Ollama (Local)" styled with local colors

#### Scenario: No enhancement
- **WHEN** LLM enhancement is disabled
- **THEN** no provider badge is shown in the preview window

### Requirement: Enhancer info event
The system SHALL emit an `enhancer_info` event with provider name and locality flag before LLM enhancement begins, so the frontend can display the data flow badge.

#### Scenario: Event payload
- **WHEN** the enhancer is invoked
- **THEN** an `enhancer_info` event is emitted with `{ "name": "<provider>", "local": <bool> }`

### Requirement: Transcription engine info event
The system SHALL emit a `transcription_engine_info` event with engine type and locality flag after transcription completes.

#### Scenario: Local engine event
- **WHEN** transcription uses the local Whisper engine
- **THEN** a `transcription_engine_info` event is emitted with `{ "engine": "local", "local": true }`

#### Scenario: Cloud engine event
- **WHEN** transcription uses the Groq Whisper engine
- **THEN** a `transcription_engine_info` event is emitted with `{ "engine": "groq", "local": false }`
