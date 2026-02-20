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
  await listen("model_download_progress", (event) => {
    const { downloaded, total } = event.payload;
    const pct = total > 0 ? (downloaded / total) * 100 : 0;
    progressContainer.style.display = "block";
    progressBar.style.width = pct + "%";
    setStatus(null, t("state.downloadingModel").replace("{pct}", Math.round(pct)));
  });

  await listen("model_ready", () => {
    progressContainer.style.display = "none";
    progressBar.style.width = "0%";
    setStatus(null, t("state.ready"));
  });

  await listen("recording_state_changed", (event) => {
    const state = event.payload;
    switch (state) {
      case "starting":
        setStatus("recording", t("state.starting"));
        transcription.textContent = "";
        break;
      case "recording":
        setStatus("recording", t("state.listening"));
        break;
      case "stopping":
        setStatus("transcribing", t("state.stopping"));
        break;
      case "transcribing":
        setStatus("transcribing", t("state.transcribing"));
        break;
      case "processing":
        setStatus("transcribing", t("state.processing"));
        break;
      case "idle":
        setStatus(null, t("state.ready"));
        appBadge.classList.remove("visible");
        appBadge.textContent = "";
        break;
    }
  });

  // Live transcription updates while recording
  await listen("partial_transcription", (event) => {
    transcription.textContent = event.payload;
  });

  await listen("transcription_complete", (event) => {
    const { text } = event.payload;
    transcription.textContent = text || "";
    setStatus("done", t("state.done"));

    setTimeout(() => {
      setStatus(null, t("state.ready"));
    }, 2000);
  });

  await listen("foreground_app_info", (event) => {
    const { name } = event.payload;
    const badge = appBadge;
    if (name && name !== "Unknown") {
      badge.textContent = name;
      badge.classList.add("visible");
    }
  });

  await listen("opacity_changed", (event) => {
    document.getElementById("app").style.background =
      `rgba(20, 20, 30, ${event.payload})`;
  });

  await listen("accessibility_error", () => {
    setStatus("error", t("state.accessibilityError"));
    transcription().textContent = t("state.accessibilityHint");
  });

  await listen("accessibility_granted", () => {
    setStatus(null, t("state.ready"));
    transcription().textContent = "";
  });

  await listen("recording_error", (event) => {
    const errorMsg = event.payload;
    transcription.textContent = errorMsg;
    setStatus("error", t("state.error"));

    setTimeout(() => {
      setStatus(null, t("state.ready"));
      transcription.textContent = "";
    }, 3000);
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
    progressContainer.style.display = "block";
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
      transcription().textContent = t("state.accessibilityHint");
    }
  } catch (_) {}
});
