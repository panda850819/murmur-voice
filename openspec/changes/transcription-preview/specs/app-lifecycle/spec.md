## ADDED Requirements

### Requirement: App badge in main bar
The system SHALL display the detected foreground application name as a small badge in the main bar when app-aware style is enabled and a transcription has been performed.

#### Scenario: App badge shown after transcription
- **WHEN** transcription completes and app-aware style is enabled
- **THEN** the main bar displays the foreground app name as a badge (e.g., "Slack", "Code")

#### Scenario: App badge hidden when disabled
- **WHEN** app-aware style is disabled in settings
- **THEN** no app badge is displayed in the main bar

#### Scenario: App badge cleared on idle
- **WHEN** the system returns to idle state and no transcription is in progress
- **THEN** the app badge is cleared from the main bar

### Requirement: Foreground app info event
The system SHALL emit a `foreground_app_info` event to the frontend containing the detected app name and style category when transcription begins.

#### Scenario: Event emitted with app info
- **WHEN** recording stops and transcription begins
- **THEN** the system emits `foreground_app_info` with `{ name: "Slack", style: "casual" }` (or equivalent for the detected app)

#### Scenario: Unknown app emits default
- **WHEN** the foreground app is not recognized
- **THEN** the system emits `foreground_app_info` with `{ name: "Unknown", style: "default" }`

## MODIFIED Requirements

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
