# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Dev Commands

```bash
# Dev mode (hot reload frontend, Rust recompiles on change)
pnpm tauri dev

# Production build
pnpm tauri build

# Rust-only commands (run from src-tauri/)
cd src-tauri
cargo check                                       # fast iteration
cargo clippy --all-targets -- -D warnings          # lint (zero warnings policy)
cargo test                                         # 6 tests in state.rs
cargo test test_valid_forward_transitions          # run a single test
```

## Architecture

Tauri 2 desktop app: Rust backend (core process) + vanilla HTML/JS/CSS frontend (webview).

```
Frontend (src/)                    Backend (src-tauri/src/)
├── index.html      main window    ├── lib.rs        app setup, commands, tray
├── settings.html   preferences    ├── audio.rs      cpal mic capture → 16kHz mono
├── onboarding.html first-run      ├── whisper.rs    whisper.cpp via whisper-rs (Metal)
├── *.js            invoke/listen  ├── hotkey.rs     CGEventTap modifier key detection
└── *.css                          ├── clipboard.rs  arboard + rdev Cmd+V simulation
                                   ├── model.rs      HuggingFace model download
                                   ├── settings.rs   JSON persistence + key mapping
                                   ├── state.rs      recording state machine
                                   ├── llm.rs        Groq LLM API post-processing
                                   └── frontapp.rs   macOS foreground app detection (FFI)
```

**IPC**: Frontend calls backend via `invoke("command")` (uses `window.__TAURI__.core.invoke` via `withGlobalTauri`), backend pushes to frontend via `app.emit("event", payload)`.

**Three windows**: `main` (420x48, always-on-top bottom bar), `settings` (460x700, on-demand), `onboarding` (560x480, first-run only). All must be listed in `src-tauri/capabilities/default.json` to invoke Tauri commands.

**Frontend is static files** served directly from `src/` (no build step, no bundler). Plain HTML/JS/CSS.

## Recording Flow

```
Hotkey press → CGEventTap → channel → do_start_recording()
  → AudioRecorder::start() → spawn live transcription thread (local engine only, peek every 2s)
Hotkey release → do_stop_recording()
  → audio.stop()
  → if engine=groq: llm::transcribe_groq() (cloud Whisper API)
    else: whisper.transcribe() (local Metal GPU)
  → if llm_enabled: llm::process_text() (Groq LLM, cleans filler words/punctuation/繁簡)
  → clipboard.insert_text(Cmd+V) → emit result → hide window after 2s
```

State machine: `Idle → Starting → Recording → Stopping → Transcribing → [Processing] → Idle`

Two recording modes: **Hold** (press=start, release=stop) and **Toggle** (press toggles, 5-min auto-stop).

## Transcription Engines

- **Local**: whisper-rs with Metal GPU. Live preview enabled. Model stored at `~/Library/Application Support/com.murmur.voice/models/`.
- **Groq**: Cloud Whisper API (`whisper-large-v3-turbo`). Audio encoded to WAV via `hound`, sent as multipart form. No live preview (too expensive). Same `groq_api_key` used for both Whisper and LLM.

## Anti-Hallucination (Local Whisper)

- `MIN_SAMPLES = 16_000` (1s minimum, shorter clips produce hallucinations)
- Audio energy check (skip if near-silent)
- `suppress_blank(true)`, `no_speech_thold(0.6)`, `temperature_inc(0.0)`, `entropy_thold(2.4)`

## Key Patterns

- **Shared state**: `MurmurState` with `Mutex<T>` fields, injected via `.manage()`
- **Threading**: hotkey listener (CFRunLoop), audio capture (cpal callback), live transcription (std::thread), model download (tokio async)
- **Hotkey mask**: `AtomicU64` updated at runtime when settings change, no restart needed
- **Settings path**: `~/Library/Application Support/com.murmur.voice/settings.json`
- **Model path**: `~/Library/Application Support/com.murmur.voice/models/ggml-large-v3-turbo.bin`
- **PTT keys**: Both legacy format (`left_option`) and JS `event.code` format (`AltLeft`) accepted in `ptt_key_mask()`

## Gotchas

- **New windows** must be added to `src-tauri/capabilities/default.json` `"windows"` array or they can't invoke any Tauri commands
- **Groq API key** is shared between Whisper transcription and LLM post-processing — stored in `settings.groq_api_key`
- **`frontapp.rs` uses raw Objective-C FFI** (objc_msgSend) — no crate dependency, but `unsafe` throughout
- **Live transcription** only runs for local engine; Groq mode skips it entirely (cost)
- **Toggle mode** checks `app_state.current()` (not a local flag) to decide start/stop — this avoids desync after auto-stop timeout

## macOS Requirements

- Microphone permission (audio capture)
- Accessibility permission (CGEventTap for hotkey + rdev for Cmd+V paste)
- Apple Silicon recommended (Metal GPU for Whisper inference)
