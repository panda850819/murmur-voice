use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn default_hold() -> String {
    "hold".to_string()
}

fn default_llm_model() -> String {
    "llama-3.3-70b-versatile".to_string()
}

fn default_true() -> bool {
    true
}

fn default_groq() -> String {
    "groq".to_string()
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    "llama3.2".to_string()
}

fn default_en() -> String {
    "en".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub ptt_key: String,
    pub language: String,
    pub engine: String,
    pub model: String,
    pub groq_api_key: String,
    pub window_opacity: f64,
    pub auto_start: bool,
    #[serde(default)]
    pub onboarding_complete: bool,
    #[serde(default = "default_hold")]
    pub recording_mode: String,
    #[serde(default)]
    pub dictionary: String,
    #[serde(default)]
    pub llm_enabled: bool,
    #[serde(default = "default_llm_model")]
    pub llm_model: String,
    #[serde(default = "default_groq")]
    pub llm_provider: String,
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,
    #[serde(default)]
    pub custom_llm_url: String,
    #[serde(default)]
    pub custom_llm_key: String,
    #[serde(default)]
    pub custom_llm_model: String,
    #[serde(default = "default_true")]
    pub app_aware_style: bool,
    #[serde(default = "default_en")]
    pub ui_locale: String,
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
            ptt_key: "left_option".to_string(),
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
            llm_model: default_llm_model(),
            llm_provider: default_groq(),
            ollama_url: default_ollama_url(),
            ollama_model: default_ollama_model(),
            custom_llm_url: String::new(),
            custom_llm_key: String::new(),
            custom_llm_model: String::new(),
            app_aware_style: true,
            ui_locale: default_en(),
        }
    }
}

impl Settings {
    /// Returns a PttKeyTarget for the configured PTT key.
    /// Supports single modifier ("AltLeft") and combo ("AltLeft+KeyZ") formats.
    pub fn ptt_key_target(&self) -> PttKeyTarget {
        if let Some(plus_pos) = self.ptt_key.find('+') {
            let modifier_str = &self.ptt_key[..plus_pos];
            let key_str = &self.ptt_key[plus_pos + 1..];
            PttKeyTarget {
                modifier_mask: modifier_mask_for(modifier_str),
                regular_key: keycode_for_code(key_str),
            }
        } else {
            PttKeyTarget {
                modifier_mask: modifier_mask_for(&self.ptt_key),
                regular_key: 0,
            }
        }
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

    /// Builds the full initial_prompt for Whisper, combining language bias and user dictionary.
    pub fn whisper_initial_prompt(&self) -> String {
        let mut parts = Vec::new();

        // Bias Whisper toward Traditional Chinese output when language is zh
        if self.language == "zh" || self.language == "auto" {
            parts.push("繁體中文語音轉錄，使用台灣正體中文。".to_string());
        }

        if !self.dictionary.is_empty() {
            parts.push(self.dictionary.clone());
        }

        parts.join(" ")
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
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub(crate) fn save_settings(settings: &Settings, base: &Path) -> Result<(), String> {
    let path = settings_path(base);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600); // Read/write by owner only
            let _ = std::fs::set_permissions(&path, perms);
        }
    }

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

        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.llm_provider, "groq");
        assert_eq!(s.ollama_url, "http://localhost:11434");
        assert_eq!(s.ollama_model, "llama3.2");
        assert!(s.custom_llm_url.is_empty());
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
            ptt_key: "AltLeft".to_string(),
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
            ptt_key: "left_option".to_string(),
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
            ptt_key: "AltLeft+KeyZ".to_string(),
            ..Settings::default()
        };
        let t = s.ptt_key_target();
        assert_eq!(t.modifier_mask, 0x20); // NX_DEVICELALTKEYMASK
        assert_eq!(t.regular_key, 0x06);  // CGKeyCode for Z
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_ptt_key_target_single_modifier_windows() {
        let s = Settings {
            ptt_key: "AltLeft".to_string(),
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
            ptt_key: "AltLeft+KeyZ".to_string(),
            ..Settings::default()
        };
        let t = s.ptt_key_target();
        assert_eq!(t.modifier_mask, 0xA4); // VK_LMENU
        assert_eq!(t.regular_key, 0x5A);   // VK_Z
    }

    #[test]
    fn test_ptt_key_target_unknown_regular_key_fallback() {
        let s = Settings {
            ptt_key: "AltLeft+KeyUnknown".to_string(),
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
        assert_eq!(s.whisper_initial_prompt(), "繁體中文語音轉錄，使用台灣正體中文。");
    }

    #[test]
    fn test_whisper_initial_prompt_auto() {
        let s = Settings {
            language: "auto".to_string(),
            dictionary: "".to_string(),
            ..Settings::default()
        };
        assert_eq!(s.whisper_initial_prompt(), "繁體中文語音轉錄，使用台灣正體中文。");
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
        assert_eq!(s.whisper_initial_prompt(), "繁體中文語音轉錄，使用台灣正體中文。 Hello World");
    }
}
