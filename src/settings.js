const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

// Key mapping: JS event.code → display name
const KEY_MAP = {
  "AltLeft": "Left Option",
  "AltRight": "Right Option",
  "MetaLeft": "Left Command",
  "MetaRight": "Right Command",
  "ShiftLeft": "Left Shift",
  "ShiftRight": "Right Shift",
  "ControlLeft": "Left Control",
  "ControlRight": "Right Control",
};

// Reverse map: legacy internal name → display name
const LEGACY_DISPLAY = {
  "left_option": "Left Option",
  "right_option": "Right Option",
  "left_command": "Left Command",
  "right_command": "Right Command",
  "left_shift": "Left Shift",
  "right_shift": "Right Shift",
  "left_control": "Left Control",
  "right_control": "Right Control",
};

// Regular key display map: JS event.code → label
const REGULAR_KEY_MAP = {
  "KeyA": "A", "KeyB": "B", "KeyC": "C", "KeyD": "D", "KeyE": "E",
  "KeyF": "F", "KeyG": "G", "KeyH": "H", "KeyI": "I", "KeyJ": "J",
  "KeyK": "K", "KeyL": "L", "KeyM": "M", "KeyN": "N", "KeyO": "O",
  "KeyP": "P", "KeyQ": "Q", "KeyR": "R", "KeyS": "S", "KeyT": "T",
  "KeyU": "U", "KeyV": "V", "KeyW": "W", "KeyX": "X", "KeyY": "Y",
  "KeyZ": "Z",
  "Digit0": "0", "Digit1": "1", "Digit2": "2", "Digit3": "3", "Digit4": "4",
  "Digit5": "5", "Digit6": "6", "Digit7": "7", "Digit8": "8", "Digit9": "9",
  "Space": "Space", "Tab": "Tab", "Enter": "Enter",
};

// State
let currentPttKey = "AltLeft";
let isRecording = false;
let recordingPhase = null; // null | "modifier" | "combo"
let capturedModifier = null;
let recordingMode = "hold";
let dictTags = [];
let dictTagsSnapshot = [];
let undoTimer = null;
let undoEntry = null; // { term, index }

const el = (id) => document.getElementById(id);

function displayNameFor(code) {
  if (code && code.includes("+")) {
    const [mod, key] = code.split("+");
    const modName = KEY_MAP[mod] || LEGACY_DISPLAY[mod] || mod;
    const keyName = REGULAR_KEY_MAP[key] || key;
    return modName + " + " + keyName;
  }
  return KEY_MAP[code] || LEGACY_DISPLAY[code] || code;
}

function setPttKey(code) {
  currentPttKey = code;
  el("ptt-record").textContent = displayNameFor(code);
}

function startRecording() {
  isRecording = true;
  recordingPhase = "modifier";
  capturedModifier = null;
  invoke("pause_hotkey_listener").catch(() => {});
  const btn = el("ptt-record");
  btn.textContent = t("ptt.holdModifier");
  btn.classList.add("recording");
}

function stopRecording() {
  isRecording = false;
  recordingPhase = null;
  capturedModifier = null;
  invoke("resume_hotkey_listener").catch(() => {});
  const btn = el("ptt-record");
  btn.classList.remove("recording");
  btn.textContent = displayNameFor(currentPttKey);
}

function handleKeyDown(e) {
  if (!isRecording) return;
  e.preventDefault();
  e.stopPropagation();

  if (e.code === "Escape") {
    stopRecording();
    return;
  }

  if (recordingPhase === "modifier") {
    if (KEY_MAP[e.code]) {
      capturedModifier = e.code;
      recordingPhase = "combo";
      el("ptt-record").textContent = t("ptt.nowPressKey");
    }
  } else if (recordingPhase === "combo") {
    // Only accept keys that the backend maps (REGULAR_KEY_MAP)
    if (REGULAR_KEY_MAP[e.code]) {
      currentPttKey = capturedModifier + "+" + e.code;
      stopRecording();
    }
  }
}

function handleKeyUp(e) {
  if (!isRecording || recordingPhase !== "combo") return;
  // Modifier released without a regular key → store modifier only
  if (e.code === capturedModifier) {
    currentPttKey = capturedModifier;
    stopRecording();
  }
}

function setRecordingMode(mode) {
  recordingMode = mode;
  const btns = document.querySelectorAll("#recording-mode .seg-btn");
  btns.forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.value === mode);
  });
}

function updateEngineVisibility() {
  const isLocal = el("engine").value === "local";
  el("model-section").classList.toggle("hidden", !isLocal);
  el("groq-section").classList.toggle("hidden", isLocal);
}

function updateLlmProviderVisibility() {
  const provider = el("llm-provider").value;
  el("groq-llm-section").classList.toggle("hidden", provider !== "groq");
  el("ollama-section").classList.toggle("hidden", provider !== "ollama");
  el("custom-llm-section").classList.toggle("hidden", provider !== "custom");
}

function updateLlmVisibility() {
  const enabled = el("llm-enabled").checked;
  el("llm-settings").classList.toggle("hidden", !enabled);
  if (enabled) {
    updateLlmProviderVisibility();
  }
}

function showStatus(message, isError) {
  const status = el("save-status");
  status.textContent = message;
  status.className = "save-status" + (isError ? " error" : "");
  setTimeout(() => { status.textContent = ""; }, 2000);
}

// --- Dictionary Tag Management ---

function renderDictTags() {
  const container = el("dict-tags");
  container.innerHTML = "";
  dictTags.forEach((term, i) => {
    const tag = document.createElement("span");
    tag.className = "dict-tag";
    tag.textContent = term;

    const x = document.createElement("button");
    x.className = "dict-tag-x";
    x.textContent = "\u00d7";
    x.addEventListener("click", () => removeDictTag(i));

    tag.appendChild(x);
    container.appendChild(tag);
  });

  const count = el("dict-count");
  count.textContent = dictTags.length > 0 ? t("dict.nTerms").replace("{n}", dictTags.length) : "";
}

function addDictTag(term) {
  const cleaned = term.trim();
  if (!cleaned) return;
  if (dictTags.includes(cleaned)) return; // no duplicates
  dictTags.push(cleaned);
  renderDictTags();
}

function removeDictTag(index) {
  const removed = dictTags.splice(index, 1)[0];
  renderDictTags();
  showDictUndo(removed, index);
}

function showDictUndo(term, index) {
  if (undoTimer) {
    clearTimeout(undoTimer);
  }
  undoEntry = { term, index };
  const container = el("dict-undo");
  el("dict-undo-text").textContent = t("dict.removedTerm").replace("{term}", term);
  container.classList.remove("hidden");
  undoTimer = setTimeout(() => {
    container.classList.add("hidden");
    undoEntry = null;
    undoTimer = null;
  }, 4000);
}

function doDictUndo() {
  if (!undoEntry) return;
  const { term, index } = undoEntry;
  const pos = Math.min(index, dictTags.length);
  dictTags.splice(pos, 0, term);
  renderDictTags();
  if (undoTimer) clearTimeout(undoTimer);
  el("dict-undo").classList.add("hidden");
  undoEntry = null;
  undoTimer = null;
}

function getDictString() {
  return dictTags.join(", ");
}

function loadDictFromString(str) {
  dictTags = (str || "")
    .split(",")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
  renderDictTags();
}

// --- Init ---

window.addEventListener("DOMContentLoaded", async () => {
  // Load settings
  try {
    const s = await invoke("get_settings");
    setPttKey(s.ptt_key);
    el("language").value = s.language;
    el("engine").value = s.engine;
    el("model").value = s.model;
    el("groq-api-key").value = s.groq_api_key;
    el("opacity").value = s.window_opacity;
    el("opacity-value").textContent = Math.round(s.window_opacity * 100) + "%";
    el("auto-start").checked = s.auto_start;
    setRecordingMode(s.recording_mode || "hold");
    loadDictFromString(s.dictionary || "");
    dictTagsSnapshot = [...dictTags];
    el("llm-enabled").checked = s.llm_enabled || false;
    el("llm-model").value = s.llm_model || "llama-3.3-70b-versatile";
    el("app-aware-style").checked = s.app_aware_style !== false;
    el("llm-provider").value = s.llm_provider || "groq";
    el("ollama-url").value = s.ollama_url || "http://localhost:11434";
    el("ollama-model").value = s.ollama_model || "llama3.2";
    el("custom-llm-url").value = s.custom_llm_url || "";
    el("custom-llm-key").value = s.custom_llm_key || "";
    el("custom-llm-model").value = s.custom_llm_model || "";
    updateEngineVisibility();
    updateLlmVisibility();
    // Apply locale
    el("ui-locale").value = s.ui_locale || "en";
    applyLocale(s.ui_locale || "en");
  } catch (e) {
    showStatus(t("status.loadFailed"), true);
  }

  // Record button
  el("ptt-record").addEventListener("click", () => {
    if (isRecording) {
      stopRecording();
    } else {
      startRecording();
    }
  });

  // Global keydown/keyup for recording
  document.addEventListener("keydown", handleKeyDown);
  document.addEventListener("keyup", handleKeyUp);

  // Engine toggle
  el("engine").addEventListener("change", updateEngineVisibility);

  // LLM toggle
  el("llm-enabled").addEventListener("change", updateLlmVisibility);

  // LLM provider
  el("llm-provider").addEventListener("change", updateLlmProviderVisibility);

  // UI locale
  el("ui-locale").addEventListener("change", () => {
    applyLocale(el("ui-locale").value);
  });

  // Recording mode segmented control
  document.querySelectorAll("#recording-mode .seg-btn").forEach((btn) => {
    btn.addEventListener("click", () => setRecordingMode(btn.dataset.value));
  });

  // Dictionary input — Enter or comma to add tag
  el("dict-input").addEventListener("keydown", (e) => {
    if (e.code === "Enter" || e.key === ",") {
      e.preventDefault();
      const input = el("dict-input");
      addDictTag(input.value.replace(",", ""));
      input.value = "";
    }
    // Backspace on empty input removes last tag
    if (e.code === "Backspace" && el("dict-input").value === "" && dictTags.length > 0) {
      removeDictTag(dictTags.length - 1);
    }
  });

  // Also add on blur (if user typed something and clicked away)
  el("dict-input").addEventListener("blur", () => {
    const input = el("dict-input");
    if (input.value.trim()) {
      addDictTag(input.value);
      input.value = "";
    }
  });

  // Undo button
  el("dict-undo-btn").addEventListener("click", doDictUndo);

  // Opacity slider
  el("opacity").addEventListener("input", () => {
    el("opacity-value").textContent = Math.round(el("opacity").value * 100) + "%";
  });

  // Save
  el("btn-save").addEventListener("click", async () => {
    const newSettings = {
      ptt_key: currentPttKey,
      language: el("language").value,
      engine: el("engine").value,
      model: el("model").value,
      groq_api_key: el("groq-api-key").value,
      window_opacity: parseFloat(el("opacity").value),
      auto_start: el("auto-start").checked,
      recording_mode: recordingMode,
      dictionary: getDictString(),
      llm_enabled: el("llm-enabled").checked,
      llm_model: el("llm-model").value,
      app_aware_style: el("app-aware-style").checked,
      llm_provider: el("llm-provider").value,
      ollama_url: el("ollama-url").value,
      ollama_model: el("ollama-model").value,
      custom_llm_url: el("custom-llm-url").value,
      custom_llm_key: el("custom-llm-key").value,
      custom_llm_model: el("custom-llm-model").value,
      ui_locale: el("ui-locale").value,
    };

    try {
      await invoke("save_settings", { newSettings });
      showStatus(t("status.saved"));
      setTimeout(() => getCurrentWindow().close(), 500);
    } catch (e) {
      showStatus(t("status.saveFailed").replace("{err}", e), true);
    }
  });

  // Check for Updates
  el("btn-check-update").addEventListener("click", async () => {
    const btn = el("btn-check-update");
    btn.textContent = t("update.checking");
    btn.disabled = true;
    try {
      const result = await invoke("check_for_updates");
      if (result.up_to_date) {
        btn.textContent = t("update.upToDate").replace("{version}", result.current_version);
        setTimeout(() => { btn.textContent = t("btn.checkUpdate"); btn.disabled = false; }, 3000);
      } else {
        btn.textContent = t("update.available").replace("{version}", result.latest_version);
        btn.classList.add("update-available");
        btn.disabled = false;
        btn.onclick = async () => {
          await invoke("open_url", { url: result.release_url });
        };
      }
    } catch (e) {
      btn.textContent = t("update.failed");
      btn.disabled = false;
      setTimeout(() => { btn.textContent = t("btn.checkUpdate"); }, 3000);
    }
  });

  // Changelog
  el("btn-changelog").addEventListener("click", async () => {
    await invoke("open_url", { url: "https://github.com/panda850819/murmur-voice/releases" });
  });

  // Roadmap
  el("btn-roadmap").addEventListener("click", async () => {
    await invoke("open_url", { url: "https://github.com/panda850819/murmur-voice/blob/main/ROADMAP.md" });
  });

  // Cancel
  el("btn-cancel").addEventListener("click", () => {
    dictTags = [...dictTagsSnapshot];
    renderDictTags();
    getCurrentWindow().close();
  });
});
