## ADDED Requirements

### Requirement: Local Whisper transcription
The system SHALL transcribe audio samples using whisper-rs with the ggml-large-v3-turbo model, performing all inference locally on the device.

#### Scenario: Transcribe English speech
- **WHEN** English audio samples are provided
- **THEN** the system returns transcribed English text

#### Scenario: Transcribe Chinese speech
- **WHEN** Chinese audio samples are provided
- **THEN** the system returns transcribed Chinese text

#### Scenario: Auto language detection
- **WHEN** audio samples are provided without explicit language selection
- **THEN** the system automatically detects the language and transcribes accordingly

### Requirement: Metal GPU acceleration
The system SHALL use Metal GPU acceleration for Whisper inference on Apple Silicon when available.

#### Scenario: Metal available
- **WHEN** the system runs on Apple Silicon with Metal support
- **THEN** whisper-rs uses GPU acceleration via use_gpu(true)

#### Scenario: Metal unavailable
- **WHEN** Metal acceleration is not available
- **THEN** the system falls back to CPU-only inference

### Requirement: Transcription configuration
The system SHALL use Greedy decoding with best_of=1 and 4 processing threads for fastest inference.

#### Scenario: Greedy decoding
- **WHEN** transcription is invoked
- **THEN** the system uses SamplingStrategy::Greedy { best_of: 1 } with 4 threads

### Requirement: Empty audio handling
The system SHALL return an empty string when audio is too short to produce meaningful transcription.

#### Scenario: Audio shorter than 0.2 seconds
- **WHEN** provided audio samples represent less than 0.2 seconds of audio
- **THEN** the system returns an empty string without invoking Whisper
