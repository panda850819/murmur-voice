## ADDED Requirements

### Requirement: Extended language support
The system SHALL support all major Whisper languages in the language selector, including but not limited to: Japanese, Korean, French, German, Spanish, Portuguese, Russian, Arabic, Hindi, Thai, Vietnamese, Indonesian.

#### Scenario: Select Japanese
- **WHEN** the user selects Japanese in settings
- **THEN** Whisper uses "ja" as the language parameter

#### Scenario: Language list in UI
- **WHEN** the user opens the language dropdown in settings
- **THEN** at least 15 languages are available for selection

## MODIFIED Requirements

### Requirement: Local Whisper transcription
The system SHALL transcribe audio samples using whisper-rs with the ggml-large-v3-turbo model, performing all inference locally on the device. When personal dictionary terms are configured, the system SHALL inject them as initial_prompt.

#### Scenario: Transcribe English speech
- **WHEN** English audio samples are provided
- **THEN** the system returns transcribed English text

#### Scenario: Transcribe Chinese speech
- **WHEN** Chinese audio samples are provided
- **THEN** the system returns transcribed Chinese text

#### Scenario: Auto language detection
- **WHEN** audio samples are provided without explicit language selection
- **THEN** the system automatically detects the language and transcribes accordingly

#### Scenario: Dictionary terms injected
- **WHEN** personal dictionary contains terms
- **THEN** Whisper initial_prompt includes those terms to improve recognition
