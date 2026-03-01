# speech-transcription Specification

## Purpose
Converts recorded audio into text using local Whisper inference or cloud Whisper API, with anti-hallucination safeguards and multi-language support.
## Requirements
### Requirement: Extended language support
The system SHALL support all major Whisper languages in the language selector, including but not limited to: Japanese, Korean, French, German, Spanish, Portuguese, Russian, Arabic, Hindi, Thai, Vietnamese, Indonesian.

#### Scenario: Select Japanese
- **WHEN** the user selects Japanese in settings
- **THEN** Whisper uses "ja" as the language parameter

#### Scenario: Language list in UI
- **WHEN** the user opens the language dropdown in settings
- **THEN** at least 15 languages are available for selection

### Requirement: Local Whisper transcription
The system SHALL transcribe audio samples using whisper-rs with the ggml-large-v3-turbo model, performing all inference locally on the device. On macOS, the system SHALL use Metal GPU acceleration. On Windows, the system SHALL use CPU inference. The system SHALL use `std::thread::available_parallelism()` to determine thread count for inference (with fallback to 4). When personal dictionary terms are configured, the system SHALL inject them as initial_prompt.

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

#### Scenario: Metal acceleration on macOS
- **WHEN** transcription runs on macOS
- **THEN** whisper-rs uses Metal GPU for inference

#### Scenario: CPU inference on Windows
- **WHEN** transcription runs on Windows
- **THEN** whisper-rs uses CPU for inference (no Metal feature)

#### Scenario: Dynamic thread count
- **WHEN** transcription runs on a machine with N available CPU threads
- **THEN** whisper-rs uses N threads for inference

### Requirement: Audio usability check
The system SHALL check audio usability before transcription using a shared `is_audio_usable()` function. The energy check SHALL use subsampled data (up to 1000 evenly-spaced samples) instead of iterating the full buffer, to reduce computation on the stop-recording hot path.

#### Scenario: Audio too short
- **WHEN** audio samples are fewer than 16,000 (1 second at 16kHz)
- **THEN** `is_audio_usable` returns false and transcription is skipped

#### Scenario: Audio is silent
- **WHEN** the subsampled energy (mean of squared amplitudes) is below 1e-6
- **THEN** `is_audio_usable` returns false and transcription is skipped

#### Scenario: Energy check uses subsampling
- **WHEN** `is_audio_usable` is called with a buffer larger than 1000 samples
- **THEN** it samples at most 1000 evenly-spaced points for the energy calculation

