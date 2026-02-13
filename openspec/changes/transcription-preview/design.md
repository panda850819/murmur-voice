## Context

Murmur Voice's main window is a 420x48 always-on-top bar at the bottom of the screen. It shows a status dot, status text, and a single truncated line of transcription. Users cannot see the full AI-processed result before it's pasted into the target app. The app-aware style feature detects the foreground app but this information is invisible to the user.

The frontend is static HTML/JS/CSS served from `src/` — no build step. Backend communicates via `app.emit()` events and `invoke()` commands. Three windows exist: `main` (bar), `settings`, `onboarding`.

## Goals / Non-Goals

**Goals:**
- Show the full transcription result in a preview window during/after recording
- Display the detected foreground app in the main bar
- Preview window appears automatically on recording start and auto-hides after result is displayed
- Maintain the minimal, non-intrusive feel of the current UI

**Non-Goals:**
- Editable text in the preview window (read-only display only)
- History/persistence of transcription results (future work)
- Preview window settings (size, position) — use sensible defaults for now
- Redesigning the main bar layout beyond adding the app badge

## Decisions

### Decision 1: Preview window as a separate Tauri window

**Choice**: Create a new Tauri window (`preview`) rather than expanding the main bar.

**Why**:
- The main bar is intentionally tiny (420x48) and always visible — making it bigger defeats its purpose
- A separate window can be sized appropriately for multi-line text (e.g., 420x280)
- It can appear/disappear based on recording state without affecting the persistent main bar
- Follows Typeless's pattern: small persistent bar + floating result window

**Alternative rejected**: Expanding the main bar height dynamically — would cause jarring layout shifts and conflict with always-on-top positioning.

### Decision 2: Preview window positioning — anchored above main bar

**Choice**: Position the preview window directly above the main bar, horizontally centered with it.

**Why**:
- Natural visual connection — the bar is the "control" and the preview is the "output"
- Both are at the bottom of the screen, minimal eye movement
- Doesn't obscure the user's work area (stays at bottom edge)

### Decision 3: Preview window lifecycle — event-driven show/hide

**Choice**: Use existing `recording_state_changed` and `transcription_complete` events to control visibility.

```
Recording starts  → show preview (empty or "Listening...")
Live preview      → update text in real-time (local engine only)
Transcription     → show "Transcribing..."
LLM processing    → show "Processing..."
Result ready      → show final text + character count
After 3 seconds   → auto-hide preview window
```

**Why**: No new backend events needed. The frontend JS in `preview.js` listens to the same events that `main.js` already handles.

### Decision 4: App badge — emit foreground app info to frontend

**Choice**: Add a new event `foreground_app_info` emitted alongside transcription, containing the app name and style category. The main bar JS renders it as a small badge.

**Why**:
- `frontapp.rs` already detects the foreground app and determines style
- Currently this info is only used internally by `llm.rs`
- Emitting it as an event lets both the main bar (badge) and preview window (context) display it
- Minimal backend change: one extra `app.emit()` call in the orchestration flow

### Decision 5: Preview window styling — dark theme matching main bar

**Choice**: Dark translucent background with blur, matching the main bar aesthetic. Monospace-ish font for the transcription text.

**Why**: Visual consistency with the existing main bar. The dark floating window is also what Typeless uses (as seen in their UI).

## Risks / Trade-offs

- **[Window focus stealing]** → Preview window must not steal focus from the user's target app, or Cmd+V paste will go to the wrong place. Mitigation: create window with `focused: false` and `skip_taskbar: true`.
- **[Multi-monitor]** → Preview window position relative to main bar may break on multi-monitor setups. Mitigation: v1 uses fixed screen-bottom positioning, same as main bar. Improve later if needed.
- **[Live preview + Groq]** → Live transcription preview only works with local Whisper engine. Groq mode will show "Listening..." until final result. This is existing behavior, not a regression.
- **[Window flicker]** → Rapid show/hide on very short recordings could cause flicker. Mitigation: minimum display time of 1 second before auto-hide kicks in.
