use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_hold() -> String {
    "hold".to_string()
}

fn default_llm_model() -> String {
    "llama-3.3-70b-versatile".to_string()
}

fn default_true() -> bool {
    true
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
    #[serde(default = "default_true")]
    pub app_aware_style: bool,
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
            app_aware_style: true,
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

fn settings_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join("Library")
            .join("Application Support")
            .join("com.murmur.voice")
            .join("settings.json")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"));
        appdata.join("com.murmur.voice").join("settings.json")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        PathBuf::from("/tmp/com.murmur.voice/settings.json")
    }
}

pub(crate) fn load_settings() -> Settings {
    let path = settings_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub(crate) fn save_settings(settings: &Settings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}
