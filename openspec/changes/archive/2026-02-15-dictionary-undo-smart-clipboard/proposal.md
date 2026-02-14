## Why

Dictionary entries are permanently lost on delete with no way to undo, and the Cancel button doesn't restore unsaved changes. The transcription output pipeline always auto-pastes regardless of context — when there's no input field focused (e.g., Finder, Desktop), the paste fails silently and the text is lost. Additionally, toggle recording mode cuts off within seconds due to duplicate modifier key events on macOS.

## What Changes

- Dictionary tag deletion becomes undoable via a toast with 4-second timeout
- Cancel button in settings restores all unsaved dictionary changes (adds + deletes)
- Transcription output detects whether a text input field is focused before deciding to auto-paste or copy-to-clipboard
- Preview window stays persistent when no input field is detected, with a Copy button
- Preview text becomes editable (contenteditable); edits are diffed against original to suggest auto-adding corrected terms to the dictionary
- Toggle mode hotkey handler gets 500ms debounce to prevent accidental double-trigger
- Preview auto-hide extended from 3s to 10s in auto-paste mode

## Capabilities

### New Capabilities
- `smart-clipboard`: Detects focused text input via Accessibility API (macOS) / UI Automation (Windows) to branch between auto-paste and clipboard-only modes

### Modified Capabilities
- `personal-dictionary`: Add undo-on-delete toast, Cancel restores snapshot, auto-add from preview edits
- `transcription-preview`: Copy button, persistent mode for no-input contexts, editable text with dictionary suggestion bar
- `text-insertion`: Output pipeline branches on input field detection; new `copy_only` mode
- `toggle-recording`: 500ms debounce on hotkey press to prevent duplicate modifier key triggers

## Impact

- **Frontend**: `settings.js/html/css` (undo toast, cancel restore), `preview.js/html/css` (copy button, editable, dict suggest, persistent mode)
- **Rust backend**: `frontapp_macos.rs` (Accessibility API FFI), `frontapp_windows.rs` (UI Automation), `clipboard.rs` (new `copy_only`), `lib.rs` (output branching, debounce, 2 new Tauri commands)
- **Dependencies**: Windows crate needs `Win32_UI_Accessibility` and `Win32_System_Com` features added to Cargo.toml
- **Event payload**: `transcription_complete` changes from `String` to `{ text, mode }` JSON — preview.js must be updated to match
