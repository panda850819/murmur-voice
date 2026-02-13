## Why

Commercial voice-to-text apps (e.g., Typeless) were found to collect browser URLs, window titles, and keystrokes, sending them to remote servers. There is no privacy-first macOS-native alternative that keeps all audio processing local. Murmur fills this gap with push-to-talk voice dictation using on-device Whisper inference.

## What Changes

- Add Tauri 2 application shell with transparent floating window
- Add local audio recording via cpal (16kHz mono for Whisper)
- Add on-device speech-to-text via whisper-rs with Metal GPU acceleration (large-v3-turbo model)
- Add push-to-talk hotkey (Right Option) via rdev global keyboard listener
- Add clipboard-based text insertion (save clipboard, paste transcription, restore clipboard)
- Add model download manager with progress reporting (HuggingFace, ~1.5GB)
- Add system tray icon with Quit option
- Add minimal dark glassmorphism UI showing recording state and transcription results

## Capabilities

### New Capabilities
- `audio-recording`: Microphone capture using cpal, 16kHz mono f32 output with resampling support
- `speech-transcription`: Local Whisper inference via whisper-rs with Metal acceleration, auto language detection (zh/en)
- `push-to-talk`: Global hotkey detection (Right Option key) via rdev for start/stop recording
- `text-insertion`: Clipboard-based text insertion into any active application via Cmd+V simulation
- `model-management`: Download, verify, and load whisper-rs GGML models from HuggingFace
- `app-lifecycle`: Tauri 2 application shell, system tray, floating window, state machine (Idle/Recording/Transcribing)

### Modified Capabilities

## Impact

- **New project**: Tauri 2 + Vanilla JS, all code is new
- **System permissions required**: Microphone access, Accessibility (for global hotkey + key simulation)
- **Dependencies**: whisper-rs (Metal), cpal, rdev, arboard, hound, tauri 2, tokio, reqwest
- **Disk**: ~1.5GB for Whisper large-v3-turbo model in ~/Library/Application Support/com.murmur.voice/models/
- **Platform**: macOS 12.0+ arm64 only (MVP)
