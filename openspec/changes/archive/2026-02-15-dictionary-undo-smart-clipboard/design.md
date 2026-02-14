## Context

Murmur Voice is a Tauri 2 desktop app (Rust backend + vanilla JS frontend) for voice-to-text. Currently:
- Dictionary tags are mutated in-memory immediately on delete; Cancel just closes the window without restoring
- `clipboard::insert_text()` always saves clipboard → pastes → restores, regardless of whether a text input is focused
- Preview window auto-hides after 3 seconds with no copy button and no editing capability
- Toggle mode hotkey handler has no debounce, causing accidental stop from macOS CGEventFlagsChanged duplicates

## Goals / Non-Goals

**Goals:**
- Dictionary changes are reversible before Save (Cancel restores, delete has undo toast)
- Transcription output adapts to context: auto-paste when text input focused, clipboard-only with persistent preview otherwise
- Preview becomes an editing surface for correcting transcription and learning new dictionary terms
- Toggle mode is resilient to duplicate modifier key events

**Non-Goals:**
- Monitoring keystrokes or reading text from other apps (security concern)
- Adding a test framework for frontend JS
- Changing the Whisper transcription pipeline itself
- Supporting dictionary import/export

## Decisions

### 1. Input field detection via platform Accessibility APIs

Use macOS Accessibility API (`AXUIElementCopyAttributeValue` for focused element role) and Windows UI Automation (`IUIAutomation::GetFocusedElement` control type). Both are called synchronously in `do_stop_recording()` after transcription completes.

**Why not app-based heuristic?** Too many false positives/negatives — most apps have both input and non-input contexts. The Accessibility API is precise and we already have the required permission (CGEventTap for hotkey).

**Why not clipboard monitoring?** Not reliable (user may not copy corrected text) and invasive.

### 2. transcription_complete payload changes from String to JSON object

The event payload becomes `{ text: String, mode: "pasted" | "clipboard" }`. This is a **breaking change** for `preview.js` — it must be updated atomically with the backend change.

**Why?** The frontend needs to know the mode to decide whether to auto-hide or show persistent preview with copy button.

### 3. Dictionary snapshot for Cancel (frontend-only)

Store a copy of `dictTags` array on settings load. Cancel restores from snapshot instead of just closing. No Rust changes needed — the backend only persists on explicit Save.

**Why frontend-only?** The backend already only writes on `save_settings`. The issue is purely that the JS `dictTags` array is mutated in-place on delete, and Cancel doesn't restore it.

### 4. Preview contenteditable with word-level diff

Make the preview `<p>` element contenteditable. On blur, diff original vs edited text at word level. New words (present in edited but not original) are candidates for dictionary addition.

**Why word-level diff?** Simple to implement, covers the primary use case (replacing a misrecognized word with the correct one). Character-level diff would be more precise but overkill.

### 5. Toggle debounce at 500ms in hotkey handler

Add an `Instant` timestamp in the hotkey receiver thread. Skip toggle presses within 500ms of the last toggle. Applied only to toggle mode — hold mode is unaffected.

**Why 500ms?** Fast enough for intentional double-tap to still work (user rarely toggles faster than 500ms on purpose), slow enough to filter duplicate CGEventFlagsChanged events (which fire within milliseconds).

## Risks / Trade-offs

- **Accessibility API failure** → If the API call fails (permission revoked, unsupported app), default to auto-paste behavior. Safe fallback.
- **contenteditable quirks** → Browser contenteditable can produce unexpected HTML. Use `textContent` for reading, not `innerHTML`.
- **Breaking event payload** → `transcription_complete` format change. Must update `preview.js` in the same commit as the backend change.
- **Windows UI Automation COM init** → `CoInitializeEx` must be called before UI Automation. Using `COINIT_APARTMENTTHREADED`. May conflict if COM is already initialized differently — `CoInitializeEx` returns `S_FALSE` in that case, which is acceptable.
