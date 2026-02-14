## ADDED Requirements

### Requirement: Input field detection on macOS
The system SHALL detect whether the foreground application has a focused text input element using the macOS Accessibility API (AXUIElementCopyAttributeValue with AXFocusedUIElement and AXRole attributes).

#### Scenario: Text input focused
- **WHEN** the focused UI element has role AXTextField, AXTextArea, AXSearchField, or AXComboBox
- **THEN** `has_focused_text_input()` returns true

#### Scenario: No text input focused
- **WHEN** the focused UI element has a non-text role (e.g., AXButton, AXGroup) or no element is focused
- **THEN** `has_focused_text_input()` returns false

#### Scenario: Accessibility API failure
- **WHEN** the Accessibility API call fails (permission denied, element query error)
- **THEN** `has_focused_text_input()` returns false (safe fallback to clipboard-only mode)

### Requirement: Input field detection on Windows
The system SHALL detect whether the foreground application has a focused text input element using Windows UI Automation (IUIAutomation::GetFocusedElement with ControlType check).

#### Scenario: Edit control focused
- **WHEN** the focused element has ControlType of Edit, Document, or ComboBox
- **THEN** `has_focused_text_input()` returns true

#### Scenario: No edit control focused
- **WHEN** the focused element has a non-edit ControlType or query fails
- **THEN** `has_focused_text_input()` returns false

### Requirement: Output mode branching
The system SHALL branch the transcription output pipeline based on input field detection: auto-paste when a text input is focused, clipboard-only when no text input is focused.

#### Scenario: Auto-paste mode
- **WHEN** transcription completes and a text input is focused
- **THEN** the system uses `insert_text()` (save clipboard → set text → simulate paste → restore clipboard)

#### Scenario: Clipboard-only mode
- **WHEN** transcription completes and no text input is focused
- **THEN** the system uses `copy_only()` (set clipboard text without paste simulation or clipboard restore)

### Requirement: Copy-only clipboard function
The system SHALL provide a `copy_only(text)` function that sets the system clipboard without simulating paste or restoring previous clipboard content.

#### Scenario: Copy only
- **WHEN** `copy_only()` is called with non-empty text
- **THEN** the system clipboard contains the text, no paste keystroke is simulated

#### Scenario: Empty text
- **WHEN** `copy_only()` is called with empty text
- **THEN** the system takes no action

### Requirement: Copy-to-clipboard Tauri command
The system SHALL expose a `copy_to_clipboard` Tauri command that the frontend can invoke to copy text to the system clipboard.

#### Scenario: Frontend invokes copy
- **WHEN** the preview window calls `invoke("copy_to_clipboard", { text })`
- **THEN** the text is placed on the system clipboard
