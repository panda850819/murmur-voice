## ADDED Requirements

### Requirement: Clipboard-based text insertion
The system SHALL insert transcribed text at the cursor position of the active application by saving the current clipboard, writing text to clipboard, simulating Cmd+V, and restoring the original clipboard.

#### Scenario: Successful text insertion
- **WHEN** transcription produces non-empty text
- **THEN** the system saves current clipboard content, writes transcription to clipboard, simulates Cmd+V keystroke, waits 100ms, and restores original clipboard

#### Scenario: Empty transcription
- **WHEN** transcription produces an empty string
- **THEN** the system does not modify the clipboard or simulate any keystrokes

### Requirement: Focus delay
The system SHALL wait 100ms before simulating Cmd+V to allow focus to return to the target application after any overlay interaction.

#### Scenario: Delay before paste
- **WHEN** text insertion is triggered
- **THEN** the system waits 100ms before simulating the Cmd+V keystroke
