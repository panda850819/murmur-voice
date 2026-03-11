# Translate Hotkey Feature

> Select text in any app, press a hotkey, get it translated and pasted back.

## Overview

Adds a global translate hotkey (default: Option+T) that copies selected text, translates it via the existing LLM provider, and pastes the translation back — replacing the original selection. Uses the same Groq/Ollama/Custom provider configured in AI Processing settings.

## User Flow

```
Option+T pressed (key event consumed — no character inserted)
  -> Set translating guard (AtomicBool) to prevent concurrent translates
  -> Show main window ("Translating..." status)
  -> Wait 150ms for modifier keys to be released
  -> Simulate Cmd+C to copy selected text
  -> Wait 150ms for clipboard to update
  -> Read clipboard content
  -> Send to LLM with translation prompt (target language from settings)
  -> Write translated text to clipboard (kept on clipboard, not restored)
  -> Simulate Cmd+V to paste (replaces selection)
  -> Show preview window with translation result
  -> Preview stays visible until user closes (ESC) or clicks "Copy"
  -> Reset main window to idle, clear translating guard
```

**Error cases:**
- No text selected (clipboard empty after Cmd+C): show error in preview, auto-hide after 3s
- LLM not configured: show error "Enable AI Processing in Settings to use translation"
- LLM API failure: show error in preview with message
- Already translating (guard active): ignore hotkey

## Settings

Two new fields in `Settings` struct:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `translate_hotkey` | `String` | `"AltLeft+KeyT"` | Global hotkey for translate |
| `translate_language` | `String` | `"en"` | Target language code |

Both fields use `#[serde(default)]` for backward compatibility with existing settings files.

### Target Language Options

| Code | Label (en) | Label (zh-TW) |
|------|-----------|---------------|
| `en` | English | English |
| `zh` | Traditional Chinese | 繁體中文 |
| `ja` | Japanese | 日本語 |
| `ko` | Korean | 한국어 |
| `fr` | French | Francais |
| `de` | German | Deutsch |
| `es` | Spanish | Espanol |
| `pt` | Portuguese | Portugues |
| `ru` | Russian | Русский |
| `ar` | Arabic | العربية |
| `th` | Thai | ไทย |
| `vi` | Vietnamese | Tieng Viet |
| `id` | Indonesian | Bahasa Indonesia |

Same language list as the existing transcription language selector (minus "auto").

## Architecture

### Hotkey Extension

Current architecture uses a single pair of atomics (`MODIFIER_MASK`, `REGULAR_KEY`) for PTT. Translate adds a second pair:

```rust
// hotkey.rs
static TRANSLATE_MODIFIER_MASK: AtomicU64 = AtomicU64::new(0);
static TRANSLATE_REGULAR_KEY: AtomicU32 = AtomicU32::new(0);

pub fn set_translate_target(modifier: u64, regular_key: u32) { ... }
pub fn pause_translate_hotkey() { ... }  // zeros out translate atomics
```

`HotkeyEvent` enum gains one variant:

```rust
pub enum HotkeyEvent {
    Pressed,
    Released,
    EscCancel,
    EventTapFailed,
    TranslatePressed,  // new
}
```

Platform callbacks (`event_tap_callback` on macOS, `keyboard_hook_proc` on Windows) check the translate key combo **before** PTT. If matched, emit `TranslatePressed` and **consume the key event** (return null on macOS / LRESULT(1) on Windows) to prevent the Option+T character (`†`) from being typed. The translate hotkey is always a combo (modifier+key), so it only needs keyDown detection — no Released event needed.

### Hotkey Parsing (DRY)

Extract `Settings::ptt_key_target()` logic into a reusable function:

```rust
// settings.rs
pub fn parse_hotkey(key: &str) -> PttKeyTarget { ... }

impl Settings {
    pub fn ptt_key_target(&self) -> PttKeyTarget { parse_hotkey(&self.ptt_key) }
    pub fn translate_key_target(&self) -> PttKeyTarget { parse_hotkey(&self.translate_hotkey) }
}
```

### Translation via LLM

New standalone function in `llm.rs` (NOT a trait method — avoids changing `TextEnhancer` trait):

```rust
/// Translates text using an OpenAI-compatible endpoint.
/// Builds its own request body with translation-specific system prompt.
pub fn translate_text(enhancer: &OpenAICompatibleEnhancer, text: &str, target_language: &str) -> Result<String, LlmError> {
    let prompt = build_translate_prompt(target_language);
    let max_tokens = (text.len() * 4).clamp(256, 4096) as u64;  // higher multiplier for CJK expansion

    let body = serde_json::json!({
        "model": &enhancer.model,
        "messages": [
            { "role": "system", "content": prompt },
            { "role": "user", "content": text }
        ],
        "temperature": 0.3,
        "max_tokens": max_tokens,
    });

    // ... same HTTP call pattern as enhance(), using enhancer.api_url and api_key
}

fn build_translate_prompt(target_language: &str) -> String {
    let lang_name = match target_language {
        "en" => "English",
        "zh" => "Traditional Chinese (zh-TW)",
        "ja" => "Japanese",
        "ko" => "Korean",
        "fr" => "French",
        "de" => "German",
        "es" => "Spanish",
        "pt" => "Portuguese",
        "ru" => "Russian",
        "ar" => "Arabic",
        "th" => "Thai",
        "vi" => "Vietnamese",
        "id" => "Indonesian",
        _ => target_language,
    };

    format!(
        r#"You are a translator. The user message contains text to translate.
Translate the entire text to {lang_name}.
Output ONLY the translated text. No explanations, no quotes, no commentary.
Preserve the original formatting (paragraphs, line breaks, lists).
Do NOT add or remove content.
If the text is already in {lang_name}, return it unchanged."#
    )
}
```

**Why standalone function instead of trait method:** `enhance()` hardcodes `build_system_prompt(style)` and prepends `[Raw transcription to clean up]` to user input — both wrong for translation. A separate function avoids trait churn and keeps translation logic isolated.

**`create_enhancer` bypass:** `create_enhancer()` returns `None` when `llm_enabled == false`. For translation, use a new `create_translator()` factory that ignores `llm_enabled` but still requires valid provider config:

```rust
pub fn create_translator(settings: &Settings) -> Option<OpenAICompatibleEnhancer> {
    // Same logic as create_enhancer but skips llm_enabled check
    // Returns concrete type (not Box<dyn>) since translate_text needs OpenAICompatibleEnhancer
    match settings.llm_provider.as_str() {
        "groq" if !settings.groq_api_key.is_empty() => Some(OpenAICompatibleEnhancer::groq(...)),
        "ollama" => Some(OpenAICompatibleEnhancer::ollama(...)),
        "custom" if !settings.custom_llm_url.is_empty() => Some(OpenAICompatibleEnhancer::custom(...)),
        _ => None,
    }
}
```

### Translation Flow (`do_translate`)

New function in `lib.rs`:

```rust
fn do_translate(app: &AppHandle, state: &MurmurState) -> Result<(), String> {
    // 1. Show main window with "translating" status
    app.emit(RECORDING_STATE_CHANGED, STATE_TRANSLATING).ok();
    show_main_window(app);

    // 2. Wait for modifier keys to be physically released
    //    (prevents Option being held during Cmd+C → producing Cmd+Option+C)
    std::thread::sleep(std::time::Duration::from_millis(150));

    // 3. Simulate Cmd+C to copy selection
    clipboard::copy_selection();

    // 4. Read clipboard
    let text = clipboard::read_text()
        .map_err(|e| format!("Failed to read clipboard: {e}"))?;
    if text.trim().is_empty() {
        return Err("No text selected".to_string());
    }

    // 5. Get translator (bypasses llm_enabled check)
    let settings = state.settings.lock().unwrap().clone();
    let translator = llm::create_translator(&settings)
        .ok_or("Enable AI Processing provider in Settings to use translation")?;

    // 6. Translate via LLM
    let translated = llm::translate_text(&translator, &text, &settings.translate_language)
        .map_err(|e| e.to_string())?;

    // 7. Write to clipboard and paste (clipboard retains translated text)
    clipboard::set_and_paste(&translated);

    // 8. Show preview (stays visible, no auto-hide)
    app.emit(TRANSCRIPTION_COMPLETE, serde_json::json!({
        "text": translated,
        "mode": "translated"
    })).ok();
    show_preview_window(app);

    // 9. Reset main window state
    app.emit(RECORDING_STATE_CHANGED, STATE_IDLE).ok();

    Ok(())
}
```

### Concurrency Guard

Use `AtomicBool` to prevent concurrent translation (simpler than adding a `Translating` state to `RecordingState`):

```rust
// In MurmurState
pub translating: AtomicBool,  // default false
```

In hotkey listener:
```rust
HotkeyEvent::TranslatePressed => {
    // Don't translate while recording or already translating
    if app_state.current() != RecordingState::Idle { continue; }
    if state.translating.swap(true, Ordering::Acquire) { continue; }  // already in progress

    let app2 = app.clone();
    let state2 = state.clone();
    std::thread::spawn(move || {
        let result = do_translate(&app2, &state2);
        state2.translating.store(false, Ordering::Release);
        if let Err(e) = result {
            app2.emit(RECORDING_ERROR, e).ok();
            app2.emit(RECORDING_STATE_CHANGED, STATE_IDLE).ok();
        }
    });
}
```

### Clipboard Extension

New functions in `clipboard.rs`:

```rust
/// Simulates Cmd+C (macOS) / Ctrl+C (Windows) to copy current selection.
pub fn copy_selection() {
    let copy_mod = if cfg!(target_os = "macos") {
        rdev::Key::MetaLeft
    } else {
        rdev::Key::ControlLeft
    };
    rdev::simulate(&EventType::KeyPress(copy_mod)).ok();
    rdev::simulate(&EventType::KeyPress(rdev::Key::KeyC)).ok();
    rdev::simulate(&EventType::KeyRelease(rdev::Key::KeyC)).ok();
    rdev::simulate(&EventType::KeyRelease(copy_mod)).ok();
    std::thread::sleep(std::time::Duration::from_millis(150));
}

/// Reads current clipboard text content.
pub fn read_text() -> Result<String, String> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.get_text())
        .map_err(|e| e.to_string())
}

/// Sets clipboard text and pastes via Cmd+V / Ctrl+V.
/// Unlike insert_text(), does NOT restore original clipboard content —
/// the translated text stays on the clipboard for subsequent pastes.
pub fn set_and_paste(text: &str) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        cb.set_text(text.to_string()).ok();
    }
    std::thread::sleep(std::time::Duration::from_millis(50));
    // Simulate paste
    let paste_mod = if cfg!(target_os = "macos") {
        rdev::Key::MetaLeft
    } else {
        rdev::Key::ControlLeft
    };
    rdev::simulate(&EventType::KeyPress(paste_mod)).ok();
    rdev::simulate(&EventType::KeyPress(rdev::Key::KeyV)).ok();
    rdev::simulate(&EventType::KeyRelease(rdev::Key::KeyV)).ok();
    rdev::simulate(&EventType::KeyRelease(paste_mod)).ok();
    std::thread::sleep(std::time::Duration::from_millis(100));
}
```

### Preview Window Behavior

For translation results, preview window does NOT auto-hide:

- `mode: "translated"` in the event payload signals to `preview.js` to skip the auto-hide timer
- The existing auto-hide logic in `lib.rs` (preview_generation timer) is NOT triggered for translation — `do_translate` does not set the auto-hide timer
- Preview stays visible until user presses ESC, clicks close, or clicks "Copy" button
- Clicking "Copy" copies the translation text to clipboard and then hides the preview

Frontend changes in `preview.js`:
- Check `mode` field from `TRANSCRIPTION_COMPLETE` event
- If `mode === "translated"`: skip auto-hide, show "Copy" button
- "Copy" button calls `invoke(COMMANDS.COPY_TO_CLIPBOARD, { text })` then hides windows

### Settings UI

New section in `settings.html` between "AI Processing" and the bottom:

```
Translation
  [Translate Hotkey: Option + T]  (recording button, same UX as PTT hotkey recording)
  [Target Language: English v]    (dropdown)
```

Translation section is always visible (not gated by LLM toggle), but shows a note if no LLM provider is configured: "Configure an AI provider above to enable translation."

**Hotkey conflict validation:** `save_settings` checks if `translate_hotkey == ptt_key`. If identical, reject with error message. Different modifier+key combos are always safe since they use separate atomics.

**Pause during recording:** When settings window opens and user starts recording a new translate hotkey, call `pause_translate_hotkey()` to zero out translate atomics (same pattern as `pause_hotkey()` for PTT).

### Events

New constants:

**Rust (`events.rs`):**
- `STATE_TRANSLATING = "translating"` (reuse `RECORDING_STATE_CHANGED` event)

**JS (`events.js`):**
- `RECORDING_STATES.TRANSLATING = "translating"`

No new event types needed — reuse `RECORDING_STATE_CHANGED` for status and `TRANSCRIPTION_COMPLETE` for results (with `mode: "translated"`).

## File Changes Summary

| File | Change |
|------|--------|
| `src-tauri/src/settings.rs` | Add `translate_hotkey`, `translate_language` fields; extract `parse_hotkey()` |
| `src-tauri/src/hotkey.rs` | Add translate atomics, `set_translate_target()`, `pause_translate_hotkey()`, `TranslatePressed` variant |
| `src-tauri/src/hotkey_macos.rs` | Check translate key combo in callback, consume key event |
| `src-tauri/src/hotkey_windows.rs` | Check translate key combo in callback, consume key event |
| `src-tauri/src/llm.rs` | Add `build_translate_prompt()`, `translate_text()`, `create_translator()` |
| `src-tauri/src/clipboard.rs` | Add `copy_selection()`, `read_text()`, `set_and_paste()` |
| `src-tauri/src/lib.rs` | Add `do_translate()`, handle `TranslatePressed` in listener loop, update `save_settings`, add `translating: AtomicBool` to `MurmurState` |
| `src-tauri/src/events.rs` | Add `STATE_TRANSLATING` |
| `src/events.js` | Add `TRANSLATING` state |
| `src/main.js` + `main.html` | Handle "translating" state display |
| `src/preview.js` + `preview.html` | Handle `mode: "translated"`, skip auto-hide, add Copy button |
| `src/settings.html` | Add Translation section |
| `src/settings.js` | Translate hotkey recording + language select + conflict validation |
| `src/i18n.js` | Translation UI strings |
| `src/styles.css` | Main window "translating" state style |
| `src/preview.css` | Copy button style |
| `src/settings.css` | Translation section style |

## Future Extensions

- **Auto-detect + reverse**: Detect source language, translate to the "other" language (e.g., zh<->en toggle). Only prompt change needed.
- **Multiple presets**: Multiple hotkeys for different target languages. Settings changes from `String` to `Vec<TranslatePreset>`, hotkey listener checks array.
- **Independent provider**: Separate LLM/API config for translation (e.g., DeepL). `create_translator()` already returns concrete type, easy to swap.
- **Translation history**: Log past translations for reference.

All of these build on top of the current design without requiring architectural changes.
