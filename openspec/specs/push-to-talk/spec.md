## MODIFIED Requirements

### Requirement: Global hotkey detection
The system SHALL detect the configured modifier key press and release events globally using platform-specific APIs (CGEventTap on macOS, `SetWindowsHookExW` on Windows), regardless of which application has focus. In hold mode, press starts and release stops recording. In toggle mode, press toggles recording state and release is ignored.

#### Scenario: Hold mode - key pressed
- **WHEN** recording_mode is "hold" and the user presses the hotkey
- **THEN** the system starts recording

#### Scenario: Hold mode - key released
- **WHEN** recording_mode is "hold" and the user releases the hotkey
- **THEN** the system stops recording and begins transcription

#### Scenario: Toggle mode - first press
- **WHEN** recording_mode is "toggle" and the system is idle and the user presses the hotkey
- **THEN** the system starts recording

#### Scenario: Toggle mode - second press
- **WHEN** recording_mode is "toggle" and the system is recording and the user presses the hotkey
- **THEN** the system stops recording and begins transcription

#### Scenario: Toggle mode - release ignored
- **WHEN** recording_mode is "toggle" and the user releases the hotkey
- **THEN** no action is taken

#### Scenario: Other keys ignored
- **WHEN** any key other than the configured hotkey is pressed or released
- **THEN** the system does not emit any hotkey event
