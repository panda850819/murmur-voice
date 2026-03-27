const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let statusDot;
let statusText;
let transcription;
let progressContainer;
let progressBar;
let appBadge;
let appEl;
let isCollapsing = false;
let recordingMaxHeight = 0;
const MAIN_BAR_HEIGHT = 48;
const MAIN_BAR_MARGIN = 8;

function setStatus(state, text) {
  const dot = statusDot;
  dot.className = "status-dot";
  if (state) {
    dot.classList.add(state);
  }
  statusText.textContent = text;
}

function expandMainBar() {
  isCollapsing = false;
  recordingMaxHeight = 0;
  appEl.classList.remove("collapsing");
  appEl.classList.add("expanded");
  transcription.classList.add("multiline");
}

function maybeExpandToFitContent() {
  if (!appEl.classList.contains("expanded")) return;
  const neededHeight = appEl.scrollHeight + MAIN_BAR_MARGIN;
  if (neededHeight > recordingMaxHeight) {
    recordingMaxHeight = neededHeight;
    invoke(COMMANDS.RESIZE_MAIN_WINDOW, { height: neededHeight });
  }
}

function collapseMainBar() {
  if (isCollapsing) return;
  isCollapsing = true;
  appEl.classList.remove("expanded");
  transcription.classList.remove("multiline");
  transcription.textContent = "";
  appEl.classList.add("collapsing");
  invoke(COMMANDS.RESIZE_MAIN_WINDOW, { height: MAIN_BAR_HEIGHT });
  appEl.addEventListener("transitionend", () => {
    isCollapsing = false;
    appEl.classList.remove("collapsing");
  }, { once: true });
  // Fallback if transitionend doesn't fire (element hidden or transition cancelled)
  setTimeout(() => {
    isCollapsing = false;
    appEl.classList.remove("collapsing");
  }, 170);
}

function resetMainBar() {
  isCollapsing = false;
  recordingMaxHeight = 0;
  appEl.classList.remove("expanded", "collapsing");
  transcription.classList.remove("multiline");
  invoke(COMMANDS.RESIZE_MAIN_WINDOW, { height: MAIN_BAR_HEIGHT });
}

window.addEventListener("DOMContentLoaded", async () => {
  statusDot = document.getElementById("status-dot");
  statusText = document.getElementById("status-text");
  transcription = document.getElementById("transcription");
  progressContainer = document.getElementById("progress-container");
  progressBar = document.getElementById("progress-bar");
  appBadge = document.getElementById("app-badge");
  appEl = document.getElementById("app");

  // Load locale
  try {
    const s = await invoke(COMMANDS.GET_SETTINGS);
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
        expandMainBar();
        break;
      case RECORDING_STATES.RECORDING:
        setStatus("recording", t("state.listening"));
        break;
      case RECORDING_STATES.STOPPING:
        setStatus("transcribing", t("state.stopping"));
        break;
      case RECORDING_STATES.TRANSCRIBING:
        setStatus("transcribing", t("state.transcribing"));
        // Stay expanded during transcription
        break;
      case RECORDING_STATES.PROCESSING:
        setStatus("transcribing", t("state.processing"));
        break;
      case RECORDING_STATES.TRANSLATING:
        setStatus("transcribing", t("state.translating"));
        transcription.textContent = "";
        break;
      case RECORDING_STATES.IDLE:
        setStatus(null, t("state.ready"));
        appBadge.classList.remove("visible");
        appBadge.textContent = "";
        if (appEl.classList.contains("expanded")) {
          collapseMainBar();
        }
        break;
      case "downloading_model":
        progressContainer.classList.remove("hidden");
        setStatus(null, t("state.downloadingModel").replace(" {pct}%", ""));
        break;
    }
  });

  // Live transcription updates while recording
  await listen(EVENTS.PARTIAL_TRANSCRIPTION, (event) => {
    transcription.textContent = event.payload;
    maybeExpandToFitContent();
  });

  await listen(EVENTS.TRANSCRIPTION_COMPLETE, (event) => {
    const { text } = event.payload;
    if (appEl.classList.contains("expanded")) {
      collapseMainBar();
    }
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
    transcription.style.cursor = "pointer";
    transcription.onclick = () => {
      invoke(COMMANDS.OPEN_URL, { url: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility" }).catch(() => {});
    };
  });

  await listen(EVENTS.ACCESSIBILITY_GRANTED, () => {
    setStatus(null, t("state.ready"));
    transcription.textContent = "";
    transcription.style.cursor = "";
    transcription.onclick = null;
  });

  await listen(EVENTS.RECORDING_ERROR, (event) => {
    const errorMsg = event.payload;
    resetMainBar();
    transcription.textContent = errorMsg;
    setStatus("error", t("state.error"));

    setTimeout(() => {
      setStatus(null, t("state.ready"));
      transcription.textContent = "";
    }, 3000);
  });

  await listen(EVENTS.RECORDING_CANCELLED, () => {
    resetMainBar();
    setStatus("error", t("state.cancelled"));

    setTimeout(() => {
      setStatus(null, t("state.ready"));
    }, 2000);
  });


  await listen(EVENTS.RECORDING_MODE_INFO, (event) => {
    const mode = event.payload;
    const modeKey = 'mode.' + mode;
    const modeText = t(modeKey);
    if (modeText && modeText !== modeKey) {
      transcription.textContent = modeText;
    }
  });
  // Right-click to open settings
  document.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    invoke(COMMANDS.OPEN_SETTINGS);
  });

  // Model download is deferred — triggered on first recording attempt for local engine.
  // Just show Ready status here.
  setStatus(null, t("state.ready"));

  // Check Accessibility permission (macOS: required for hotkey)
  try {
    const accessible = await invoke(COMMANDS.CHECK_ACCESSIBILITY);
    if (!accessible) {
      setStatus("error", t("state.accessibilityError"));
      transcription.textContent = t("state.accessibilityHint");
      transcription.style.cursor = "pointer";
      transcription.onclick = () => {
        invoke(COMMANDS.OPEN_URL, { url: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility" }).catch(() => {});
      };
    }
  } catch (_) {}
});
