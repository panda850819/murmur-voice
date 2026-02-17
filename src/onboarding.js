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

let currentStep = 1;
let pttKey = "AltLeft";
let isRecording = false;
let chosenLocale = "en";

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

function startPttRecording() {
  isRecording = true;
  const btn = el("onboard-ptt-record");
  btn.textContent = t("ptt.pressKey");
  btn.classList.add("recording");
}

function stopPttRecording() {
  isRecording = false;
  const btn = el("onboard-ptt-record");
  btn.classList.remove("recording");
  btn.textContent = KEY_MAP[pttKey] || pttKey;
}

window.addEventListener("DOMContentLoaded", async () => {
  // Locale picker buttons
  document.querySelectorAll(".locale-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      chosenLocale = btn.dataset.locale;
      document.querySelectorAll(".locale-btn").forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      applyLocale(chosenLocale);
    });
  });

  // Next buttons
  document.querySelectorAll(".next-btn[data-next]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const next = parseInt(btn.dataset.next, 10);
      goToStep(next);
    });
  });

  // Back buttons
  document.querySelectorAll(".back-btn[data-back]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const back = parseInt(btn.dataset.back, 10);
      goToStep(back);
    });
  });

  // Step 3: Model download
  const modelReady = await invoke("is_model_ready");
  if (modelReady) {
    el("btn-download").style.display = "none";
    el("btn-step3-next").style.display = "";
  }

  el("btn-download").addEventListener("click", async () => {
    el("btn-download").disabled = true;
    el("btn-download").textContent = t("onboard.downloading");
    el("model-progress-wrap").style.display = "flex";

    try {
      await invoke("download_model_cmd");
      el("model-progress-wrap").style.display = "none";
      el("btn-download").style.display = "none";
      el("btn-step3-next").style.display = "";
    } catch (e) {
      el("btn-download").disabled = false;
      el("btn-download").textContent = t("onboard.retryDownload");
    }
  });

  await listen("model_download_progress", (event) => {
    const { downloaded, total } = event.payload;
    if (total > 0) {
      const pct = Math.round((downloaded / total) * 100);
      el("model-progress-bar").style.width = pct + "%";
      el("model-progress-text").textContent = pct + "%";
    }
  });

  // Step 4: PTT key recording
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

    if (KEY_MAP[e.code]) {
      pttKey = e.code;
      stopPttRecording();
    }
  });

  // Done button — save PTT key + locale
  el("btn-done").addEventListener("click", async () => {
    el("btn-done").disabled = true;
    try {
      const settings = await invoke("get_settings");
      settings.ptt_key = pttKey;
      settings.ui_locale = chosenLocale;
      settings.onboarding_complete = true;
      await invoke("save_settings", { newSettings: settings });
      await invoke("complete_onboarding");
    } catch (e) {
      el("btn-done").disabled = false;
    }
  });
});
