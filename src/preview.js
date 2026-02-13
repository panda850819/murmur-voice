const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const headerText = () => document.getElementById("header-text");
const previewText = () => document.getElementById("preview-text");
const charCount = () => document.getElementById("char-count");
const appBadge = () => document.getElementById("app-badge");
const previewBody = () => document.getElementById("preview-body");

let autoHideTimer = null;
let dotsInterval = null;

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
  el.textContent = text.length + " chars";
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

function reset() {
  clearAutoHide();
  if (dotsInterval) {
    clearInterval(dotsInterval);
    dotsInterval = null;
  }
  setHeader("Listening...", false);
  setText("Listening...", "placeholder");
  setCharCount("");
  setAppBadge(null);
}

function scrollToBottom() {
  const body = previewBody();
  body.scrollTop = body.scrollHeight;
}

window.addEventListener("DOMContentLoaded", async () => {
  await listen("recording_state_changed", (event) => {
    const state = event.payload;
    switch (state) {
      case "starting":
        reset();
        break;
      case "recording":
        setHeader("Listening...", false);
        break;
      case "stopping":
        setHeader("Stopping...", true);
        break;
      case "transcribing":
        setHeader("Transcribing...", true);
        setText("", null);
        break;
      case "processing":
        setHeader("Processing...", true);
        break;
      case "idle":
        // Auto-hide is handled by the result display timer.
        // If we get idle without a result, just keep current state.
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
    const text = event.payload;
    clearAutoHide();

    if (!text || text.trim().length === 0) {
      setHeader("Done", false);
      setText("No speech detected", "no-speech");
      setCharCount("");
    } else {
      setHeader("Done", false);
      setText(text, null);
      setCharCount(text);
      scrollToBottom();
    }

    // Auto-hide after 3 seconds (minimum 1s display guaranteed by show timing)
    autoHideTimer = setTimeout(() => {
      invoke("hide_preview").catch(() => {});
    }, 3000);
  });

  await listen("foreground_app_info", (event) => {
    const { name } = event.payload;
    if (name && name !== "Unknown") {
      setAppBadge(name);
    }
  });
});
