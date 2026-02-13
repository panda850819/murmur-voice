## Context

Murmur Voice is a greenfield macOS application built from zero. The target is Apple Silicon (arm64) running macOS 12.0+. The user already has Node v22 + pnpm, cmake; Rust needs to be installed. The project repo exists at `~/site/projects/murmur-voice/` with only README/LICENSE/.gitignore.

The application provides push-to-talk voice dictation that processes all audio locally using Whisper, inserting transcribed text at the cursor position in any app.

## Goals / Non-Goals

**Goals:**
- Functional MVP: hold hotkey to record, release to transcribe and insert text
- All audio stays local (privacy-first)
- Metal GPU acceleration for fast transcription on Apple Silicon
- Minimal floating window showing recording/transcription state
- Auto-download Whisper model on first launch

**Non-Goals:**
- LLM text polishing (deferred to future change)
- Settings UI / configuration panel
- Windows/Linux support
- Streaming transcription (batch only)
- Custom hotkey configuration

## Decisions

### Tauri 2 + Vanilla JS frontend
- **Why**: Smallest bundle size (~30-50MB), Rust-native backend, no React/Vue overhead for a simple status indicator
- **Alternatives**: Electron (too heavy, 200MB+), pure CLI (no visual feedback), SwiftUI (ties to Apple ecosystem, harder to port later)

### whisper-rs with large-v3-turbo model
- **Why**: Best accuracy for Chinese+English auto-detection at reasonable size (~1.5GB). Metal acceleration via whisper.cpp backend.
- **Alternatives**: whisper.cpp direct FFI (more work, same result), cloud APIs (violates privacy goal), Vosk (lower accuracy for Chinese)

### rdev for global hotkey
- **Why**: Cross-platform keyboard listener, no extra frameworks. Listens at OS level for key events.
- **Alternatives**: tauri-plugin-global-shortcut (only handles shortcuts, not push-to-talk press/release), CGEventTap direct (more code, macOS-only)
- **Risk**: rdev may not distinguish left/right Option keys. Fallback: use raw keycode 61 for Right Option, or switch to Fn key.

### Clipboard-based text insertion (arboard + rdev simulate)
- **Why**: Most reliable cross-app text insertion on macOS. Direct typing simulation has issues with non-ASCII characters and input methods.
- **Alternatives**: enigo (similar approach but extra dependency), CGEvent direct (more code), Accessibility API insertText (app-specific)

### cpal for audio recording
- **Why**: Rust-native, well-maintained, supports macOS CoreAudio. Can capture at device default sample rate and resample.
- **Alternatives**: coreaudio-rs (macOS-only, less maintained), portaudio (C dependency)

### State machine architecture
- **Why**: Clear state transitions prevent race conditions (e.g., starting a new recording while transcribing). States: Idle -> Starting -> Recording -> Stopping -> Transcribing -> Idle.
- **Error recovery**: Any state can transition to Idle on error.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| rdev cannot distinguish Left/Right Option | Test `Key::AltGr` and raw keycode 61; fallback to different key |
| cpal default sample rate != 16kHz | Record at device default, linear interpolation resample to 16kHz |
| whisper-rs Metal compilation fails | Ensure cmake installed; fallback to CPU-only (`use_gpu(false)`) |
| Transparent window flicker on macOS | Use semi-transparent dark background instead of full transparency |
| Accessibility permission not granted | Display guidance message in floating window |
| Model download interrupted | Verify file size after download; re-download if incomplete |
| rdev::simulate needs Accessibility permission | Same permission needed for hotkey detection; guide user once |

## Key File Structure

```
src-tauri/
  Cargo.toml
  tauri.conf.json
  capabilities/default.json
  src/
    main.rs          # Entry point
    lib.rs           # Tauri commands, events, tray, hotkey orchestration
    state.rs         # State machine
    audio.rs         # cpal recording
    whisper.rs       # whisper-rs transcription engine
    hotkey.rs        # rdev push-to-talk listener
    clipboard.rs     # Text insertion via clipboard
    model.rs         # Model download manager
index.html           # Frontend entry
src/
  main.js            # Frontend logic (Tauri event listeners)
  style.css          # Glassmorphism dark theme
```
