## ADDED Requirements

### Requirement: Toggle recording mode
The system SHALL support a toggle recording mode where the first hotkey press starts recording and the second press stops recording.

#### Scenario: First press starts recording
- **WHEN** recording_mode is "toggle" and the system is idle and the user presses the hotkey
- **THEN** the system starts recording

#### Scenario: Second press stops recording
- **WHEN** recording_mode is "toggle" and the system is recording and the user presses the hotkey
- **THEN** the system stops recording and begins transcription

#### Scenario: Key release ignored in toggle mode
- **WHEN** recording_mode is "toggle" and the user releases the hotkey
- **THEN** the system takes no action

### Requirement: Recording mode setting
The system SHALL store the recording mode preference ("hold" or "toggle") in settings, defaulting to "hold".

#### Scenario: Default mode
- **WHEN** no recording_mode is configured
- **THEN** the system uses "hold" mode (current behavior)

#### Scenario: Mode persisted
- **WHEN** the user changes recording_mode in settings
- **THEN** the preference is saved and applied immediately without restart

### Requirement: Maximum recording duration
The system SHALL automatically stop recording after 5 minutes in toggle mode to prevent accidental indefinite recording.

#### Scenario: Recording exceeds 5 minutes
- **WHEN** recording has been active for 5 minutes in toggle mode
- **THEN** the system automatically stops recording and begins transcription

#### Scenario: Hold mode unaffected
- **WHEN** recording_mode is "hold"
- **THEN** no maximum duration is enforced
