# push-to-talk Specification

## Purpose
TBD - created by archiving change build-mvp-voice-to-text. Update Purpose after archive.
## Requirements
### Requirement: Global hotkey detection
The system SHALL detect Right Option key press and release events globally using rdev, regardless of which application has focus.

#### Scenario: Right Option key pressed
- **WHEN** the user presses the Right Option key
- **THEN** the system emits a HotkeyEvent::Pressed via mpsc channel

#### Scenario: Right Option key released
- **WHEN** the user releases the Right Option key
- **THEN** the system emits a HotkeyEvent::Released via mpsc channel

#### Scenario: Other keys ignored
- **WHEN** any key other than Right Option is pressed or released
- **THEN** the system does not emit any hotkey event

### Requirement: Background listener thread
The system SHALL run the rdev keyboard listener on a dedicated background thread that starts at application launch and runs until application exit.

#### Scenario: Listener starts with app
- **WHEN** the application starts
- **THEN** rdev::listen is spawned on a dedicated std::thread

#### Scenario: Listener thread independence
- **WHEN** the listener thread is running
- **THEN** it does not block the main Tauri event loop or UI thread

