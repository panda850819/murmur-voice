# Dictionary Undo + Smart Clipboard Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add dictionary undo/cancel, smart clipboard with input field detection, toggle debounce, and editable preview with auto-dictionary.

**Architecture:** Four independent features sharing the same codebase. Tasks 1-2 are standalone. Task 3 modifies the transcription output pipeline (Rust backend + preview frontend). Task 4 extends the preview UI from Task 3.

**Tech Stack:** Rust (Tauri 2, macOS Accessibility API, Windows UI Automation), vanilla JS/HTML/CSS (no bundler)

---

### Task 1: Dictionary Cancel Restore

**Files:**
- Modify: `src/settings.js:32` (add snapshot variable)
- Modify: `src/settings.js:164` (snapshot after load)
- Modify: `src/settings.js:252` (cancel handler)

**Step 1: Add snapshot variable and capture on load**

In `src/settings.js`, add a new state variable at line 32 (after `let dictTags = [];`):

```js
let dictTagsSnapshot = [];
```

In the `DOMContentLoaded` handler, after `loadDictFromString(s.dictionary || "")` (line 164), add:

```js
dictTagsSnapshot = [...dictTags];
```

**Step 2: Update Cancel to restore snapshot**

Replace the cancel handler (line 252):

```js
// Cancel — restore dictionary to last-saved state, then close
el("btn-cancel").addEventListener("click", () => {
  dictTags = [...dictTagsSnapshot];
  getCurrentWindow().close();
});
```

**Step 3: Verify manually**

1. Run `pnpm tauri dev`
2. Open Settings, add a dictionary term, click Cancel
3. Reopen Settings — the added term should NOT be there
4. Open Settings, delete a term, click Cancel
5. Reopen Settings — the deleted term should still be there

**Step 4: Commit**

```bash
git add src/settings.js
git commit -m "feat(settings): restore dictionary on Cancel"
```

---

### Task 2: Dictionary Delete Undo Toast

**Files:**
- Modify: `src/settings.html:82-83` (add undo container after dict-input-wrap)
- Modify: `src/settings.css` (add undo toast styles)
- Modify: `src/settings.js:101-135` (undo logic in removeDictTag)

**Step 1: Add undo container to HTML**

In `src/settings.html`, after the closing `</div>` of `dict-input-wrap` (line 82) and before the closing `</div>` of `dict-section` (line 83), add:

```html
<div id="dict-undo" class="dict-undo" style="display: none;">
  <span id="dict-undo-text"></span>
  <button id="dict-undo-btn" class="dict-undo-btn">Undo</button>
</div>
```

**Step 2: Add undo toast styles**

Append to `src/settings.css`:

```css
/* ── Dictionary Undo Toast ── */

.dict-undo {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  background: var(--bg-elevated);
  border-top: 1px solid var(--row-border);
  font-size: 11px;
  color: var(--text-secondary);
  animation: fadeIn 0.15s ease;
}

.dict-undo-btn {
  font-family: inherit;
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  transition: background 0.12s;
}

.dict-undo-btn:hover {
  background: var(--accent-glow);
}
```

**Step 3: Add undo logic to settings.js**

Add undo state variable after `dictTagsSnapshot` declaration:

```js
let undoTimer = null;
let undoEntry = null;
```

Replace `removeDictTag` function (lines 132-135):

```js
function removeDictTag(index) {
  const removed = dictTags.splice(index, 1)[0];
  renderDictTags();
  showDictUndo(removed, index);
}

function showDictUndo(term, index) {
  // Clear any previous undo timer
  if (undoTimer) clearTimeout(undoTimer);

  undoEntry = { term, index };
  const undo = el("dict-undo");
  el("dict-undo-text").textContent = `"${term}" removed`;
  undo.style.display = "flex";

  undoTimer = setTimeout(() => {
    undo.style.display = "none";
    undoEntry = null;
    undoTimer = null;
  }, 4000);
}

function doDictUndo() {
  if (!undoEntry) return;
  const { term, index } = undoEntry;
  // Insert back at original position (clamped to current length)
  const pos = Math.min(index, dictTags.length);
  dictTags.splice(pos, 0, term);
  renderDictTags();

  // Hide toast
  if (undoTimer) clearTimeout(undoTimer);
  el("dict-undo").style.display = "none";
  undoEntry = null;
  undoTimer = null;
}
```

In the `DOMContentLoaded` handler, after the dict-input blur listener (after line 218), add:

```js
// Dictionary undo button
el("dict-undo-btn").addEventListener("click", doDictUndo);
```

**Step 4: Verify manually**

1. Run `pnpm tauri dev`
2. Open Settings, delete a dictionary term
3. Undo toast should appear with term name and "Undo" button
4. Click Undo — term reappears at original position
5. Delete a term and wait 4 seconds — toast disappears
6. Delete two terms rapidly — only the most recent shows undo

**Step 5: Commit**

```bash
git add src/settings.js src/settings.html src/settings.css
git commit -m "feat(settings): add undo toast for dictionary deletions"
```

---

### Task 3: Toggle Mode Debounce

**Files:**
- Modify: `src-tauri/src/lib.rs:727-813` (hotkey handler thread)

**Step 1: Add debounce to toggle mode handler**

In `lib.rs`, inside the hotkey handler thread spawn (line 727), add a timestamp variable after `let mut is_recording = false;` (line 728):

```rust
let mut is_recording = false;
let mut last_toggle: Option<std::time::Instant> = None;
```

Inside the `"toggle"` match arm (around line 741), wrap the existing toggle logic with a debounce check. Replace the entire `"toggle" => { ... }` block:

```rust
"toggle" => {
    // Debounce: ignore presses within 500ms of last toggle
    let now = std::time::Instant::now();
    if let Some(last) = last_toggle {
        if now.duration_since(last).as_millis() < 500 {
            continue;
        }
    }
    last_toggle = Some(now);

    let murmur_state = app_handle.state::<MurmurState>();
    let current = murmur_state.app_state.current();
    if current == state::RecordingState::Recording {
        is_recording = false;
        if let Err(e) = do_stop_recording(&app_handle) {
            log::error!("failed to stop recording: {}", e);
            let _ = murmur_state
                .app_state
                .transition(state::RecordingState::Idle);
            let _ = app_handle
                .emit("recording_state_changed", "idle");
            let _ = app_handle
                .emit("recording_error", e.to_string());
            hide_preview_window(&app_handle);
            hide_main_window(&app_handle);
        }
    } else if current == state::RecordingState::Idle {
        match do_start_recording(&app_handle) {
            Ok(()) => {
                is_recording = true;
            }
            Err(e) => {
                log::error!(
                    "failed to start recording: {}",
                    e
                );
            }
        }
    }
}
```

**Step 2: Verify with cargo check**

Run: `cd src-tauri && cargo check`
Expected: compiles without errors

**Step 3: Run existing tests**

Run: `cd src-tauri && cargo test`
Expected: all 6 tests pass (debounce is in runtime handler, not unit-testable)

**Step 4: Manual verification**

1. Run `pnpm tauri dev`
2. Set recording mode to Toggle
3. Rapidly double-tap the PTT key — should only start recording (not start then immediately stop)
4. Single tap to start, single tap to stop — should work normally

**Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "fix: add 500ms debounce to toggle mode hotkey"
```

---

### Task 4: clipboard::copy_only Function

**Files:**
- Modify: `src-tauri/src/clipboard.rs` (add `copy_only` function)

**Step 1: Add copy_only function**

In `src-tauri/src/clipboard.rs`, after the `insert_text` function (after line 51), add:

```rust
/// Copies text to the system clipboard without pasting or restoring.
/// Used when no text input field is focused — the user will paste manually.
pub(crate) fn copy_only(text: &str) -> Result<(), ClipboardError> {
    if text.is_empty() {
        return Ok(());
    }

    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| ClipboardError::Access(e.to_string()))?;

    clipboard
        .set_text(text)
        .map_err(|e| ClipboardError::Access(e.to_string()))?;

    Ok(())
}
```

**Step 2: Verify with cargo check**

Run: `cd src-tauri && cargo check`
Expected: compiles (function unused for now, that's OK)

**Step 3: Commit**

```bash
git add src-tauri/src/clipboard.rs
git commit -m "feat(clipboard): add copy_only for clipboard-without-paste mode"
```

---

### Task 5: has_focused_text_input — macOS

**Files:**
- Modify: `src-tauri/src/frontapp_macos.rs` (add new function + FFI bindings)

**Step 1: Add Accessibility API FFI bindings and function**

In `src-tauri/src/frontapp_macos.rs`, add the following after the existing `style_for_app` function (after line 100) and before the `// --- Raw Objective-C FFI bindings ---` comment (line 102):

```rust
/// Checks whether the current foreground app has a focused text input element.
///
/// Uses macOS Accessibility API: AXUIElementCreateSystemWide → focused element → role check.
/// Returns `true` if the focused element is a text input (AXTextField, AXTextArea, etc.).
/// Returns `false` if no text input is focused, or if the query fails.
pub(crate) fn has_focused_text_input() -> bool {
    unsafe {
        let system_wide = AXUIElementCreateSystemWide();
        if system_wide.is_null() {
            return false;
        }

        // Get the focused UI element
        let attr_name = cfstring_from_static("AXFocusedUIElement");
        let mut focused_value: CFTypeRef = std::ptr::null();
        let err = AXUIElementCopyAttributeValue(system_wide, attr_name, &mut focused_value);
        CFRelease(system_wide as CFTypeRef);
        CFRelease(attr_name as CFTypeRef);

        if err != 0 || focused_value.is_null() {
            return false;
        }

        // Get the role of the focused element
        let role_attr = cfstring_from_static("AXRole");
        let mut role_value: CFTypeRef = std::ptr::null();
        let err = AXUIElementCopyAttributeValue(
            focused_value as AXUIElementRef,
            role_attr,
            &mut role_value,
        );
        CFRelease(focused_value);
        CFRelease(role_attr as CFTypeRef);

        if err != 0 || role_value.is_null() {
            return false;
        }

        // Convert CFString role to Rust string and check
        let role_str = cfstring_to_string(role_value as CFStringRef);
        CFRelease(role_value);

        matches!(
            role_str.as_deref(),
            Some("AXTextField" | "AXTextArea" | "AXSearchField" | "AXComboBox")
        )
    }
}

// --- Accessibility API FFI ---

type AXUIElementRef = *mut c_void;
type CFTypeRef = *const c_void;
type AXError = i32;
type CFStringRef = *const c_void;
type CFIndex = isize;
type CFAllocatorRef2 = *const c_void; // avoid conflict with existing typedef

extern "C" {
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    fn CFRelease(cf: CFTypeRef);
    fn CFStringCreateWithBytes(
        alloc: CFAllocatorRef2,
        bytes: *const u8,
        num_bytes: CFIndex,
        encoding: u32,
        is_external: bool,
    ) -> CFStringRef;
    fn CFStringGetLength(string: CFStringRef) -> CFIndex;
    fn CFStringGetCString(
        string: CFStringRef,
        buffer: *mut u8,
        buffer_size: CFIndex,
        encoding: u32,
    ) -> bool;
}

const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

/// Creates a CFString from a static Rust str. Caller must CFRelease.
unsafe fn cfstring_from_static(s: &str) -> CFStringRef {
    CFStringCreateWithBytes(
        std::ptr::null(),
        s.as_ptr(),
        s.len() as CFIndex,
        K_CF_STRING_ENCODING_UTF8,
        false,
    )
}

/// Converts a CFString to a Rust String. Returns None if conversion fails.
unsafe fn cfstring_to_string(cf: CFStringRef) -> Option<String> {
    let len = CFStringGetLength(cf);
    // UTF-8 can be up to 4 bytes per character
    let buf_size = (len * 4 + 1) as usize;
    let mut buf = vec![0u8; buf_size];
    if CFStringGetCString(cf, buf.as_mut_ptr(), buf_size as CFIndex, K_CF_STRING_ENCODING_UTF8) {
        let nul_pos = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8(buf[..nul_pos].to_vec()).ok()
    } else {
        None
    }
}
```

**Step 2: Verify with cargo check**

Run: `cd src-tauri && cargo check`
Expected: compiles. Note: the `CFAllocatorRef` typedef already exists in the file for `CFMachPortCreateRunLoopSource` — we use `CFAllocatorRef2` to avoid conflict. Alternatively, if the existing `CFAllocatorRef` is already `*const c_void`, reuse it.

Actually, looking at the existing code, `CFAllocatorRef` is already defined at line 33 as `type CFAllocatorRef = *const c_void;`. So we should reuse that. Replace `CFAllocatorRef2` with `CFAllocatorRef` in the `CFStringCreateWithBytes` declaration and `cfstring_from_static` function.

**Step 3: Commit**

```bash
git add src-tauri/src/frontapp_macos.rs
git commit -m "feat(frontapp): add has_focused_text_input via Accessibility API (macOS)"
```

---

### Task 6: has_focused_text_input — Windows

**Files:**
- Modify: `src-tauri/Cargo.toml:41-45` (add Windows features)
- Modify: `src-tauri/src/frontapp_windows.rs` (add new function)

**Step 1: Add required Windows crate features**

In `src-tauri/Cargo.toml`, update the `[target.'cfg(target_os = "windows")'.dependencies]` windows features (line 41-45):

```toml
windows = { version = "0.58", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Accessibility",
    "Win32_System_Threading",
    "Win32_System_Com",
    "Win32_Foundation",
] }
```

**Step 2: Add has_focused_text_input function**

In `src-tauri/src/frontapp_windows.rs`, after the `style_for_app` function (after line 89), add:

```rust
/// Checks whether the current foreground app has a focused text input element.
///
/// Uses Windows UI Automation: CoCreateInstance → GetFocusedElement → check ControlType.
/// Returns `true` if the focused element is a text edit control.
/// Returns `false` if no text input is focused, or if the query fails.
pub(crate) fn has_focused_text_input() -> bool {
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, UIA_ControlTypePropertyId,
        UIA_EditControlTypeId, UIA_DocumentControlTypeId, UIA_ComboBoxControlTypeId,
    };
    use windows::core::VARIANT;

    unsafe {
        // Initialize COM (ignore already-initialized error)
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let automation: IUIAutomation = match CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) {
            Ok(a) => a,
            Err(_) => return false,
        };

        let focused = match automation.GetFocusedElement() {
            Ok(el) => el,
            Err(_) => return false,
        };

        let control_type: VARIANT = match focused.GetCurrentPropertyValue(UIA_ControlTypePropertyId) {
            Ok(v) => v,
            Err(_) => return false,
        };

        // Extract i32 from VARIANT
        let ct: i32 = match i32::try_from(&control_type) {
            Ok(v) => v,
            Err(_) => return false,
        };

        ct == UIA_EditControlTypeId
            || ct == UIA_DocumentControlTypeId
            || ct == UIA_ComboBoxControlTypeId
    }
}
```

**Step 2b: Verify with cargo check (macOS cross-check only)**

Run: `cd src-tauri && cargo check`
Expected: compiles on macOS (Windows code is behind `cfg(target_os = "windows")` so it's not compiled). Full verification happens in CI or on a Windows machine.

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/frontapp_windows.rs
git commit -m "feat(frontapp): add has_focused_text_input via UI Automation (Windows)"
```

---

### Task 7: Smart Clipboard — Backend Branching

**Files:**
- Modify: `src-tauri/src/lib.rs:386-411` (do_stop_recording output pipeline)
- Modify: `src-tauri/src/lib.rs:573-584` (invoke_handler — add new commands)

**Step 1: Change transcription_complete payload to include mode**

In `lib.rs`, replace the output pipeline section (lines 386-411) with:

```rust
    // Determine output mode based on whether a text input is focused
    let has_input = frontapp::has_focused_text_input();

    if !text.is_empty() {
        if has_input {
            // Auto-paste mode: simulate paste + restore clipboard (existing behavior)
            if let Err(e) = clipboard::insert_text(&text) {
                let _ = app.emit("recording_error", format!("clipboard error: {e}"));
                log::error!("failed to insert text: {}", e);
            }
        } else {
            // Clipboard mode: copy to clipboard, user pastes manually
            if let Err(e) = clipboard::copy_only(&text) {
                let _ = app.emit("recording_error", format!("clipboard error: {e}"));
                log::error!("failed to copy text: {}", e);
            }
        }
    }

    let _ = state.app_state.transition(state::RecordingState::Idle);
    let _ = app.emit("recording_state_changed", "idle");

    let mode = if has_input { "pasted" } else { "clipboard" };
    let _ = app.emit(
        "transcription_complete",
        serde_json::json!({ "text": &text, "mode": mode }),
    );

    // Auto-hide: 10s for pasted mode, no auto-hide for clipboard mode
    if has_input {
        let generation = state.preview_generation.load(Ordering::SeqCst);
        let app_clone = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(10));
            let ms = app_clone.state::<MurmurState>();
            if ms.preview_generation.load(Ordering::SeqCst) == generation {
                hide_preview_window(&app_clone);
                hide_main_window(&app_clone);
            }
        });
    }
    // clipboard mode: no auto-hide — preview stays until copy or next recording

    Ok(text)
```

**Step 2: Add new Tauri commands**

In `lib.rs`, after the `hide_preview` command (line 546-548), add:

```rust
#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    clipboard::copy_only(&text).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_dictionary_term(
    term: String,
    state: tauri::State<'_, MurmurState>,
) -> Result<(), String> {
    let mut s = state.settings.lock().map_err(|e| e.to_string())?;
    let trimmed = term.trim().to_string();
    if trimmed.is_empty() {
        return Ok(());
    }
    // Append to dictionary (comma-separated), avoiding duplicates
    let existing: Vec<&str> = s.dictionary.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
    if !existing.iter().any(|&e| e == trimmed) {
        if s.dictionary.is_empty() {
            s.dictionary = trimmed;
        } else {
            s.dictionary = format!("{}, {}", s.dictionary, trimmed);
        }
        settings::save_settings(&s, &state.app_data_dir)?;
    }
    Ok(())
}
```

Register both commands in the invoke_handler (line 573-584). Add `copy_to_clipboard` and `add_dictionary_term` to the `generate_handler!` macro:

```rust
.invoke_handler(tauri::generate_handler![
    get_recording_state,
    is_model_ready,
    download_model_cmd,
    start_recording,
    stop_recording,
    get_settings,
    save_settings,
    open_settings,
    hide_preview,
    complete_onboarding,
    copy_to_clipboard,
    add_dictionary_term,
])
```

**Step 3: Verify with cargo check**

Run: `cd src-tauri && cargo check`
Expected: compiles without errors

**Step 4: Run tests**

Run: `cd src-tauri && cargo test`
Expected: all 6 tests pass

**Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: smart clipboard branching + copy_to_clipboard and add_dictionary_term commands"
```

---

### Task 8: Preview UI — Copy Button + Persistent Mode

**Files:**
- Modify: `src/preview.html:18-20` (add copy button to footer)
- Modify: `src/preview.css` (add copy button styles)
- Modify: `src/preview.js` (update event handler, add copy logic)

**Step 1: Add copy button to preview.html**

Replace the footer section (lines 18-20):

```html
<div id="preview-footer" class="preview-footer">
  <span id="char-count" class="char-count"></span>
  <div class="footer-actions">
    <button id="copy-btn" class="copy-btn" style="display: none;">Copy</button>
    <span id="app-badge" class="app-badge"></span>
  </div>
</div>
```

**Step 2: Add copy button styles to preview.css**

Append to `src/preview.css`:

```css
/* -- Copy Button -- */

.footer-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.copy-btn {
  font-family: system-ui, -apple-system, sans-serif;
  font-size: 11px;
  font-weight: 500;
  padding: 3px 10px;
  border: none;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.12s ease;
}

.copy-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
}

.copy-btn.copied {
  background: rgba(52, 199, 89, 0.25);
  color: #34c759;
}
```

**Step 3: Update preview.js for new payload format and copy logic**

Rewrite `src/preview.js` to handle the new `{ text, mode }` payload:

Replace the `transcription_complete` listener (lines 117-136):

```js
let currentText = "";
let currentMode = "pasted"; // "pasted" or "clipboard"

// ... inside DOMContentLoaded:

await listen("transcription_complete", (event) => {
    const { text, mode } = event.payload;
    currentText = text || "";
    currentMode = mode || "pasted";
    clearAutoHide();

    const copyBtn = document.getElementById("copy-btn");

    if (!currentText || currentText.trim().length === 0) {
      setHeader("Done", false);
      setText("No speech detected", "no-speech");
      setCharCount("");
      copyBtn.style.display = "none";
    } else {
      setHeader("Done", false);
      setText(currentText, null);
      setCharCount(currentText);
      scrollToBottom();
      copyBtn.style.display = "inline-block";
      copyBtn.textContent = "Copy";
      copyBtn.classList.remove("copied");
    }

    // Auto-hide only in pasted mode (backend handles the timer for main+preview windows)
    // In clipboard mode, preview stays until copy or next recording
    if (currentMode === "pasted") {
      autoHideTimer = setTimeout(() => {
        invoke("hide_preview").catch(() => {});
      }, 10000);
    }
  });
```

Add copy button handler inside `DOMContentLoaded`, after all the listeners:

```js
// Copy button
document.getElementById("copy-btn").addEventListener("click", async () => {
  const textToCopy = previewText().textContent;
  if (!textToCopy || textToCopy === "Listening..." || textToCopy === "No speech detected") return;

  try {
    await invoke("copy_to_clipboard", { text: textToCopy });
  } catch (e) {
    console.error("copy failed:", e);
    return;
  }

  const btn = document.getElementById("copy-btn");
  btn.textContent = "Copied!";
  btn.classList.add("copied");

  // Auto-hide after showing "Copied!" for 1.5s
  setTimeout(() => {
    invoke("hide_preview").catch(() => {});
  }, 1500);
});
```

**Step 4: Verify manually**

1. Run `pnpm tauri dev`
2. Record speech while focused on a text editor → text auto-pastes, preview shows with Copy button, hides after 10s
3. Record speech while focused on Finder/Desktop → text NOT pasted, preview stays with Copy button
4. Click Copy → shows "Copied!", hides after 1.5s
5. Verify clipboard contains the transcription text

**Step 5: Commit**

```bash
git add src/preview.html src/preview.css src/preview.js
git commit -m "feat(preview): add copy button and persistent mode for non-input contexts"
```

---

### Task 9: Editable Preview + Auto-add Dictionary

**Files:**
- Modify: `src/preview.js` (contenteditable + diff + suggest)
- Modify: `src/preview.html` (dict suggestion bar)
- Modify: `src/preview.css` (editing + suggestion styles)

**Step 1: Add dict suggestion bar to preview.html**

In `src/preview.html`, after the `preview-footer` div (before closing `</div>` of `#preview`), add:

```html
<div id="dict-suggest" class="dict-suggest" style="display: none;">
  <span id="dict-suggest-text" class="dict-suggest-text"></span>
  <div class="dict-suggest-actions">
    <button id="dict-suggest-yes" class="dict-suggest-btn yes">Add</button>
    <button id="dict-suggest-dismiss" class="dict-suggest-btn dismiss">Dismiss</button>
  </div>
</div>
```

**Step 2: Add editing and suggestion styles to preview.css**

Append to `src/preview.css`:

```css
/* -- Editable state -- */

.preview-text.editable {
  outline: none;
  border-radius: 4px;
  cursor: text;
}

.preview-text.editing {
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.1);
  padding: 4px;
  margin: -4px;
}

/* -- Dictionary Suggestion Bar -- */

.dict-suggest {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 16px;
  background: rgba(108, 92, 231, 0.12);
  border-top: 1px solid rgba(108, 92, 231, 0.2);
  flex-shrink: 0;
  animation: fadeIn 0.15s ease;
}

.dict-suggest-text {
  color: rgba(255, 255, 255, 0.7);
  font-size: 11px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dict-suggest-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.dict-suggest-btn {
  font-family: system-ui, -apple-system, sans-serif;
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  border: none;
  border-radius: 3px;
  cursor: pointer;
  transition: all 0.12s;
}

.dict-suggest-btn.yes {
  background: rgba(108, 92, 231, 0.3);
  color: #b8aaff;
}

.dict-suggest-btn.yes:hover {
  background: rgba(108, 92, 231, 0.5);
}

.dict-suggest-btn.dismiss {
  background: transparent;
  color: rgba(255, 255, 255, 0.4);
}

.dict-suggest-btn.dismiss:hover {
  color: rgba(255, 255, 255, 0.6);
}
```

**Step 3: Add editing and diff logic to preview.js**

In `src/preview.js`, add the following variables at the top (after `let dotsInterval = null;`):

```js
let originalText = "";
let suggestedTerms = [];
```

Add the following functions before the `DOMContentLoaded` handler:

```js
function enableEditing() {
  const el = previewText();
  el.setAttribute("contenteditable", "true");
  el.classList.add("editable");
}

function disableEditing() {
  const el = previewText();
  el.setAttribute("contenteditable", "false");
  el.classList.remove("editable", "editing");
}

function diffWords(original, edited) {
  // Simple word-level diff: find words in edited that replaced words in original
  const origWords = original.split(/\s+/).filter(w => w.length > 0);
  const editWords = edited.split(/\s+/).filter(w => w.length > 0);

  const newTerms = [];
  for (const word of editWords) {
    if (!origWords.includes(word) && word.length >= 2) {
      newTerms.push(word);
    }
  }
  return [...new Set(newTerms)]; // deduplicate
}

function showDictSuggestion(terms) {
  if (terms.length === 0) return;
  suggestedTerms = terms;
  const suggest = document.getElementById("dict-suggest");
  const text = document.getElementById("dict-suggest-text");
  const termList = terms.map(t => `"${t}"`).join(", ");
  text.textContent = `Add ${termList} to dictionary?`;
  suggest.style.display = "flex";
}

function hideDictSuggestion() {
  document.getElementById("dict-suggest").style.display = "none";
  suggestedTerms = [];
}
```

Inside the `transcription_complete` listener, after setting `currentText`, add:

```js
originalText = currentText;
hideDictSuggestion();
```

After the `setText(currentText, null)` call in the transcription_complete handler, add:

```js
// Enable editing (click to activate)
enableEditing();
```

In the `reset()` function, add:

```js
disableEditing();
hideDictSuggestion();
originalText = "";
```

Add event listeners inside `DOMContentLoaded`, after the copy button handler:

```js
// Editable preview: visual feedback on focus
previewText().addEventListener("focus", () => {
  previewText().classList.add("editing");
  // Cancel auto-hide while editing
  clearAutoHide();
});

// On blur: diff and suggest dictionary additions
previewText().addEventListener("blur", () => {
  previewText().classList.remove("editing");
  const edited = previewText().textContent || "";
  if (originalText && edited !== originalText) {
    const newTerms = diffWords(originalText, edited);
    if (newTerms.length > 0) {
      showDictSuggestion(newTerms);
    }
    // Update currentText so Copy copies the edited version
    currentText = edited;
    setCharCount(edited);
  }
});

// Dictionary suggestion buttons
document.getElementById("dict-suggest-yes").addEventListener("click", async () => {
  for (const term of suggestedTerms) {
    try {
      await invoke("add_dictionary_term", { term });
    } catch (e) {
      console.error("failed to add term:", e);
    }
  }
  const text = document.getElementById("dict-suggest-text");
  text.textContent = "Added!";
  setTimeout(hideDictSuggestion, 1500);
});

document.getElementById("dict-suggest-dismiss").addEventListener("click", hideDictSuggestion);
```

**Step 4: Verify manually**

1. Run `pnpm tauri dev`
2. Record speech → preview shows text
3. Click on preview text → editing visual feedback (subtle background)
4. Edit a word (e.g., change "台機電" to "台積電")
5. Click outside preview text → suggestion bar appears "Add '台積電' to dictionary?"
6. Click "Add" → shows "Added!" briefly
7. Open Settings → verify "台積電" appears in Dictionary tags
8. Click Copy → copies the edited (corrected) text

**Step 5: Commit**

```bash
git add src/preview.html src/preview.css src/preview.js
git commit -m "feat(preview): editable text with auto-dictionary suggestions"
```

---

### Task 10: Final Integration Test + Cargo Clippy

**Files:**
- All modified files

**Step 1: Run clippy**

Run: `cd src-tauri && cargo clippy --all-targets -- -D warnings`
Expected: zero warnings. Fix any issues found.

**Step 2: Run tests**

Run: `cd src-tauri && cargo test`
Expected: all tests pass

**Step 3: Full manual integration test**

Test matrix:

| Scenario | Expected |
|----------|----------|
| Settings: add dict term, Cancel | Term NOT saved |
| Settings: delete dict term, Undo | Term restored |
| Settings: delete dict term, wait 4s | Undo disappears |
| Settings: delete, Save | Term gone permanently |
| Toggle mode: double-tap PTT | Only starts (no immediate stop) |
| Toggle mode: normal start/stop | Works as before |
| Hold mode: press/release | Works as before (no regression) |
| Record in text editor (Slack, VS Code) | Auto-paste, preview 10s, Copy visible |
| Record on Finder/Desktop | No paste, clipboard has text, preview stays |
| Preview: click Copy | "Copied!" shown, hides after 1.5s |
| Preview: edit text, blur | Dict suggestion appears |
| Preview: click Add on suggestion | Term added to dictionary |
| Preview: click Dismiss | Suggestion hidden |
| Preview: edit then Copy | Copies edited text |

**Step 4: Commit (if any fixes were needed)**

```bash
git add -A
git commit -m "fix: integration test fixes"
```
