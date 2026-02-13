## MODIFIED Requirements

### Requirement: Clipboard-based text insertion
The system SHALL insert transcribed text at the cursor position of the active application by saving the current clipboard, writing text to clipboard, simulating the platform paste shortcut (Cmd+V on macOS, Ctrl+V on Windows), and restoring the original clipboard.

#### Scenario: Successful text insertion
- **WHEN** transcription produces non-empty text
- **THEN** the system saves current clipboard content, writes transcription to clipboard, simulates the platform-appropriate paste keystroke, waits 100ms, and restores original clipboard

#### Scenario: Empty transcription
- **WHEN** transcription produces an empty string
- **THEN** the system does not modify the clipboard or simulate any keystrokes

#### Scenario: Paste uses Cmd+V on macOS
- **WHEN** text insertion is triggered on macOS
- **THEN** the system simulates Cmd+V

#### Scenario: Paste uses Ctrl+V on Windows
- **WHEN** text insertion is triggered on Windows
- **THEN** the system simulates Ctrl+V
