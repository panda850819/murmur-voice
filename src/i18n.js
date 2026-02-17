const I18N = {
  en: {
    "header.title": "Preferences",
    "group.input": "Input",
    "group.transcription": "Transcription",
    "group.ai": "AI Processing",
    "group.recording": "Recording",
    "group.appearance": "Appearance",
    "group.system": "System",
    "row.ptt": "Push-to-Talk",
    "row.language": "Language",
    "row.engine": "Engine",
    "row.model": "Model",
    "row.dict": "Dictionary",
    "row.llm": "LLM Post-Processing",
    "row.provider": "Provider",
    "row.apiKey": "API Key",
    "row.url": "URL",
    "row.endpoint": "Endpoint",
    "row.appAware": "App-Aware Style",
    "row.mode": "Mode",
    "row.opacity": "Opacity",
    "row.autoStart": "Launch at Login",
    "row.updates": "Updates",
    "row.changelog": "What's New",
    "row.uiLocale": "UI Language",
    "btn.checkUpdate": "Check for Updates",
    "btn.changelog": "Changelog",
    "btn.cancel": "Cancel",
    "btn.save": "Save",
    "btn.holdToTalk": "Hold to Talk",
    "btn.pressToToggle": "Press to Toggle",
    "hint.groqKey": "API Key is in AI Processing below",
    "dict.placeholder": "Type a term and press Enter",
    "vision.tagline": "Your voice, unheard by others.",
    "vision.roadmap": "Roadmap",
  },
  "zh-TW": {
    "header.title": "偏好設定",
    "group.input": "輸入",
    "group.transcription": "語音轉錄",
    "group.ai": "AI 處理",
    "group.recording": "錄音",
    "group.appearance": "外觀",
    "group.system": "系統",
    "row.ptt": "按鍵說話",
    "row.language": "語言",
    "row.engine": "引擎",
    "row.model": "模型",
    "row.dict": "自訂辭典",
    "row.llm": "LLM 後處理",
    "row.provider": "供應商",
    "row.apiKey": "API Key",
    "row.url": "URL",
    "row.endpoint": "端點",
    "row.appAware": "應用感知風格",
    "row.mode": "模式",
    "row.opacity": "透明度",
    "row.autoStart": "登入時啟動",
    "row.updates": "更新",
    "row.changelog": "新功能",
    "row.uiLocale": "介面語言",
    "btn.checkUpdate": "檢查更新",
    "btn.changelog": "更新日誌",
    "btn.cancel": "取消",
    "btn.save": "儲存",
    "btn.holdToTalk": "長按錄音",
    "btn.pressToToggle": "點按錄音",
    "hint.groqKey": "API Key 在下方 AI 處理區",
    "dict.placeholder": "輸入詞彙後按 Enter",
    "vision.tagline": "你的聲音，不被他人聽見。",
    "vision.roadmap": "產品路線圖",
  },
};

let currentLocale = "en";

function t(key) {
  const map = I18N[currentLocale] || I18N.en;
  return map[key] || I18N.en[key] || key;
}

function applyLocale(locale) {
  currentLocale = locale;
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n");
    const text = t(key);
    if (text && text !== key) {
      if (el.tagName === "INPUT" && el.type !== "checkbox") {
        el.placeholder = text;
      } else {
        el.textContent = text;
      }
    }
  });
}
