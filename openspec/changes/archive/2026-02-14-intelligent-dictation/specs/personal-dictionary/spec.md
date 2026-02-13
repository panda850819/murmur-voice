## ADDED Requirements

### Requirement: Personal dictionary storage
The system SHALL store a list of user-defined terms (names, jargon, proper nouns) in settings as a comma-separated string.

#### Scenario: Dictionary saved
- **WHEN** the user enters terms in the dictionary field and saves settings
- **THEN** the terms are persisted in settings.json

#### Scenario: Empty dictionary
- **WHEN** no dictionary terms are configured
- **THEN** the system operates without initial_prompt injection

### Requirement: Whisper initial_prompt injection
The system SHALL inject dictionary terms into the Whisper initial_prompt parameter to bias recognition toward those terms.

#### Scenario: Dictionary with terms
- **WHEN** the user has configured dictionary terms like "Murmur, Tauri, Whisper"
- **THEN** the system passes these as initial_prompt to Whisper transcription

#### Scenario: Terms improve recognition
- **WHEN** the user speaks a term that matches a dictionary entry
- **THEN** Whisper is more likely to produce the correct spelling/form

### Requirement: Dictionary UI
The system SHALL provide a text input field in the settings window for editing dictionary terms.

#### Scenario: Edit dictionary
- **WHEN** the user opens settings
- **THEN** a text field displays current dictionary terms, editable as comma-separated values
