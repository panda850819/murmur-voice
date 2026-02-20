const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let statusDot;
let statusText;
let transcription;
let progressContainer;
let progressBar;
let appBadge;

function setStatus(state, text) {
  const dot = statusDot;
  dot.className = "status-dot";
  if (state) {
    dot.classList.add(state);
  }
  statusText.textContent = text;
}

window.addEventListener("DOMContentLoaded", async () => {
  statusDot = document.getElementById("status-dot");
  statusText = document.getElementById("status-text");
  transcription = document.getElementById("transcription");
  progressContainer = document.getElementById("progress-container");
  progressBar = document.getElementById("progress-bar");
  appBadge = document.getElementById("app-badge");

  // Load locale
  try {
    const s = await invoke("get_settings");
    currentLocale = s.ui_locale || "en";
  } catch (_) {}

  // Register ALL event listeners FIRST, before triggering any commands.
  await listen(EVENTS.MODEL_DOWNLOAD_PROGRESS, (event) => {
    const { downloaded, total } = event.payload;
    const pct = total > 0 ? (downloaded / total) * 100 : 0;
    progressContainer.classList.remove("hidden");
    progressBar.style.width = pct + "%";
    setStatus(null, t("state.downloadingModel").replace("{pct}", Math.round(pct)));
  });

  await listen(EVENTS.MODEL_READY, () => {
    progressContainer.classList.add("hidden");
    progressBar.style.width = "0%";
    setStatus(null, t("state.ready"));
  });

  await listen(EVENTS.RECORDING_STATE_CHANGED, (event) => {
    const state = event.payload;
    switch (state) {
      case RECORDING_STATES.STARTING:
        setStatus("recording", t("state.starting"));
        transcription.textContent = "";
        break;
      case RECORDING_STATES.RECORDING:
        setStatus("recording", t("state.listening"));
        break;
      case RECORDING_STATES.STOPPING:
        setStatus("transcribing", t("state.stopping"));
        break;
      case RECORDING_STATES.TRANSCRIBING:
        setStatus("transcribing", t("state.transcribing"));
        break;
      case RECORDING_STATES.PROCESSING:
        setStatus("transcribing", t("state.processing"));
        break;
      case RECORDING_STATES.IDLE:
        setStatus(null, t("state.ready"));
        appBadge.classList.remove("visible");
        appBadge.textContent = "";
        break;
    }
  });

  // Live transcription updates while recording
  await listen(EVENTS.PARTIAL_TRANSCRIPTION, (event) => {
    transcription.textContent = event.payload;
  });

  await listen(EVENTS.TRANSCRIPTION_COMPLETE, (event) => {
    const { text } = event.payload;
    transcription.textContent = text || "";
    setStatus("done", t("state.done"));

    setTimeout(() => {
      setStatus(null, t("state.ready"));
    }, 2000);
  });

  await listen(EVENTS.FOREGROUND_APP_INFO, (event) => {
    const { name } = event.payload;
    const badge = appBadge;
    if (name && name !== "Unknown") {
      badge.textContent = name;
      badge.classList.add("visible");
    }
  });

  await listen(EVENTS.OPACITY_CHANGED, (event) => {
    document.getElementById("app").style.background =
      `rgba(20, 20, 30, ${event.payload})`;
  });

  await listen(EVENTS.ACCESSIBILITY_ERROR, () => {
    setStatus("error", t("state.accessibilityError"));
    transcription.textContent = t("state.accessibilityHint");
  });

  await listen(EVENTS.ACCESSIBILITY_GRANTED, () => {
    setStatus(null, t("state.ready"));
    transcription.textContent = "";
  });

  await listen(EVENTS.RECORDING_ERROR, (event) => {
    const errorMsg = event.payload;
    transcription.textContent = errorMsg;
    setStatus("error", t("state.error"));

    setTimeout(() => {
      setStatus(null, t("state.ready"));
      transcription.textContent = "";
    }, 3000);
  });

  await listen(EVENTS.RECORDING_CANCELLED, () => {
    setStatus("error", t("state.cancelled"));
    transcription.textContent = "";

    setTimeout(() => {
      setStatus(null, t("state.ready"));
    }, 2000);
  });

  // Right-click to open settings
  document.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    invoke("open_settings");
  });

  // Now check model status and trigger download if needed.
  let modelReady = false;
  try {
    modelReady = await invoke("is_model_ready");
  } catch (_) {}

  if (!modelReady) {
    progressContainer.classList.remove("hidden");
    setStatus(null, t("state.downloadingModel").replace(" {pct}%", ""));
    // Fire and forget — progress updates come via events above.
    invoke("download_model_cmd").catch((err) => {
      setStatus("error", t("state.downloadFailed"));
      transcription.textContent = String(err);
    });
  } else {
    setStatus(null, t("state.ready"));
  }

  // Check Accessibility permission (macOS: required for hotkey)
  try {
    const accessible = await invoke("check_accessibility");
    if (!accessible) {
      setStatus("error", t("state.accessibilityError"));
      transcription.textContent = t("state.accessibilityHint");
    }
  } catch (_) {}
});
