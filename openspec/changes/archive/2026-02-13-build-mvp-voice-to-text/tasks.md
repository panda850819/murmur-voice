## 1. Environment Setup

- [ ] 1.1 Install Rust via rustup
- [ ] 1.2 Initialize Tauri 2 project with Vanilla JS template, preserving existing README/LICENSE/.gitignore

## 2. Cargo Dependencies

- [ ] 2.1 Add all crate dependencies to src-tauri/Cargo.toml (tauri, cpal, whisper-rs, rdev, arboard, hound, tokio, reqwest, serde, log, thiserror, etc.)
- [ ] 2.2 Verify `cargo check` passes

## 3. State Machine (state.rs)

- [ ] 3.1 Implement RecordingState enum and AppState with Mutex-wrapped state and transition validation
- [ ] 3.2 Write unit tests covering all valid transitions and error recovery to Idle

## 4. Audio Recording (audio.rs)

- [ ] 4.1 Implement AudioRecorder using cpal with 16kHz mono f32 output and resampling for non-16kHz devices
- [ ] 4.2 Implement start/stop via AtomicBool and sample collection into Arc<Mutex<Vec<f32>>>

## 5. Whisper Engine (whisper.rs)

- [ ] 5.1 Implement TranscriptionEngine wrapping WhisperContext with Metal GPU, auto language detection, Greedy decoding
- [ ] 5.2 Handle empty/short audio protection (< 0.2s returns empty string)

## 6. Push-to-Talk Hotkey (hotkey.rs)

- [ ] 6.1 Implement rdev::listen on background thread detecting Right Option press/release
- [ ] 6.2 Send HotkeyEvent::Pressed/Released via mpsc channel

## 7. Text Insertion (clipboard.rs)

- [ ] 7.1 Implement save clipboard -> write text -> simulate Cmd+V -> wait 100ms -> restore clipboard
- [ ] 7.2 Skip insertion for empty transcription text

## 8. Model Management (model.rs)

- [ ] 8.1 Implement model path resolution and existence/size check
- [ ] 8.2 Implement streaming download from HuggingFace with progress events
- [ ] 8.3 Implement file size validation after download

## 9. Tauri Integration (lib.rs)

- [ ] 9.1 Register Tauri commands: start_recording, stop_recording, get_recording_state, is_model_ready, download_model_cmd
- [ ] 9.2 Setup system tray with Quit option
- [ ] 9.3 Setup hotkey listener thread with event-driven recording/transcription/insertion pipeline
- [ ] 9.4 Emit frontend events: recording_state_changed, transcription_complete, model_download_progress, model_ready

## 10. Tauri Configuration

- [ ] 10.1 Configure tauri.conf.json: 320x120 window, decorations off, transparent, always on top, skip taskbar, bundle ID
- [ ] 10.2 Configure capabilities/default.json with core:default, core:event:default, global-shortcut:default

## 11. Frontend UI

- [ ] 11.1 Create index.html with status indicator, state text, and transcription result display
- [ ] 11.2 Create style.css with dark glassmorphism theme, state-colored indicator lights, 16px border-radius
- [ ] 11.3 Create main.js with Tauri event listeners, model download progress display, auto-reset after 2s

## 12. End-to-End Verification

- [ ] 12.1 Verify `pnpm tauri dev` launches floating window with tray icon
- [ ] 12.2 Verify model download with progress display on first launch
- [ ] 12.3 Verify push-to-talk recording and transcription flow
- [ ] 12.4 Verify text insertion at cursor position
- [ ] 12.5 Verify second launch skips download and loads model directly
