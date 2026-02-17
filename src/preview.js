const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const headerText = () => document.getElementById("header-text");
const previewText = () => document.getElementById("preview-text");
const charCount = () => document.getElementById("char-count");
const appBadge = () => document.getElementById("app-badge");
const previewBody = () => document.getElementById("preview-body");
const copyBtn = () => document.getElementById("copy-btn");
const dictSuggest = () => document.getElementById("dict-suggest");
const dictSuggestText = () => document.getElementById("dict-suggest-text");

let autoHideTimer = null;
let dotsInterval = null;
let currentMode = null;
let originalText = "";
let pendingSuggestion = null;

function setHeader(text, processing) {
  const el = headerText();
  if (dotsInterval) {
    clearInterval(dotsInterval);
    dotsInterval = null;
  }
  if (processing) {
    let dots = 0;
    el.textContent = text;
    dotsInterval = setInterval(() => {
      dots = (dots + 1) % 4;
      el.textContent = text + ".".repeat(dots);
    }, 400);
  } else {
    el.textContent = text;
  }
}

function setText(text, cls) {
  const el = previewText();
  el.textContent = text;
  el.className = "preview-text";
  if (cls) el.classList.add(cls);
}

function setCharCount(text) {
  const el = charCount();
  if (!text || text.length === 0) {
    el.textContent = "";
    return;
  }
  el.textContent = t("preview.nChars").replace("{n}", text.length);
}

function setAppBadge(name) {
  const el = appBadge();
  if (!name) {
    el.classList.remove("visible");
    el.textContent = "";
    return;
  }
  el.textContent = name;
  el.classList.add("visible");
}

function clearAutoHide() {
  if (autoHideTimer) {
    clearTimeout(autoHideTimer);
    autoHideTimer = null;
  }
}

function enableEditing() {
  previewText().setAttribute("contenteditable", "true");
  clearAutoHide();
}

function disableEditing() {
  previewText().removeAttribute("contenteditable");
}

function wordDiff(original, edited) {
  const origWords = original.split(/\s+/).filter(Boolean).map((w) => w.toLowerCase());
  const editWords = edited.split(/\s+/).filter(Boolean);
  return editWords.filter((w) => !origWords.includes(w.toLowerCase()));
}

function showDictSuggest(word) {
  pendingSuggestion = word;
  dictSuggestText().textContent = t("preview.dictPrompt").replace("{word}", word);
  dictSuggest().style.display = "";
}

function hideDictSuggest() {
  dictSuggest().style.display = "none";
  pendingSuggestion = null;
}

function reset() {
  clearAutoHide();
  if (dotsInterval) {
    clearInterval(dotsInterval);
    dotsInterval = null;
  }
  setHeader(t("state.listening"), false);
  setText(t("state.listening"), "placeholder");
  setCharCount("");
  setAppBadge(null);
  disableEditing();
  copyBtn().style.display = "none";
  hideDictSuggest();
  currentMode = null;
  originalText = "";
}

function scrollToBottom() {
  const body = previewBody();
  body.scrollTop = body.scrollHeight;
}

window.addEventListener("DOMContentLoaded", async () => {
  // Load locale
  try {
    const s = await invoke("get_settings");
    currentLocale = s.ui_locale || "en";
  } catch (_) {}

  // Copy button handler
  copyBtn().addEventListener("click", async () => {
    const btn = copyBtn();
    const text = previewText().textContent;
    try {
      await invoke("copy_to_clipboard", { text });
      btn.textContent = t("preview.copied");
      btn.classList.add("copied");
      setTimeout(() => {
        btn.textContent = t("preview.copy");
        btn.classList.remove("copied");
        invoke("hide_preview").catch(() => {});
      }, 1500);
    } catch (_) {
      // silently fail
    }
  });

  // Dict suggestion handlers
  document.getElementById("dict-add-btn").addEventListener("click", async () => {
    if (!pendingSuggestion) return;
    try {
      await invoke("add_dictionary_term", { term: pendingSuggestion });
      dictSuggestText().textContent = t("preview.dictAdded");
      setTimeout(() => hideDictSuggest(), 1200);
    } catch (_) {
      hideDictSuggest();
    }
  });

  document.getElementById("dict-dismiss-btn").addEventListener("click", () => {
    hideDictSuggest();
  });

  // Cancel auto-hide when user focuses on text to edit
  previewText().addEventListener("focus", () => {
    clearAutoHide();
  });

  // Detect edits on blur and suggest dictionary additions
  previewText().addEventListener("blur", () => {
    const edited = previewText().textContent;
    if (!originalText || originalText === edited) return;
    const newWords = wordDiff(originalText, edited);
    if (newWords.length > 0) {
      showDictSuggest(newWords[0]);
    }

    // Restart auto-hide after editing
    if (currentMode === "pasted") {
      autoHideTimer = setTimeout(async () => {
        try {
          await invoke("hide_overlay_windows");
        } catch (_) {}
      }, 5000);
    }
  });

  await listen("recording_state_changed", (event) => {
    const state = event.payload;
    switch (state) {
      case "starting":
        reset();
        break;
      case "recording":
        setHeader(t("state.listening"), false);
        break;
      case "stopping":
        setHeader(t("state.stopping"), true);
        break;
      case "transcribing":
        setHeader(t("state.transcribing"), true);
        setText("", null);
        break;
      case "processing":
        setHeader(t("state.processing"), true);
        break;
      case "idle":
        // Auto-hide is handled by the transcription_complete handler below.
        break;
    }
  });

  await listen("partial_transcription", (event) => {
    const text = event.payload;
    if (text) {
      setText(text, null);
      scrollToBottom();
    }
  });

  await listen("transcription_complete", (event) => {
    const { text, mode } = event.payload;
    clearAutoHide();
    currentMode = mode;

    if (!text || text.trim().length === 0) {
      setHeader(t("state.done"), false);
      setText(t("preview.noSpeech"), "no-speech");
      setCharCount("");
      copyBtn().style.display = "none";
      disableEditing();
    } else {
      setHeader(t("state.done"), false);
      setText(text, null);
      setCharCount(text);
      scrollToBottom();
      originalText = text;
      copyBtn().style.display = "";
      enableEditing();

      // Auto-hide: 5s for pasted mode, cancelled by editing
      if (mode === "pasted" && text && text.trim().length > 0) {
        autoHideTimer = setTimeout(async () => {
          try {
            await invoke("hide_overlay_windows");
          } catch (_) {}
        }, 5000);
      }
    }
  });

  await listen("foreground_app_info", (event) => {
    const { name } = event.payload;
    if (name && name !== "Unknown") {
      setAppBadge(name);
    }
  });

  await listen("enhancer_info", (event) => {
    const { name, local } = event.payload;
    const el = appBadge();
    if (el) {
      el.textContent = local ? `${name} (Local)` : `${name} (Cloud)`;
      el.className = "app-badge " + (local ? "badge-local" : "badge-cloud") + " visible";
    }
  });
});
