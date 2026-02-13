# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Dev Commands

```bash
# Dev mode (hot reload frontend, Rust recompiles on change)
pnpm tauri dev

# Production build
pnpm tauri build

# Rust-only check (fast iteration, no frontend)
cargo check --manifest-path src-tauri/Cargo.toml

# Lint
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings

# Run tests (state machine tests in state.rs)
cargo test --manifest-path src-tauri/Cargo.toml
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
                                   └── state.rs      recording state machine
```

**IPC**: Frontend calls backend via `invoke("command")`, backend pushes to frontend via `app.emit("event", payload)`.

**Three windows**: `main` (420x48, always-on-top bottom bar), `settings` (460x560, on-demand), `onboarding` (560x480, first-run only). All must be listed in `src-tauri/capabilities/default.json` to invoke Tauri commands.

## Recording Flow

```
Hotkey press → CGEventTap → channel → do_start_recording()
  → AudioRecorder::start() → spawn live transcription thread (peek every 2s)
Hotkey release → do_stop_recording()
  → audio.stop() → whisper.transcribe() → clipboard.insert_text(Cmd+V) → emit result
```

State machine: `Idle → Starting → Recording → Stopping → Transcribing → Idle`

## Key Patterns

- **Shared state**: `MurmurState` with `Mutex<T>` fields, injected via `.manage()`
- **Threading**: hotkey listener (CFRunLoop), audio capture (cpal callback), live transcription (std::thread), model download (tokio async)
- **Hotkey mask**: `AtomicU64` updated at runtime when settings change, no restart needed
- **Settings path**: `~/Library/Application Support/com.murmur.voice/settings.json`
- **Model path**: `~/Library/Application Support/com.murmur.voice/models/ggml-large-v3-turbo.bin`
- **PTT keys**: Both legacy format (`left_option`) and JS `event.code` format (`AltLeft`) accepted in `ptt_key_mask()`

## macOS Requirements

- Microphone permission (audio capture)
- Accessibility permission (CGEventTap for hotkey + rdev for Cmd+V paste)
- Apple Silicon recommended (Metal GPU for Whisper inference)
