## Requirements

### Requirement: Auto-hide after result
The system SHALL automatically hide the preview and main windows 3 seconds after the final result is displayed in auto-paste mode, using a frontend timer that can be cancelled by user interaction. In clipboard-only mode, auto-hide after 30 seconds.

#### Scenario: Auto-hide after delay (pasted mode)
- **WHEN** the final result has been displayed for 3 seconds in "pasted" mode and the user has not interacted with the preview text
- **THEN** the frontend calls `hide_overlay_windows` to hide both preview and main windows

#### Scenario: No auto-hide in clipboard mode
- **WHEN** the final result is displayed in "clipboard" mode
- **THEN** the preview window auto-hides after 30 seconds

#### Scenario: Editing cancels auto-hide
- **WHEN** the user focuses on the preview text to edit it
- **THEN** the auto-hide timer is cancelled

#### Scenario: Blur restarts auto-hide
- **WHEN** the user blurs the preview text (stops editing) in pasted mode
- **THEN** a new 3-second auto-hide timer starts

#### Scenario: New recording cancels auto-hide
- **WHEN** a new recording starts while the auto-hide timer is running
- **THEN** the timer is cancelled and the preview resets to "Listening..."

#### Scenario: Hide overlay windows command
- **WHEN** the frontend invokes the `hide_overlay_windows` Tauri command
- **THEN** both the preview window and main window are hidden
