## MODIFIED Requirements

### Requirement: Auto-hide after result
The system SHALL automatically hide the preview and main windows 5 seconds after the final result is displayed in auto-paste mode, using a frontend timer that can be cancelled by user interaction. No auto-hide in clipboard-only mode.

#### Scenario: Auto-hide after delay (pasted mode)
- **WHEN** the final result has been displayed for 5 seconds in "pasted" mode and the user has not interacted with the preview text
- **THEN** the frontend calls `hide_overlay_windows` to hide both preview and main windows

#### Scenario: No auto-hide in clipboard mode
- **WHEN** the final result is displayed in "clipboard" mode
- **THEN** the preview window remains visible until copy or next recording

#### Scenario: Editing cancels auto-hide
- **WHEN** the user focuses on the preview text to edit it
- **THEN** the auto-hide timer is cancelled

#### Scenario: Blur restarts auto-hide
- **WHEN** the user blurs the preview text (stops editing)
- **THEN** a new 5-second auto-hide timer starts

#### Scenario: New recording cancels auto-hide
- **WHEN** a new recording starts while the auto-hide timer is running
- **THEN** the timer is cancelled and the preview resets to "Listening..."

#### Scenario: Hide overlay windows command
- **WHEN** the frontend invokes the `hide_overlay_windows` Tauri command
- **THEN** both the preview window and main window are hidden
