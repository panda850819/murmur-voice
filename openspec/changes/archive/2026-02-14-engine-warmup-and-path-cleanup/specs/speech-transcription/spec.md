## MODIFIED Requirements

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
