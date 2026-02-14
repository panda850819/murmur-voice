## ADDED Requirements

### Requirement: Toggle mode hotkey debounce
The system SHALL enforce a minimum 500ms interval between toggle actions to prevent accidental double-trigger from duplicate modifier key events.

#### Scenario: Rapid double-press ignored
- **WHEN** recording_mode is "toggle" and the user presses the hotkey twice within 500ms
- **THEN** only the first press is processed (recording starts but does not immediately stop)

#### Scenario: Normal toggle timing
- **WHEN** recording_mode is "toggle" and the user presses the hotkey, waits more than 500ms, then presses again
- **THEN** both presses are processed normally (start then stop)

#### Scenario: Hold mode unaffected
- **WHEN** recording_mode is "hold"
- **THEN** no debounce is applied to hotkey events
