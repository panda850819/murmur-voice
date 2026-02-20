const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const headerText = () => document.getElementById("header-text");
const previewText = () => document.getElementById("preview-text");
const charCount = () => document.getElementById("char-count");
const appBadge = () => document.getElementById("app-badge");
const previewBody = () => document.getElementById("preview-body");
const copyBtn = () => document.getElementById("copy-btn");
const dictSuggest = () => document.getElementById("dict-suggest");
const dictChips = () => document.getElementById("dict-chips");

let autoHideTimer = null;
let dotsInterval = null;
let currentMode = null;
let originalText = "";
let addedWords = new Set();
let dismissedWords = new Set();
let debounceTimer = null;

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

function tokenize(text) {
  if (typeof Intl !== "undefined" && Intl.Segmenter) {
    const segmenter = new Intl.Segmenter(undefined, { granularity: "word" });
    return [...segmenter.segment(text)]
      .filter((s) => s.isWordLike)
      .map((s) => s.segment);
  }
  return text.split(/\s+/).filter(Boolean);
}

function wordDiff(original, edited) {
  const origSet = new Set(tokenize(original).map((w) => w.toLowerCase()));
  const editWords = tokenize(edited);
  return editWords.filter((w) => !origSet.has(w.toLowerCase()) && w.length >= 2);
}

function detectNewWords() {
  const edited = previewText().textContent;
  if (!originalText || originalText === edited) {
    hideDictSuggest();
    return;
  }
  const newWords = wordDiff(originalText, edited)
    .filter((w) => !addedWords.has(w.toLowerCase()) && !dismissedWords.has(w.toLowerCase()));
  if (newWords.length > 0) {
    showDictSuggestions(newWords);
  } else {
    hideDictSuggest();
  }
}

function showDictSuggestions(words) {
  const container = dictChips();
  while (container.firstChild) container.removeChild(container.firstChild);
  words.forEach((word) => {
    const chip = document.createElement("button");
    chip.className = "dict-chip";
    chip.textContent = "+ " + word;
    chip.title = t("preview.add");
    chip.addEventListener("click", () => addDictWord(word, chip));
    container.appendChild(chip);
  });
  const addAllBtn = document.getElementById("dict-add-all-btn");
  addAllBtn.classList.toggle("hidden", words.length < 2);
  dictSuggest().classList.remove("hidden");
}

async function addDictWord(word, chipEl) {
  try {
    await invoke("add_dictionary_term", { term: word });
    addedWords.add(word.toLowerCase());
    chipEl.textContent = t("preview.dictAdded");
    chipEl.classList.add("added");
    setTimeout(() => {
      chipEl.remove();
      updateDictBar();
    }, 800);
  } catch (e) {
    console.error("add_dictionary_term failed:", e);
    chipEl.textContent = "Error";
    chipEl.classList.add("added");
    setTimeout(() => {
      chipEl.remove();
      updateDictBar();
    }, 1200);
  }
}

function updateDictBar() {
  const chips = dictChips().querySelectorAll(".dict-chip:not(.added)");
  if (chips.length === 0) {
    hideDictSuggest();
    return;
  }
  const addAllBtn = document.getElementById("dict-add-all-btn");
  addAllBtn.classList.toggle("hidden", chips.length < 2);
}

function hideDictSuggest() {
  dictSuggest().classList.add("hidden");
  const container = dictChips();
  while (container.firstChild) container.removeChild(container.firstChild);
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
  copyBtn().classList.add("hidden");
  hideDictSuggest();
  currentMode = null;
  originalText = "";
  addedWords.clear();
  dismissedWords.clear();
  if (debounceTimer) {
    clearTimeout(debounceTimer);
    debounceTimer = null;
  }
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
  document.getElementById("dict-add-all-btn").addEventListener("click", async () => {
    const chips = Array.from(dictChips().querySelectorAll(".dict-chip:not(.added)"));
    const words = chips.map((chip) => {
      let w = chip.textContent;
      if (w.startsWith("+ ")) w = w.substring(2);
      return w;
    });

    try {
      await invoke("add_dictionary_terms", { terms: words });
      chips.forEach((chip, i) => {
        addedWords.add(words[i].toLowerCase());
        chip.textContent = t("preview.dictAdded");
        chip.classList.add("added");
      });
    } catch (_) {}
    setTimeout(() => hideDictSuggest(), 800);
  });

  document.getElementById("dict-dismiss-btn").addEventListener("click", () => {
    const chips = dictChips().querySelectorAll(".dict-chip:not(.added)");
    chips.forEach((chip) => dismissedWords.add(chip.textContent.toLowerCase()));
    hideDictSuggest();
  });

  // Cancel auto-hide when user focuses on text to edit
  previewText().addEventListener("focus", () => {
    clearAutoHide();
  });

  // Detect new words in real-time while editing
  previewText().addEventListener("input", () => {
    setCharCount(previewText().textContent);
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(detectNewWords, 500);
  });

  // Final detection on blur + restart auto-hide
  previewText().addEventListener("blur", () => {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }
    detectNewWords();

    // Restart auto-hide after editing
    if (currentMode === TRANSCRIPTION_MODES.PASTED) {
      autoHideTimer = setTimeout(async () => {
        try {
          await invoke("hide_overlay_windows");
        } catch (_) {}
      }, 5000);
    }
  });

  await listen(EVENTS.RECORDING_STATE_CHANGED, (event) => {
    const state = event.payload;
    switch (state) {
      case RECORDING_STATES.STARTING:
        reset();
        break;
      case RECORDING_STATES.RECORDING:
        setHeader(t("state.listening"), false);
        break;
      case RECORDING_STATES.STOPPING:
        setHeader(t("state.stopping"), true);
        break;
      case RECORDING_STATES.TRANSCRIBING:
        setHeader(t("state.transcribing"), true);
        setText("", null);
        break;
      case RECORDING_STATES.PROCESSING:
        setHeader(t("state.processing"), true);
        break;
      case RECORDING_STATES.IDLE:
        // Auto-hide is handled by the transcription_complete handler below.
        break;
    }
  });

  await listen(EVENTS.PARTIAL_TRANSCRIPTION, (event) => {
    const text = event.payload;
    if (text) {
      setText(text, null);
      scrollToBottom();
    }
  });

  await listen(EVENTS.TRANSCRIPTION_COMPLETE, (event) => {
    const { text, mode } = event.payload;
    clearAutoHide();
    currentMode = mode;

    if (!text || text.trim().length === 0) {
      setHeader(t("state.done"), false);
      setText(t("preview.noSpeech"), "no-speech");
      setCharCount("");
      copyBtn().classList.add("hidden");
      disableEditing();
    } else {
      setHeader(t("state.done"), false);
      setText(text, null);
      setCharCount(text);
      scrollToBottom();
      originalText = text;
      copyBtn().classList.remove("hidden");
      enableEditing();

      // Auto-hide: 5s for pasted mode, cancelled by editing
      if (mode === TRANSCRIPTION_MODES.PASTED && text && text.trim().length > 0) {
        autoHideTimer = setTimeout(async () => {
          try {
            await invoke("hide_overlay_windows");
          } catch (_) {}
        }, 5000);
      }
    }
  });

  await listen(EVENTS.FOREGROUND_APP_INFO, (event) => {
    const { name } = event.payload;
    if (name && name !== "Unknown") {
      setAppBadge(name);
    }
  });

  await listen(EVENTS.ENHANCER_INFO, (event) => {
    const { name, local } = event.payload;
    const el = appBadge();
    if (el) {
      el.textContent = local ? `${name} (Local)` : `${name} (Cloud)`;
      el.className = "app-badge " + (local ? "badge-local" : "badge-cloud") + " visible";
    }
  });
});
