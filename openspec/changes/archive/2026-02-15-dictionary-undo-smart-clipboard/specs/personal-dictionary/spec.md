## ADDED Requirements

### Requirement: Dictionary cancel restores snapshot
The system SHALL restore dictionary tags to their last-saved state when the user clicks Cancel in the settings window.

#### Scenario: Cancel after adding terms
- **WHEN** the user adds dictionary terms and clicks Cancel without saving
- **THEN** the added terms are discarded and the dictionary returns to its saved state

#### Scenario: Cancel after deleting terms
- **WHEN** the user deletes dictionary terms and clicks Cancel without saving
- **THEN** the deleted terms are restored and the dictionary returns to its saved state

### Requirement: Dictionary delete undo toast
The system SHALL display an undo toast for 4 seconds after a dictionary tag is deleted, allowing the user to restore the deleted term.

#### Scenario: Delete shows undo
- **WHEN** the user deletes a dictionary tag
- **THEN** an undo bar appears below the dictionary area showing the removed term and an "Undo" button

#### Scenario: Undo restores term
- **WHEN** the user clicks "Undo" within 4 seconds
- **THEN** the deleted term is restored at its original position in the tag list

#### Scenario: Undo expires
- **WHEN** 4 seconds pass without clicking Undo
- **THEN** the undo bar disappears

#### Scenario: Consecutive deletes
- **WHEN** the user deletes multiple tags in sequence
- **THEN** only the most recent deletion's undo is shown (replaces previous)

### Requirement: Auto-add dictionary term from preview
The system SHALL provide an `add_dictionary_term` Tauri command that appends a term to the dictionary settings, avoiding duplicates.

#### Scenario: Add new term
- **WHEN** `add_dictionary_term` is invoked with a term not already in the dictionary
- **THEN** the term is appended to the dictionary string and settings are saved to disk

#### Scenario: Duplicate term
- **WHEN** `add_dictionary_term` is invoked with a term already in the dictionary
- **THEN** the dictionary is not modified

#### Scenario: Empty term
- **WHEN** `add_dictionary_term` is invoked with an empty or whitespace-only string
- **THEN** the dictionary is not modified
