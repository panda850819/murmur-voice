# app-lifecycle Specification

## Purpose
Manages the application lifecycle: state machine, window visibility, system tray, hotkey orchestration, and frontend event communication.

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
The system SHALL display a system tray icon with Settings, Show/Hide, and Quit menu options.

#### Scenario: Quit from tray
- **WHEN** the user clicks Quit in the tray menu
- **THEN** the application exits cleanly, stopping any active recording

#### Scenario: Show window from tray
- **WHEN** the user clicks "Show" in the tray menu and the main window is hidden
- **THEN** the main window becomes visible and the menu item text changes to "Hide"

#### Scenario: Hide window from tray
- **WHEN** the user clicks "Hide" in the tray menu and the main window is visible
- **THEN** the main window hides and the menu item text changes to "Show"

### Requirement: Floating window
The system SHALL display a 420x48 transparent, always-on-top window without title bar decorations that shows recording state and transcription results. The window SHALL be hidden by default and only become visible when a recording starts. After transcription completes, the window SHALL auto-hide after 3 seconds. The window SHALL NOT be shown in the taskbar.

#### Scenario: Window properties
- **WHEN** the application starts
- **THEN** the window is 420x48, always on top, no decorations, transparent background, not shown in taskbar, and hidden

#### Scenario: Window shows on recording start
- **WHEN** a recording starts successfully (audio capture begins)
- **THEN** the main window becomes visible at its bottom-center position

#### Scenario: Window hides after transcription
- **WHEN** transcription completes and the result is delivered (pasted or copied)
- **THEN** the main window auto-hides after 3 seconds

#### Scenario: Window hides on empty result
- **WHEN** recording stops with no audio samples or silent audio
- **THEN** the main window hides immediately (no delay)

#### Scenario: Auto-hide cancelled by new recording
- **WHEN** a new recording starts while the auto-hide timer is pending
- **THEN** the auto-hide timer is cancelled and the window remains visible

#### Scenario: Manual show suppresses auto-hide
- **WHEN** the user manually shows the window via tray menu
- **THEN** the auto-hide timer does not fire until the next recording cycle

### Requirement: Frontend state events
The system SHALL emit events to the frontend: recording_state_changed, transcription_complete, model_download_progress, model_ready, foreground_app_info, recording_cancelled.

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
The system SHALL connect hotkey events to recording and transcription actions: pressed starts recording, released stops recording and triggers transcription followed by text insertion. The system SHALL hide both main and preview windows after the recording flow completes. The window visibility lifecycle is: hidden -> show on recording start -> visible during recording/transcription/processing -> auto-hide 3 seconds after completion.

#### Scenario: Press starts recording
- **WHEN** HotkeyEvent::Pressed is received and state is Idle
- **THEN** the system starts recording and transitions to Recording state

#### Scenario: Release triggers transcription pipeline
- **WHEN** HotkeyEvent::Released is received and state is Recording
- **THEN** the system stops recording, transcribes audio, inserts text at cursor, and transitions back to Idle

#### Scenario: Normal recording flow window lifecycle
- **WHEN** the user performs a complete recording cycle (start -> stop -> transcribe -> result)
- **THEN** the main window shows at recording start, remains visible during transcription, and auto-hides 3 seconds after the result is delivered

#### Scenario: Error during recording hides windows
- **WHEN** an error occurs during recording or transcription
- **THEN** both main and preview windows hide immediately after the error is displayed (3 seconds)
