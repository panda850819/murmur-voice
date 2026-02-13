# audio-recording Specification

## Purpose
TBD - created by archiving change build-mvp-voice-to-text. Update Purpose after archive.
## Requirements
### Requirement: Microphone capture
The system SHALL capture audio from the default input device using cpal and produce 16kHz mono f32 samples suitable for Whisper inference.

#### Scenario: Record audio at native sample rate and resample
- **WHEN** recording is started on a device with sample rate other than 16kHz
- **THEN** the system records at the device's default sample rate and resamples to 16kHz mono f32

#### Scenario: Record audio at 16kHz natively
- **WHEN** recording is started on a device supporting 16kHz
- **THEN** the system records directly at 16kHz mono f32 without resampling

### Requirement: Recording lifecycle
The system SHALL support start and stop operations controlled by an external signal (AtomicBool), collecting samples into a shared buffer.

#### Scenario: Start recording
- **WHEN** start is called
- **THEN** a dedicated recording thread begins capturing samples into an Arc<Mutex<Vec<f32>>> buffer

#### Scenario: Stop recording
- **WHEN** the stop signal (AtomicBool) is set to true
- **THEN** the recording thread stops capturing and the collected samples are available for retrieval

#### Scenario: Short recording protection
- **WHEN** recording duration is less than 0.2 seconds (fewer than ~3200 samples at 16kHz)
- **THEN** the system returns an empty sample buffer to avoid transcription of noise

