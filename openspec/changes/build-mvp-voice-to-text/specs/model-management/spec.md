## ADDED Requirements

### Requirement: Model storage location
The system SHALL store Whisper models in ~/Library/Application Support/com.murmur.voice/models/.

#### Scenario: Model directory creation
- **WHEN** the model directory does not exist
- **THEN** the system creates it before downloading

### Requirement: Model download
The system SHALL download ggml-large-v3-turbo.bin from HuggingFace with streaming progress reporting.

#### Scenario: First launch download
- **WHEN** the application starts and the model file does not exist
- **THEN** the system initiates a streaming download and emits model_download_progress events to the frontend

#### Scenario: Download progress reporting
- **WHEN** model download is in progress
- **THEN** the system emits progress events with bytes downloaded and total bytes

#### Scenario: Download completion
- **WHEN** download finishes
- **THEN** the system verifies the file size matches expected size and emits a model_ready event

### Requirement: Model availability check
The system SHALL check if the model file exists and has the correct size on startup, skipping download if valid.

#### Scenario: Model already present
- **WHEN** the application starts and the model file exists with correct size
- **THEN** the system skips download and loads the model directly

#### Scenario: Model file corrupted or incomplete
- **WHEN** the model file exists but size does not match expected size
- **THEN** the system re-downloads the model
