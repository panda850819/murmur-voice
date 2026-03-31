use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn default_true() -> bool {
    true
}

fn default_hotkey_dictation() -> String {
    "left_option".to_string()
}

fn default_hotkey_translate() -> String {
    "AltLeft+KeyT".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextReplacement {
    pub find: String,
    pub replace: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    // Legacy field — kept for backward compatibility with old settings files.
    // New code should use hotkey_dictation instead.
    #[serde(skip_serializing)]
    pub ptt_key: String,
    pub language: String,
    pub engine: String,
    pub model: String,
    pub groq_api_key: String,
    pub window_opacity: f64,
    pub auto_start: bool,
    pub onboarding_complete: bool,
    pub recording_mode: String,
    pub dictionary: String,
    pub llm_enabled: bool,
    pub llm_model: String,
    pub llm_provider: String,
    pub ollama_url: String,
    pub ollama_model: String,
    pub custom_llm_url: String,
    pub custom_llm_key: String,
    pub custom_llm_model: String,
    pub app_aware_style: bool,
    pub ui_locale: String,
    // Legacy field — kept for backward compatibility. Use hotkey_translate instead.
    #[serde(skip_serializing)]
    pub translate_hotkey: String,
    pub translate_language: String,
    pub dictionary_packs: Vec<String>,
    #[serde(default)]
    pub text_replacements: Vec<TextReplacement>,

    // --- Multi-mode hotkey fields (v0.5.0+) ---
    #[serde(default = "default_hotkey_dictation")]
    pub hotkey_dictation: String,
    #[serde(default = "default_hotkey_translate")]
    pub hotkey_translate: String,
    #[serde(default)]
    pub hotkey_voice_command: String,
    #[serde(default)]
    pub hotkey_clipboard_rewrite: String,
}

/// Target for PTT key matching — supports single modifier or modifier+key combos.
#[derive(Debug)]
pub(crate) struct PttKeyTarget {
    pub modifier_mask: u64,
    pub regular_key: u32, // 0 = modifier-only
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ptt_key: String::new(),
            language: "auto".to_string(),
            engine: "local".to_string(),
            model: "large-v3-turbo".to_string(),
            groq_api_key: String::new(),
            window_opacity: 0.78,
            auto_start: false,
            onboarding_complete: false,
            recording_mode: "hold".to_string(),
            dictionary: String::new(),
            llm_enabled: false,
            llm_model: "llama-3.3-70b-versatile".to_string(),
            llm_provider: "groq".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3.2".to_string(),
            custom_llm_url: String::new(),
            custom_llm_key: String::new(),
            custom_llm_model: String::new(),
            app_aware_style: true,
            ui_locale: "en".to_string(),
            translate_hotkey: String::new(),
            translate_language: "en".to_string(),
            dictionary_packs: Vec::new(),
            text_replacements: Vec::new(),
            hotkey_dictation: "left_option".to_string(),
            hotkey_translate: "AltLeft+KeyT".to_string(),
            hotkey_voice_command: String::new(),
            hotkey_clipboard_rewrite: String::new(),
        }
    }
}

impl Settings {
    /// Migrate legacy field names to new hotkey fields.
    /// Called after deserialization — if legacy fields have values but new fields
    /// have defaults, copy the legacy values over.
    pub(crate) fn migrate_legacy_hotkeys(&mut self) {
        // ptt_key → hotkey_dictation
        if !self.ptt_key.is_empty() && self.hotkey_dictation == default_hotkey_dictation() {
            self.hotkey_dictation = self.ptt_key.clone();
        }
        // translate_hotkey → hotkey_translate
        if !self.translate_hotkey.is_empty() && self.hotkey_translate == default_hotkey_translate()
        {
            self.hotkey_translate = self.translate_hotkey.clone();
        }
    }
}

pub(crate) fn parse_hotkey(key: &str) -> PttKeyTarget {
    if key.is_empty() {
        return PttKeyTarget {
            modifier_mask: 0,
            regular_key: 0,
        };
    }
    let parts: Vec<&str> = key.split('+').collect();
    if parts.len() >= 2 {
        let last = *parts.last().unwrap();
        let regular_key = keycode_for_code(last);
        if regular_key != 0 {
            // Last part is a regular key; all preceding parts are modifiers
            let combined_mask = combine_modifier_masks(&parts[..parts.len() - 1]);
            PttKeyTarget {
                modifier_mask: combined_mask,
                regular_key,
            }
        } else {
            // No regular key recognized — treat first part as modifier-only
            PttKeyTarget {
                modifier_mask: modifier_mask_for(parts[0]),
                regular_key: 0,
            }
        }
    } else {
        PttKeyTarget {
            modifier_mask: modifier_mask_for(key),
            regular_key: 0,
        }
    }
}

/// Combine multiple modifier masks into a single u64.
/// macOS: bitwise OR of CGEventFlags device-dependent bits.
/// Windows: pack VK codes into 16-bit slots (up to 4 modifiers).
fn combine_modifier_masks(modifiers: &[&str]) -> u64 {
    #[cfg(target_os = "macos")]
    {
        modifiers
            .iter()
            .fold(0u64, |acc, &m| acc | modifier_mask_for(m))
    }
    #[cfg(target_os = "windows")]
    {
        debug_assert!(modifiers.len() <= 4, "Windows supports max 4 modifiers");
        modifiers.iter().enumerate().fold(0u64, |acc, (i, &m)| {
            acc | (modifier_mask_for(m) << (i * 16))
        })
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = modifiers;
        0
    }
}

impl Settings {
    /// Returns a PttKeyTarget for the configured dictation hotkey.
    pub fn ptt_key_target(&self) -> PttKeyTarget {
        parse_hotkey(&self.hotkey_dictation)
    }

    pub fn translate_key_target(&self) -> PttKeyTarget {
        parse_hotkey(&self.hotkey_translate)
    }

    /// Returns a PttKeyTarget for the voice command hotkey.
    pub fn voice_command_key_target(&self) -> PttKeyTarget {
        parse_hotkey(&self.hotkey_voice_command)
    }

    /// Returns a PttKeyTarget for the clipboard rewrite hotkey.
    pub fn clipboard_rewrite_key_target(&self) -> PttKeyTarget {
        parse_hotkey(&self.hotkey_clipboard_rewrite)
    }

    /// Apply text replacement rules to the given text.
    pub fn apply_replacements(&self, text: &str) -> String {
        let mut result = text.to_string();
        for rule in &self.text_replacements {
            if rule.enabled && !rule.find.is_empty() {
                result = result.replace(&rule.find, &rule.replace);
            }
        }
        result
    }

    /// Returns the whisper language code.
    pub fn whisper_language(&self) -> &str {
        match self.language.as_str() {
            "zh" => "zh",
            "en" => "en",
            "ja" => "ja",
            "ko" => "ko",
            "fr" => "fr",
            "de" => "de",
            "es" => "es",
            "pt" => "pt",
            "ru" => "ru",
            "ar" => "ar",
            "hi" => "hi",
            "th" => "th",
            "vi" => "vi",
            "id" => "id",
            _ => "auto",
        }
    }

    /// Builds the full initial_prompt for Whisper, combining language bias,
    /// enabled dictionary packs, and user custom dictionary.
    ///
    /// Whisper limits initial_prompt to ~224 tokens (max_initial_prompt_tokens).
    /// Order matters: Whisper weighs tokens closest to the audio (end of prompt)
    /// more heavily. We put dictionary terms first (can be truncated) and
    /// language bias + custom dictionary last (most important).
    pub fn whisper_initial_prompt(&self) -> String {
        let mut parts = Vec::new();

        // Dictionary pack terms first (lowest priority, truncated first by Whisper)
        for pack in &self.dictionary_packs {
            if let Some(content) = dict_pack_content(pack) {
                let terms: String = content
                    .lines()
                    .flat_map(|line| line.split(','))
                    .map(|t| t.trim())
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                if !terms.is_empty() {
                    parts.push(terms);
                }
            }
        }

        // Custom dictionary (higher priority than packs)
        if !self.dictionary.is_empty() {
            parts.push(self.dictionary.clone());
        }

        // Language bias last (highest priority — closest to audio)
        if self.language == "zh" || self.language == "auto" {
            parts.push("繁體中文語音轉錄，使用台灣正體中文。".to_string());
        }

        parts.join(" ")
    }
}

/// Returns the embedded content of a built-in dictionary pack.
fn dict_pack_content(name: &str) -> Option<&'static str> {
    match name {
        "crypto" => Some(include_str!("../../src/dictionaries/crypto.txt")),
        "ai-ml" => Some(include_str!("../../src/dictionaries/ai-ml.txt")),
        "dev-tools" => Some(include_str!("../../src/dictionaries/dev-tools.txt")),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn modifier_mask_for(key: &str) -> u64 {
    match key {
        "left_option" | "AltLeft" => 0x20,
        "right_option" | "AltRight" => 0x40,
        "left_command" | "MetaLeft" => 0x08,
        "right_command" | "MetaRight" => 0x10,
        "left_shift" | "ShiftLeft" => 0x02,
        "right_shift" | "ShiftRight" => 0x04,
        "left_control" | "ControlLeft" => 0x01,
        "right_control" | "ControlRight" => 0x2000,
        _ => 0x20,
    }
}

#[cfg(target_os = "windows")]
fn modifier_mask_for(key: &str) -> u64 {
    match key {
        "left_option" | "AltLeft" => 0xA4,
        "right_option" | "AltRight" => 0xA5,
        "left_command" | "MetaLeft" => 0x5B,
        "right_command" | "MetaRight" => 0x5C,
        "left_shift" | "ShiftLeft" => 0xA0,
        "right_shift" | "ShiftRight" => 0xA1,
        "left_control" | "ControlLeft" => 0xA2,
        "right_control" | "ControlRight" => 0xA3,
        _ => 0xA4,
    }
}

/// Maps JS `event.code` strings to macOS CGKeyCode values.
#[cfg(target_os = "macos")]
fn keycode_for_code(code: &str) -> u32 {
    match code {
        "KeyA" => 0x00,
        "KeyS" => 0x01,
        "KeyD" => 0x02,
        "KeyF" => 0x03,
        "KeyH" => 0x04,
        "KeyG" => 0x05,
        "KeyZ" => 0x06,
        "KeyX" => 0x07,
        "KeyC" => 0x08,
        "KeyV" => 0x09,
        "KeyB" => 0x0B,
        "KeyQ" => 0x0C,
        "KeyW" => 0x0D,
        "KeyE" => 0x0E,
        "KeyR" => 0x0F,
        "KeyY" => 0x10,
        "KeyT" => 0x11,
        "Digit1" => 0x12,
        "Digit2" => 0x13,
        "Digit3" => 0x14,
        "Digit4" => 0x15,
        "Digit6" => 0x16,
        "Digit5" => 0x17,
        "Digit9" => 0x19,
        "Digit7" => 0x1A,
        "Digit8" => 0x1C,
        "Digit0" => 0x1D,
        "KeyO" => 0x1F,
        "KeyU" => 0x20,
        "KeyI" => 0x22,
        "KeyP" => 0x23,
        "Return" | "Enter" => 0x24,
        "KeyL" => 0x25,
        "KeyJ" => 0x26,
        "KeyK" => 0x28,
        "KeyN" => 0x2D,
        "KeyM" => 0x2E,
        "Tab" => 0x30,
        "Space" => 0x31,
        _ => 0,
    }
}

/// Maps JS `event.code` strings to Windows Virtual Key codes.
#[cfg(target_os = "windows")]
fn keycode_for_code(code: &str) -> u32 {
    match code {
        "KeyA" => 0x41,
        "KeyB" => 0x42,
        "KeyC" => 0x43,
        "KeyD" => 0x44,
        "KeyE" => 0x45,
        "KeyF" => 0x46,
        "KeyG" => 0x47,
        "KeyH" => 0x48,
        "KeyI" => 0x49,
        "KeyJ" => 0x4A,
        "KeyK" => 0x4B,
        "KeyL" => 0x4C,
        "KeyM" => 0x4D,
        "KeyN" => 0x4E,
        "KeyO" => 0x4F,
        "KeyP" => 0x50,
        "KeyQ" => 0x51,
        "KeyR" => 0x52,
        "KeyS" => 0x53,
        "KeyT" => 0x54,
        "KeyU" => 0x55,
        "KeyV" => 0x56,
        "KeyW" => 0x57,
        "KeyX" => 0x58,
        "KeyY" => 0x59,
        "KeyZ" => 0x5A,
        "Digit0" => 0x30,
        "Digit1" => 0x31,
        "Digit2" => 0x32,
        "Digit3" => 0x33,
        "Digit4" => 0x34,
        "Digit5" => 0x35,
        "Digit6" => 0x36,
        "Digit7" => 0x37,
        "Digit8" => 0x38,
        "Digit9" => 0x39,
        "Space" => 0x20,
        "Return" | "Enter" => 0x0D,
        "Tab" => 0x09,
        _ => 0,
    }
}

fn settings_path(base: &Path) -> PathBuf {
    base.join("settings.json")
}

pub(crate) fn load_settings(base: &Path) -> Settings {
    let path = settings_path(base);
    let mut settings = match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(settings) => settings,
            Err(e) => {
                log::error!("failed to parse settings.json, using defaults: {e}");
                Settings::default()
            }
        },
        Err(_) => Settings::default(),
    };
    settings.migrate_legacy_hotkeys();
    settings
}

pub(crate) fn save_settings(settings: &Settings, base: &Path) -> Result<(), String> {
    let path = settings_path(base);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;

    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    use std::io::Write;
    let mut file = options.open(&path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_legacy_settings() {
        let json = r#"{
            "ptt_key": "AltLeft",
            "language": "auto",
            "engine": "local",
            "model": "large-v3-turbo",
            "groq_api_key": "gsk_test",
            "window_opacity": 0.78,
            "auto_start": false,
            "llm_enabled": true,
            "llm_model": "llama-3.3-70b-versatile"
        }"#;

        let mut s: Settings = serde_json::from_str(json).unwrap();
        s.migrate_legacy_hotkeys();
        assert_eq!(s.llm_provider, "groq");
        assert_eq!(s.ollama_url, "http://localhost:11434");
        assert_eq!(s.ollama_model, "llama3.2");
        assert!(s.custom_llm_url.is_empty());
        // Legacy ptt_key should migrate to hotkey_dictation
        assert_eq!(s.hotkey_dictation, "AltLeft");
    }

    #[test]
    fn test_deserialize_without_ui_locale() {
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
        assert_eq!(s.ui_locale, "en");
    }

    #[test]
    fn test_deserialize_new_settings() {
        let json = r#"{
            "ptt_key": "AltLeft",
            "language": "auto",
            "engine": "local",
            "model": "large-v3-turbo",
            "groq_api_key": "",
            "window_opacity": 0.78,
            "auto_start": false,
            "llm_enabled": true,
            "llm_model": "llama-3.3-70b-versatile",
            "llm_provider": "ollama",
            "ollama_url": "http://192.168.1.100:11434",
            "ollama_model": "mistral"
        }"#;

        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.llm_provider, "ollama");
        assert_eq!(s.ollama_url, "http://192.168.1.100:11434");
        assert_eq!(s.ollama_model, "mistral");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_ptt_key_target_single_modifier() {
        let s = Settings {
            hotkey_dictation: "AltLeft".to_string(),
            ..Settings::default()
        };
        let t = s.ptt_key_target();
        assert_eq!(t.modifier_mask, 0x20);
        assert_eq!(t.regular_key, 0);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_ptt_key_target_legacy_modifier() {
        let s = Settings {
            hotkey_dictation: "left_option".to_string(),
            ..Settings::default()
        };
        let t = s.ptt_key_target();
        assert_eq!(t.modifier_mask, 0x20);
        assert_eq!(t.regular_key, 0);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_ptt_key_target_combo() {
        let s = Settings {
            hotkey_dictation: "AltLeft+KeyZ".to_string(),
            ..Settings::default()
        };
        let t = s.ptt_key_target();
        assert_eq!(t.modifier_mask, 0x20); // NX_DEVICELALTKEYMASK
        assert_eq!(t.regular_key, 0x06); // CGKeyCode for Z
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_ptt_key_target_single_modifier_windows() {
        let s = Settings {
            hotkey_dictation: "AltLeft".to_string(),
            ..Settings::default()
        };
        let t = s.ptt_key_target();
        assert_eq!(t.modifier_mask, 0xA4);
        assert_eq!(t.regular_key, 0);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_ptt_key_target_combo_windows() {
        let s = Settings {
            hotkey_dictation: "AltLeft+KeyZ".to_string(),
            ..Settings::default()
        };
        let t = s.ptt_key_target();
        assert_eq!(t.modifier_mask, 0xA4); // VK_LMENU
        assert_eq!(t.regular_key, 0x5A); // VK_Z
    }

    #[test]
    fn test_ptt_key_target_unknown_regular_key_fallback() {
        let s = Settings {
            hotkey_dictation: "AltLeft+KeyUnknown".to_string(),
            ..Settings::default()
        };
        let t = s.ptt_key_target();
        assert_eq!(t.regular_key, 0);
    }

    #[test]
    fn test_whisper_initial_prompt_zh() {
        let s = Settings {
            language: "zh".to_string(),
            dictionary: "".to_string(),
            ..Settings::default()
        };
        assert_eq!(
            s.whisper_initial_prompt(),
            "繁體中文語音轉錄，使用台灣正體中文。"
        );
    }

    #[test]
    fn test_whisper_initial_prompt_auto() {
        let s = Settings {
            language: "auto".to_string(),
            dictionary: "".to_string(),
            ..Settings::default()
        };
        assert_eq!(
            s.whisper_initial_prompt(),
            "繁體中文語音轉錄，使用台灣正體中文。"
        );
    }

    #[test]
    fn test_whisper_initial_prompt_en() {
        let s = Settings {
            language: "en".to_string(),
            dictionary: "".to_string(),
            ..Settings::default()
        };
        assert_eq!(s.whisper_initial_prompt(), "");
    }

    #[test]
    fn test_whisper_initial_prompt_with_dictionary() {
        let s = Settings {
            language: "en".to_string(),
            dictionary: "Hello World".to_string(),
            ..Settings::default()
        };
        assert_eq!(s.whisper_initial_prompt(), "Hello World");
    }

    #[test]
    fn test_whisper_initial_prompt_zh_with_dictionary() {
        let s = Settings {
            language: "zh".to_string(),
            dictionary: "Hello World".to_string(),
            ..Settings::default()
        };
        assert_eq!(
            s.whisper_initial_prompt(),
            "Hello World 繁體中文語音轉錄，使用台灣正體中文。"
        );
    }

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
        assert_eq!(s.hotkey_translate, "AltLeft+KeyT");
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
    fn test_parse_hotkey_empty() {
        let t = parse_hotkey("");
        assert_eq!(t.modifier_mask, 0);
        assert_eq!(t.regular_key, 0);
    }

    #[test]
    fn test_translate_key_target() {
        let s = Settings::default();
        let t = s.translate_key_target();
        assert_ne!(t.modifier_mask, 0);
        assert_ne!(t.regular_key, 0);
    }

    #[test]
    fn test_parse_hotkey_multi_modifier() {
        // MetaLeft+ShiftLeft+KeyT → two modifiers + regular key
        let t = parse_hotkey("MetaLeft+ShiftLeft+KeyT");
        assert_ne!(t.regular_key, 0); // T key
        assert_ne!(t.modifier_mask, 0);

        // Combined mask should differ from single modifier
        let single = parse_hotkey("MetaLeft+KeyT");
        assert_ne!(t.modifier_mask, single.modifier_mask);
        assert_eq!(t.regular_key, single.regular_key); // same regular key
    }

    #[test]
    fn test_apply_replacements_no_rules() {
        let s = Settings::default();
        assert_eq!(s.apply_replacements("hello world"), "hello world");
    }

    #[test]
    fn test_apply_replacements_single_rule() {
        let s = Settings {
            text_replacements: vec![TextReplacement {
                find: "foo".to_string(),
                replace: "bar".to_string(),
                enabled: true,
            }],
            ..Settings::default()
        };
        assert_eq!(s.apply_replacements("foo baz foo"), "bar baz bar");
    }

    #[test]
    fn test_apply_replacements_multiple_rules() {
        let s = Settings {
            text_replacements: vec![
                TextReplacement {
                    find: "a".to_string(),
                    replace: "b".to_string(),
                    enabled: true,
                },
                TextReplacement {
                    find: "c".to_string(),
                    replace: "d".to_string(),
                    enabled: true,
                },
            ],
            ..Settings::default()
        };
        assert_eq!(s.apply_replacements("a c"), "b d");
    }

    #[test]
    fn test_apply_replacements_disabled_rule_skipped() {
        let s = Settings {
            text_replacements: vec![TextReplacement {
                find: "foo".to_string(),
                replace: "bar".to_string(),
                enabled: false,
            }],
            ..Settings::default()
        };
        assert_eq!(s.apply_replacements("foo"), "foo");
    }

    #[test]
    fn test_apply_replacements_empty_find_skipped() {
        let s = Settings {
            text_replacements: vec![TextReplacement {
                find: String::new(),
                replace: "bar".to_string(),
                enabled: true,
            }],
            ..Settings::default()
        };
        assert_eq!(s.apply_replacements("hello"), "hello");
    }

    #[test]
    fn test_apply_replacements_empty_replace_deletes() {
        let s = Settings {
            text_replacements: vec![TextReplacement {
                find: "remove".to_string(),
                replace: String::new(),
                enabled: true,
            }],
            ..Settings::default()
        };
        assert_eq!(s.apply_replacements("please remove this"), "please  this");
    }

    #[test]
    fn test_text_replacements_serialization_roundtrip() {
        let s = Settings {
            text_replacements: vec![
                TextReplacement {
                    find: "GPT".to_string(),
                    replace: "LLM".to_string(),
                    enabled: true,
                },
                TextReplacement {
                    find: "typo".to_string(),
                    replace: "type".to_string(),
                    enabled: false,
                },
            ],
            ..Settings::default()
        };
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.text_replacements.len(), 2);
        assert_eq!(deserialized.text_replacements[0].find, "GPT");
        assert_eq!(deserialized.text_replacements[0].replace, "LLM");
        assert!(deserialized.text_replacements[0].enabled);
        assert_eq!(deserialized.text_replacements[1].find, "typo");
        assert!(!deserialized.text_replacements[1].enabled);
    }

    // --- Migration tests ---

    #[test]
    fn test_migrate_legacy_ptt_key() {
        let json = r#"{
            "ptt_key": "right_option",
            "language": "auto",
            "engine": "local"
        }"#;
        let mut s: Settings = serde_json::from_str(json).unwrap();
        s.migrate_legacy_hotkeys();
        assert_eq!(s.hotkey_dictation, "right_option");
    }

    #[test]
    fn test_migrate_legacy_translate_hotkey() {
        let json = r#"{
            "translate_hotkey": "MetaLeft+KeyT",
            "language": "auto"
        }"#;
        let mut s: Settings = serde_json::from_str(json).unwrap();
        s.migrate_legacy_hotkeys();
        assert_eq!(s.hotkey_translate, "MetaLeft+KeyT");
    }

    #[test]
    fn test_no_migration_when_new_fields_present() {
        let json = r#"{
            "ptt_key": "old_value",
            "hotkey_dictation": "new_value",
            "translate_hotkey": "old_translate",
            "hotkey_translate": "new_translate"
        }"#;
        let mut s: Settings = serde_json::from_str(json).unwrap();
        s.migrate_legacy_hotkeys();
        // New fields should NOT be overwritten by legacy fields
        assert_eq!(s.hotkey_dictation, "new_value");
        assert_eq!(s.hotkey_translate, "new_translate");
    }

    #[test]
    fn test_voice_command_and_clipboard_rewrite_defaults() {
        let s = Settings::default();
        assert!(s.hotkey_voice_command.is_empty());
        assert!(s.hotkey_clipboard_rewrite.is_empty());
        // Empty hotkey should parse to disabled (mask = 0)
        let vc = s.voice_command_key_target();
        assert_eq!(vc.modifier_mask, 0);
        let cr = s.clipboard_rewrite_key_target();
        assert_eq!(cr.modifier_mask, 0);
    }
}
