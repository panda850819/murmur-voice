## ADDED Requirements

### Requirement: Windows global hotkey detection
The system SHALL detect the configured modifier key press and release events globally on Windows using a low-level keyboard hook (`SetWindowsHookExW` with `WH_KEYBOARD_LL`), regardless of which application has focus. The hook SHALL run in a dedicated thread with a Windows message pump (`GetMessageW` loop).

#### Scenario: Hold mode - key pressed on Windows
- **WHEN** the platform is Windows and recording_mode is "hold" and the user presses the configured hotkey
- **THEN** the system starts recording

#### Scenario: Hold mode - key released on Windows
- **WHEN** the platform is Windows and recording_mode is "hold" and the user releases the configured hotkey
- **THEN** the system stops recording and begins transcription

#### Scenario: Toggle mode - first press on Windows
- **WHEN** the platform is Windows and recording_mode is "toggle" and the system is idle and the user presses the configured hotkey
- **THEN** the system starts recording

#### Scenario: Toggle mode - second press on Windows
- **WHEN** the platform is Windows and recording_mode is "toggle" and the system is recording and the user presses the configured hotkey
- **THEN** the system stops recording and begins transcription

#### Scenario: Other keys ignored on Windows
- **WHEN** any key other than the configured hotkey is pressed or released on Windows
- **THEN** the system does not emit any hotkey event

### Requirement: Windows PTT key mapping
The system SHALL map JS `event.code` key identifiers (e.g., `AltLeft`, `ShiftRight`, `MetaLeft`) to Windows virtual key codes (`VK_LMENU`, `VK_RMENU`, `VK_LWIN`, `VK_RWIN`, `VK_LSHIFT`, `VK_RSHIFT`, `VK_LCONTROL`, `VK_RCONTROL`) for hotkey matching.

#### Scenario: Map AltLeft to VK_LMENU
- **WHEN** the PTT key setting is "AltLeft"
- **THEN** the system uses `VK_LMENU` (0xA4) for hotkey detection

#### Scenario: Map MetaLeft to VK_LWIN
- **WHEN** the PTT key setting is "MetaLeft"
- **THEN** the system uses `VK_LWIN` (0x5B) for hotkey detection

#### Scenario: Runtime hotkey mask update on Windows
- **WHEN** the user changes the PTT key in settings
- **THEN** the hotkey mask is updated atomically without restarting the hook thread

### Requirement: No special permissions required on Windows
The system SHALL register low-level keyboard hooks on Windows without requiring elevated privileges or special accessibility permissions.

#### Scenario: Hook registers without admin
- **WHEN** the application starts on Windows as a standard user
- **THEN** the keyboard hook is successfully registered
