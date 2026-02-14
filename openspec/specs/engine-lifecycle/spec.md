# engine-lifecycle Specification

## Purpose
Manages TranscriptionEngine initialization, warmup, readiness signaling, and retry logic.

## Requirements

### Requirement: Background engine initialization
The system SHALL initialize the TranscriptionEngine (including model loading and GPU kernel warmup) in a background thread during app startup, before the user triggers any recording.

#### Scenario: Engine ready before first recording
- **WHEN** the application starts and the model file is available
- **THEN** the TranscriptionEngine is initialized in a background thread and ready for use when the user first presses the hotkey

#### Scenario: Engine not ready when user records
- **WHEN** the user triggers recording before background initialization completes
- **THEN** the system waits for initialization to complete before proceeding with transcription

#### Scenario: Engine init failure
- **WHEN** engine initialization fails in the background (e.g., model file missing or corrupt)
- **THEN** the system logs the error and attempts initialization again on the first recording attempt

### Requirement: GPU kernel warmup
The system SHALL run a 1-second dummy inference during engine initialization to force CUDA/Metal kernel compilation, using a thread count derived from `std::thread::available_parallelism()`.

#### Scenario: Warmup uses dynamic thread count
- **WHEN** warmup runs on a machine with 8 available CPU threads
- **THEN** the warmup inference uses 8 threads (not a hardcoded value)

#### Scenario: Warmup fallback thread count
- **WHEN** `std::thread::available_parallelism()` is unavailable
- **THEN** the warmup inference uses 4 threads as a fallback

#### Scenario: Warmup completes before first use
- **WHEN** the engine is initialized in the background
- **THEN** GPU kernels are compiled and cached before the first real transcription
