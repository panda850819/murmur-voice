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
    /// Returns the platform-specific key mask for the configured PTT key.
    #[cfg(target_os = "macos")]
    pub fn ptt_key_mask(&self) -> u64 {
        match self.ptt_key.as_str() {
            "left_option" | "AltLeft" => 0x20,       // NX_DEVICELALTKEYMASK
            "right_option" | "AltRight" => 0x40,     // NX_DEVICERALTKEYMASK
            "left_command" | "MetaLeft" => 0x08,     // NX_DEVICELCMDKEYMASK
            "right_command" | "MetaRight" => 0x10,   // NX_DEVICERCMDKEYMASK
            "left_shift" | "ShiftLeft" => 0x02,      // NX_DEVICELSSHIFTKEYMASK
            "right_shift" | "ShiftRight" => 0x04,    // NX_DEVICERSHIFTKEYMASK
            "left_control" | "ControlLeft" => 0x01,  // NX_DEVICELCTLKEYMASK
            "right_control" | "ControlRight" => 0x2000, // NX_DEVICERCTLKEYMASK
            _ => 0x20,
        }
    }

    /// Returns the platform-specific key mask for the configured PTT key.
    #[cfg(target_os = "windows")]
    pub fn ptt_key_mask(&self) -> u64 {
        match self.ptt_key.as_str() {
            "left_option" | "AltLeft" => 0xA4,        // VK_LMENU
            "right_option" | "AltRight" => 0xA5,       // VK_RMENU
            "left_command" | "MetaLeft" => 0x5B,       // VK_LWIN
            "right_command" | "MetaRight" => 0x5C,     // VK_RWIN
            "left_shift" | "ShiftLeft" => 0xA0,        // VK_LSHIFT
            "right_shift" | "ShiftRight" => 0xA1,      // VK_RSHIFT
            "left_control" | "ControlLeft" => 0xA2,    // VK_LCONTROL
            "right_control" | "ControlRight" => 0xA3,  // VK_RCONTROL
            _ => 0xA4, // default: left alt
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
}
