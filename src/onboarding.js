const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

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

function pttDisplayName(code) {
  if (code && code.includes("+")) {
    const [mod, key] = code.split("+");
    const modName = KEY_MAP[mod] || mod;
    const keyName = REGULAR_KEY_MAP[key] || key;
    return modName + " + " + keyName;
  }
  return KEY_MAP[code] || code;
}

let currentStep = 1;
let pttKey = "AltLeft";
let isRecording = false;
let recordingPhase = null; // null | "modifier" | "combo"
let capturedModifier = null;
let chosenLocale = "en";
let chosenEngine = "local";

const el = (id) => document.getElementById(id);

function goToStep(n) {
  document.querySelectorAll(".step").forEach((s) => s.classList.remove("active"));
  document.querySelectorAll(".dot").forEach((d) => d.classList.remove("active"));

  const step = document.querySelector(`.step[data-step="${n}"]`);
  const dot = document.querySelector(`.dot[data-dot="${n}"]`);
  if (step) step.classList.add("active");
  if (dot) dot.classList.add("active");
  currentStep = n;
}

function updateDots() {
  const dot4 = document.querySelector('.dot[data-dot="4"]');
  if (dot4) {
    dot4.classList.toggle("hidden", chosenEngine === "groq");
  }
}

function startPttRecording() {
  isRecording = true;
  recordingPhase = "modifier";
  capturedModifier = null;
  invoke("pause_hotkey_listener").catch(() => {});
  const btn = el("onboard-ptt-record");
  btn.textContent = t("ptt.holdModifier");
  btn.classList.add("recording");
}

function stopPttRecording() {
  isRecording = false;
  recordingPhase = null;
  capturedModifier = null;
  invoke("resume_hotkey_listener").catch(() => {});
  const btn = el("onboard-ptt-record");
  btn.classList.remove("recording");
  btn.textContent = pttDisplayName(pttKey);
}

function updateEngineNext() {
  const btn = el("btn-engine-next");
  if (chosenEngine === "groq") {
    btn.disabled = !el("onboard-groq-key").value.trim();
  } else {
    btn.disabled = false;
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  // Auto-detect locale from system language
  const systemLang = navigator.language || "en";
  if (systemLang.startsWith("zh")) {
    chosenLocale = "zh-TW";
    document.querySelectorAll(".locale-btn").forEach((b) => {
      b.classList.toggle("active", b.dataset.locale === "zh-TW");
    });
  }
  applyLocale(chosenLocale);

  // Locale picker buttons
  document.querySelectorAll(".locale-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      chosenLocale = btn.dataset.locale;
      document.querySelectorAll(".locale-btn").forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      applyLocale(chosenLocale);
    });
  });

  // Next buttons (static data-next)
  document.querySelectorAll(".next-btn[data-next]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const next = parseInt(btn.dataset.next, 10);
      goToStep(next);
    });
  });

  // Back buttons (static data-back)
  document.querySelectorAll(".back-btn[data-back]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const back = parseInt(btn.dataset.back, 10);
      goToStep(back);
    });
  });

  // Poll permission statuses
  let micGranted = false;
  let accGranted = false;

  async function checkMicStatus() {
    try {
      micGranted = await invoke("check_microphone");
    } catch (_) {
      micGranted = false;
    }
    const badge = el("mic-status");
    if (badge) {
      badge.textContent = micGranted ? "\u2705" : "\u274C";
    }
  }

  async function checkAccStatus() {
    try {
      accGranted = await invoke("check_accessibility");
    } catch (_) {
      accGranted = false;
    }
    const badge = el("acc-status");
    if (badge) {
      badge.textContent = accGranted ? "\u2705" : "\u274C";
    }
  }

  // Trigger macOS mic permission dialog (no-op if already granted/denied)
  invoke("request_microphone").catch(() => {});

  await checkMicStatus();
  await checkAccStatus();
  const permPoll = setInterval(async () => {
    if (!micGranted) await checkMicStatus();
    if (!accGranted) await checkAccStatus();
    if (micGranted && accGranted) clearInterval(permPoll);
  }, 3000);

  // Step 3: Engine choice
  document.querySelectorAll(".engine-card").forEach((card) => {
    card.addEventListener("click", () => {
      document.querySelectorAll(".engine-card").forEach((c) => c.classList.remove("active"));
      card.classList.add("active");
      chosenEngine = card.dataset.engine;
      el("groq-key-wrap").classList.toggle("hidden", chosenEngine !== "groq");
      updateEngineNext();
      updateDots();
    });
  });

  el("onboard-groq-key").addEventListener("input", updateEngineNext);

  // Engine next button (dynamic: local -> step 4, groq -> step 5)
  el("btn-engine-next").addEventListener("click", () => {
    goToStep(chosenEngine === "local" ? 4 : 5);
  });

  // Hotkey back button (dynamic: local -> step 4, groq -> step 3)
  el("btn-hotkey-back").addEventListener("click", () => {
    goToStep(chosenEngine === "local" ? 4 : 3);
  });

  // Step 4: Model download
  const modelReady = await invoke("is_model_ready");
  if (modelReady) {
    el("btn-download").classList.add("hidden");
    el("btn-step4-next").classList.remove("hidden");
  }

  el("btn-download").addEventListener("click", async () => {
    el("btn-download").disabled = true;
    el("btn-download").textContent = t("onboard.downloading");
    el("model-progress-wrap").classList.remove("hidden");

    try {
      await invoke("download_model_cmd");
      el("model-progress-wrap").classList.add("hidden");
      el("btn-download").classList.add("hidden");
      el("btn-step4-next").classList.remove("hidden");
    } catch (e) {
      el("btn-download").disabled = false;
      el("btn-download").textContent = t("onboard.retryDownload");
    }
  });

  await listen(EVENTS.MODEL_DOWNLOAD_PROGRESS, (event) => {
    const { downloaded, total } = event.payload;
    if (total > 0) {
      const pct = Math.round((downloaded / total) * 100);
      el("model-progress-bar").style.width = pct + "%";
      el("model-progress-text").textContent = pct + "%";
    }
  });

  // Step 5: PTT key recording
  el("onboard-ptt-record").addEventListener("click", () => {
    if (isRecording) {
      stopPttRecording();
    } else {
      startPttRecording();
    }
  });

  document.addEventListener("keydown", (e) => {
    if (!isRecording) return;
    e.preventDefault();
    e.stopPropagation();

    if (e.code === "Escape") {
      stopPttRecording();
      return;
    }

    if (recordingPhase === "modifier") {
      if (KEY_MAP[e.code]) {
        capturedModifier = e.code;
        recordingPhase = "combo";
        el("onboard-ptt-record").textContent = t("ptt.nowPressKey");
      }
    } else if (recordingPhase === "combo") {
      if (REGULAR_KEY_MAP[e.code]) {
        pttKey = capturedModifier + "+" + e.code;
        stopPttRecording();
      }
    }
  });

  document.addEventListener("keyup", (e) => {
    if (!isRecording || recordingPhase !== "combo") return;
    if (e.code === capturedModifier) {
      pttKey = capturedModifier;
      stopPttRecording();
    }
  });

  // Done button — save PTT key + locale + engine + groq key
  el("btn-done").addEventListener("click", async () => {
    el("btn-done").disabled = true;
    try {
      const settings = await invoke("get_settings");
      settings.ptt_key = pttKey;
      settings.ui_locale = chosenLocale;
      settings.engine = chosenEngine;
      if (chosenEngine === "groq") {
        settings.groq_api_key = el("onboard-groq-key").value.trim();
      }
      settings.onboarding_complete = true;
      await invoke("save_settings", { newSettings: settings });
      await invoke("complete_onboarding");
    } catch (e) {
      el("btn-done").disabled = false;
    }
  });
});
