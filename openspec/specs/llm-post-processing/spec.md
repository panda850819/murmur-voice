## ADDED Requirements

### Requirement: LLM post-processing pipeline
The system SHALL pass Whisper transcription output through a Groq LLM API call for text refinement before inserting into clipboard, when LLM processing is enabled in settings.

#### Scenario: LLM enabled with valid API key
- **WHEN** llm_enabled is true and groq_api_key is set
- **THEN** the system sends transcribed text to Groq LLM API and uses the refined output

#### Scenario: LLM disabled
- **WHEN** llm_enabled is false
- **THEN** the system inserts raw Whisper output directly (current behavior)

#### Scenario: API call fails
- **WHEN** LLM API call fails (network error, rate limit, invalid key)
- **THEN** the system falls back to raw Whisper output and emits a warning event

### Requirement: Filler word removal
The system SHALL instruct the LLM to remove filler words (um, uh, 嗯, 啊, 那個, 就是) from transcribed text.

#### Scenario: Text with filler words
- **WHEN** transcribed text contains filler words
- **THEN** the LLM output has filler words removed while preserving meaning

### Requirement: Repetition and self-correction handling
The system SHALL instruct the LLM to detect and remove repeated phrases and self-corrections, keeping only the final intended version.

#### Scenario: Speaker corrects themselves
- **WHEN** transcribed text contains "我想要去台北 不對 我想要去台中"
- **THEN** the LLM output contains only "我想要去台中"

#### Scenario: Repeated words
- **WHEN** transcribed text contains unnecessarily repeated words
- **THEN** the LLM output removes the repetition

### Requirement: Auto-formatting
The system SHALL instruct the LLM to organize text into structured format when appropriate (lists, paragraphs, punctuation).

#### Scenario: Spoken list
- **WHEN** the user dictates items in sequence ("first... second... third...")
- **THEN** the LLM output formats them as a structured list

#### Scenario: Punctuation insertion
- **WHEN** transcribed text lacks proper punctuation
- **THEN** the LLM output includes appropriate punctuation

### Requirement: Simplified to Traditional Chinese conversion
The system SHALL instruct the LLM to output Traditional Chinese (zh-TW) when the transcription language is Chinese.

#### Scenario: Chinese transcription
- **WHEN** Whisper outputs Simplified Chinese text
- **THEN** the LLM converts to Traditional Chinese in the output

### Requirement: LLM processing state feedback
The system SHALL emit a "processing" state event while LLM API call is in progress.

#### Scenario: LLM processing in progress
- **WHEN** the LLM API call is running
- **THEN** the system emits recording_state_changed with "processing" state
