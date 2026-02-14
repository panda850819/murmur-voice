## MODIFIED Requirements

### Requirement: Model storage location
The system SHALL store Whisper models in the platform app data directory resolved via Tauri's `app.path().app_data_dir()`, under a `models/` subdirectory. The system SHALL NOT use hardcoded platform paths or manual `#[cfg]` path assembly.

#### Scenario: Model directory creation
- **WHEN** the model directory does not exist
- **THEN** the system creates it before downloading

#### Scenario: Path resolution on macOS
- **WHEN** the app runs on macOS
- **THEN** the model directory resolves to `~/Library/Application Support/com.murmur.voice/models/`

#### Scenario: Path resolution on Windows
- **WHEN** the app runs on Windows
- **THEN** the model directory resolves to `%APPDATA%\com.murmur.voice\models\`

#### Scenario: Path resolution failure
- **WHEN** Tauri cannot resolve the app data directory
- **THEN** the system returns an error (not a silent fallback to `/tmp` or other volatile path)

### Requirement: Model availability check
The system SHALL check if the model file exists and has the correct size on startup, skipping download if valid.

#### Scenario: Model already present
- **WHEN** the application starts and the model file exists with correct size
- **THEN** the system skips download and loads the model directly

#### Scenario: Model file corrupted or incomplete
- **WHEN** the model file exists but size does not match expected size
- **THEN** the system re-downloads the model

### Requirement: Windows model migration
The system SHALL migrate the model from the old macOS-style path (`$HOME/Library/Application Support/...`) to the correct platform path on Windows. The migration SHALL execute at most once per process using `std::sync::Once`.

#### Scenario: Migration runs once
- **WHEN** `is_model_ready()` is called multiple times during a session
- **THEN** the migration check runs only on the first call

#### Scenario: Model at old path
- **WHEN** the model exists at the old macOS-style path on Windows and not at the new path
- **THEN** the system moves (or copies + deletes for cross-drive) the model to the correct path

#### Scenario: Model already at correct path
- **WHEN** the model already exists at the correct platform path
- **THEN** the migration is skipped immediately

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
