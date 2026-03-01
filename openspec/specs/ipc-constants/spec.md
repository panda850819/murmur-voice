# ipc-constants Specification

## Purpose
TBD - created by archiving change simplify-followup. Update Purpose after archive.
## Requirements
### Requirement: Rust event constants module
The system SHALL define all IPC event names and state strings in a dedicated `events.rs` module. All backend code SHALL use these constants instead of raw string literals when emitting events.

#### Scenario: Event name constant used for recording state
- **WHEN** the backend emits a recording state change event
- **THEN** it uses `events::RECORDING_STATE_CHANGED` constant, not `"recording_state_changed"` literal

#### Scenario: State string constant used for idle
- **WHEN** the backend emits the idle state
- **THEN** it uses `events::states::IDLE` constant, not `"idle"` literal

#### Scenario: All event names covered
- **WHEN** inspecting the events module
- **THEN** constants exist for all 14 event types: model_download_progress, model_ready, recording_state_changed, partial_transcription, transcription_complete, foreground_app_info, opacity_changed, recording_error, accessibility_error, accessibility_granted, enhancer_info, recording_cancelled

#### Scenario: All state strings covered
- **WHEN** inspecting the events::states submodule
- **THEN** constants exist for all 6 states: idle, starting, recording, stopping, transcribing, processing

### Requirement: JS command constants
The system SHALL define all Tauri command names in the `events.js` module as a `COMMANDS` object. All frontend code SHALL use `COMMANDS.X` instead of raw string literals in `invoke()` calls.

#### Scenario: Command constant used for get_settings
- **WHEN** the frontend invokes the get_settings command
- **THEN** it uses `COMMANDS.GET_SETTINGS`, not `invoke("get_settings")`

#### Scenario: All command names covered
- **WHEN** inspecting the COMMANDS object in events.js
- **THEN** constants exist for all Tauri commands used in the frontend

