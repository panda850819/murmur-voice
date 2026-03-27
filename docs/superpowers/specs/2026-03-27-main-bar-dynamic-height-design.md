# Main Bar Dynamic Height Expansion

**Date:** 2026-03-27
**Status:** Approved

## Summary

The main bar (420x48px) dynamically expands upward during recording to show multi-line live transcription text, then collapses back to 48px when the result is ready and the preview window takes over.

## Current State

- Main bar: fixed 420x48px, live transcription displayed as single-line with `text-overflow: ellipsis`
- Preview window: separate 420x280px window, appears above main bar for final results
- Main bar is bottom-anchored at screen center with 80px margin

## Design

### Approach: Frontend ResizeObserver + Backend resize command

The frontend CSS drives the visual layout. A `ResizeObserver` on `#app` detects height changes and invokes a backend command to resize the native window and reposition it (bottom-anchored).

### Behavior Matrix

| State | Main bar height | Behavior |
|-------|----------------|----------|
| Idle | 48px | Single-line status text |
| Recording (live text) | 48px → max 120px | Expand only, never shrink during recording. Latest text at bottom |
| Transcribing/Processing | Maintains current height | No collapse until result ready |
| Result ready | Collapse to 48px | 150ms ease-out, preview fades in simultaneously |
| Error/Cancel | Collapse to 48px | Immediate reset |

### Frontend Changes

**`src/styles.css`:**

- `#app`: Change `height: calc(100vh - 8px)` to `min-height: calc(48px - 8px)` with `transition: height 150ms ease-out`
- `#app.expanded`: Enable flex-wrap behavior for transcription area
- `.transcription`: Default remains `white-space: nowrap; text-overflow: ellipsis`
- `.transcription.multiline`: `white-space: pre-wrap; word-break: break-word; line-height: 1.5`
- Max content height capped so window never exceeds 120px total

**`src/main.js`:**

- On `RECORDING_STATE_CHANGED` → `RECORDING`: Add `expanded` class to `#app`, `multiline` class to `.transcription`
- On `RECORDING_STATE_CHANGED` → `IDLE` / result ready / error / cancel: Remove classes
- `ResizeObserver` on `#app`: When height changes, invoke `resize_main_window` with new height (in logical pixels)
- **Expand-only during recording**: Track `maxHeight` during recording session. ResizeObserver only sends resize if new height > current max. Reset on state exit.
- Debounce resize calls (avoid flooding backend during rapid text updates)
- **Collapse bypass**: On state exit, call `resize_main_window(48)` directly instead of relying on ResizeObserver. Set `collapsing` flag before removing classes; ignore ResizeObserver callbacks while `collapsing` is true. Clear flag on `transitionend`.

**`src/index.html`:**

- No structural changes needed

### Backend Changes

**`src-tauri/src/lib.rs`:**

New Tauri command:

```rust
#[tauri::command]
fn resize_main_window(app: tauri::AppHandle, height: f64) {
    let h = height.clamp(MAIN_WINDOW_HEIGHT, MAIN_WINDOW_MAX_HEIGHT);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_size(tauri::LogicalSize::new(MAIN_WINDOW_WIDTH, h));
        // Reposition to maintain bottom anchor (all logical coordinates)
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

Constants:

```rust
const MAIN_WINDOW_MAX_HEIGHT: f64 = 120.0;
```

Register `resize_main_window` in the `invoke_handler`.

### Transition Choreography

1. Recording stops → state changes to Transcribing (main bar stays expanded)
2. Result arrives → frontend removes `expanded`/`multiline` classes, resets `maxHeight` tracker
3. CSS transition collapses `#app` height over 150ms
4. `ResizeObserver` fires → invokes `resize_main_window(48)` → backend shrinks window + repositions
5. Preview window appears independently based on result-ready state (not chained to collapse timing)

### Edge Cases

- **No live text** (Groq mode): Main bar stays at 48px during recording (no live preview for cloud engine)
- **Very short text** (fits one line): Main bar stays at 48px even with `expanded` class (min-height)
- **ESC cancel during expansion**: Remove classes immediately, resize to 48px
- **Window already at target height**: ResizeObserver won't fire, no unnecessary backend calls
- **Empty live text during recording**: Bar stays at 48px until actual text appears (don't add `expanded` class on empty string)

## Files Changed

| File | Change |
|------|--------|
| `src/styles.css` | Add `#app.expanded`, `.transcription.multiline` styles, adjust `#app` height model |
| `src/main.js` | Add/remove CSS classes on state changes, ResizeObserver + resize invoke |
| `src-tauri/src/lib.rs` | Add `MAIN_WINDOW_MAX_HEIGHT` const, `resize_main_window` command, register in handler |
| `src-tauri/capabilities/default.json` | Add `resize_main_window` to allowed commands (if needed) |

## Not In Scope

- Preview window changes (stays as-is)
- Settings for max height (hardcoded 120px)
- Horizontal expansion
