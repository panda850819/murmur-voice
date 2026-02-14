# Dictionary Undo + Smart Clipboard + Editable Preview

## Goal

Four improvements to Murmur Voice:
1. Dictionary entries can be undone/cancelled before saving
2. Smart clipboard: auto-paste when input field detected, persistent preview otherwise
3. Toggle mode debounce to prevent accidental stop
4. Editable preview with auto-add dictionary suggestions

## Current State

- **Dictionary**: `dictTags` array mutated immediately on delete. Cancel button only closes window (no restore). Save writes to Rust backend.
- **Preview**: Read-only `<p>` element. Auto-hides 3 seconds after transcription. No copy button.
- **Clipboard**: `insert_text()` saves clipboard → sets text → simulates Cmd+V → restores clipboard. Always auto-pastes.
- **Toggle mode**: No debounce on hotkey events. macOS `CGEventFlagsChanged` can fire duplicates, causing premature stop within seconds.

## Design

### 1. Dictionary Cancel + Undo (frontend only)

**Cancel restores all unsaved changes:**
- On `DOMContentLoaded`, after loading dictionary from backend, snapshot `dictTags` into `dictTagsSnapshot = [...dictTags]`
- Cancel button: restore `dictTags = [...dictTagsSnapshot]`, then close window

**Delete shows undo toast:**
- `removeDictTag()` saves `{ term, index }` before splicing
- Shows undo bar below dictionary area: "Term removed — Undo" (4 second timeout)
- Undo restores the term at original index via `splice(index, 0, term)`
- New delete replaces previous undo (only one active at a time)

Files: `src/settings.js`, `src/settings.html`, `src/settings.css`

### 2. Smart Clipboard — Input Field Detection

**macOS: Accessibility API**
- New function `has_focused_text_input() -> bool` in `frontapp_macos.rs`
- Uses `AXUIElementCopySystemWideElement` → `AXUIElementCopyAttributeValue(kAXFocusedUIElementAttribute)` → check `AXRole` for text input roles (AXTextField, AXTextArea, AXSearchField, AXComboBox, AXWebArea)
- Already requires Accessibility permission (CGEventTap needs it)

**Windows: UI Automation**
- Equivalent `has_focused_text_input()` in `frontapp_windows.rs`
- Uses `IUIAutomation::GetFocusedElement()` → check control type

**Dispatcher** in `frontapp.rs`: re-exports `has_focused_text_input()`

**`do_stop_recording()` branching in `lib.rs`:**
- After transcription + LLM processing:
  - `has_focused_text_input() == true` → `clipboard::insert_text()` (auto-paste + restore clipboard, current behavior)
  - `has_focused_text_input() == false` → `clipboard::copy_only()` (set clipboard, no paste, no restore)
- Emit `transcription_complete` with payload `{ text, mode: "pasted" | "clipboard" }`
- Preview auto-hide: "pasted" mode → 10 seconds. "clipboard" mode → no auto-hide.

**New Rust function** `clipboard::copy_only(text)`: sets clipboard text without paste simulation or restore.

**New Tauri command** `copy_to_clipboard(text)`: for preview Copy button to invoke.

**Preview changes:**
- Add Copy button in preview footer (visible always, but primary action in "clipboard" mode)
- Copy button: `invoke("copy_to_clipboard", { text })` → show "Copied!" badge 1.5s → auto-hide preview + main window

Files: `src-tauri/src/frontapp_macos.rs`, `src-tauri/src/frontapp_windows.rs`, `src-tauri/src/frontapp.rs`, `src-tauri/src/clipboard.rs`, `src-tauri/src/lib.rs`, `src/preview.html`, `src/preview.js`, `src/preview.css`

### 3. Toggle Mode Debounce

- In `lib.rs` hotkey handler, add `last_toggle_time: Instant` tracking
- On toggle press: check if `elapsed() < 500ms` → skip
- Prevents accidental double-trigger from macOS `CGEventFlagsChanged` duplicate events

Files: `src-tauri/src/lib.rs`

### 4. Editable Preview + Auto-add Dictionary

**Preview text becomes editable:**
- Change `<p id="preview-text">` to `contenteditable="false"` by default
- On click: set `contenteditable="true"`, add `editing` class for visual feedback
- Store original text in `originalText` variable on `transcription_complete`

**Diff and suggest:**
- On blur or explicit "Done" action: compare `originalText` vs current `textContent`
- Simple word-level diff: split both by whitespace, find replaced segments
- For each replaced segment: show suggestion bar in footer "Add 'X' to dictionary? [Yes] [Dismiss]"
- "Yes" → `invoke("add_dictionary_term", { term })` → update settings

**New Tauri command** `add_dictionary_term(term)`:
- Lock settings, append term to `dictionary` field (comma-separated), save

**After edit + copy:**
- Copy button copies the edited (corrected) text, not the original

Files: `src-tauri/src/lib.rs` (new command), `src/preview.html`, `src/preview.js`, `src/preview.css`

## File Impact Summary

| File | Changes |
|------|---------|
| `src/settings.js` | dictTags snapshot + undo logic |
| `src/settings.html` | undo container element |
| `src/settings.css` | undo toast styles |
| `src-tauri/src/frontapp_macos.rs` | `has_focused_text_input()` via Accessibility API |
| `src-tauri/src/frontapp_windows.rs` | `has_focused_text_input()` via UI Automation |
| `src-tauri/src/frontapp.rs` | dispatch `has_focused_text_input()` |
| `src-tauri/src/clipboard.rs` | new `copy_only()` function |
| `src-tauri/src/lib.rs` | `do_stop_recording` branching, debounce, `copy_to_clipboard` + `add_dictionary_term` commands |
| `src/preview.html` | Copy button, contenteditable |
| `src/preview.js` | Copy logic, edit mode, diff, dict suggestion, persistent mode |
| `src/preview.css` | Copy button, editing state, suggestion bar styles |
