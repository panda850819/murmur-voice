# Murmur Voice: Vision & Roadmap

## End Goal

> The fastest, safest, verifiably private open-source voice-to-text tool. macOS and Windows.

### Success Criteria

| Dimension | Target | Measurement |
|-----------|--------|-------------|
| Speed | Hotkey release to text output < 1s (short utterances) | Measurable latency benchmark |
| Accuracy | Output usable without manual correction after AI enhancement | Users don't need to go back and edit |
| Privacy | Full pipeline works offline, all code open source and auditable | Zero network requests in local mode |
| Cross-platform | macOS and Windows feature parity | Same Rust core, zero feature gap |
| Experience | One key to use. No config, no account | First transcription within 30s of install |

### Non-goals

| Out of Scope | Reason |
|-------------|--------|
| Voice assistant / conversational AI | Focus on transcription, not dialogue |
| Meeting recording / speaker diarization | Different product |
| iOS / Android | Desktop tool |
| Enterprise SSO / admin panel | SuperWhisper's lane |
| Platform / ecosystem / plugin system | Tool, not platform |

---

## Competitive Position

### Market Landscape

```
            Paid                        Free
             |                           |
  Closed   SuperWhisper ($8.49/mo)     macOS Dictation
             |                           |
  Open     VoiceInk ($39.99, GPL)      Murmur  <-- here
             |                           |
           macOS only                  macOS + Windows
```

### Why Murmur Wins

1. **Verifiable privacy**: Open-source Rust binary. No runtime, no bytecode, no hidden network calls. Users can audit every line from mic input to text output.
2. **Windows is first-class**: SuperWhisper's Windows is crippled (no local models, no FileSync). VoiceInk is macOS-only. Murmur treats Windows equally, including CUDA GPU acceleration.
3. **Full-chain offline**: Local Whisper + local LLM (Ollama) = zero network dependency. No other tool offers free, open-source, fully-offline voice-to-text WITH AI enhancement.
4. **Free**: No subscription, no license key, no account.

### Rust as Trust Foundation

Rust is not just a technical choice. It's the trust architecture:

- Native binary = no runtime can phone home
- Open-source Rust = auditable (not obfuscated bytecode)
- Memory safety without GC = efficient on-device processing
- Cross-platform = privacy promise works on both macOS and Windows
- Trait system = pluggable engines with zero runtime overhead

---

## Current State (v0.3.0)

### What We Have

- Local Whisper transcription (whisper-rs → whisper.cpp, Metal GPU on macOS, CUDA on Windows)
- Groq cloud transcription (whisper-large-v3-turbo)
- Multi-provider LLM enhancement via TextEnhancer trait (Groq, Ollama, Custom OpenAI-compatible)
- Full-chain offline mode: local Whisper + Ollama
- Data flow indicator (Local/Cloud badge in preview window)
- Smart clipboard (input field detection, clipboard-only fallback)
- Personal dictionary
- Hold + Toggle recording modes
- Preview window (editable, copy button, dictionary suggestions)
- Cross-platform CI: macOS (.dmg), Windows CPU (.msi), Windows CUDA (-cuda.msi)

### What's Missing to Reach End Goal

1. Only whisper.cpp for local inference — same speed as every competitor
2. No data sovereignty features (history, export, privacy dashboard)
3. No context awareness (screen, clipboard context for LLM)
4. No streaming transcription (current "live preview" is 2s peek intervals)

---

## Roadmap

### Phase 1: Engine Abstraction + Local-First (v0.3) -- COMPLETED

**Theme**: Build the foundation. Make offline-everything possible.

#### Completed

- `TextEnhancer` trait (`Send + Sync`) with `name()`, `is_local()`, `enhance()` methods
- `OpenAICompatibleEnhancer` struct with `groq()`, `ollama()`, `custom()` factory presets
- `create_enhancer(&Settings) -> Option<Box<dyn TextEnhancer>>` factory function
- Provider dropdown in Settings UI (Groq/Ollama/Custom) with conditional config sections
- Data flow indicator: Local/Cloud badges in preview window
- 6 new settings fields with backward-compatible serde defaults
- 16 unit tests (state, LLM, settings)

#### Remaining for Future Phases

| Engine | Type | Priority | Value |
|--------|------|----------|-------|
| Distil-Whisper | Transcription (local) | P1 | 4-6x faster than whisper-large-v3 |
| Moonshine | Transcription (local) | P2 | 5x faster than whisper-tiny, best for live preview |
| TranscriptionEngine trait | Transcription abstraction | P1 | Same trait pattern as TextEnhancer |

#### Deliverable

Users can run Murmur with zero network dependency: local Whisper + local Ollama. The "fully offline, fully open-source voice-to-text with AI enhancement" story is now real.

---

### Phase 2: Speed + Data Sovereignty (v0.4)

**Theme**: Be measurably faster. Give users complete data control.

#### Fast Model Integration

- Distil-Whisper or Moonshine as "Speed Mode" option
- User chooses: "Accurate" (whisper-large) vs "Fast" (distil/moonshine)
- Benchmark: release-to-text < 1s for short utterances in Fast mode

#### Pipeline Optimization

- VAD (Voice Activity Detection) before inference — skip silence segments
- Zero-copy audio buffer passing between pipeline stages
- Overlap LLM enhancement with final transcription (stream partial results)

#### Data Sovereignty Features

| Feature | Description |
|---------|-------------|
| Transcription history | Local storage (SQLite), user chooses location |
| Full export/import | Settings, dictionary, history in standard JSON format |
| Zero-retention mode | Audio deleted after transcription, text optional |
| Privacy dashboard | "Today: X minutes processed, Y% local, Z% cloud" |

#### Deliverable

Users can say "Murmur is the fastest open-source voice-to-text" with benchmark data. Users fully control their data — export, delete, verify.

---

### Phase 3: Context-Aware Intelligence (v0.5)

**Theme**: Smart transcription that understands context. All local-first.

#### Context Capture (local, no data leaves device)

| Context Source | macOS | Windows |
|----------------|-------|---------|
| Screen text | CGWindowListCopyWindowInfo + Vision OCR | UI Automation |
| Clipboard | arboard (already integrated) | arboard (already integrated) |
| Selected text | Accessibility API (already have `has_focused_text_input`) | UI Automation |
| Active app/URL | frontapp module (already have) | frontapp module (already have) |

#### Power Mode

- Per-app/URL configuration profiles
- Auto-switch transcription language, AI prompt, model based on foreground app
- Example: VS Code → English, technical mode. LINE → Chinese, casual mode

#### Enhanced AI Context Injection

```
LLM prompt = system instructions
           + screen context (local OCR)
           + clipboard context
           + selected text context
           + custom vocabulary
           + user prompt (per Power Mode)
```

All context captured and processed locally. Cloud LLM only receives the assembled prompt text, never raw screen captures.

#### Deliverable

VoiceInk-level intelligence, but cross-platform and with verifiable local processing. Context never leaves the device unless explicitly sent to a cloud LLM.

---

### Phase 4: Pure Rust Pipeline (v1.0)

**Theme**: Graduate from whisper.cpp. Every line of code is Rust.

#### Rust-Native Inference

Replace whisper-rs (C++ FFI) with Rust-native ML inference:

```
Before: Rust → FFI → whisper.cpp (C++) → Metal/CUDA
After:  Rust → candle/burn/tract → Metal/CUDA (all Rust, no FFI)
```

| Candidate | Pros | Cons |
|-----------|------|------|
| candle (Hugging Face) | Active development, Metal+CUDA, Whisper impl exists | Not as mature as whisper.cpp |
| burn (Tracel) | Pure Rust, multi-backend | Whisper not yet implemented |
| tract (Sonos) | ONNX support, production-tested | CPU-focused, GPU support limited |

#### Benefits

- No C++ toolchain dependency (simpler builds, easier contribution)
- No unsafe FFI boundary — full memory safety
- Auditable from audio capture to text output — every line is Rust
- Potentially easier cross-compilation

#### Timeline Dependency

This phase depends on candle/burn ecosystem maturity. Monitor quarterly. Begin prototyping when candle's Whisper implementation reaches parity with whisper.cpp on Metal.

#### Deliverable

The tagline becomes reality: "From microphone to text, every line of code is Rust, open-source, and auditable. Cross-platform, fully offline, free."

---

## Prior Plans Status

| Plan | Status | Notes |
|------|--------|-------|
| `2026-02-14-dictionary-undo-smart-clipboard` | Completed (v0.2.1) | All 4 features shipped |
| `2026-02-17-v03-engine-abstraction` | Completed (v0.3.0) | TextEnhancer trait, multi-provider LLM, data flow indicator |

---

## Competitive Feature Matrix (End State)

| Feature | Murmur (v1.0) | SuperWhisper | VoiceInk |
|---------|---------------|-------------|----------|
| Open source | Yes (MIT/Apache) | No | GPL v3 |
| Free | Yes | No ($8.49/mo) | No ($39.99) |
| macOS | Yes | Yes | Yes |
| Windows (full features) | Yes | Partial | No |
| Full-chain offline | Yes (Whisper + Ollama) | Partial | Partial |
| Pure Rust pipeline | Yes (v1.0) | No (Swift + whisper.cpp) | No (Swift + whisper.cpp) |
| Verifiable privacy | Yes (auditable binary) | No (closed source) | Partial (GPL but Swift) |
| Multi-engine support | Yes (trait-based) | Yes | Yes |
| Context awareness | Yes (local OCR) | Yes | Yes |
| Power Mode | Yes | Yes (Modes) | Yes |
| Speed mode | Yes (Distil/Moonshine) | Yes (Parakeet) | Yes (Parakeet) |
| Data export | Yes (JSON) | Limited | CSV |
| Privacy dashboard | Yes | No | No |
