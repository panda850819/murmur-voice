# Murmur

> Your voice, unheard by others.

Privacy-first voice-to-text for macOS, built with Rust.

## What is Murmur?

Murmur is a local-first voice dictation tool that transcribes your speech and inserts polished text at your cursor position -- in any app. Unlike cloud-based alternatives, your audio never leaves your machine.

## Features

- **Push-to-Talk** -- Hold a hotkey to speak, release to insert text
- **Local Transcription** -- Runs Whisper on-device via whisper-rs (Metal accelerated on Apple Silicon)
- **AI Polish** -- Optional text refinement via local LLM (Ollama) or Groq API (text-only, no audio sent)
- **System-wide** -- Works in any text field across all apps
- **Lightweight** -- Tauri-based, ~30-50MB vs 200MB+ Electron apps
- **Open Source** -- Fully auditable, no telemetry, no tracking

## Architecture

```
Hotkey (rdev) -> Record (cpal) -> Transcribe (whisper-rs) -> Polish (LLM) -> Insert (enigo)
```

All audio processing happens locally. If you opt into LLM polishing via cloud API, only the transcribed text is sent -- never your audio.

## Tech Stack

| Component | Crate | Purpose |
|-----------|-------|---------|
| App Framework | `tauri` | Lightweight desktop app |
| Audio Capture | `cpal` | Cross-platform microphone input |
| Speech-to-Text | `whisper-rs` | Local Whisper inference (Metal/CoreML) |
| Hotkey Detection | `rdev` | Global keyboard event listener |
| Text Insertion | `enigo` | Simulate typing at cursor position |
| Clipboard | `arboard` | Fallback text insertion |
| LLM Polish | `ollama-rs` / `reqwest` | Optional text refinement |

## Requirements

- macOS 12.0+ (Apple Silicon recommended)
- Microphone permission
- Accessibility permission (for global hotkey + text insertion)

## Getting Started

```bash
# Clone
git clone https://github.com/panda850819/murmur-voice.git
cd murmur-voice

# Install dependencies
cargo build --release

# Run
cargo run --release
```

## Privacy

Murmur was born from a security audit of a commercial voice-to-text app that was found to:
- Capture browser URLs and window titles
- Monitor all keystrokes via CGEventTap
- Send application context to remote servers
- Include session recording analytics (Microsoft Clarity)

Murmur does none of this. Your audio is processed locally, and the source code is fully auditable.

## License

MIT
