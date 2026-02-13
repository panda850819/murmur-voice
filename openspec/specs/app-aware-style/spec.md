## ADDED Requirements

### Requirement: Foreground app detection
The system SHALL detect the currently focused application using macOS NSWorkspace API when transcription completes.

#### Scenario: App detected
- **WHEN** transcription completes
- **THEN** the system identifies the foreground app's bundle identifier

#### Scenario: Detection fails
- **WHEN** foreground app cannot be determined
- **THEN** the system uses the default style

### Requirement: Style presets by app category
The system SHALL apply different LLM prompt styles based on the detected app category.

#### Scenario: Email app (Mail, Outlook, Gmail in browser)
- **WHEN** the foreground app is an email client
- **THEN** the LLM uses a formal, professional tone

#### Scenario: Chat app (Slack, Discord, Messages, Telegram)
- **WHEN** the foreground app is a messaging client
- **THEN** the LLM uses a casual, concise tone

#### Scenario: Code editor (VS Code, Xcode, cursor)
- **WHEN** the foreground app is a code editor
- **THEN** the LLM preserves technical terms and code-related vocabulary without reformatting

#### Scenario: Unknown app
- **WHEN** the foreground app is not in any known category
- **THEN** the LLM uses a neutral default style

### Requirement: App-aware style toggle
The system SHALL provide a setting to enable or disable app-aware styling, defaulting to enabled.

#### Scenario: Style disabled
- **WHEN** app_aware_style is disabled
- **THEN** the system uses the default style regardless of foreground app
