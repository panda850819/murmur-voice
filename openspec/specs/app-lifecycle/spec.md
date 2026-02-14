# app-lifecycle Specification

## Purpose
TBD - created by archiving change build-mvp-voice-to-text. Update Purpose after archive.
## Requirements
### Requirement: Application state machine
The system SHALL maintain a state machine with transitions: Idle -> Starting -> Recording -> Stopping -> Transcribing -> Idle. Any state MAY transition to Idle on error.

#### Scenario: Normal recording flow
- **WHEN** the user presses and releases the hotkey
- **THEN** state transitions through Idle -> Starting -> Recording -> Stopping -> Transcribing -> Idle

#### Scenario: Error recovery
- **WHEN** an error occurs in any state
- **THEN** state transitions to Idle and an error event is emitted to the frontend

#### Scenario: Invalid transition rejected
- **WHEN** a state transition is attempted that is not in the valid transition map
- **THEN** the transition is rejected and the current state is preserved

### Requirement: System tray
The system SHALL display a system tray icon with a Quit menu option.

#### Scenario: Quit from tray
- **WHEN** the user clicks Quit in the tray menu
- **THEN** the application exits cleanly, stopping any active recording

### Requirement: Floating window
The system SHALL display a 320x120 transparent, always-on-top window without title bar decorations that shows recording state and transcription results.

#### Scenario: Window properties
- **WHEN** the application starts
- **THEN** the window is 320x120, always on top, no decorations, transparent background, not shown in taskbar

### Requirement: Frontend state events
The system SHALL emit events to the frontend: recording_state_changed, transcription_complete, model_download_progress, model_ready, foreground_app_info.

#### Scenario: State change notification
- **WHEN** the recording state changes
- **THEN** the system emits recording_state_changed with the new state name

#### Scenario: Transcription result notification
- **WHEN** transcription completes with non-empty text
- **THEN** the system emits transcription_complete with the transcribed text

#### Scenario: Foreground app notification
- **WHEN** transcription begins
- **THEN** the system emits foreground_app_info with the detected app name and style

### Requirement: Hotkey-to-action orchestration
The system SHALL connect hotkey events to recording and transcription actions: pressed starts recording, released stops recording and triggers transcription followed by text insertion.

#### Scenario: Press starts recording
- **WHEN** HotkeyEvent::Pressed is received and state is Idle
- **THEN** the system starts recording and transitions to Recording state

#### Scenario: Release triggers transcription pipeline
- **WHEN** HotkeyEvent::Released is received and state is Recording
- **THEN** the system stops recording, transcribes audio, inserts text at cursor, and transitions back to Idle

