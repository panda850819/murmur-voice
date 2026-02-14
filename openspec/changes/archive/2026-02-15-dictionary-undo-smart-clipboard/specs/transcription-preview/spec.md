## ADDED Requirements

### Requirement: Copy button in preview
The system SHALL display a Copy button in the preview footer that copies the current transcription text to the system clipboard.

#### Scenario: Copy button appears after result
- **WHEN** transcription completes with non-empty text
- **THEN** a Copy button is visible in the preview footer

#### Scenario: Copy button action
- **WHEN** the user clicks the Copy button
- **THEN** the displayed text is copied to the clipboard, the button shows "Copied!" for 1.5 seconds, and the preview auto-hides

#### Scenario: Copy button hidden for empty result
- **WHEN** transcription produces no text
- **THEN** the Copy button is not shown

### Requirement: Persistent preview in clipboard-only mode
The system SHALL keep the preview window visible indefinitely when the transcription output mode is "clipboard" (no text input focused), until the user copies the text or starts a new recording.

#### Scenario: No auto-hide in clipboard mode
- **WHEN** transcription completes in "clipboard" mode
- **THEN** the preview window remains visible with no auto-hide timer

#### Scenario: Next recording resets
- **WHEN** a new recording starts while persistent preview is showing
- **THEN** the preview resets to "Listening..." as normal

### Requirement: Editable preview text
The system SHALL allow the user to edit the transcription text directly in the preview window by clicking on it.

#### Scenario: Click to edit
- **WHEN** the user clicks on the preview text after transcription completes
- **THEN** the text becomes editable with visual feedback (subtle background highlight)

#### Scenario: Edit cancels auto-hide
- **WHEN** the user starts editing the preview text
- **THEN** any active auto-hide timer is cancelled

#### Scenario: Copy after edit
- **WHEN** the user edits the text and then clicks Copy
- **THEN** the edited (corrected) text is copied, not the original

### Requirement: Dictionary suggestion from edits
The system SHALL detect word-level changes when the user edits preview text and suggest adding new words to the dictionary.

#### Scenario: Word replacement detected
- **WHEN** the user edits the preview text and clicks away (blur)
- **THEN** the system diffs original vs edited text and identifies new words

#### Scenario: Suggestion bar shown
- **WHEN** new words are detected from the edit
- **THEN** a suggestion bar appears at the bottom of the preview: 'Add "term" to dictionary?' with Add and Dismiss buttons

#### Scenario: User accepts suggestion
- **WHEN** the user clicks "Add"
- **THEN** the suggested terms are added to the dictionary via the `add_dictionary_term` command

#### Scenario: User dismisses suggestion
- **WHEN** the user clicks "Dismiss"
- **THEN** the suggestion bar is hidden without modifying the dictionary

## MODIFIED Requirements

### Requirement: Auto-hide after result
The system SHALL automatically hide the preview window 10 seconds after the final result is displayed in auto-paste mode, with no auto-hide in clipboard-only mode. A minimum display time of 1 second prevents flicker on short recordings.

#### Scenario: Auto-hide after delay (pasted mode)
- **WHEN** the final result has been displayed for 10 seconds in "pasted" mode
- **THEN** the preview window hides automatically

#### Scenario: No auto-hide in clipboard mode
- **WHEN** the final result is displayed in "clipboard" mode
- **THEN** the preview window remains visible until copy or next recording

#### Scenario: Minimum display time
- **WHEN** a recording is very short (under 1 second of processing)
- **THEN** the preview window remains visible for at least 1 second before auto-hiding

#### Scenario: New recording interrupts auto-hide
- **WHEN** a new recording starts while the preview window is showing a previous result
- **THEN** the auto-hide timer is cancelled and the preview window resets to "Listening..."

### Requirement: Final result display
The system SHALL display the complete AI-processed transcription result in the preview window with a character count when transcription (and optional LLM processing) completes. The event payload SHALL include both the text and the output mode ("pasted" or "clipboard").

#### Scenario: Result with LLM processing
- **WHEN** LLM post-processing completes and produces final text
- **THEN** the preview window displays the full processed text and character count

#### Scenario: Result without LLM processing
- **WHEN** transcription completes and LLM is disabled
- **THEN** the preview window displays the raw transcription text and character count

#### Scenario: Empty transcription
- **WHEN** transcription produces an empty result
- **THEN** the preview window displays a "No speech detected" message
