const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let statusDot;
let statusText;
let transcription;
let progressContainer;
let progressBar;
let appBadge;
let appEl;
let isRecording = false;
let isCollapsing = false;
let recordingMaxHeight = 0;
let resizeDebounceTimer = null;

function setStatus(state, text) {
  const dot = statusDot;
  dot.className = "status-dot";
  if (state) {
    dot.classList.add(state);
  }
  statusText.textContent = text;
}

function expandMainBar() {
  isRecording = true;
  isCollapsing = false;
  recordingMaxHeight = 0;
  appEl.classList.remove("collapsing");
  appEl.classList.add("expanded");
  transcription.classList.add("multiline");
}

function collapseMainBar() {
  isRecording = false;
  isCollapsing = true;
  appEl.classList.remove("expanded");
  transcription.classList.remove("multiline");
  transcription.textContent = "";
  appEl.classList.add("collapsing");
  // Directly invoke resize to 48px (bypass ResizeObserver)
  invoke(COMMANDS.RESIZE_MAIN_WINDOW, { height: 48 });
  // Clear collapsing flag after transition
  appEl.addEventListener("transitionend", () => {
    isCollapsing = false;
    appEl.classList.remove("collapsing");
  }, { once: true });
  // Fallback: clear flag after 200ms if transitionend doesn't fire
  setTimeout(() => {
    isCollapsing = false;
    appEl.classList.remove("collapsing");
  }, 200);
}

function resetMainBar() {
  // Immediate reset without animation (error/cancel)
  isRecording = false;
  isCollapsing = false;
  recordingMaxHeight = 0;
  appEl.classList.remove("expanded", "collapsing");
  transcription.classList.remove("multiline");
  invoke(COMMANDS.RESIZE_MAIN_WINDOW, { height: 48 });
}

window.addEventListener("DOMContentLoaded", async () => {
  statusDot = document.getElementById("status-dot");
  statusText = document.getElementById("status-text");
  transcription = document.getElementById("transcription");
  progressContainer = document.getElementById("progress-container");
  progressBar = document.getElementById("progress-bar");
  appBadge = document.getElementById("app-badge");
  appEl = document.getElementById("app");

  // ResizeObserver: notify backend when #app height changes during recording
  const resizeObserver = new ResizeObserver((entries) => {
    if (isCollapsing) return;
    if (!isRecording) return;
    const entry = entries[0];
    const newHeight = entry.contentBoxSize?.[0]?.blockSize
      ?? entry.contentRect.height;
    // Add margin (4px top + 4px bottom = 8px)
    const windowHeight = newHeight + 8;
    // Expand-only: never shrink during recording
    if (windowHeight <= recordingMaxHeight) return;
    recordingMaxHeight = windowHeight;
    // Debounce backend calls
    clearTimeout(resizeDebounceTimer);
    resizeDebounceTimer = setTimeout(() => {
      invoke(COMMANDS.RESIZE_MAIN_WINDOW, { height: windowHeight });
    }, 50);
  });
  resizeObserver.observe(appEl);

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
        if (isRecording || appEl.classList.contains("expanded")) {
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
