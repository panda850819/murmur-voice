## 1. Dictionary Cancel + Undo

- [x] 1.1 Add `dictTagsSnapshot` variable and capture snapshot after loading settings in `src/settings.js`
- [x] 1.2 Update Cancel button handler to restore `dictTags` from snapshot before closing in `src/settings.js`
- [x] 1.3 Add undo container HTML to `src/settings.html` after dict-input-wrap
- [x] 1.4 Add undo toast styles to `src/settings.css`
- [x] 1.5 Replace `removeDictTag()` with undo-aware version and add `showDictUndo`/`doDictUndo` functions in `src/settings.js`
- [x] 1.6 Add undo button click listener in DOMContentLoaded handler

## 2. Toggle Mode Debounce

- [x] 2.1 Add `last_toggle: Option<Instant>` variable in hotkey handler thread in `src-tauri/src/lib.rs`
- [x] 2.2 Add 500ms debounce check at the start of the toggle match arm
- [x] 2.3 Run `cargo check` and `cargo test` to verify no regressions

## 3. Clipboard copy_only Function

- [x] 3.1 Add `copy_only(text)` function to `src-tauri/src/clipboard.rs`
- [x] 3.2 Run `cargo check` to verify compilation

## 4. Input Field Detection — macOS

- [x] 4.1 Add Accessibility API FFI bindings (AXUIElement, CFString helpers) to `src-tauri/src/frontapp_macos.rs`
- [x] 4.2 Implement `has_focused_text_input()` using AXUIElementCreateSystemWide + AXRole check in `src-tauri/src/frontapp_macos.rs`
- [x] 4.3 Run `cargo check` to verify compilation

## 5. Input Field Detection — Windows

- [x] 5.1 Add `Win32_UI_Accessibility` and `Win32_System_Com` features to windows crate in `src-tauri/Cargo.toml`
- [x] 5.2 Implement `has_focused_text_input()` using IUIAutomation in `src-tauri/src/frontapp_windows.rs`

## 6. Smart Clipboard Backend

- [x] 6.1 Replace output pipeline in `do_stop_recording()` with input-field-aware branching in `src-tauri/src/lib.rs`
- [x] 6.2 Change `transcription_complete` event payload from String to `{ text, mode }` JSON
- [x] 6.3 Change auto-hide timer: 10s for pasted mode, no timer for clipboard mode
- [x] 6.4 Add `copy_to_clipboard` Tauri command in `src-tauri/src/lib.rs`
- [x] 6.5 Add `add_dictionary_term` Tauri command in `src-tauri/src/lib.rs`
- [x] 6.6 Register both new commands in `generate_handler!` macro
- [x] 6.7 Run `cargo clippy --all-targets -- -D warnings` and `cargo test`

## 7. Preview UI — Copy Button + Persistent Mode

- [x] 7.1 Add Copy button and footer-actions wrapper to `src/preview.html`
- [x] 7.2 Add copy button and copied state styles to `src/preview.css`
- [x] 7.3 Update `transcription_complete` listener in `src/preview.js` to handle `{ text, mode }` payload
- [x] 7.4 Implement copy button click handler with `invoke("copy_to_clipboard")` and "Copied!" feedback
- [x] 7.5 Implement persistent mode (no auto-hide) when mode is "clipboard"

## 8. Editable Preview + Auto-add Dictionary

- [x] 8.1 Add dict suggestion bar HTML to `src/preview.html`
- [x] 8.2 Add editing and suggestion bar styles to `src/preview.css`
- [x] 8.3 Add contenteditable enable/disable functions and word-level diff function to `src/preview.js`
- [x] 8.4 Enable contenteditable on transcription complete, store originalText
- [x] 8.5 Add blur handler for diff detection and suggestion display
- [x] 8.6 Add suggestion bar button handlers (Add invokes `add_dictionary_term`, Dismiss hides)
- [x] 8.7 Cancel auto-hide timer when user starts editing

## 9. Integration Verification

- [x] 9.1 Run `cargo clippy --all-targets -- -D warnings` with zero warnings
- [x] 9.2 Run `cargo test` — all tests pass
- [ ] 9.3 Manual test: dictionary Cancel restores, undo toast works
- [ ] 9.4 Manual test: toggle mode debounce prevents accidental stop
- [ ] 9.5 Manual test: auto-paste in text editor, clipboard-only on Desktop/Finder
- [ ] 9.6 Manual test: preview Copy button, edit + dict suggestion
