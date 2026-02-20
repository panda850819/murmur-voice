## Requirements

### Requirement: ESC key cancels active recording
The system SHALL detect the ESC key globally (same mechanism as hotkey detection) and cancel the current recording when the state is Recording. Cancellation discards captured audio without transcription, transitions state to Idle, and hides the main and preview windows after a 2-second delay.

#### Scenario: ESC pressed during recording
- **WHEN** the recording state is Recording and the user presses ESC
- **THEN** the system stops audio capture, discards the captured samples, transitions state to Idle, emits `recording_state_changed` with "idle", emits `recording_cancelled` event, and hides both main and preview windows after 2 seconds

#### Scenario: ESC pressed during non-recording states
- **WHEN** the recording state is not Recording (e.g., Idle, Transcribing, Processing) and the user presses ESC
- **THEN** the system takes no action and the ESC key event passes through to the foreground application

#### Scenario: ESC cancel shows cancelled status
- **WHEN** the user cancels a recording via ESC
- **THEN** the main window displays a "cancelled" status indicator for 2 seconds before hiding

#### Scenario: ESC during hold mode
- **WHEN** recording_mode is "hold" and the user presses ESC while holding the hotkey
- **THEN** the recording is cancelled (same as releasing the hotkey, but without transcription)
