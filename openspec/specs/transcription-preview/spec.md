## ADDED Requirements

### Requirement: Preview window display
The system SHALL display a floating preview window (approximately 420x280) with a dark translucent background above the main bar during recording and transcription. The window SHALL NOT steal focus from the foreground application.

#### Scenario: Window appears on recording start
- **WHEN** the recording state changes to "recording"
- **THEN** the preview window appears above the main bar with "Listening..." placeholder text

#### Scenario: Window does not steal focus
- **WHEN** the preview window appears
- **THEN** the foreground application retains focus and keyboard input

#### Scenario: Window not shown in taskbar
- **WHEN** the preview window is visible
- **THEN** it does not appear in the macOS Dock or taskbar

### Requirement: Live transcription display
The system SHALL display live transcription text in the preview window when using the local Whisper engine, updating in real-time as audio is processed.

#### Scenario: Local engine live preview
- **WHEN** the transcription engine is "local" and live transcription updates are received
- **THEN** the preview window displays the current partial transcription text

#### Scenario: Groq engine no live preview
- **WHEN** the transcription engine is "groq" and recording is in progress
- **THEN** the preview window displays "Listening..." without live updates

### Requirement: Final result display
The system SHALL display the complete AI-processed transcription result in the preview window with a character count when transcription (and optional LLM processing) completes.

#### Scenario: Result with LLM processing
- **WHEN** LLM post-processing completes and produces final text
- **THEN** the preview window displays the full processed text and character count

#### Scenario: Result without LLM processing
- **WHEN** transcription completes and LLM is disabled
- **THEN** the preview window displays the raw transcription text and character count

#### Scenario: Empty transcription
- **WHEN** transcription produces an empty result
- **THEN** the preview window displays a "No speech detected" message

### Requirement: Processing state indicators
The system SHALL display the current processing stage in the preview window header during the transcription pipeline.

#### Scenario: Transcribing state
- **WHEN** the recording state changes to "transcribing"
- **THEN** the preview window header displays "Transcribing..."

#### Scenario: LLM processing state
- **WHEN** the recording state changes to "processing"
- **THEN** the preview window header displays "Processing..."

#### Scenario: Result ready state
- **WHEN** the final text is available
- **THEN** the preview window header displays the first line or truncated summary of the result

### Requirement: Auto-hide after result
The system SHALL automatically hide the preview window 3 seconds after the final result is displayed, with a minimum display time of 1 second to prevent flicker on short recordings.

#### Scenario: Auto-hide after delay
- **WHEN** the final result has been displayed for 3 seconds
- **THEN** the preview window hides automatically

#### Scenario: Minimum display time
- **WHEN** a recording is very short (under 1 second of processing)
- **THEN** the preview window remains visible for at least 1 second before auto-hiding

#### Scenario: New recording interrupts auto-hide
- **WHEN** a new recording starts while the preview window is showing a previous result
- **THEN** the auto-hide timer is cancelled and the preview window resets to "Listening..."

### Requirement: Foreground app display in preview
The system SHALL display the detected foreground application name in the preview window when app-aware style is enabled.

#### Scenario: App detected and displayed
- **WHEN** transcription completes and app-aware style is enabled
- **THEN** the preview window displays the foreground app name (e.g., "Slack", "VS Code") near the character count

#### Scenario: App-aware style disabled
- **WHEN** app-aware style is disabled in settings
- **THEN** the preview window does not display any app name
