## MODIFIED Requirements

### Requirement: Clipboard-based text insertion
The system SHALL insert transcribed text at the cursor position of the active application by detecting whether a text input is focused, then either: (a) saving the current clipboard, writing text to clipboard, simulating the platform paste shortcut, and restoring the original clipboard when a text input IS focused; or (b) copying text to clipboard without pasting when no text input is focused.

#### Scenario: Successful text insertion with input field
- **WHEN** transcription produces non-empty text and a text input field is focused
- **THEN** the system saves current clipboard content, writes transcription to clipboard, simulates the platform-appropriate paste keystroke, waits 100ms, and restores original clipboard

#### Scenario: Clipboard-only when no input field
- **WHEN** transcription produces non-empty text and no text input field is focused
- **THEN** the system writes transcription to clipboard without simulating paste or restoring previous clipboard content

#### Scenario: Empty transcription
- **WHEN** transcription produces an empty string
- **THEN** the system does not modify the clipboard or simulate any keystrokes

#### Scenario: Paste uses Cmd+V on macOS
- **WHEN** text insertion is triggered on macOS with a focused text input
- **THEN** the system simulates Cmd+V

#### Scenario: Paste uses Ctrl+V on Windows
- **WHEN** text insertion is triggered on Windows with a focused text input
- **THEN** the system simulates Ctrl+V
