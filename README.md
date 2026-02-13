# Murmur

> Your voice, unheard by others.

Privacy-first voice-to-text for macOS, built with Rust.

## What is Murmur?

Murmur is a local-first voice dictation tool that transcribes your speech and inserts polished text at your cursor position -- in any app. Unlike cloud-based alternatives, your audio never leaves your machine.

## Features

- **Push-to-Talk** -- Hold a modifier key to speak, release to insert text
- **Custom Hotkey** -- Choose any modifier key (Option, Command, Shift, Control, left or right)
- **Local Transcription** -- Runs Whisper on-device via whisper-rs (Metal accelerated on Apple Silicon)
- **Live Preview** -- See partial transcription while you speak
- **System-wide** -- Works in any text field across all apps
- **Lightweight** -- Tauri-based, ~30-50MB vs 200MB+ Electron apps
- **Open Source** -- Fully auditable, no telemetry, no tracking

## Architecture

```
Hotkey (CGEventTap) -> Record (cpal) -> Transcribe (whisper-rs) -> Insert (clipboard + Cmd+V)
```

All audio processing happens locally.

## Tech Stack

| Component | Crate | Purpose |
|-----------|-------|---------|
| App Framework | `tauri` 2 | Lightweight desktop app |
| Audio Capture | `cpal` | Microphone input |
| Speech-to-Text | `whisper-rs` | Local Whisper inference (Metal) |
| Hotkey Detection | CGEventTap (CoreGraphics FFI) | Global modifier key listener |
| Text Insertion | `arboard` + `rdev` | Clipboard write + Cmd+V simulation |

## Requirements

- macOS 12.0+ (Apple Silicon recommended)
- Microphone permission
- Accessibility permission (for global hotkey + text insertion)

## Getting Started

```bash
git clone https://github.com/panda850819/murmur-voice.git
cd murmur-voice
pnpm install
pnpm tauri dev
```

## Roadmap

- [ ] LLM text polishing (local Ollama / Groq)
- [ ] Multiple language model sizes (currently large-v3-turbo only)
- [ ] Auto-start at login
- [ ] Windows support
- [ ] Linux support

## Privacy

Murmur was born from a security audit of a commercial voice-to-text app that was found to:
- Capture browser URLs and window titles
- Monitor all keystrokes via CGEventTap
- Send application context to remote servers
- Include session recording analytics (Microsoft Clarity)

Murmur does none of this. Your audio is processed locally, and the source code is fully auditable.

## License

MIT
