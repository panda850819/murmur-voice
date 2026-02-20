# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Dev Commands

```bash
# Dev mode (hot reload frontend, Rust recompiles on change)
pnpm tauri dev

# Production build
pnpm tauri build

# Production build with CUDA (Windows only, requires CUDA toolkit)
pnpm tauri build --features cuda

# Rust-only commands (run from src-tauri/)
cd src-tauri
cargo check                                       # fast iteration
cargo clippy --all-targets -- -D warnings          # lint (zero warnings policy)
cargo test --lib                                   # 36 tests (llm, settings, state, whisper, model, audio)
cargo test test_valid_forward_transitions          # run a single test
```

## Architecture

Tauri 2 desktop app: Rust backend (core process) + vanilla HTML/JS/CSS frontend (webview). Cross-platform (macOS + Windows).

```
Frontend (src/)                    Backend (src-tauri/src/)
├── index.html      main window    ├── lib.rs        app setup, commands, tray
├── settings.html   preferences    ├── audio.rs      cpal mic capture → 16kHz mono
├── preview.html    transcription  ├── whisper.rs    whisper-rs (Metal/CUDA)
├── onboarding.html first-run      ├── hotkey.rs     cfg dispatcher → _macos.rs / _windows.rs
├── events.js       event consts   ├── frontapp.rs   cfg dispatcher → _macos.rs / _windows.rs
├── i18n.js         localization   ├── clipboard.rs  arboard + rdev (cfg gates for paste key)
├── *.js            invoke/listen  ├── model.rs      ModelConfig + HuggingFace download
└── *.css                          ├── settings.rs   JSON persistence + key mapping
                                   ├── state.rs      recording state machine
                                   └── llm.rs        TextEnhancer trait + multi-provider LLM
```

**IPC**: Frontend calls backend via `invoke("command")` (uses `window.__TAURI__.core.invoke` via `withGlobalTauri`), backend pushes to frontend via `app.emit("event", payload)`.

**Four windows**: `main` (420x48, always-on-top bottom bar, starts hidden), `preview` (420x280, transcription result), `settings` (460x700, on-demand), `onboarding` (560x480, first-run only). `main` and `preview` are defined in `tauri.conf.json`; `settings` and `onboarding` are created dynamically via `WebviewWindowBuilder` in `lib.rs`. All must be listed in `src-tauri/capabilities/default.json` to invoke Tauri commands.

**NSPanel overlay (macOS)**: `main` and `preview` windows are converted to NSPanel via `tauri-nspanel` (v2.1 branch, git dep) at startup. Panel config: `NonactivatingPanel` style + `CanJoinAllSpaces | Stationary | FullScreenAuxiliary` + level 1001. This enables overlay on fullscreen apps. The `panel!` macro lives in `mod overlay_panel` with required traits in scope.

**Tray menu**: Settings, Show/Hide toggle, Quit. Show/Hide controls `main_visible` and sets `manual_show` flag to suppress auto-hide after transcription.

**Frontend is static files** served directly from `src/` (no build step, no bundler). Plain HTML/JS/CSS.

## Cross-Platform Pattern

Platform-specific modules use a cfg dispatcher pattern:

```rust
// hotkey.rs, frontapp.rs — thin dispatchers
#[cfg(target_os = "macos")]
#[path = "hotkey_macos.rs"]
mod platform;

#[cfg(target_os = "windows")]
#[path = "hotkey_windows.rs"]
mod platform;

pub(crate) use platform::*;
```

Each platform file exports the same public API. `clipboard.rs` uses inline `#[cfg]` gates instead (simpler, only the paste key differs: MetaLeft vs ControlLeft).

Platform-conditional deps in `Cargo.toml`:
- macOS: `whisper-rs` with `metal` feature
- Windows: `whisper-rs` CPU-only by default; `cuda` crate feature enables GPU acceleration (requires CUDA toolkit at build time). `windows` crate for Win32 APIs

## Recording Flow

```
Hotkey press → platform hotkey listener → channel → do_start_recording()
  → AudioRecorder::start() → spawn live transcription thread (local engine only, peek every 2s)
Hotkey release → do_stop_recording()
  → audio.stop()
  → if engine=groq: llm::transcribe_groq() (cloud Whisper API)
    else: whisper.transcribe() (local GPU)
  → if llm_enabled: create_enhancer() → TextEnhancer::enhance() (Groq/Ollama/Custom)
  → clipboard.insert_text(paste simulation) → emit result → preview window
  → auto-hide main+preview after 3s (unless manual_show)
```

State machine: `Idle → Starting → Recording → Stopping → Transcribing → [Processing] → Idle`

Two recording modes: **Hold** (press=start, release=stop) and **Toggle** (press toggles, 5-min auto-stop).

**Window visibility**: Main window starts hidden. Auto-shows when recording starts, auto-hides ~3s after transcription completes. `preview_generation: AtomicU64` prevents stale hide timers from closing newer results. If user manually shows via tray (`manual_show` flag), auto-hide is suppressed until next recording cycle.

**ESC cancel**: Global ESC key cancels active recording. Emits `recording_cancelled` event to frontend. Resets state to Idle without transcribing.

## Transcription Engines

- **Local**: whisper-rs (Metal GPU on macOS). Live preview enabled. Model stored at app data dir `/models/`.
- **Groq**: Cloud Whisper API (`whisper-large-v3-turbo`). Audio encoded to WAV via `hound`, sent as multipart form. No live preview (too expensive). `groq_api_key` shared between Whisper transcription and Groq LLM enhancement.

## LLM Enhancement (Multi-Provider)

`TextEnhancer` trait (`Send + Sync`) with `name()`, `is_local()`, `enhance()` methods. Single implementation `OpenAICompatibleEnhancer` with three factory presets:

- **Groq**: `OpenAICompatibleEnhancer::groq()` — cloud, requires `groq_api_key`
- **Ollama**: `OpenAICompatibleEnhancer::ollama()` — local, no auth, skips `frequency_penalty`
- **Custom**: `OpenAICompatibleEnhancer::custom()` — any OpenAI-compatible endpoint

`create_enhancer(&Settings) -> Option<Box<dyn TextEnhancer>>` factory returns `None` if disabled or config missing. `enhance()` is sync (creates internal `tokio::runtime::Runtime` for HTTP calls).

Settings fields: `llm_provider` (groq|ollama|custom), `ollama_url`, `ollama_model`, `custom_llm_url`, `custom_llm_key`, `custom_llm_model`. All use `#[serde(default)]` for backward compatibility.

## Anti-Hallucination (Local Whisper)

- `MIN_SAMPLES = 16_000` (1s minimum, shorter clips produce hallucinations)
- Audio energy check (skip if near-silent)
- `suppress_blank(true)`, `no_speech_thold(0.6)`, `temperature_inc(0.0)`, `entropy_thold(2.4)`

## Key Patterns

- **Shared state**: `MurmurState` with `Mutex<T>` fields, injected via `.manage()`. Includes `engine_init_done: (Mutex<bool>, Condvar)` for background engine readiness signaling. Atomic fields: `preview_generation` (u64, timer invalidation), `main_visible` (bool), `manual_show` (bool, tray-triggered show suppresses auto-hide).
- **Background engine init**: TranscriptionEngine loads in a background `std::thread` during startup. Recording waits on Condvar if engine isn't ready yet. On failure, retries on first recording attempt.
- **Threading**: hotkey listener (CFRunLoop on macOS / SetWindowsHookEx on Windows), audio capture (cpal callback), live transcription (std::thread), model download (tokio async), engine init (std::thread)
- **Dynamic thread count**: `whisper.rs::optimal_threads()` → `calculate_threads()`: <=4 cores uses all, >4 reserves 2 for system/UI, caps at 8. Fallback to 4 if parallelism query fails.
- **Hotkey mask**: `AtomicU64` updated at runtime when settings change, no restart needed
- **Path resolution**: All paths resolved via Tauri's `app.path().app_data_dir()`, stored in `MurmurState.app_data_dir`. No `#[cfg]` path assembly — `model.rs` and `settings.rs` accept `base: &Path`.
- **PTT keys**: Both legacy format (`left_option`) and JS `event.code` format (`AltLeft`) accepted in `ptt_key_mask()`

## Gotchas

- **New windows** must be added to `src-tauri/capabilities/default.json` `"windows"` array or they can't invoke any Tauri commands
- **Groq API key** is shared between Whisper transcription and Groq LLM enhancement — stored in `settings.groq_api_key`
- **`frontapp_macos.rs` uses raw Objective-C FFI** (objc_msgSend) — no crate dependency, but `unsafe` throughout
- **`frontapp_windows.rs` uses `windows` crate** — `PWSTR` wrapper required for Win32 string buffer APIs
- **Live transcription** only runs for local engine; Groq mode skips it entirely (cost)
- **Toggle mode** checks `app_state.current()` (not a local flag) to decide start/stop — this avoids desync after auto-stop timeout
- **LLM post-processing prompt** must explicitly state input is raw transcription, not a question — otherwise the model answers instead of cleaning up. User message is prefixed with `[Raw transcription to clean up]`
- **Windows model migration** runs at most once per process via `std::sync::Once` in `is_model_ready()`
- **`macos-private-api` feature is required** in Cargo.toml — needed for transparent windows on macOS. Do NOT remove.
- **Frontend event strings** are centralized in `src/events.js` (`EVENTS`, `RECORDING_STATES`, `TRANSCRIPTION_MODES`). Use these constants instead of magic strings.
- **CSP**: `style-src` does NOT allow `unsafe-inline` — use `.hidden` CSS class (in each page's CSS) instead of inline `style="display:none"`. Toggle visibility with `classList.add/remove("hidden")`.
- **Model config**: `ModelConfig` struct in `model.rs` holds URL, filename, expected size. Functions like `is_model_ready()`, `download_model()` accept `&ModelConfig`.
- **CI uses `macos-14`** (not `macos-latest`) due to whisper-rs-sys i8mm build failure on newer ARM runners. whisper-rs is maintained on Codeberg (archived on GitHub); crates.io v0.15.1 predates the whisper.cpp fix
- **Release workflow** produces 3 builds: macOS (.dmg), Windows CPU (.msi/.exe), Windows CUDA (-cuda.msi/-cuda.exe). CUDA job uses manual `pnpm tauri build` + `softprops/action-gh-release` (not `tauri-action`) to avoid filename collision with CPU build

## Platform Requirements

### macOS
- macOS 12.0+ (Apple Silicon recommended for local Whisper)
- Microphone permission (audio capture)
- Accessibility permission (CGEventTap for hotkey + rdev for Cmd+V paste)

### Windows
- Windows 10+
- Microphone permission
