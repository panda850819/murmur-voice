## ADDED Requirements

### Requirement: Windows foreground application detection
The system SHALL detect the foreground application on Windows by calling `GetForegroundWindow` to get the active window handle, then `GetWindowThreadProcessId` to get the process ID, then `OpenProcess` + `QueryFullProcessImageNameW` to get the executable path, and finally extract the executable name (e.g., `Code.exe`, `slack.exe`).

#### Scenario: Detect VS Code as foreground app
- **WHEN** VS Code is the foreground application on Windows
- **THEN** the system returns "Code.exe" as the foreground app identifier

#### Scenario: Detect Slack as foreground app
- **WHEN** Slack is the foreground application on Windows
- **THEN** the system returns "Slack.exe" as the foreground app identifier

#### Scenario: No foreground window
- **WHEN** no window has focus (e.g., desktop is focused)
- **THEN** the system returns a default/empty identifier

### Requirement: Windows app-to-style mapping
The system SHALL map Windows executable names to the same style presets used for macOS bundle IDs in `style_for_app()`. The mapping SHALL cover common applications (e.g., `Code.exe` maps to the code style, `Slack.exe` maps to the chat style).

#### Scenario: Code.exe maps to code style
- **WHEN** the foreground app is "Code.exe"
- **THEN** `style_for_app()` returns the code/technical writing style preset

#### Scenario: Unknown exe maps to default style
- **WHEN** the foreground app executable name has no configured style mapping
- **THEN** `style_for_app()` returns the default style preset
