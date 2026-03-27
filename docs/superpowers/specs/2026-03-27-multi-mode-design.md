# Multi-Mode + Template Variables Design

**Date**: 2026-03-27
**Status**: Draft
**Version target**: v0.5.0

## Overview

Extend Murmur Voice from 2 hardcoded hotkey actions (PTT dictation + translate) to 4 built-in recording modes, each with an independent hotkey. Two new modes (Voice Command, Clipboard Rewrite) introduce template variables `{selected}` and `{clipboard}` — context captured at hotkey press and sent to the LLM alongside the voice command.

## Modes

| Mode | Hotkey default | Walks recording pipeline? | Requires LLM? | Context captured |
|------|---------------|--------------------------|---------------|-----------------|
| **Dictation** | `left_option` (existing PTT) | Yes | Optional (enhance) | None |
| **Translate** | `AltLeft+KeyT` (existing) | No (copy → LLM → paste) | Yes | `{selected}` via Cmd+C |
| **VoiceCommand** | `""` (disabled) | Yes | **Required** | `{selected}` via Cmd+C at press |
| **ClipboardRewrite** | `""` (disabled) | Yes | **Required** | `{clipboard}` read at press |

### Mode behaviors

**Dictation** — unchanged from current behavior. Hotkey → record → Whisper → optional text replacement → optional LLM enhance → paste.

**Translate** — unchanged. Hotkey fires `do_translate()` which copies selection, sends to LLM for translation, pastes result. Does not enter the recording pipeline or state machine.

**VoiceCommand** — on hotkey press: `copy_selection()` to capture selected text, then start recording. User speaks a command (e.g., "make this more formal", "translate to Japanese", "summarize in 3 bullets"). On release: Whisper transcribes the command → LLM receives command + captured selection → result pasted back, replacing selection.

**ClipboardRewrite** — on hotkey press: `clipboard::read_text()` to capture current clipboard content, then start recording. User speaks a command. On release: Whisper transcribes → LLM receives command + clipboard content → result pasted.

### Error cases

- VoiceCommand with empty selection → emit error "No text selected", abort before recording starts.
- ClipboardRewrite with empty clipboard → emit error "Clipboard is empty", abort before recording starts.
- VoiceCommand/ClipboardRewrite with LLM disabled → emit error "Enable AI Processing in Settings to use this mode".

## Architecture

### Mode Enum (Rust)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RecordingMode {
    Dictation,
    Translate,
    VoiceCommand,
    ClipboardRewrite,
}
```

Defined in a new section of `state.rs` or a small `mode.rs` file — no new file needed, `state.rs` is appropriate.

### HotkeyEvent changes

```rust
pub(crate) enum HotkeyEvent {
    Pressed(RecordingMode),    // replaces Pressed
    Released(RecordingMode),   // replaces Released
    EscCancel,                 // unchanged
    EventTapFailed,            // unchanged
}
```

`TranslatePressed` is absorbed into `Pressed(Translate)`. The main loop in lib.rs dispatches to `do_translate()` when mode is Translate, same as today.

### Hotkey system (hotkey_macos.rs / hotkey_windows.rs)

Replace the 2 pairs of static atomics (PTT + Translate) with an array-based design:

```rust
struct HotkeySlot {
    modifier_mask: AtomicU64,
    regular_key: AtomicU32,
}

static HOTKEY_SLOTS: [HotkeySlot; 4] = [...]; // indexed by RecordingMode as usize
```

Event tap callback iterates slots to find a match. Match order: VoiceCommand and ClipboardRewrite checked before Dictation (combo hotkeys are more specific than modifier-only).

**Dictation** retains modifier-only support. The other 3 modes are combo-only (modifier+key) to avoid conflicts.

Slot with `modifier_mask == 0` is disabled (not checked).

Public API:

```rust
pub(crate) fn set_hotkey(mode: RecordingMode, modifier: u64, regular_key: u32);
pub(crate) fn pause_hotkey(mode: RecordingMode);
pub(crate) fn pause_all_hotkeys();
```

### Settings fields

```rust
// Replace ptt_key + translate_hotkey with:
pub hotkey_dictation: String,           // default: "left_option"
pub hotkey_translate: String,           // default: "AltLeft+KeyT"
pub hotkey_voice_command: String,       // default: "" (disabled)
pub hotkey_clipboard_rewrite: String,   // default: "" (disabled)
```

**Migration**: On deserialization, if `ptt_key` exists but `hotkey_dictation` does not, copy `ptt_key` → `hotkey_dictation`. Same for `translate_hotkey` → `hotkey_translate`. This preserves existing user settings. The old field names get `#[serde(alias)]` or a custom deserialize.

### MurmurState additions

```rust
pub active_mode: Mutex<RecordingMode>,
pub captured_context: Mutex<Option<String>>,
```

`active_mode` is set on `Pressed(mode)`, read in `do_stop_recording`. `captured_context` holds the selected text or clipboard content for VoiceCommand/ClipboardRewrite.

### Recording pipeline changes

**do_start_recording(app, mode)**:

1. If VoiceCommand: `copy_selection()` → `read_text()` → store in `captured_context`. If empty → error, abort.
2. If ClipboardRewrite: `read_text()` → store in `captured_context`. If empty → error, abort.
3. Rest of start_recording unchanged (audio capture, live transcription).

Live transcription: only enabled for Dictation mode. VoiceCommand/ClipboardRewrite are recording voice commands — live preview of the command text is not useful.

**do_stop_recording(app)**:

The pipeline up to and including Whisper transcription is **unchanged**. After getting `raw_text`:

```
match active_mode {
    Dictation => {
        // existing flow: text_replacement → optional enhance → paste
    }
    VoiceCommand | ClipboardRewrite => {
        // skip text_replacement
        // require LLM — error if not configured
        let context = captured_context.take();
        let result = enhancer.execute_command(&raw_text, &context, context_type);
        // paste result
    }
}
```

### TextEnhancer trait addition

```rust
fn execute_command(&self, command: &str, context: &str, context_type: &str) -> Result<String, Box<dyn Error + Send + Sync>>;
```

Implemented in `OpenAICompatibleEnhancer`. Prompt:

```
System: You are a text processing assistant. The user gives you a voice command
and a piece of text. Execute the command on the text. Output ONLY the processed
result — no explanations, no markdown formatting, no preamble.

User: [Voice command]: {command}
[{context_type}]: {context}
```

`context_type` is "Selected text" or "Clipboard content".

## Frontend

### Main bar — mode indicator

On recording start, emit `recording_mode_info` event with mode-specific i18n text. Main bar displays:

| Mode | Text (zh-TW) | Text (en) |
|------|-------------|-----------|
| Dictation (LLM on) | 語音輸入，大模型文本優化 | Voice dictation with AI enhancement |
| Dictation (LLM off) | 語音輸入 | Voice dictation |
| VoiceCommand | 語音指令，處理選取文字 | Voice command on selected text |
| ClipboardRewrite | 語音指令，處理剪貼簿內容 | Voice command on clipboard |

Red recording indicator dot + mode description text, consistent with macOS dictation UI style.

### Settings page

"Hotkeys" section with 4 hotkey recorders:

1. **Dictation Hotkey** — existing PTT recorder, relabeled
2. **Translate Hotkey** — existing translate recorder, relabeled
3. **Voice Command Hotkey** — new, default empty, shows "Not set" placeholder
4. **Clipboard Rewrite Hotkey** — new, default empty, shows "Not set" placeholder

Empty hotkey = mode disabled. Each recorder uses the existing hotkey recording UI (already supports modifier-only and combo).

Collision detection: if user sets a hotkey that conflicts with another mode, show warning inline.

### Preview window

`TRANSCRIPTION_COMPLETE` event `mode` field extended:

| Mode | mode value |
|------|-----------|
| Dictation | `"dictated"` (existing, was implicit) |
| Translate | `"translated"` (existing) |
| VoiceCommand | `"voice_command"` |
| ClipboardRewrite | `"clipboard_rewrite"` |

Preview shows mode badge matching the value.

### i18n

All new strings added to `i18n.js` for both `en` and `zh-TW` locales.

## What does NOT change

- **state.rs** — state machine unchanged. Mode is an orthogonal dimension to recording state.
- **audio.rs** — audio capture unchanged.
- **whisper.rs** — transcription unchanged.
- **Hold/Toggle recording mode** — applies to all modes that walk the recording pipeline.
- **Anti-hallucination** — unchanged, applies to all modes.
- **Text replacement** — only applied in Dictation mode (not VoiceCommand/ClipboardRewrite).
- **Model download / engine init** — unchanged.
- **NSPanel overlay** — unchanged.
- **Tray menu** — unchanged.

## Migration & backward compatibility

- `ptt_key` → `hotkey_dictation` with serde alias
- `translate_hotkey` → `hotkey_translate` with serde alias
- `translate_language` — kept, used by Translate mode
- Old settings files load seamlessly via `#[serde(alias)]` + `#[serde(default)]`

## Testing

- Unit tests for `RecordingMode` enum serialization
- Unit tests for hotkey slot matching (ensure combo-first priority)
- Unit tests for `execute_command` prompt construction
- Unit tests for settings migration (old field names → new)
- Existing 36 tests must continue to pass unchanged
