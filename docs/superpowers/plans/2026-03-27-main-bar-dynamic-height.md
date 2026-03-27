# Main Bar Dynamic Height Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Main bar expands upward during recording to show multi-line live transcription, then collapses back to 48px when results are ready.

**Architecture:** Frontend CSS drives layout (expanded/collapsed classes). ResizeObserver detects height changes and invokes a backend Tauri command to resize the native window while keeping it bottom-anchored. Expand-only during recording; collapse bypasses ResizeObserver via direct invoke.

**Tech Stack:** CSS transitions, ResizeObserver API, Tauri `set_size`/`set_position` (LogicalSize/LogicalPosition)

**Spec:** `docs/superpowers/specs/2026-03-27-main-bar-dynamic-height-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `src/styles.css` | Modify | Add expanded/multiline CSS rules, change `#app` height model |
| `src/main.js` | Modify | State-driven class toggling, ResizeObserver, resize invoke |
| `src/events.js` | Modify | Add `RESIZE_MAIN_WINDOW` command constant |
| `src-tauri/src/lib.rs` | Modify | Add `resize_main_window` command, `MAIN_WINDOW_MAX_HEIGHT` const, register in handler |

---

### Task 1: Backend — Add `resize_main_window` command

**Files:**
- Modify: `src-tauri/src/lib.rs:36-40` (constants)
- Modify: `src-tauri/src/lib.rs:1208-1233` (invoke_handler)

- [ ] **Step 1: Add `MAIN_WINDOW_MAX_HEIGHT` constant**

In `src-tauri/src/lib.rs`, after line 39 (`const MAIN_WINDOW_HEIGHT: f64 = 48.0;`), add:

```rust
const MAIN_WINDOW_MAX_HEIGHT: f64 = 120.0;
```

- [ ] **Step 2: Add `resize_main_window` command function**

In `src-tauri/src/lib.rs`, add the command before the `run()` function (near the other command functions):

```rust
#[tauri::command]
fn resize_main_window(app: tauri::AppHandle, height: f64) {
    let h = height.clamp(MAIN_WINDOW_HEIGHT, MAIN_WINDOW_MAX_HEIGHT);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_size(tauri::LogicalSize::new(MAIN_WINDOW_WIDTH, h));
        if let Some(monitor) = w.current_monitor().ok().flatten() {
            let scale = monitor.scale_factor();
            let screen = monitor.size();
            let x = (screen.width as f64 / scale - MAIN_WINDOW_WIDTH) / 2.0;
            let y = screen.height as f64 / scale - h - MAIN_WINDOW_BOTTOM_MARGIN;
            let _ = w.set_position(tauri::LogicalPosition::new(x, y));
        }
    }
}
```

- [ ] **Step 3: Register command in invoke_handler**

In the `.invoke_handler(tauri::generate_handler![...])` block, add `resize_main_window` after `open_url`:

```rust
            open_url,
            resize_main_window,
```

- [ ] **Step 4: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors (warnings OK)

- [ ] **Step 5: Commit**

```
feat(backend): add resize_main_window command

Bottom-anchored resize: adjusts window height and Y position
to keep the main bar pinned to its bottom edge.
```

---

### Task 2: Frontend CSS — Expanded and multiline styles

**Files:**
- Modify: `src/styles.css:14-26` (`#app` rule)
- Modify: `src/styles.css:81-90` (`.transcription` rule)

- [ ] **Step 1: Change `#app` height model**

In `src/styles.css`, replace the `#app` rule (lines 14-26) with:

```css
#app {
  background: rgba(20, 20, 30, 0.78);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border-radius: 14px;
  margin: 4px;
  padding: 0 16px;
  height: calc(100vh - 8px);
  display: flex;
  align-items: center;
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  transition: none;
}

#app.expanded {
  align-items: flex-end;
  padding: 8px 16px;
  transition: none;
}

#app.collapsing {
  transition: height 150ms ease-out;
}
```

The key: `#app` fills the webview viewport (`100vh - 8px`). The actual native window size is what changes. No CSS transition during recording expansion (instant growth). The `collapsing` class enables a smooth 150ms transition only during the collapse phase.

`align-items: flex-end` keeps the status dot and text at the bottom when expanded, so the bar grows upward visually.

- [ ] **Step 2: Add multiline transcription style**

After the `.transcription` rule (line 90), add:

```css
.transcription.multiline {
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.5;
  text-overflow: clip;
  overflow-y: hidden;
}
```

- [ ] **Step 3: Visual check**

Run: `pnpm tauri dev`
Open app. The main bar should look identical to before (no visual regression at 48px idle state).

- [ ] **Step 4: Commit**

```
feat(css): add expanded and multiline styles for main bar

Expanded state uses flex-end alignment for upward growth.
Collapsing class enables 150ms transition only on collapse.
```

---

### Task 3: Frontend JS — Add command constant

**Files:**
- Modify: `src/events.js:35-57` (COMMANDS object)

- [ ] **Step 1: Add RESIZE_MAIN_WINDOW to COMMANDS**

In `src/events.js`, add after the `TRANSLATE_TEXT` line (line 56):

```javascript
  RESIZE_MAIN_WINDOW: "resize_main_window",
```

- [ ] **Step 2: Commit**

```
feat(events): add RESIZE_MAIN_WINDOW command constant
```

---

### Task 4: Frontend JS — ResizeObserver + state-driven expansion

**Files:**
- Modify: `src/main.js` (full file)

- [ ] **Step 1: Add expansion state variables**

In `src/main.js`, after line 9 (`let appBadge;`), add:

```javascript
let appEl;
let isRecording = false;
let isCollapsing = false;
let recordingMaxHeight = 0;
let resizeDebounceTimer = null;
```

- [ ] **Step 2: Initialize `appEl` and set up ResizeObserver**

In the `DOMContentLoaded` handler, after `appBadge = document.getElementById("app-badge");` (line 26), add:

```javascript
  appEl = document.getElementById("app");

  // ResizeObserver: notify backend when #app height changes during recording
  const resizeObserver = new ResizeObserver((entries) => {
    if (isCollapsing) return;
    if (!isRecording) return;
    const entry = entries[0];
    const newHeight = entry.contentBoxSize?.[0]?.blockSize
      ?? entry.contentRect.height;
    // Add margin (4px top + 4px bottom = 8px)
    const windowHeight = newHeight + 8;
    // Expand-only: never shrink during recording
    if (windowHeight <= recordingMaxHeight) return;
    recordingMaxHeight = windowHeight;
    // Debounce backend calls
    clearTimeout(resizeDebounceTimer);
    resizeDebounceTimer = setTimeout(() => {
      invoke(COMMANDS.RESIZE_MAIN_WINDOW, { height: windowHeight });
    }, 50);
  });
  resizeObserver.observe(appEl);
```

- [ ] **Step 3: Add expand/collapse helper functions**

After the `setStatus` function (line 18), add:

```javascript
function expandMainBar() {
  isRecording = true;
  isCollapsing = false;
  recordingMaxHeight = 0;
  appEl.classList.remove("collapsing");
  appEl.classList.add("expanded");
  transcription.classList.add("multiline");
}

function collapseMainBar() {
  isRecording = false;
  isCollapsing = true;
  appEl.classList.remove("expanded");
  transcription.classList.remove("multiline");
  transcription.textContent = "";
  appEl.classList.add("collapsing");
  // Directly invoke resize to 48px (bypass ResizeObserver)
  invoke(COMMANDS.RESIZE_MAIN_WINDOW, { height: 48 });
  // Clear collapsing flag after transition
  appEl.addEventListener("transitionend", () => {
    isCollapsing = false;
    appEl.classList.remove("collapsing");
  }, { once: true });
  // Fallback: clear flag after 200ms if transitionend doesn't fire
  setTimeout(() => {
    isCollapsing = false;
    appEl.classList.remove("collapsing");
  }, 200);
}

function resetMainBar() {
  // Immediate reset without animation (error/cancel)
  isRecording = false;
  isCollapsing = false;
  recordingMaxHeight = 0;
  appEl.classList.remove("expanded", "collapsing");
  transcription.classList.remove("multiline");
  invoke(COMMANDS.RESIZE_MAIN_WINDOW, { height: 48 });
}
```

- [ ] **Step 4: Wire up state changes**

Replace the `RECORDING_STATE_CHANGED` listener (lines 49-82) with:

```javascript
  await listen(EVENTS.RECORDING_STATE_CHANGED, (event) => {
    const state = event.payload;
    switch (state) {
      case RECORDING_STATES.STARTING:
        setStatus("recording", t("state.starting"));
        transcription.textContent = "";
        expandMainBar();
        break;
      case RECORDING_STATES.RECORDING:
        setStatus("recording", t("state.listening"));
        break;
      case RECORDING_STATES.STOPPING:
        setStatus("transcribing", t("state.stopping"));
        break;
      case RECORDING_STATES.TRANSCRIBING:
        setStatus("transcribing", t("state.transcribing"));
        // Stay expanded during transcription
        break;
      case RECORDING_STATES.PROCESSING:
        setStatus("transcribing", t("state.processing"));
        break;
      case RECORDING_STATES.TRANSLATING:
        setStatus("transcribing", t("state.translating"));
        transcription.textContent = "";
        break;
      case RECORDING_STATES.IDLE:
        setStatus(null, t("state.ready"));
        appBadge.classList.remove("visible");
        appBadge.textContent = "";
        if (isRecording || appEl.classList.contains("expanded")) {
          collapseMainBar();
        }
        break;
      case "downloading_model":
        progressContainer.classList.remove("hidden");
        setStatus(null, t("state.downloadingModel").replace(" {pct}%", ""));
        break;
    }
  });
```

- [ ] **Step 5: Update TRANSCRIPTION_COMPLETE to trigger collapse**

Replace the `TRANSCRIPTION_COMPLETE` listener (lines 89-97) with:

```javascript
  await listen(EVENTS.TRANSCRIPTION_COMPLETE, (event) => {
    const { text } = event.payload;
    if (appEl.classList.contains("expanded")) {
      collapseMainBar();
    }
    transcription.textContent = text || "";
    setStatus("done", t("state.done"));

    setTimeout(() => {
      setStatus(null, t("state.ready"));
    }, 2000);
  });
```

- [ ] **Step 6: Update error/cancel handlers to reset**

Replace the `RECORDING_ERROR` listener (lines 129-138) with:

```javascript
  await listen(EVENTS.RECORDING_ERROR, (event) => {
    const errorMsg = event.payload;
    resetMainBar();
    transcription.textContent = errorMsg;
    setStatus("error", t("state.error"));

    setTimeout(() => {
      setStatus(null, t("state.ready"));
      transcription.textContent = "";
    }, 3000);
  });
```

Replace the `RECORDING_CANCELLED` listener (lines 140-147) with:

```javascript
  await listen(EVENTS.RECORDING_CANCELLED, () => {
    resetMainBar();
    setStatus("error", t("state.cancelled"));

    setTimeout(() => {
      setStatus(null, t("state.ready"));
    }, 2000);
  });
```

- [ ] **Step 7: Verify compile + visual test**

Run: `pnpm tauri dev`

Test flow:
1. Press hotkey → main bar should expand as live text appears
2. Release hotkey → bar should smoothly collapse, preview appears
3. ESC during recording → bar should instantly reset
4. Short recording (one line of text) → bar stays at 48px

- [ ] **Step 8: Commit**

```
feat(ux): dynamic main bar height expansion during recording

- Expand upward (max 120px) as live transcription text wraps
- Expand-only during recording (never shrink mid-session)
- Smooth 150ms collapse on result/cancel/error
- ResizeObserver + debounce drives native window resize
- Bottom-anchored: bar grows upward, bottom edge stays fixed
```

---

### Task 5: Manual E2E verification

- [ ] **Step 1: Test local engine flow**

1. Start app with local engine configured
2. Press and hold hotkey, speak a long sentence
3. Verify: main bar expands as multi-line text appears
4. Release hotkey
5. Verify: bar collapses smoothly, preview window appears with result

- [ ] **Step 2: Test Groq engine flow**

1. Switch to Groq engine in settings
2. Press and hold hotkey, speak
3. Verify: main bar stays at 48px (no live preview for Groq)
4. Release hotkey
5. Verify: preview window appears normally

- [ ] **Step 3: Test ESC cancel**

1. Press hotkey, start speaking
2. While bar is expanded, press ESC
3. Verify: bar instantly resets to 48px, no preview appears

- [ ] **Step 4: Test rapid re-recording**

1. Complete a recording (bar collapses, preview appears)
2. Immediately start a new recording
3. Verify: bar expands again cleanly, no stuck state

- [ ] **Step 5: Test toggle mode**

1. Switch to toggle recording mode in settings
2. Press hotkey (starts), speak, press hotkey again (stops)
3. Verify: same expand/collapse behavior as hold mode
