use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
    static MAP: OnceLock<HashMap<&'static str, u32>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("KeyA", 0x00);
        m.insert("KeyS", 0x01);
        m.insert("KeyD", 0x02);
        m.insert("KeyF", 0x03);
        m.insert("KeyH", 0x04);
        m.insert("KeyG", 0x05);
        m.insert("KeyZ", 0x06);
        m.insert("KeyX", 0x07);
        m.insert("KeyC", 0x08);
        m.insert("KeyV", 0x09);
        m.insert("KeyB", 0x0B);
        m.insert("KeyQ", 0x0C);
        m.insert("KeyW", 0x0D);
        m.insert("KeyE", 0x0E);
        m.insert("KeyR", 0x0F);
        m.insert("KeyY", 0x10);
        m.insert("KeyT", 0x11);
        m.insert("Digit1", 0x12);
        m.insert("Digit2", 0x13);
        m.insert("Digit3", 0x14);
        m.insert("Digit4", 0x15);
        m.insert("Digit6", 0x16);
        m.insert("Digit5", 0x17);
        m.insert("Digit9", 0x19);
        m.insert("Digit7", 0x1A);
        m.insert("Digit8", 0x1C);
        m.insert("Digit0", 0x1D);
        m.insert("KeyO", 0x1F);
        m.insert("KeyU", 0x20);
        m.insert("KeyI", 0x22);
        m.insert("KeyP", 0x23);
        m.insert("Return", 0x24);
        m.insert("Enter", 0x24);
        m.insert("KeyL", 0x25);
        m.insert("KeyJ", 0x26);
        m.insert("KeyK", 0x28);
        m.insert("KeyN", 0x2D);
        m.insert("KeyM", 0x2E);
        m.insert("Tab", 0x30);
        m.insert("Space", 0x31);
        m
    });
    *map.get(code).unwrap_or(&0)
}

/// Maps JS `event.code` strings to Windows Virtual Key codes.
#[cfg(target_os = "windows")]
fn keycode_for_code(code: &str) -> u32 {
    static MAP: OnceLock<HashMap<&'static str, u32>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("KeyA", 0x41);
        m.insert("KeyB", 0x42);
        m.insert("KeyC", 0x43);
        m.insert("KeyD", 0x44);
        m.insert("KeyE", 0x45);
        m.insert("KeyF", 0x46);
        m.insert("KeyG", 0x47);
        m.insert("KeyH", 0x48);
        m.insert("KeyI", 0x49);
        m.insert("KeyJ", 0x4A);
        m.insert("KeyK", 0x4B);
        m.insert("KeyL", 0x4C);
        m.insert("KeyM", 0x4D);
        m.insert("KeyN", 0x4E);
        m.insert("KeyO", 0x4F);
        m.insert("KeyP", 0x50);
        m.insert("KeyQ", 0x51);
        m.insert("KeyR", 0x52);
        m.insert("KeyS", 0x53);
        m.insert("KeyT", 0x54);
        m.insert("KeyU", 0x55);
        m.insert("KeyV", 0x56);
        m.insert("KeyW", 0x57);
        m.insert("KeyX", 0x58);
        m.insert("KeyY", 0x59);
        m.insert("KeyZ", 0x5A);
        m.insert("Digit0", 0x30);
        m.insert("Digit1", 0x31);
        m.insert("Digit2", 0x32);
        m.insert("Digit3", 0x33);
        m.insert("Digit4", 0x34);
        m.insert("Digit5", 0x35);
        m.insert("Digit6", 0x36);
        m.insert("Digit7", 0x37);
        m.insert("Digit8", 0x38);
        m.insert("Digit9", 0x39);
        m.insert("Space", 0x20);
        m.insert("Return", 0x0D);
        m.insert("Enter", 0x0D);
        m.insert("Tab", 0x09);
        m
    });
    *map.get(code).unwrap_or(&0)
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
    #[cfg(target_os = "macos")]
    fn test_keycode_for_code_macos() {
        assert_eq!(keycode_for_code("KeyA"), 0x00);
        assert_eq!(keycode_for_code("Space"), 0x31);
        assert_eq!(keycode_for_code("Return"), 0x24);
        assert_eq!(keycode_for_code("Enter"), 0x24);
        assert_eq!(keycode_for_code("Unknown"), 0);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_keycode_for_code_windows() {
        assert_eq!(keycode_for_code("KeyA"), 0x41);
        assert_eq!(keycode_for_code("Space"), 0x20);
        assert_eq!(keycode_for_code("Return"), 0x0D);
        assert_eq!(keycode_for_code("Enter"), 0x0D);
        assert_eq!(keycode_for_code("Unknown"), 0);
    }
}
