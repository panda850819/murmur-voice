# Translate Hotkey Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a global translate hotkey (Option+T) that copies selected text, translates it via the existing LLM provider, and pastes the translation back.

**Architecture:** Extends the existing hotkey listener with a second key combo for translate. Reuses the LLM provider infrastructure (Groq/Ollama/Custom) with a translation-specific prompt. Translation flow runs on a background thread guarded by AtomicBool to prevent concurrency.

**Tech Stack:** Rust (Tauri 2), vanilla JS/HTML/CSS, arboard (clipboard), rdev (key simulation), CGEventTap (macOS) / SetWindowsHookEx (Windows)

**Spec:** `docs/superpowers/specs/2026-03-11-translate-hotkey-design.md`

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src-tauri/src/settings.rs` | Modify | Add `translate_hotkey`, `translate_language` fields; extract `parse_hotkey()` |
| `src-tauri/src/events.rs` | Modify | Add `STATE_TRANSLATING`, `MODE_TRANSLATED` constants |
| `src-tauri/src/llm.rs` | Modify | Add `build_translate_prompt()`, `translate_text()`, `create_translator()` |
| `src-tauri/src/clipboard.rs` | Modify | Add `copy_selection()`, `read_text()`, `set_and_paste()` |
| `src-tauri/src/hotkey_macos.rs` | Modify | Add translate key atomics + detection in callback |
| `src-tauri/src/hotkey_windows.rs` | Modify | Add translate key atomics + detection in callback |
| `src-tauri/src/lib.rs` | Modify | Add `translating: AtomicBool` to MurmurState, `do_translate()`, handle `TranslatePressed` in listener, update `save_settings` + `resume_hotkey_listener` + `pause_hotkey_listener` |
| `src/events.js` | Modify | Add `TRANSLATING` state, `TRANSLATED` mode, `PAUSE_TRANSLATE_HOTKEY` command |
| `src/i18n.js` | Modify | Add translation UI strings |
| `src/main.js` | Modify | Handle "translating" state |
| `src/preview.js` | Modify | Handle `mode: "translated"` (no auto-hide, show copy button) |
| `src/settings.html` | Modify | Add Translation section |
| `src/settings.js` | Modify | Add translate hotkey recording + language select + save/load |
| `src/settings.css` | Modify | Minor styles if needed for translation section |

---

## Chunk 1: Backend Core (Settings + Events + LLM + Clipboard)

### Task 1: Settings — Add translate fields and extract parse_hotkey

**Files:**
- Modify: `src-tauri/src/settings.rs`

- [ ] **Step 1: Add default functions for translate settings**

```rust
// Add after the existing default functions (around line 28)
fn default_translate_hotkey() -> String {
    "AltLeft+KeyT".to_string()
}

fn default_translate_language() -> String {
    "en".to_string()
}
```

- [ ] **Step 2: Add fields to Settings struct**

Add after `ui_locale` field (line 68):

```rust
    #[serde(default = "default_translate_hotkey")]
    pub translate_hotkey: String,
    #[serde(default = "default_translate_language")]
    pub translate_language: String,
```

- [ ] **Step 3: Add defaults to Default impl**

Add in `Default::default()` after `ui_locale` (line 100):

```rust
            translate_hotkey: default_translate_hotkey(),
            translate_language: default_translate_language(),
```

- [ ] **Step 4: Extract parse_hotkey function**

Refactor `ptt_key_target` to use a shared function. Replace `ptt_key_target` method (lines 108-122) with:

```rust
    pub fn ptt_key_target(&self) -> PttKeyTarget {
        parse_hotkey(&self.ptt_key)
    }

    pub fn translate_key_target(&self) -> PttKeyTarget {
        parse_hotkey(&self.translate_hotkey)
    }
```

And add a standalone function before the `impl Settings`:

```rust
pub(crate) fn parse_hotkey(key: &str) -> PttKeyTarget {
    if let Some(plus_pos) = key.find('+') {
        let modifier_str = &key[..plus_pos];
        let key_str = &key[plus_pos + 1..];
        PttKeyTarget {
            modifier_mask: modifier_mask_for(modifier_str),
            regular_key: keycode_for_code(key_str),
        }
    } else {
        PttKeyTarget {
            modifier_mask: modifier_mask_for(key),
            regular_key: 0,
        }
    }
}
```

- [ ] **Step 5: Add tests for new settings**

Add at the end of the `mod tests` block:

```rust
    #[test]
    fn test_deserialize_without_translate_settings() {
        let json = r#"{
            "ptt_key": "AltLeft",
            "language": "auto",
            "engine": "local",
            "model": "large-v3-turbo",
            "groq_api_key": "",
            "window_opacity": 0.78,
            "auto_start": false
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.translate_hotkey, "AltLeft+KeyT");
        assert_eq!(s.translate_language, "en");
    }

    #[test]
    fn test_parse_hotkey_combo() {
        let t = parse_hotkey("AltLeft+KeyT");
        assert_ne!(t.modifier_mask, 0);
        assert_ne!(t.regular_key, 0);
    }

    #[test]
    fn test_parse_hotkey_single() {
        let t = parse_hotkey("AltLeft");
        assert_ne!(t.modifier_mask, 0);
        assert_eq!(t.regular_key, 0);
    }

    #[test]
    fn test_translate_key_target() {
        let s = Settings::default();
        let t = s.translate_key_target();
        assert_ne!(t.modifier_mask, 0);
        assert_ne!(t.regular_key, 0);
    }
```

- [ ] **Step 6: Run tests**

Run: `cd src-tauri && cargo test --lib settings`
Expected: All tests pass, including new ones.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/settings.rs
git commit -m "feat(settings): add translate_hotkey and translate_language fields"
```

---

### Task 2: Events — Add translate state and mode constants

**Files:**
- Modify: `src-tauri/src/events.rs`
- Modify: `src/events.js`

- [ ] **Step 1: Add Rust constants**

In `src-tauri/src/events.rs`, add after `STATE_PROCESSING` (line 25):

```rust
pub const STATE_TRANSLATING: &str = "translating";
```

Add after `MODE_CLIPBOARD` (line 29):

```rust
pub const MODE_TRANSLATED: &str = "translated";
```

- [ ] **Step 2: Add JS constants**

In `src/events.js`, add `TRANSLATING` to `RECORDING_STATES` (after line 23):

```javascript
  TRANSLATING: "translating",
```

Add `TRANSLATED` to `TRANSCRIPTION_MODES` (after line 28):

```javascript
  TRANSLATED: "translated",
```

Add to `COMMANDS` (after `HIDE_OVERLAY_WINDOWS`, line 48):

```javascript
  PAUSE_TRANSLATE_HOTKEY: "pause_translate_hotkey",
  RESUME_TRANSLATE_HOTKEY: "resume_translate_hotkey",
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/events.rs src/events.js
git commit -m "feat(events): add translate state and mode constants"
```

---

### Task 3: LLM — Add translation functions

**Files:**
- Modify: `src-tauri/src/llm.rs`

- [ ] **Step 1: Add build_translate_prompt function**

Add after `strip_llm_prefix` (around line 160, before the TextEnhancer trait):

```rust
fn translate_language_name(code: &str) -> &str {
    match code {
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
        _ => code,
    }
}

fn build_translate_prompt(target_language: &str) -> String {
    let lang_name = translate_language_name(target_language);
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

- [ ] **Step 2: Make private fields `pub(crate)` for cross-function access**

The `OpenAICompatibleEnhancer` fields `api_key`, `model`, and `local` are currently private. `translate_text()` (same module) needs to access them. Change visibility in the struct definition (around line 173):

```rust
pub(crate) struct OpenAICompatibleEnhancer {
    pub api_url: String,
    pub(crate) api_key: String,
    pub(crate) model: String,
    pub(crate) local: bool,
    provider_name: String,
}
```

Note: `api_url` is already `pub`. `provider_name` stays private (only used via `name()` trait method).

- [ ] **Step 3: Add translate_text function**

Add as an `impl OpenAICompatibleEnhancer` method after the `TextEnhancer` impl block (after line 292):

```rust
impl OpenAICompatibleEnhancer {
    /// Translates text using this endpoint with a translation-specific prompt.
    pub(crate) fn translate(&self, text: &str, target_language: &str) -> Result<String, LlmError> {
        let prompt = build_translate_prompt(target_language);
        let max_tokens = (text.len() * 4).clamp(256, 4096) as u64;

        let mut body = serde_json::json!({
            "model": &self.model,
            "messages": [
                { "role": "system", "content": prompt },
                { "role": "user", "content": text }
            ],
            "temperature": 0.3,
            "max_tokens": max_tokens,
        });

        if !self.local {
            body["frequency_penalty"] = serde_json::json!(0.0);
        }

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| LlmError::Api(format!("failed to create runtime: {e}")))?;

        let response = rt.block_on(async {
            let client = reqwest::Client::new();
            let mut req = client
                .post(&self.api_url)
                .header("Content-Type", "application/json")
                .json(&body);

            if !self.api_key.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", self.api_key));
            }

            req.send().await
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = rt
                .block_on(response.text())
                .unwrap_or_else(|e| format!("(failed to read body: {e})"));
            return Err(LlmError::Api(format!("{status}: {body_text}")));
        }

        let chat: ChatResponse = rt.block_on(response.json())?;
        let content = chat
            .choices
            .into_iter()
            .next()
            .ok_or(LlmError::Format)?
            .message
            .content;

        Ok(content.trim().to_string())
    }
}
```

- [ ] **Step 3: Add create_translator factory**

Add after `create_enhancer` (after line 330):

```rust
/// Creates an OpenAICompatibleEnhancer for translation.
/// Unlike create_enhancer, this ignores llm_enabled — translation has its own toggle.
pub(crate) fn create_translator(
    settings: &crate::settings::Settings,
) -> Option<OpenAICompatibleEnhancer> {
    match settings.llm_provider.as_str() {
        "groq" => {
            if settings.groq_api_key.is_empty() {
                return None;
            }
            Some(OpenAICompatibleEnhancer::groq(
                &settings.groq_api_key,
                &settings.llm_model,
            ))
        }
        "ollama" => Some(OpenAICompatibleEnhancer::ollama(
            &settings.ollama_url,
            &settings.ollama_model,
        )),
        "custom" => {
            if settings.custom_llm_url.is_empty() {
                return None;
            }
            Some(OpenAICompatibleEnhancer::custom(
                &settings.custom_llm_url,
                &settings.custom_llm_key,
                &settings.custom_llm_model,
            ))
        }
        _ => None,
    }
}
```

- [ ] **Step 4: Add tests for translation functions**

Add at the end of `mod tests`:

```rust
    #[test]
    fn test_build_translate_prompt() {
        let prompt = build_translate_prompt("zh");
        assert!(prompt.contains("Traditional Chinese (zh-TW)"));
        assert!(prompt.contains("Output ONLY the translated text"));
    }

    #[test]
    fn test_build_translate_prompt_unknown() {
        let prompt = build_translate_prompt("xyz");
        assert!(prompt.contains("xyz"));
    }

    #[test]
    fn test_translate_language_name() {
        assert_eq!(translate_language_name("en"), "English");
        assert_eq!(translate_language_name("zh"), "Traditional Chinese (zh-TW)");
        assert_eq!(translate_language_name("unknown"), "unknown");
    }

    #[test]
    fn test_create_translator_groq() {
        let s = Settings {
            groq_api_key: "gsk_test".to_string(),
            llm_provider: "groq".to_string(),
            llm_enabled: false, // should still work
            ..Default::default()
        };
        let t = create_translator(&s);
        assert!(t.is_some());
        assert_eq!(t.unwrap().name(), "Groq");
    }

    #[test]
    fn test_create_translator_no_key() {
        let s = Settings {
            groq_api_key: String::new(),
            llm_provider: "groq".to_string(),
            ..Default::default()
        };
        assert!(create_translator(&s).is_none());
    }
```

- [ ] **Step 5: Run tests**

Run: `cd src-tauri && cargo test --lib llm`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/llm.rs
git commit -m "feat(llm): add translate_text() and create_translator() functions"
```

---

### Task 4: Clipboard — Add copy_selection, read_text, set_and_paste

**Files:**
- Modify: `src-tauri/src/clipboard.rs`

- [ ] **Step 1: Add copy_selection function**

Add after `copy_only` (after line 64):

```rust
/// Simulates Cmd+C (macOS) / Ctrl+C (Windows) to copy the current text selection.
pub(crate) fn copy_selection() -> Result<(), ClipboardError> {
    simulate_copy()?;
    std::thread::sleep(std::time::Duration::from_millis(150));
    Ok(())
}

/// Reads current clipboard text content.
pub(crate) fn read_text() -> Result<String, ClipboardError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| ClipboardError::Access(e.to_string()))?;
    clipboard
        .get_text()
        .map_err(|e| ClipboardError::Access(e.to_string()))
}

/// Sets clipboard text and pastes via Cmd+V / Ctrl+V.
/// Unlike insert_text(), does NOT restore original clipboard content —
/// the translated text stays on the clipboard for subsequent pastes.
pub(crate) fn set_and_paste(text: &str) -> Result<(), ClipboardError> {
    if text.is_empty() {
        return Ok(());
    }

    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| ClipboardError::Access(e.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|e| ClipboardError::Access(e.to_string()))?;

    std::thread::sleep(std::time::Duration::from_millis(50));

    simulate_paste()?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}
```

- [ ] **Step 2: Add simulate_copy functions (platform-specific)**

Add after the existing `simulate_paste` functions:

```rust
#[cfg(target_os = "macos")]
fn simulate_copy() -> Result<(), ClipboardError> {
    use rdev::{simulate, EventType, Key};

    let events = [
        EventType::KeyPress(Key::MetaLeft),
        EventType::KeyPress(Key::KeyC),
        EventType::KeyRelease(Key::KeyC),
        EventType::KeyRelease(Key::MetaLeft),
    ];

    for event in &events {
        simulate(event).map_err(|e| ClipboardError::Simulate(format!("{:?}", e)))?;
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn simulate_copy() -> Result<(), ClipboardError> {
    use rdev::{simulate, EventType, Key};

    let events = [
        EventType::KeyPress(Key::ControlLeft),
        EventType::KeyPress(Key::KeyC),
        EventType::KeyRelease(Key::KeyC),
        EventType::KeyRelease(Key::ControlLeft),
    ];

    for event in &events {
        simulate(event).map_err(|e| ClipboardError::Simulate(format!("{:?}", e)))?;
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}
```

- [ ] **Step 3: Run cargo check**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully (no unit tests for clipboard — requires OS-level access).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/clipboard.rs
git commit -m "feat(clipboard): add copy_selection, read_text, set_and_paste"
```

---

### Task 5: Hotkey — Add translate key detection (macOS + Windows)

**Files:**
- Modify: `src-tauri/src/hotkey_macos.rs`
- Modify: `src-tauri/src/hotkey_windows.rs`

- [ ] **Step 1: Add TranslatePressed to HotkeyEvent (macOS)**

In `hotkey_macos.rs`, add `TranslatePressed` to the enum (line 6-11):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HotkeyEvent {
    Pressed,
    Released,
    EscCancel,
    EventTapFailed,
    TranslatePressed,
}
```

- [ ] **Step 2: Add translate atomics and functions (macOS)**

After the existing `pause_hotkey` function (line 30), add:

```rust
/// The active translate modifier mask. Updated at runtime via settings.
static TRANSLATE_MODIFIER_MASK: AtomicU64 = AtomicU64::new(0);
/// The translate regular key CGKeyCode (always a combo — never 0 in practice).
static TRANSLATE_REGULAR_KEY: AtomicU32 = AtomicU32::new(0);

/// Update the translate hotkey target at runtime.
pub(crate) fn set_translate_target(modifier: u64, regular_key: u32) {
    TRANSLATE_MODIFIER_MASK.store(modifier, Ordering::SeqCst);
    TRANSLATE_REGULAR_KEY.store(regular_key, Ordering::SeqCst);
}

/// Temporarily pause translate hotkey detection.
pub(crate) fn pause_translate_hotkey() {
    TRANSLATE_MODIFIER_MASK.store(0, Ordering::SeqCst);
    TRANSLATE_REGULAR_KEY.store(0, Ordering::SeqCst);
}
```

- [ ] **Step 3: Add translate detection in macOS callback**

In `event_tap_callback`, add translate key check right **after** the ESC detection block (after line 108) and **before** the `if regular_key == 0` block (line 111). This must come before PTT check so translate takes priority:

```rust
    // Translate hotkey detection — always a combo (modifier+key)
    let tr_modifier = TRANSLATE_MODIFIER_MASK.load(Ordering::SeqCst);
    let tr_key = TRANSLATE_REGULAR_KEY.load(Ordering::SeqCst);
    if tr_key != 0 && event_type == K_CG_EVENT_KEY_DOWN {
        let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u32;
        if keycode == tr_key {
            let flags = CGEventGetFlags(event);
            if (flags & tr_modifier) != 0 {
                let _ = sender.send(HotkeyEvent::TranslatePressed);
                return std::ptr::null_mut(); // consume the key event
            }
        }
    }
```

- [ ] **Step 4: Add TranslatePressed to HotkeyEvent (Windows)**

In `hotkey_windows.rs`, add `TranslatePressed` to the enum (line 10-16):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HotkeyEvent {
    Pressed,
    Released,
    EscCancel,
    EventTapFailed,
    TranslatePressed,
}
```

- [ ] **Step 5: Add translate atomics and functions (Windows)**

After `pause_hotkey` (line 47), add:

```rust
/// The active translate modifier virtual key code.
static TRANSLATE_MODIFIER_MASK: AtomicU64 = AtomicU64::new(0);
/// The translate regular key virtual key code.
static TRANSLATE_REGULAR_KEY: AtomicU32 = AtomicU32::new(0);

/// Update the translate hotkey target at runtime.
pub(crate) fn set_translate_target(modifier: u64, regular_key: u32) {
    TRANSLATE_MODIFIER_MASK.store(modifier, Ordering::SeqCst);
    TRANSLATE_REGULAR_KEY.store(regular_key, Ordering::SeqCst);
}

/// Temporarily pause translate hotkey detection.
pub(crate) fn pause_translate_hotkey() {
    TRANSLATE_MODIFIER_MASK.store(0, Ordering::SeqCst);
    TRANSLATE_REGULAR_KEY.store(0, Ordering::SeqCst);
}
```

- [ ] **Step 6: Add translate detection in Windows callback**

In `keyboard_hook_proc`, add translate check right **after** the ESC detection block (after line 68) and **before** `if regular_vk != 0` (line 70):

```rust
        // Translate hotkey detection — always a combo (modifier+key)
        let tr_modifier_vk = TRANSLATE_MODIFIER_MASK.load(Ordering::SeqCst) as u32;
        let tr_regular_vk = TRANSLATE_REGULAR_KEY.load(Ordering::SeqCst);
        if tr_regular_vk != 0 && tr_modifier_vk != 0 {
            if let Some(ref sender) = GLOBAL_SENDER {
                if kb.vkCode == tr_regular_vk && is_down {
                    // Check if translate modifier is held via GetAsyncKeyState
                    let mod_held = {
                        use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
                        GetAsyncKeyState(tr_modifier_vk as i32) < 0
                    };
                    if mod_held {
                        let _ = sender.send(HotkeyEvent::TranslatePressed);
                        return LRESULT(1); // consume the key event
                    }
                }
            }
        }
```

Note: Windows needs `GetAsyncKeyState` import. Add to the use block at top of file:

```rust
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
```

- [ ] **Step 7: Run cargo check**

Run: `cd src-tauri && cargo check`
Expected: Compiles on current platform. (Cross-platform compilation handled by CI.)

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/hotkey_macos.rs src-tauri/src/hotkey_windows.rs
git commit -m "feat(hotkey): add TranslatePressed event and translate key detection"
```

---

### Task 6: lib.rs — Add do_translate, TranslatePressed handler, update save_settings

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add translating AtomicBool to MurmurState**

In the `MurmurState` struct (around line 42), add after `downloading: AtomicBool` (line 63):

```rust
    /// Guard to prevent concurrent translation operations.
    translating: AtomicBool,
```

- [ ] **Step 2: Initialize translating in MurmurState construction**

Find where `MurmurState` is created (in the `.setup()` closure). Add `translating: AtomicBool::new(false),` after the `downloading` field initialization.

- [ ] **Step 3: Add do_translate function**

Add after `reset_to_idle` (line 167), before `do_start_recording`:

```rust
fn do_translate(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<MurmurState>();

    // 1. Show main window with "translating" status
    let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_TRANSLATING);
    show_main_window(app);

    // 2. Wait for modifier keys to be physically released
    std::thread::sleep(std::time::Duration::from_millis(150));

    // 3. Simulate Cmd+C to copy selection
    clipboard::copy_selection().map_err(|e| format!("Failed to copy selection: {e}"))?;

    // 4. Read clipboard
    let text = clipboard::read_text()
        .map_err(|e| format!("Failed to read clipboard: {e}"))?;
    if text.trim().is_empty() {
        return Err("No text selected".to_string());
    }

    // 5. Get translator (bypasses llm_enabled check)
    let settings = state.settings.lock().map_err(|e| format!("settings mutex poisoned: {e}"))?.clone();
    let translator = llm::create_translator(&settings)
        .ok_or("Enable AI Processing provider in Settings to use translation")?;

    // 6. Translate via LLM
    let translated = translator.translate(&text, &settings.translate_language)
        .map_err(|e| e.to_string())?;

    // 7. Write to clipboard and paste (clipboard retains translated text)
    clipboard::set_and_paste(&translated).map_err(|e| e.to_string())?;

    // 8. Show preview (stays visible, no auto-hide)
    let _ = app.emit(
        events::TRANSCRIPTION_COMPLETE,
        serde_json::json!({
            "text": translated,
            "mode": events::MODE_TRANSLATED
        }),
    );
    show_preview_window(app);

    // 9. Reset main window state
    let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_IDLE);

    Ok(())
}
```

- [ ] **Step 4: Add TranslatePressed handler in hotkey listener loop**

In the hotkey event loop (around line 1162), add a new match arm before the existing `hotkey::HotkeyEvent::Pressed` arm:

```rust
                    match event {
                        hotkey::HotkeyEvent::TranslatePressed => {
                            let murmur_state = app_handle.state::<MurmurState>();
                            // Don't translate while recording or already translating
                            if murmur_state.app_state.current()
                                != state::RecordingState::Idle
                            {
                                continue;
                            }
                            if murmur_state
                                .translating
                                .swap(true, Ordering::Acquire)
                            {
                                continue; // already in progress
                            }
                            let app2 = app_handle.clone();
                            std::thread::spawn(move || {
                                let ms = app2.state::<MurmurState>();
                                let result = do_translate(&app2);
                                ms.translating.store(false, Ordering::Release);
                                if let Err(e) = result {
                                    let _ = app2.emit(events::RECORDING_ERROR, e);
                                    let _ = app2.emit(
                                        events::RECORDING_STATE_CHANGED,
                                        events::STATE_IDLE,
                                    );
                                }
                            });
                            continue;
                        }
                        hotkey::HotkeyEvent::Pressed => {
```

- [ ] **Step 5: Update save_settings to apply translate hotkey + conflict validation**

In the `save_settings` function (around line 722), add after the PTT hotkey application (line 729):

```rust
    // Reject if translate hotkey conflicts with PTT key
    if new_settings.translate_hotkey == new_settings.ptt_key {
        return Err("Translate hotkey cannot be the same as Push-to-Talk key".to_string());
    }

    // Apply translate hotkey change
    let tr = new_settings.translate_key_target();
    hotkey::set_translate_target(tr.modifier_mask, tr.regular_key);
```

- [ ] **Step 6: Add pause/resume translate hotkey commands**

Add after `resume_hotkey_listener` (line 832):

```rust
#[tauri::command]
fn pause_translate_hotkey() {
    hotkey::pause_translate_hotkey();
}

#[tauri::command]
fn resume_translate_hotkey(state: tauri::State<'_, MurmurState>) {
    if let Ok(s) = state.settings.lock() {
        let t = s.translate_key_target();
        hotkey::set_translate_target(t.modifier_mask, t.regular_key);
    }
}
```

- [ ] **Step 7: Register new commands in invoke_handler**

Find the `.invoke_handler(tauri::generate_handler![...])` call and add:

```rust
    pause_translate_hotkey,
    resume_translate_hotkey,
```

- [ ] **Step 8: Initialize translate hotkey at startup**

In the `.setup()` closure, add translate key initialization **immediately after** the PTT `set_hotkey_target` call, and **before** `initial_settings` is moved into `MurmurState` via `app.manage()`. This is critical because `initial_settings` is consumed by the move:

```rust
    // Existing PTT hotkey init (already present):
    // let t = initial_settings.ptt_key_target();
    // hotkey::set_hotkey_target(t.modifier_mask, t.regular_key);

    // Add right after:
    let tr = initial_settings.translate_key_target();
    hotkey::set_translate_target(tr.modifier_mask, tr.regular_key);

    // ... later, initial_settings is moved into MurmurState
```

- [ ] **Step 9: Run cargo check**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully.

- [ ] **Step 10: Run all tests**

Run: `cd src-tauri && cargo test --lib`
Expected: All tests pass.

- [ ] **Step 11: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add do_translate flow and TranslatePressed hotkey handler"
```

---

## Chunk 2: Frontend (i18n + main + preview + settings)

### Task 7: i18n — Add translation UI strings

**Files:**
- Modify: `src/i18n.js`

- [ ] **Step 1: Add English strings**

In the `en` object, add after `"preview.dismiss"` (line 107):

```javascript
    // Translation
    "group.translation": "Translation",
    "row.translateHotkey": "Translate Hotkey",
    "row.translateLang": "Target Language",
    "hint.translateProvider": "Uses the AI provider configured above",
    "state.translating": "Translating...",
    "state.translated": "Translated",
```

- [ ] **Step 2: Add Chinese strings**

In the `"zh-TW"` object, add after `"preview.dismiss"` (line 211):

```javascript
    "group.translation": "翻譯",
    "row.translateHotkey": "翻譯快捷鍵",
    "row.translateLang": "目標語言",
    "hint.translateProvider": "使用上方設定的 AI 供應商",
    "state.translating": "翻譯中...",
    "state.translated": "已翻譯",
```

- [ ] **Step 3: Commit**

```bash
git add src/i18n.js
git commit -m "feat(i18n): add translation feature UI strings"
```

---

### Task 8: main.js — Handle translating state

**Files:**
- Modify: `src/main.js`

- [ ] **Step 1: Add translating case to state handler**

In the `RECORDING_STATE_CHANGED` listener (around line 49), add a case after `PROCESSING` (line 67):

```javascript
      case RECORDING_STATES.TRANSLATING:
        setStatus("transcribing", t("state.translating"));
        transcription.textContent = "";
        break;
```

- [ ] **Step 2: Commit**

```bash
git add src/main.js
git commit -m "feat(main): handle translating state display"
```

---

### Task 9: preview.js — Handle translated mode (no auto-hide)

**Files:**
- Modify: `src/preview.js`

- [ ] **Step 1: Update TRANSCRIPTION_COMPLETE handler**

In the `TRANSCRIPTION_COMPLETE` listener (lines 308-338), modify the auto-hide logic. Replace the block starting at line 328:

```javascript
      // Auto-hide: 3s for pasted mode, 30s for clipboard mode, none for translated
      if (text && text.trim().length > 0 && mode !== TRANSCRIPTION_MODES.TRANSLATED) {
        const delay = mode === TRANSCRIPTION_MODES.PASTED ? 3000 : 30000;
        autoHideTimer = setTimeout(async () => {
          try {
            await invoke(COMMANDS.HIDE_OVERLAY_WINDOWS);
          } catch (_) {}
        }, delay);
      }
```

Also update the header text for translated mode. In the same handler, after `setHeader(t("state.done"), false);` (line 320), add:

```javascript
      if (mode === TRANSCRIPTION_MODES.TRANSLATED) {
        setHeader(t("state.translated"), false);
      }
```

- [ ] **Step 2: Add translating case to RECORDING_STATE_CHANGED handler in preview**

In the preview's `RECORDING_STATE_CHANGED` listener (lines 275-298), add after the `PROCESSING` case (line 293):

```javascript
      case RECORDING_STATES.TRANSLATING:
        clearAutoHide();
        setHeader(t("state.translating"), true);
        setText("", null);
        setCharCount("");
        copyBtn().classList.add("hidden");
        disableEditing();
        hideDictSuggest();
        break;
```

- [ ] **Step 3: Skip auto-hide on blur for translated mode**

In the blur event handler (lines 258-273), update the auto-hide condition:

```javascript
    // Restart auto-hide after editing (not for translated mode)
    if (currentMode === TRANSCRIPTION_MODES.PASTED) {
```

(This line is already correct — just verify it says `PASTED` not something else.)

- [ ] **Step 4: Commit**

```bash
git add src/preview.js
git commit -m "feat(preview): handle translated mode with no auto-hide"
```

---

### Task 10: Settings UI — Add Translation section

**Files:**
- Modify: `src/settings.html`
- Modify: `src/settings.js`

- [ ] **Step 1: Add Translation section HTML**

In `settings.html`, add a new section after the "AI Processing" section (after line 162, before the "Recording" section):

```html
      <section class="group">
        <div class="group-label" data-i18n="group.translation">Translation</div>
        <div class="group-card">
          <div class="row">
            <span class="row-label" data-i18n="row.translateHotkey">Translate Hotkey</span>
            <button id="translate-record" class="record-btn">Option + T</button>
          </div>
          <div class="row">
            <span class="row-label" data-i18n="row.translateLang">Target Language</span>
            <select id="translate-language">
              <option value="en">English</option>
              <option value="zh">繁體中文</option>
              <option value="ja">日本語</option>
              <option value="ko">한국어</option>
              <option value="fr">Francais</option>
              <option value="de">Deutsch</option>
              <option value="es">Espanol</option>
              <option value="pt">Portugues</option>
              <option value="ru">Русский</option>
              <option value="ar">العربية</option>
              <option value="th">ไทย</option>
              <option value="vi">Tieng Viet</option>
              <option value="id">Bahasa Indonesia</option>
            </select>
          </div>
          <div class="row-desc" data-i18n="hint.translateProvider">Uses the AI provider configured above</div>
        </div>
      </section>
```

- [ ] **Step 2: Add translate hotkey recording state in settings.js**

Add state variables after `undoEntry` (line 50):

```javascript
let currentTranslateKey = "AltLeft+KeyT";
let isRecordingTranslate = false;
let recordingTranslatePhase = null;
let capturedTranslateModifier = null;
```

- [ ] **Step 3: Update existing PTT startRecording/stopRecording to also pause/resume translate hotkey**

In `startRecording()` (line 69-77), add after the `PAUSE_HOTKEY_LISTENER` invoke:

```javascript
  invoke(COMMANDS.PAUSE_TRANSLATE_HOTKEY).catch(() => {});
```

In `stopRecording()` (line 79-87), add after the `RESUME_HOTKEY_LISTENER` invoke:

```javascript
  invoke(COMMANDS.RESUME_TRANSLATE_HOTKEY).catch(() => {});
```

- [ ] **Step 4: Add translate hotkey display and recording functions**

Add after `stopRecording` (after line 87):

```javascript
function setTranslateKey(code) {
  currentTranslateKey = code;
  el("translate-record").textContent = displayNameFor(code);
}

function startTranslateRecording() {
  isRecordingTranslate = true;
  recordingTranslatePhase = "modifier";
  capturedTranslateModifier = null;
  invoke(COMMANDS.PAUSE_TRANSLATE_HOTKEY).catch(() => {});
  invoke(COMMANDS.PAUSE_HOTKEY_LISTENER).catch(() => {});
  const btn = el("translate-record");
  btn.textContent = t("ptt.holdModifier");
  btn.classList.add("recording");
}

function stopTranslateRecording() {
  isRecordingTranslate = false;
  recordingTranslatePhase = null;
  capturedTranslateModifier = null;
  invoke(COMMANDS.RESUME_TRANSLATE_HOTKEY).catch(() => {});
  invoke(COMMANDS.RESUME_HOTKEY_LISTENER).catch(() => {});
  const btn = el("translate-record");
  btn.classList.remove("recording");
  btn.textContent = displayNameFor(currentTranslateKey);
}
```

- [ ] **Step 5: Update keydown/keyup handlers to support translate recording**

Modify `handleKeyDown` (line 89) to also handle translate recording:

```javascript
function handleKeyDown(e) {
  if (!isRecording && !isRecordingTranslate) return;
  e.preventDefault();
  e.stopPropagation();

  if (e.code === "Escape") {
    if (isRecording) stopRecording();
    if (isRecordingTranslate) stopTranslateRecording();
    return;
  }

  if (isRecording) {
    if (recordingPhase === "modifier") {
      if (KEY_MAP[e.code]) {
        capturedModifier = e.code;
        recordingPhase = "combo";
        el("ptt-record").textContent = t("ptt.nowPressKey");
      }
    } else if (recordingPhase === "combo") {
      if (REGULAR_KEY_MAP[e.code]) {
        currentPttKey = capturedModifier + "+" + e.code;
        stopRecording();
      }
    }
  }

  if (isRecordingTranslate) {
    if (recordingTranslatePhase === "modifier") {
      if (KEY_MAP[e.code]) {
        capturedTranslateModifier = e.code;
        recordingTranslatePhase = "combo";
        el("translate-record").textContent = t("ptt.nowPressKey");
      }
    } else if (recordingTranslatePhase === "combo") {
      if (REGULAR_KEY_MAP[e.code]) {
        currentTranslateKey = capturedTranslateModifier + "+" + e.code;
        stopTranslateRecording();
      }
    }
  }
}

function handleKeyUp(e) {
  if (isRecording && recordingPhase === "combo") {
    if (e.code === capturedModifier) {
      currentPttKey = capturedModifier;
      stopRecording();
    }
  }
  if (isRecordingTranslate && recordingTranslatePhase === "combo") {
    if (e.code === capturedTranslateModifier) {
      // Translate hotkey must be a combo — don't allow modifier-only
      stopTranslateRecording();
    }
  }
}
```

- [ ] **Step 6: Load translate settings on init**

In the `DOMContentLoaded` handler, after loading existing settings (around line 264), add:

```javascript
    setTranslateKey(s.translate_hotkey || "AltLeft+KeyT");
    el("translate-language").value = s.translate_language || "en";
```

- [ ] **Step 7: Add translate record button event listener**

After the PTT record button listener (around line 277), add:

```javascript
  el("translate-record").addEventListener("click", () => {
    if (isRecordingTranslate) {
      stopTranslateRecording();
    } else {
      if (isRecording) stopRecording(); // cancel PTT recording if active
      startTranslateRecording();
    }
  });
```

- [ ] **Step 8: Add translate fields to save**

In the save button handler (line 334), add to the `newSettings` object:

```javascript
      translate_hotkey: currentTranslateKey,
      translate_language: el("translate-language").value,
```

- [ ] **Step 9: Run the app to verify**

Run: `pnpm tauri dev`
Expected: Settings window shows Translation section with hotkey button and language dropdown. Hotkey recording works. Save persists new settings.

- [ ] **Step 10: Commit**

```bash
git add src/settings.html src/settings.js src/settings.css
git commit -m "feat(settings): add Translation section with hotkey and language select"
```

Note: Tauri 2 does NOT gate individual `#[tauri::command]` functions in `capabilities/default.json`. Commands registered via `invoke_handler` are available to all windows listed in the capabilities `"windows"` array. No changes needed to `default.json`.

---

### Task 11: Final integration test

- [ ] **Step 1: Run full cargo test**

Run: `cd src-tauri && cargo test --lib`
Expected: All tests pass (settings, llm, state tests).

- [ ] **Step 2: Run clippy**

Run: `cd src-tauri && cargo clippy --all-targets -- -D warnings`
Expected: Zero warnings.

- [ ] **Step 3: Manual smoke test**

Run: `pnpm tauri dev`

Test checklist:
1. Open Settings → Translation section visible with "Option + T" button and language dropdown
2. Change translate hotkey → saves and persists
3. Select text in any app → press Option+T → main window shows "Translating..."
4. Translation completes → preview shows result with "Translated" header
5. Preview does NOT auto-hide
6. Click "Copy" in preview → text copied, preview hides
7. Press ESC → preview hides
8. PTT recording still works normally (no regression)
9. Translate during recording → ignored
10. Double-press Option+T rapidly → only one translation runs

- [ ] **Step 4: Commit any fixes**

```bash
git add -A
git commit -m "fix: address integration test findings"
```
