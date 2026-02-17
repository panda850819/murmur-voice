# Murmur Voice: Roadmap

> The fastest, safest, verifiably private open-source voice-to-text tool. macOS and Windows.

## Current: v0.3.x — Engine Abstraction + Local-First

- Local Whisper transcription (Metal GPU on macOS, CUDA on Windows)
- Groq cloud transcription (whisper-large-v3-turbo)
- Multi-provider LLM enhancement (Groq, Ollama, Custom OpenAI-compatible)
- Full-chain offline mode: local Whisper + Ollama
- Smart clipboard, editable preview, personal dictionary
- Hold + Toggle recording modes
- UI language switching (English / 繁體中文)

## Next: v0.4 — Speed + Data Sovereignty

### Fast Model Integration

- Distil-Whisper or Moonshine as "Speed Mode" option
- User chooses: "Accurate" (whisper-large) vs "Fast" (distil/moonshine)
- Target: release-to-text < 1s for short utterances in Fast mode

### Pipeline Optimization

- VAD (Voice Activity Detection) — skip silence segments
- Zero-copy audio buffer passing between pipeline stages
- Overlap LLM enhancement with final transcription

### Data Sovereignty

| Feature | Description |
|---------|-------------|
| Transcription history | Local storage (SQLite), user chooses location |
| Full export/import | Settings, dictionary, history in standard JSON |
| Zero-retention mode | Audio deleted after transcription, text optional |
| Privacy dashboard | "Today: X minutes processed, Y% local, Z% cloud" |

## v0.5 — Context-Aware Intelligence

### Context Capture (all local, no data leaves device)

| Context Source | macOS | Windows |
|----------------|-------|---------|
| Screen text | Vision OCR | UI Automation |
| Clipboard | arboard | arboard |
| Selected text | Accessibility API | UI Automation |
| Active app/URL | frontapp module | frontapp module |

### Power Mode

- Per-app/URL configuration profiles
- Auto-switch language, AI prompt, model based on foreground app
- Example: VS Code → English, technical. LINE → Chinese, casual.

## v1.0 — Pure Rust Pipeline

Replace whisper-rs (C++ FFI) with Rust-native ML inference:

```
Before: Rust → FFI → whisper.cpp (C++) → Metal/CUDA
After:  Rust → candle/burn → Metal/CUDA (all Rust, no FFI)
```

No C++ toolchain. No unsafe FFI. Auditable from mic to text — every line is Rust.

---

## Competitive Position

```
            Paid                        Free
             |                           |
  Closed   SuperWhisper ($8.49/mo)     macOS Dictation
             |                           |
  Open     VoiceInk ($39.99, GPL)      Murmur  <-- here
             |                           |
           macOS only                  macOS + Windows
```

### Why Murmur

1. **Verifiable privacy**: Open-source Rust binary. No runtime, no hidden network calls.
2. **Windows is first-class**: CUDA GPU acceleration, full feature parity.
3. **Full-chain offline**: Local Whisper + local LLM = zero network dependency.
4. **Free**: No subscription, no license key, no account.
