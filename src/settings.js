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

// State
let currentPttKey = "AltLeft";
let isRecording = false;

const el = (id) => document.getElementById(id);

function displayNameFor(code) {
  return KEY_MAP[code] || LEGACY_DISPLAY[code] || code;
}

function setPttKey(code) {
  currentPttKey = code;
  el("ptt-record").textContent = displayNameFor(code);
}

function startRecording() {
  isRecording = true;
  const btn = el("ptt-record");
  btn.textContent = "Press a key...";
  btn.classList.add("recording");
}

function stopRecording() {
  isRecording = false;
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

  if (KEY_MAP[e.code]) {
    currentPttKey = e.code;
    stopRecording();
  }
}

function updateEngineVisibility() {
  const isLocal = el("engine").value === "local";
  el("model-section").style.display = isLocal ? "flex" : "none";
  el("groq-section").style.display = isLocal ? "none" : "flex";
}

function showStatus(message, isError) {
  const status = el("save-status");
  status.textContent = message;
  status.className = "save-status" + (isError ? " error" : "");
  setTimeout(() => { status.textContent = ""; }, 2000);
}

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
    updateEngineVisibility();
  } catch (e) {
    showStatus("Failed to load settings", true);
  }

  // Record button
  el("ptt-record").addEventListener("click", () => {
    if (isRecording) {
      stopRecording();
    } else {
      startRecording();
    }
  });

  // Global keydown for recording
  document.addEventListener("keydown", handleKeyDown);

  // Engine toggle
  el("engine").addEventListener("change", updateEngineVisibility);

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
    };

    try {
      await invoke("save_settings", { newSettings });
      showStatus("Saved");
      setTimeout(() => getCurrentWindow().close(), 500);
    } catch (e) {
      showStatus("Failed: " + e, true);
    }
  });

  // Cancel
  el("btn-cancel").addEventListener("click", () => getCurrentWindow().close());
});
