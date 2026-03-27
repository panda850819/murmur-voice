mod audio;
mod clipboard;
mod events;
mod frontapp;
mod hotkey;
mod llm;
mod model;
mod settings;
mod state;
mod whisper;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{Emitter, Manager};

// NSPanel type for overlay windows that render above fullscreen apps.
// NSWindow cannot do this — only NSPanel subclasses can.
#[cfg(target_os = "macos")]
mod overlay_panel {
    use tauri::Manager;
    use tauri_nspanel::objc2::runtime::NSObjectProtocol;
    use tauri_nspanel::objc2::{ClassType, Message};
    tauri_nspanel::panel!(OverlayPanel {
        config: {
            is_floating_panel: true,
            hides_on_deactivate: false,
        }
    });
}
#[cfg(target_os = "macos")]
use overlay_panel::OverlayPanel;

// UI Dimensions
const PREVIEW_WINDOW_HEIGHT: f64 = 280.0;
const PREVIEW_WINDOW_GAP: f64 = 8.0;
const MAIN_WINDOW_WIDTH: f64 = 420.0;
const MAIN_WINDOW_HEIGHT: f64 = 48.0;
const MAIN_WINDOW_BOTTOM_MARGIN: f64 = 80.0;

pub(crate) struct MurmurState {
    app_data_dir: PathBuf,
    app_state: state::AppState,
    recorder: Mutex<Option<audio::AudioRecorder>>,
    engine: Mutex<Option<whisper::TranscriptionEngine>>,
    /// Signals when background engine initialization is complete (success or not-needed).
    /// `(Mutex<bool>, Condvar)` — `true` means init is done (or no init was needed).
    engine_init_done: (Mutex<bool>, std::sync::Condvar),
    settings: Mutex<settings::Settings>,
    live_stop: AtomicBool,
    /// Join handle for the live transcription thread so stop_recording can wait for it.
    live_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// Generation counter for preview auto-hide timer cancellation.
    /// Incremented on each new recording; stale timers compare and bail out.
    preview_generation: AtomicU64,
    /// Tracks whether the main window is currently visible.
    main_visible: AtomicBool,
    /// Set when user manually shows window via tray; suppresses auto-hide.
    manual_show: AtomicBool,
    /// Guard to prevent concurrent model downloads (main + onboarding can both trigger).
    downloading: AtomicBool,
    /// Guard to prevent concurrent translation operations.
    translating: AtomicBool,
    /// Active recording mode for the current recording cycle.
    active_mode: Mutex<state::RecordingMode>,
    /// Captured context (selected text / clipboard) for VoiceCommand/ClipboardRewrite.
    captured_context: Mutex<Option<String>>,
}

/// Signal that engine initialization is complete (success or failure).
fn signal_engine_init_done(app: &tauri::AppHandle) {
    let ms = app.state::<MurmurState>();
    // Lock, set flag, drop guard, then notify — avoids borrow lifetime issues
    let locked = ms.engine_init_done.0.lock();
    if let Ok(mut done) = locked {
        *done = true;
        drop(done);
        ms.engine_init_done.1.notify_all();
    }
}

/// Spawn a background thread to load the Whisper engine.
/// Used by startup, post-download, and engine-switch paths.
fn spawn_engine_load(app: tauri::AppHandle, model_path: std::path::PathBuf, context: &'static str) {
    std::thread::spawn(move || {
        let model_path_str = match model_path.to_str() {
            Some(s) => s.to_string(),
            None => {
                log::error!("model path contains invalid UTF-8");
                signal_engine_init_done(&app);
                return;
            }
        };
        match whisper::TranscriptionEngine::new(&model_path_str) {
            Ok(engine) => {
                let ms = app.state::<MurmurState>();
                if let Ok(mut lock) = ms.engine.lock() {
                    *lock = Some(engine);
                }
                signal_engine_init_done(&app);
                log::info!("whisper engine loaded ({})", context);
            }
            Err(e) => {
                log::error!("engine init failed ({}): {}", context, e);
                signal_engine_init_done(&app);
            }
        }
    });
}

#[cfg(target_os = "macos")]
fn is_accessibility_trusted() -> bool {
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
fn is_accessibility_trusted() -> bool {
    true
}

fn is_microphone_authorized() -> &'static str {
    frontapp::is_microphone_authorized()
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

fn show_preview_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("preview") {
        // Position preview directly above the main bar
        if let Some(main_win) = app.get_webview_window("main") {
            if let Ok(main_pos) = main_win.outer_position() {
                let preview_h = PREVIEW_WINDOW_HEIGHT;
                let gap = PREVIEW_WINDOW_GAP;
                let scale = main_win
                    .current_monitor()
                    .ok()
                    .flatten()
                    .map(|m| m.scale_factor())
                    .unwrap_or(1.0);
                let new_y = main_pos.y as f64 - (preview_h + gap) * scale;
                use tauri::PhysicalPosition;
                let _ = w.set_position(PhysicalPosition::new(main_pos.x, new_y as i32));
            }
        }
        let _ = w.show();
        // Do not call set_focus — preview must not steal focus
    }
}

fn hide_preview_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("preview") {
        let _ = w.hide();
    }
}

fn reset_to_idle(state: &MurmurState, app: &tauri::AppHandle) {
    let _ = state.app_state.transition(state::RecordingState::Idle);
    let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_IDLE);
}

/// Stop live transcription thread and audio recorder, discarding all captured audio.
/// Does NOT reset state or emit events — call `reset_to_idle` separately.
fn cancel_active_recording(state: &MurmurState) {
    state.live_stop.store(true, Ordering::SeqCst);
    if let Ok(mut lt) = state.live_thread.lock() {
        if let Some(handle) = lt.take() {
            let _ = handle.join();
        }
    }
    if let Ok(mut rec) = state.recorder.lock() {
        if let Some(recorder) = rec.as_mut() {
            let _ = recorder.stop();
        }
    }
}

/// Max time between PTT modifier press and translate key press to be considered
/// a modifier conflict (rather than an intentional recording).
const MODIFIER_CONFLICT_WINDOW: std::time::Duration = std::time::Duration::from_millis(500);

/// Emit recording mode info to the frontend for display in the main bar.
fn emit_mode_info(app: &tauri::AppHandle, mode: state::RecordingMode, llm_enabled: bool) {
    let mode_str = match mode {
        state::RecordingMode::Dictation => {
            if llm_enabled { "dictation_llm" } else { "dictation" }
        }
        state::RecordingMode::VoiceCommand => "voice_command",
        state::RecordingMode::ClipboardRewrite => "clipboard_rewrite",
        state::RecordingMode::Translate => "translate",
    };
    let _ = app.emit(events::RECORDING_MODE_INFO, mode_str);
}

fn do_translate(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<MurmurState>();

    // 1. Show main window with "translating" status
    let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_TRANSLATING);
    show_main_window(app);

    // 2. Simulate Cmd+C to copy selection (copy_selection releases modifiers first)
    clipboard::copy_selection().map_err(|e| format!("Failed to copy selection: {e}"))?;

    // 4. Read clipboard
    let text = clipboard::read_text()
        .map_err(|e| format!("Failed to read clipboard: {e}"))?;
    if text.trim().is_empty() {
        return Err("No text selected".to_string());
    }

    // 5. Get translator (bypasses llm_enabled check)
    let settings = state.settings.lock().map_err(|e| format!("settings mutex poisoned: {e}"))?.clone();
    let translator = llm::create_translator(&settings)
        .ok_or("Enable AI Processing provider in Settings to use translation")?;

    // 6. Translate via LLM (auto-detect direction)
    let target = llm::detect_target_language(&text);
    let translated = translator.translate(&text, target)
        .map_err(|e| e.to_string())?;

    // 7. Write to clipboard and paste (clipboard retains translated text)
    clipboard::set_and_paste(&translated).map_err(|e| e.to_string())?;

    // 8. Show preview (stays visible, no auto-hide)
    let _ = app.emit(
        events::TRANSCRIPTION_COMPLETE,
        serde_json::json!({
            "text": translated,
            "mode": events::MODE_TRANSLATED
        }),
    );
    show_preview_window(app);

    // 9. Reset main window state and hide it (preview stays visible)
    let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_IDLE);
    hide_main_window(app);
    state.main_visible.store(false, Ordering::SeqCst);

    Ok(())
}

fn do_start_recording(app: &tauri::AppHandle, mode: state::RecordingMode) -> Result<(), String> {
    let state = app.state::<MurmurState>();

    // Store the active mode
    if let Ok(mut m) = state.active_mode.lock() {
        *m = mode;
    }

    // For VoiceCommand/ClipboardRewrite, check prerequisites
    match mode {
        state::RecordingMode::VoiceCommand => {
            // Check LLM is enabled
            let llm_enabled = state.settings.lock().map(|s| s.llm_enabled).unwrap_or(false);
            if !llm_enabled {
                return Err("Enable AI Processing in Settings to use Voice Command mode".to_string());
            }
            // Copy selection to get context
            clipboard::copy_selection().map_err(|e| format!("Failed to copy selection: {e}"))?;
            let text = clipboard::read_text()
                .map_err(|e| format!("Failed to read clipboard: {e}"))?;
            if text.trim().is_empty() {
                return Err("No text selected".to_string());
            }
            if let Ok(mut ctx) = state.captured_context.lock() {
                *ctx = Some(text);
            }
        }
        state::RecordingMode::ClipboardRewrite => {
            // Check LLM is enabled
            let llm_enabled = state.settings.lock().map(|s| s.llm_enabled).unwrap_or(false);
            if !llm_enabled {
                return Err("Enable AI Processing in Settings to use Clipboard Rewrite mode".to_string());
            }
            // Read clipboard content
            let text = clipboard::read_text()
                .map_err(|e| format!("Failed to read clipboard: {e}"))?;
            if text.trim().is_empty() {
                return Err("Clipboard is empty".to_string());
            }
            if let Ok(mut ctx) = state.captured_context.lock() {
                *ctx = Some(text);
            }
        }
        _ => {
            // Clear any stale context
            if let Ok(mut ctx) = state.captured_context.lock() {
                *ctx = None;
            }
        }
    }

    // If local engine and model not downloaded, trigger download instead of recording
    let is_local = state.settings.lock().map(|s| s.engine != "groq").unwrap_or(true);
    if is_local && !model::is_model_ready(&state.app_data_dir, &model::ModelConfig::default()) {
        log::info!("model not ready, triggering download");
        show_main_window(app);
        state.main_visible.store(true, Ordering::SeqCst);
        // Emit downloading state so frontend shows progress bar
        let _ = app.emit(events::RECORDING_STATE_CHANGED, "downloading_model");

        let app_clone = app.clone();
        let base = state.app_data_dir.clone();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    log::error!("failed to create tokio runtime for download: {}", e);
                    let _ = app_clone.emit(events::RECORDING_ERROR, e.to_string());
                    return;
                }
            };
            let config = model::ModelConfig::default();
            let app_progress = app_clone.clone();
            let result = rt.block_on(model::download_model(&base, &config, move |downloaded, total| {
                let _ = app_progress.emit(events::MODEL_DOWNLOAD_PROGRESS, serde_json::json!({
                    "downloaded": downloaded,
                    "total": total,
                }));
            }));
            match result {
                Ok(()) => {
                    let _ = app_clone.emit(events::MODEL_READY, ());
                    // Load engine in background
                    let ms = app_clone.state::<MurmurState>();
                    if let Ok(mut done) = ms.engine_init_done.0.lock() {
                        *done = false;
                    }
                    let model_path = model::model_path(&base, &config.filename);
                    spawn_engine_load(app_clone, model_path, "first-record download");
                }
                Err(e) => {
                    log::error!("model download failed: {}", e);
                    let _ = app_clone.emit(events::RECORDING_ERROR, e.to_string());
                }
            }
        });
        return Ok(());
    }

    // Cancel any pending preview auto-hide timer
    state.preview_generation.fetch_add(1, Ordering::SeqCst);
    state.manual_show.store(false, Ordering::SeqCst);

    show_main_window(app);
    state.main_visible.store(true, Ordering::SeqCst);
    show_preview_window(app);

    // Emit mode info for main bar display
    let llm_enabled = state.settings.lock().map(|s| s.llm_enabled).unwrap_or(false);
    emit_mode_info(app, mode, llm_enabled);

    state
        .app_state
        .transition(state::RecordingState::Starting)
        .map_err(|e| e.to_string())?;
    let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_STARTING);

    let mut recorder = audio::AudioRecorder::new();
    if let Err(e) = recorder.start() {
        reset_to_idle(&state, app);
        let _ = app.emit(events::RECORDING_ERROR, e.to_string());
        hide_main_window(app);
        state.main_visible.store(false, Ordering::SeqCst);
        hide_preview_window(app);
        return Err(e.to_string());
    }

    let mut recorder_lock = state
        .recorder
        .lock()
        .map_err(|e| format!("recorder mutex poisoned: {e}"))?;
    *recorder_lock = Some(recorder);

    let _ = state
        .app_state
        .transition(state::RecordingState::Recording);
    let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_RECORDING);

    // Auto-stop after 5 minutes in toggle mode
    let rec_mode = state
        .settings
        .lock()
        .map(|s| s.recording_mode.clone())
        .unwrap_or_default();
    if rec_mode == "toggle" {
        let app_timeout = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(300));
            let ms = app_timeout.state::<MurmurState>();
            if ms.app_state.current() == state::RecordingState::Recording {
                let _ = do_stop_recording(&app_timeout);
            }
        });
    }

    // Start live transcription thread — only for Dictation mode with local engine.
    // VoiceCommand/ClipboardRewrite record voice commands, live preview not useful.
    let enable_live_preview = mode == state::RecordingMode::Dictation
        && state
            .settings
            .lock()
            .map(|s| s.engine != "groq")
            .unwrap_or(true);

    state.live_stop.store(false, Ordering::SeqCst);
    if enable_live_preview {
        let app_clone = app.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(1500));

            loop {
                let ms = app_clone.state::<MurmurState>();
                if ms.live_stop.load(Ordering::SeqCst) {
                    break;
                }

                let samples = {
                    let lock = match ms.recorder.lock() {
                        Ok(l) => l,
                        Err(_) => break,
                    };
                    match lock.as_ref() {
                        Some(rec) => rec.peek_samples().unwrap_or_else(|e| {
                            log::warn!("failed to peek samples: {}", e);
                            Vec::new()
                        }),
                        None => break,
                    }
                };

                if samples.len() < 3200 {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }

                let (language, initial_prompt) = ms
                    .settings
                    .lock()
                    .map(|s| (s.whisper_language().to_string(), s.whisper_initial_prompt()))
                    .unwrap_or_else(|_| ("auto".to_string(), String::new()));

                let text = {
                    let engine_lock = match ms.engine.try_lock() {
                        Ok(l) => l,
                        Err(_) => {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            continue;
                        }
                    };
                    match engine_lock.as_ref() {
                        Some(engine) => engine.transcribe(&samples, &language, &initial_prompt).unwrap_or_default(),
                        None => {
                            // Engine not ready yet — wait and retry on next loop iteration
                            drop(engine_lock);
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            continue;
                        }
                    }
                };

                if ms.live_stop.load(Ordering::SeqCst) {
                    break;
                }

                if !text.is_empty() {
                    let _ = app_clone.emit(events::PARTIAL_TRANSCRIPTION, &text);
                }

                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        });
        if let Ok(mut lt) = state.live_thread.lock() {
            *lt = Some(handle);
        }
    }

    Ok(())
}

fn do_stop_recording(app: &tauri::AppHandle) -> Result<String, String> {
    let state = app.state::<MurmurState>();

    state.live_stop.store(true, Ordering::SeqCst);

    // Wait for the live transcription thread to finish so we don't block on engine lock
    if let Ok(mut lt) = state.live_thread.lock() {
        if let Some(handle) = lt.take() {
            let _ = handle.join();
        }
    }

    state
        .app_state
        .transition(state::RecordingState::Stopping)
        .map_err(|e| e.to_string())?;
    let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_STOPPING);

    let samples = {
        let mut recorder_lock = state
            .recorder
            .lock()
            .map_err(|e| format!("recorder mutex poisoned: {e}"))?;
        match recorder_lock.as_mut() {
            Some(recorder) => recorder.stop(),
            None => Vec::new(),
        }
    };

    if samples.is_empty() {
        reset_to_idle(&state, app);
        hide_main_window(app);
        state.main_visible.store(false, Ordering::SeqCst);
        hide_preview_window(app);
        return Ok(String::new());
    }

    let _ = state
        .app_state
        .transition(state::RecordingState::Transcribing);
    let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_TRANSCRIBING);

    // Read active mode
    let active_mode = state.active_mode.lock()
        .map(|m| *m)
        .unwrap_or(state::RecordingMode::Dictation);

    // Emit foreground app info for the frontend (preview window + main bar badge)
    {
        let app_aware = state
            .settings
            .lock()
            .map(|s| s.app_aware_style)
            .unwrap_or(false);
        if app_aware {
            let (name, style) = frontapp::foreground_app_bundle_id()
                .as_deref()
                .map(|bid| (frontapp::display_name_for_app(bid), frontapp::style_for_app(bid)))
                .unwrap_or(("Unknown", "default"));
            let _ = app.emit(
                events::FOREGROUND_APP_INFO,
                serde_json::json!({ "name": name, "style": style }),
            );
        }
    }

    let (engine_type, language, initial_prompt, api_key_for_whisper) = state
        .settings
        .lock()
        .map(|s| (
            s.engine.clone(),
            s.whisper_language().to_string(),
            s.whisper_initial_prompt(),
            s.groq_api_key.clone(),
        ))
        .unwrap_or_else(|_| ("local".to_string(), "auto".to_string(), String::new(), String::new()));

    // Anti-hallucination: skip if audio is too short or silent (applies to all engines)
    if !audio::is_audio_usable(&samples) {
        reset_to_idle(&state, app);
        let _ = app.emit(
            events::TRANSCRIPTION_COMPLETE,
            serde_json::json!({ "text": "", "mode": active_mode.event_mode_str() }),
        );
        return Ok(String::new());
    }

    let raw_text = if engine_type == "groq" && !api_key_for_whisper.is_empty() {
        // Groq cloud Whisper
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        rt.block_on(llm::transcribe_groq(
            &api_key_for_whisper,
            &samples,
            &language,
            &initial_prompt,
        ))
        .map_err(|e| e.to_string())?
    } else {
        // Local Whisper — wait for background engine init if still running
        {
            let (init_lock, cvar) = &state.engine_init_done;
            let guard = init_lock
                .lock()
                .map_err(|e| format!("engine init mutex poisoned: {e}"))?;
            if !*guard {
                let (guard, _timeout) = cvar
                    .wait_timeout(guard, std::time::Duration::from_secs(30))
                    .map_err(|e| format!("engine init wait failed: {e}"))?;
                if !*guard {
                    log::warn!("engine init wait timed out after 30s");
                }
            }
        }

        let engine_lock = state
            .engine
            .lock()
            .map_err(|e| format!("engine mutex poisoned: {e}"))?;
        match engine_lock.as_ref() {
            Some(engine) => engine.transcribe(&samples, &language, &initial_prompt).map_err(|e| e.to_string())?,
            None => {
                // Engine not available — retry init synchronously (task 4.4)
                drop(engine_lock);
                let model_path = model::model_path(&state.app_data_dir, &model::ModelConfig::default().filename);
                let model_path_str = model_path
                    .to_str()
                    .ok_or("model path contains invalid UTF-8")?;
                log::info!("retrying engine init synchronously");
                let engine = whisper::TranscriptionEngine::new(model_path_str)
                    .map_err(|e| format!("engine init retry failed: {e}"))?;
                let text = engine
                    .transcribe(&samples, &language, &initial_prompt)
                    .map_err(|e| e.to_string())?;
                // Store engine for future use
                if let Ok(mut lock) = state.engine.lock() {
                    *lock = Some(engine);
                }
                text
            }
        }
    };

    let _ = app.emit(
        events::TRANSCRIPTION_ENGINE_INFO,
        serde_json::json!({
            "engine": &engine_type,
            "local": engine_type != "groq",
        }),
    );

    log::debug!("[whisper raw] {}", raw_text);

    // Branch based on active mode
    let text = match active_mode {
        state::RecordingMode::Dictation => {
            // Existing dictation flow: text_replacement → optional LLM enhance → paste
            let (enhancer, app_aware_style, raw_text) = {
                let s = state
                    .settings
                    .lock()
                    .map_err(|e| format!("settings mutex poisoned: {e}"))?;
                (llm::create_enhancer(&s), s.app_aware_style, s.apply_replacements(&raw_text))
            };

            if let Some(enhancer) = enhancer {
                if raw_text.is_empty() {
                    raw_text
                } else {
                    let _ = state
                        .app_state
                        .transition(state::RecordingState::Processing);
                    let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_PROCESSING);

                    let _ = app.emit(
                        events::ENHANCER_INFO,
                        serde_json::json!({
                            "name": enhancer.name(),
                            "local": enhancer.is_local(),
                        }),
                    );

                    let style = if app_aware_style {
                        frontapp::foreground_app_bundle_id()
                            .as_deref()
                            .map(frontapp::style_for_app)
                            .unwrap_or("default")
                    } else {
                        "default"
                    };

                    match enhancer.enhance(&raw_text, style) {
                        Ok(processed) => {
                            log::debug!("[llm output] {}", processed);
                            processed
                        }
                        Err(e) => {
                            log::error!("LLM post-processing failed: {}", e);
                            let _ = app.emit(
                                events::RECORDING_ERROR,
                                format!("LLM processing failed, using raw text: {e}"),
                            );
                            raw_text
                        }
                    }
                }
            } else {
                raw_text
            }
        }
        state::RecordingMode::VoiceCommand | state::RecordingMode::ClipboardRewrite => {
            // Voice command flow: skip text_replacement, require LLM
            let context = state.captured_context.lock()
                .ok()
                .and_then(|mut ctx| ctx.take())
                .unwrap_or_default();

            if raw_text.trim().is_empty() {
                reset_to_idle(&state, app);
                return Ok(String::new());
            }

            let _ = state.app_state.transition(state::RecordingState::Processing);
            let _ = app.emit(events::RECORDING_STATE_CHANGED, events::STATE_PROCESSING);

            let enhancer = {
                let s = state.settings.lock()
                    .map_err(|e| format!("settings mutex poisoned: {e}"))?;
                llm::create_enhancer(&s)
            };

            match enhancer {
                Some(enhancer) => {
                    let _ = app.emit(
                        events::ENHANCER_INFO,
                        serde_json::json!({
                            "name": enhancer.name(),
                            "local": enhancer.is_local(),
                        }),
                    );

                    let context_type = active_mode.context_type();
                    match enhancer.execute_command(&raw_text, &context, context_type) {
                        Ok(result) => {
                            log::debug!("[llm command output] {}", result);
                            result
                        }
                        Err(e) => {
                            log::error!("LLM execute_command failed: {}", e);
                            let _ = app.emit(events::RECORDING_ERROR, format!("LLM processing failed: {e}"));
                            return Err(e.to_string());
                        }
                    }
                }
                None => {
                    let err = "Enable AI Processing in Settings to use this mode";
                    let _ = app.emit(events::RECORDING_ERROR, err);
                    return Err(err.to_string());
                }
            }
        }
        state::RecordingMode::Translate => {
            // Translate mode shouldn't go through the recording pipeline,
            // but if it somehow does, just return the raw text.
            raw_text
        }
    };

    // Detect if foreground app can accept paste (default: true, only false for Desktop/Finder)
    let has_input = if !text.is_empty() {
        std::panic::catch_unwind(frontapp::has_focused_text_input).unwrap_or(true)
    } else {
        false
    };
    if !text.is_empty() {
        if has_input {
            // For VoiceCommand/ClipboardRewrite, use set_and_paste (replaces selection)
            match active_mode {
                state::RecordingMode::VoiceCommand | state::RecordingMode::ClipboardRewrite => {
                    if let Err(e) = clipboard::set_and_paste(&text) {
                        let _ = app.emit(events::RECORDING_ERROR, format!("clipboard error: {e}"));
                        log::error!("failed to paste text: {}", e);
                    }
                }
                _ => {
                    // Auto-paste mode: save clipboard → paste → restore
                    if let Err(e) = clipboard::insert_text(&text) {
                        let _ = app.emit(events::RECORDING_ERROR, format!("clipboard error: {e}"));
                        log::error!("failed to insert text: {}", e);
                    }
                }
            }
        } else {
            // Clipboard-only mode: just copy, no paste simulation
            if let Err(e) = clipboard::copy_only(&text) {
                let _ = app.emit(events::RECORDING_ERROR, format!("clipboard error: {e}"));
                log::error!("failed to copy text: {}", e);
            }
        }
    }

    // Use mode-specific event mode string
    let mode_str = active_mode.event_mode_str();

    reset_to_idle(&state, app);
    let _ = app.emit(
        events::TRANSCRIPTION_COMPLETE,
        serde_json::json!({ "text": text, "mode": mode_str }),
    );

    // Auto-hide main window 8s after transcription complete
    {
        let app_hide = app.clone();
        let gen = state.preview_generation.load(Ordering::SeqCst);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(8));
            let ms = app_hide.state::<MurmurState>();
            // Cancel if a new recording started (generation changed) or user manually showed
            if ms.preview_generation.load(Ordering::SeqCst) == gen
                && !ms.manual_show.load(Ordering::SeqCst)
            {
                hide_main_window(&app_hide);
                ms.main_visible.store(false, Ordering::SeqCst);
            }
        });
    }

    Ok(text)
}

// --- Tauri Commands ---

#[derive(serde::Serialize)]
struct UpdateCheckResult {
    up_to_date: bool,
    current_version: String,
    latest_version: String,
    release_url: String,
}

#[tauri::command]
async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let current = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.github.com/repos/panda850819/murmur-voice/releases/latest")
        .header("User-Agent", "murmur-voice")
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("parse failed: {e}"))?;
    let tag = json["tag_name"].as_str().unwrap_or("unknown");
    let latest = tag.trim_start_matches('v');
    let url = json["html_url"]
        .as_str()
        .unwrap_or("https://github.com/panda850819/murmur-voice/releases")
        .to_string();

    Ok(UpdateCheckResult {
        up_to_date: current == latest,
        current_version: current.to_string(),
        latest_version: latest.to_string(),
        release_url: url,
    })
}

#[tauri::command]
async fn open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    const ALLOWED_SCHEMES: &[&str] = &["http://", "https://", "x-apple.systempreferences:"];
    if !ALLOWED_SCHEMES.iter().any(|s| url.starts_with(s)) {
        log::error!("Blocked attempt to open unauthorized URL scheme: {}", url);
        return Err("Unauthorized URL scheme".to_string());
    }

    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| format!("failed to open URL: {e}"))
}

#[tauri::command]
fn get_recording_state(state: tauri::State<'_, MurmurState>) -> String {
    state.app_state.current().to_string()
}

#[tauri::command]
fn is_model_ready(state: tauri::State<'_, MurmurState>) -> bool {
    model::is_model_ready(&state.app_data_dir, &model::ModelConfig::default())
}

#[tauri::command]
async fn download_model_cmd(app: tauri::AppHandle) -> Result<(), String> {
    let murmur_state = app.state::<MurmurState>();

    // Prevent concurrent downloads (main window + onboarding can both trigger).
    // First caller wins; second caller silently returns Ok (it will receive progress events).
    if murmur_state.downloading.swap(true, Ordering::Acquire) {
        return Ok(());
    }

    let base = murmur_state.app_data_dir.clone();

    let app_clone = app.clone();
    let result = model::download_model(&base, &model::ModelConfig::default(), move |downloaded, total| {
        let _ = app_clone.emit(
            events::MODEL_DOWNLOAD_PROGRESS,
            serde_json::json!({
                "downloaded": downloaded,
                "total": total,
            }),
        );
    })
    .await;

    result.map_err(|e| {
        murmur_state.downloading.store(false, Ordering::Release);
        e.to_string()
    })?;

    let _ = app.emit(events::MODEL_READY, ());
    murmur_state.downloading.store(false, Ordering::Release);

    // Spawn background engine init (don't block the command)
    if let Ok(mut done) = murmur_state.engine_init_done.0.lock() {
        *done = false; // Mark as pending — background thread will set true
    }
    let model_path = model::model_path(&base, &model::ModelConfig::default().filename);
    spawn_engine_load(app.clone(), model_path, "post-download");

    Ok(())
}

#[tauri::command]
fn start_recording(app: tauri::AppHandle) -> Result<(), String> {
    do_start_recording(&app, state::RecordingMode::Dictation)
}

#[tauri::command]
fn stop_recording(app: tauri::AppHandle) -> Result<String, String> {
    do_stop_recording(&app)
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, MurmurState>) -> settings::Settings {
    state
        .settings
        .lock()
        .map(|s| s.clone())
        .unwrap_or_default()
}

#[tauri::command]
fn save_settings(
    new_settings: settings::Settings,
    state: tauri::State<'_, MurmurState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Apply all hotkey changes
    let dict_t = new_settings.ptt_key_target();
    hotkey::set_hotkey(state::RecordingMode::Dictation, dict_t.modifier_mask, dict_t.regular_key);

    let tr = new_settings.translate_key_target();
    hotkey::set_hotkey(state::RecordingMode::Translate, tr.modifier_mask, tr.regular_key);

    // Set VoiceCommand and ClipboardRewrite hotkeys
    let vc = new_settings.voice_command_key_target();
    hotkey::set_hotkey(state::RecordingMode::VoiceCommand, vc.modifier_mask, vc.regular_key);

    let cr = new_settings.clipboard_rewrite_key_target();
    hotkey::set_hotkey(state::RecordingMode::ClipboardRewrite, cr.modifier_mask, cr.regular_key);

    // Apply window opacity
    if let Some(window) = app.get_webview_window("main") {
        let _ = app.emit(events::OPACITY_CHANGED, new_settings.window_opacity);
        let _ = window.set_always_on_top(true);
    }

    // Persist first, then update in-memory state in a single lock
    settings::save_settings(&new_settings, &state.app_data_dir)?;

    let engine_changed = {
        let mut s = state.settings.lock().map_err(|e| format!("settings mutex poisoned: {e}"))?;
        let changed = s.engine != new_settings.engine;
        *s = new_settings;
        changed
    };

    // Handle engine lifecycle on switch
    if engine_changed {
        let new_engine = &state.settings.lock().map_err(|e| format!("settings mutex poisoned: {e}"))?.engine.clone();
        if new_engine == "groq" {
            // Unload whisper engine (frees ~2GB of memory)
            if let Ok(mut lock) = state.engine.lock() {
                if lock.take().is_some() {
                    log::info!("unloaded whisper engine (switched to groq)");
                }
            }
        } else {
            // Load whisper engine in background
            let model_config = model::ModelConfig::default();
            if model::is_model_ready(&state.app_data_dir, &model_config) {
                if let Ok(mut done) = state.engine_init_done.0.lock() {
                    *done = false;
                }
                let model_path = model::model_path(&state.app_data_dir, &model_config.filename);
                spawn_engine_load(app.clone(), model_path, "engine switch");
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn complete_onboarding(
    state: tauri::State<'_, MurmurState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if let Ok(mut s) = state.settings.lock() {
        s.onboarding_complete = true;
        settings::save_settings(&s, &state.app_data_dir)?;
    }
    if let Some(w) = app.get_webview_window("onboarding") {
        let _ = w.close();
    }
    Ok(())
}

#[tauri::command]
fn hide_preview(app: tauri::AppHandle) {
    hide_preview_window(&app);
}

#[tauri::command]
fn hide_overlay_windows(app: tauri::AppHandle) {
    hide_preview_window(&app);
    hide_main_window(&app);
    let ms = app.state::<MurmurState>();
    ms.main_visible.store(false, Ordering::SeqCst);
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    clipboard::copy_only(&text).map_err(|e| e.to_string())
}

#[tauri::command]
fn translate_text(text: String, app: tauri::AppHandle) -> Result<String, String> {
    let state = app.state::<MurmurState>();
    let settings = state
        .settings
        .lock()
        .map_err(|e| format!("settings mutex poisoned: {e}"))?
        .clone();
    let translator = llm::create_translator(&settings)
        .ok_or("Enable AI Processing provider in Settings to use translation")?;
    let target = llm::detect_target_language(&text);
    let translated = translator.translate(&text, target).map_err(|e| e.to_string())?;
    clipboard::copy_only(&translated).map_err(|e| e.to_string())?;
    Ok(translated)
}

#[tauri::command]
fn check_accessibility() -> bool {
    is_accessibility_trusted()
}

#[tauri::command]
fn check_microphone() -> String {
    is_microphone_authorized().to_string()
}

#[tauri::command]
fn request_microphone() {
    frontapp::request_microphone_access();
}

#[tauri::command]
fn pause_hotkey_listener() {
    hotkey::pause_hotkey(state::RecordingMode::Dictation);
}

#[tauri::command]
fn resume_hotkey_listener(state: tauri::State<'_, MurmurState>) {
    if let Ok(s) = state.settings.lock() {
        let t = s.ptt_key_target();
        hotkey::set_hotkey(state::RecordingMode::Dictation, t.modifier_mask, t.regular_key);
    }
}

#[tauri::command]
fn pause_translate_hotkey() {
    hotkey::pause_hotkey(state::RecordingMode::Translate);
}

#[tauri::command]
fn resume_translate_hotkey(state: tauri::State<'_, MurmurState>) {
    if let Ok(s) = state.settings.lock() {
        let t = s.translate_key_target();
        hotkey::set_hotkey(state::RecordingMode::Translate, t.modifier_mask, t.regular_key);
    }
}

#[tauri::command]
fn add_dictionary_term(
    term: String,
    state: tauri::State<'_, MurmurState>,
) -> Result<(), String> {
    let trimmed = term.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    if let Ok(mut s) = state.settings.lock() {
        // Check for duplicate
        let existing: Vec<&str> = s.dictionary.split(',').map(|t| t.trim()).collect();
        if existing.iter().any(|&t| t.eq_ignore_ascii_case(trimmed)) {
            return Ok(()); // Already exists
        }

        // Append
        if s.dictionary.is_empty() {
            s.dictionary = trimmed.to_string();
        } else {
            s.dictionary = format!("{}, {}", s.dictionary, trimmed);
        }

        // Persist
        settings::save_settings(&s, &state.app_data_dir)?;
    }

    Ok(())
}

#[tauri::command]
fn add_dictionary_terms(
    terms: Vec<String>,
    state: tauri::State<'_, MurmurState>,
) -> Result<(), String> {
    if terms.is_empty() {
        return Ok(());
    }

    if let Ok(mut s) = state.settings.lock() {
        let mut existing_vec: Vec<String> = s
            .dictionary
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        let mut existing_set: std::collections::HashSet<String> = existing_vec
            .iter()
            .map(|t| t.to_lowercase())
            .collect();

        let mut added = false;
        for term in terms {
            let trimmed = term.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !existing_set.contains(&trimmed.to_lowercase()) {
                existing_vec.push(trimmed.to_string());
                existing_set.insert(trimmed.to_lowercase());
                added = true;
            }
        }

        if added {
            s.dictionary = existing_vec.join(", ");
            settings::save_settings(&s, &state.app_data_dir)?;
        }
    }

    Ok(())
}

#[tauri::command]
fn open_settings(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.set_focus();
    } else {
        let _ = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("settings.html".into()),
        )
        .title("Murmur Voice Settings")
        .inner_size(460.0, 700.0)
        .resizable(false)
        .build();
    }
}

// --- App Setup ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init());

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            get_recording_state,
            is_model_ready,
            download_model_cmd,
            start_recording,
            stop_recording,
            get_settings,
            save_settings,
            open_settings,
            hide_preview,
            hide_overlay_windows,
            complete_onboarding,
            copy_to_clipboard,
            translate_text,
            pause_hotkey_listener,
            resume_hotkey_listener,
            pause_translate_hotkey,
            resume_translate_hotkey,
            add_dictionary_term,
            add_dictionary_terms,
            check_for_updates,
            check_accessibility,
            check_microphone,
            request_microphone,
            open_url,
        ])
        .setup(|app| {
            // Resolve app data directory from Tauri
            let app_data_dir = app.path().app_data_dir()
                .map_err(|e| format!("failed to resolve app data dir: {e}"))?;

            // Load settings from the resolved path
            let initial_settings = settings::load_settings(&app_data_dir);
            log::info!(
                "loaded settings: onboarding_complete={}, recording_mode={}",
                initial_settings.onboarding_complete,
                initial_settings.recording_mode
            );

            // Set all hotkey slots
            let t = initial_settings.ptt_key_target();
            hotkey::set_hotkey(state::RecordingMode::Dictation, t.modifier_mask, t.regular_key);
            let tr = initial_settings.translate_key_target();
            hotkey::set_hotkey(state::RecordingMode::Translate, tr.modifier_mask, tr.regular_key);
            let vc = initial_settings.voice_command_key_target();
            hotkey::set_hotkey(state::RecordingMode::VoiceCommand, vc.modifier_mask, vc.regular_key);
            let cr = initial_settings.clipboard_rewrite_key_target();
            hotkey::set_hotkey(state::RecordingMode::ClipboardRewrite, cr.modifier_mask, cr.regular_key);

            // Register MurmurState with the resolved app_data_dir.
            // engine_init_done starts as `false` only when background engine init will run
            // (local engine + model ready). Otherwise `true` (nothing to wait for).
            let model_ready = model::is_model_ready(&app_data_dir, &model::ModelConfig::default());
            let is_local_engine = initial_settings.engine != "groq";
            let will_load_engine = model_ready && is_local_engine;
            app.manage(MurmurState {
                app_data_dir: app_data_dir.clone(),
                app_state: state::AppState::new(),
                recorder: Mutex::new(None),
                engine: Mutex::new(None),
                engine_init_done: (Mutex::new(!will_load_engine), std::sync::Condvar::new()),
                settings: Mutex::new(initial_settings),
                live_stop: AtomicBool::new(false),
                live_thread: Mutex::new(None),
                preview_generation: AtomicU64::new(0),
                main_visible: AtomicBool::new(false),
                manual_show: AtomicBool::new(false),
                downloading: AtomicBool::new(false),
                translating: AtomicBool::new(false),
                active_mode: Mutex::new(state::RecordingMode::Dictation),
                captured_context: Mutex::new(None),
            });

            // Create system tray with Settings + Show/Hide + Quit
            let settings_item =
                tauri::menu::MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let show_item =
                tauri::menu::MenuItem::with_id(app, "show_toggle", "Show", true, None::<&str>)?;
            let quit =
                tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&settings_item, &show_item, &quit])?;
            let show_item_ref = show_item.clone();
            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&menu)
                .tooltip("Murmur Voice")
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "settings" => open_settings(app.clone()),
                    "show_toggle" => {
                        let ms = app.state::<MurmurState>();
                        let visible = ms.main_visible.load(Ordering::SeqCst);
                        if visible {
                            hide_main_window(app);
                            hide_preview_window(app);
                            ms.main_visible.store(false, Ordering::SeqCst);
                            ms.manual_show.store(false, Ordering::SeqCst);
                            let _ = show_item_ref.set_text("Show");
                        } else {
                            show_main_window(app);
                            ms.main_visible.store(true, Ordering::SeqCst);
                            ms.manual_show.store(true, Ordering::SeqCst);
                            let _ = show_item_ref.set_text("Hide");
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // Check onboarding status
            let needs_onboarding = {
                let s = app.state::<MurmurState>();
                s.settings.lock().map(|s| !s.onboarding_complete).unwrap_or(false)
            };

            if needs_onboarding {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
                let _ = tauri::WebviewWindowBuilder::new(
                    app,
                    "onboarding",
                    tauri::WebviewUrl::App("onboarding.html".into()),
                )
                .title("Welcome to Murmur Voice")
                .inner_size(560.0, 520.0)
                .resizable(false)
                .center()
                .build();
            }

            // Position main window at bottom center
            if let Some(window) = app.get_webview_window("main") {
                if let Some(monitor) = window.current_monitor().ok().flatten() {
                    let screen = monitor.size();
                    let scale = monitor.scale_factor();
                    let win_w = MAIN_WINDOW_WIDTH;
                    let win_h = MAIN_WINDOW_HEIGHT;
                    let margin = MAIN_WINDOW_BOTTOM_MARGIN;
                    let x = (screen.width as f64 / scale - win_w) / 2.0;
                    let y = screen.height as f64 / scale - win_h - margin;
                    use tauri::PhysicalPosition;
                    let _ = window.set_position(PhysicalPosition::new(
                        (x * scale) as i32,
                        (y * scale) as i32,
                    ));
                }
            }

            // Convert overlay windows to NSPanel so they render above fullscreen apps.
            // NSWindow cannot overlay fullscreen apps regardless of level — only NSPanel can.
            #[cfg(target_os = "macos")]
            {
                use tauri_nspanel::WebviewWindowExt;
                use tauri_nspanel::panel::{NSWindowCollectionBehavior, NSWindowStyleMask};

                for name in &["main", "preview"] {
                    if let Some(w) = app.get_webview_window(name) {
                        match w.to_panel::<OverlayPanel>() {
                            Ok(panel) => {
                                panel.set_level(1001);
                                panel.set_style_mask(NSWindowStyleMask::NonactivatingPanel);
                                panel.set_collection_behavior(
                                    NSWindowCollectionBehavior::CanJoinAllSpaces
                                    | NSWindowCollectionBehavior::Stationary
                                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
                                );
                                log::info!("converted '{}' to NSPanel for fullscreen overlay", name);
                            }
                            Err(e) => {
                                log::warn!("failed to convert '{}' to NSPanel: {}", name, e);
                            }
                        }
                    }
                }
            }

            // Position preview window above main bar (hidden by default)
            if let (Some(main_win), Some(preview_win)) = (
                app.get_webview_window("main"),
                app.get_webview_window("preview"),
            ) {
                if let Ok(main_pos) = main_win.outer_position() {
                    let preview_h = PREVIEW_WINDOW_HEIGHT;
                    let gap = PREVIEW_WINDOW_GAP;
                    let scale = main_win
                        .current_monitor()
                        .ok()
                        .flatten()
                        .map(|m| m.scale_factor())
                        .unwrap_or(1.0);
                    let new_y = main_pos.y as f64 - (preview_h + gap) * scale;
                    use tauri::PhysicalPosition;
                    let _ = preview_win.set_position(PhysicalPosition::new(main_pos.x, new_y as i32));
                }
            }

            // Load whisper engine in background thread only for local engine users.
            // Groq users don't need the local model at all — saves ~2GB of memory.
            if will_load_engine {
                let model_path = model::model_path(&app_data_dir, &model::ModelConfig::default().filename);
                spawn_engine_load(app.handle().clone(), model_path, "startup");
            }

            // Start hotkey listener
            let app_handle = app.handle().clone();
            let (sender, receiver) = std::sync::mpsc::channel();
            let retry_sender = sender.clone();
            hotkey::start_listener(sender);

            std::thread::spawn(move || {
                let mut is_recording = false;
                let mut last_toggle: Option<Instant> = None;
                let mut recording_started_at: Option<Instant> = None;
                while let Ok(event) = receiver.recv() {
                    if event == hotkey::HotkeyEvent::EventTapFailed {
                        let _ = app_handle.emit(events::ACCESSIBILITY_ERROR, ());
                        // Poll until Accessibility is granted, then retry
                        loop {
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            if is_accessibility_trusted() {
                                hotkey::start_listener(retry_sender.clone());
                                let _ = app_handle.emit(events::ACCESSIBILITY_GRANTED, ());
                                break;
                            }
                        }
                        continue;
                    }

                    match event {
                        hotkey::HotkeyEvent::Pressed(mode) => {
                            // Translate mode doesn't use the recording pipeline
                            if mode == state::RecordingMode::Translate {
                                let murmur_state = app_handle.state::<MurmurState>();
                                let current = murmur_state.app_state.current();

                                if current != state::RecordingState::Idle {
                                    // Check for modifier conflict
                                    let is_modifier_conflict = is_recording
                                        && matches!(
                                            current,
                                            state::RecordingState::Starting
                                                | state::RecordingState::Recording
                                        )
                                        && recording_started_at
                                            .map(|t| t.elapsed() < MODIFIER_CONFLICT_WINDOW)
                                            .unwrap_or(false);

                                    if !is_modifier_conflict {
                                        continue; // genuinely busy
                                    }

                                    log::info!("translate hotkey: cancelling modifier-conflict recording");
                                    cancel_active_recording(&murmur_state);
                                    is_recording = false;
                                    recording_started_at = None;
                                    reset_to_idle(&murmur_state, &app_handle);
                                }
                                if murmur_state
                                    .translating
                                    .swap(true, Ordering::Acquire)
                                {
                                    continue; // already in progress
                                }
                                let app2 = app_handle.clone();
                                std::thread::spawn(move || {
                                    let ms = app2.state::<MurmurState>();
                                    let result = do_translate(&app2);
                                    ms.translating.store(false, Ordering::Release);
                                    if let Err(e) = result {
                                        let _ = app2.emit(events::RECORDING_ERROR, e);
                                        let _ = app2.emit(
                                            events::RECORDING_STATE_CHANGED,
                                            events::STATE_IDLE,
                                        );
                                    }
                                });
                                continue;
                            }

                            // Recording modes (Dictation, VoiceCommand, ClipboardRewrite)
                            let rec_mode = {
                                let ms = app_handle.state::<MurmurState>();
                                ms.settings
                                    .lock()
                                    .map(|s| s.recording_mode.clone())
                                    .unwrap_or_else(|_| "hold".to_string())
                            };

                            match rec_mode.as_str() {
                                "toggle" => {
                                    // Debounce: skip if last toggle was < 500ms ago
                                    if let Some(last) = last_toggle {
                                        if last.elapsed()
                                            < std::time::Duration::from_millis(500)
                                        {
                                            continue;
                                        }
                                    }
                                    last_toggle = Some(Instant::now());

                                    let murmur_state = app_handle.state::<MurmurState>();
                                    let current = murmur_state.app_state.current();
                                    if current == state::RecordingState::Recording {
                                        is_recording = false;
                                        if let Err(e) = do_stop_recording(&app_handle) {
                                            log::error!("failed to stop recording: {}", e);
                                            reset_to_idle(&murmur_state, &app_handle);
                                            let _ = app_handle
                                                .emit(events::RECORDING_ERROR, e.to_string());
                                            hide_preview_window(&app_handle);
                                            hide_main_window(&app_handle);
                                        }
                                    } else if current == state::RecordingState::Idle {
                                        match do_start_recording(&app_handle, mode) {
                                            Ok(()) => {
                                                is_recording = true;
                                                recording_started_at = Some(Instant::now());
                                            }
                                            Err(e) => {
                                                log::error!(
                                                    "failed to start recording: {}",
                                                    e
                                                );
                                                let _ = app_handle.emit(events::RECORDING_ERROR, e);
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    // Hold mode (default behavior)
                                    if is_recording {
                                        continue;
                                    }
                                    let murmur_state = app_handle.state::<MurmurState>();
                                    if murmur_state.app_state.current()
                                        != state::RecordingState::Idle
                                    {
                                        continue;
                                    }
                                    match do_start_recording(&app_handle, mode) {
                                        Ok(()) => {
                                            is_recording = true;
                                            recording_started_at = Some(Instant::now());
                                        }
                                        Err(e) => {
                                            log::error!("failed to start recording: {}", e);
                                            let _ = app_handle.emit(events::RECORDING_ERROR, e);
                                        }
                                    }
                                }
                            }
                        }
                        hotkey::HotkeyEvent::Released(_mode) => {
                            let rec_mode = {
                                let ms = app_handle.state::<MurmurState>();
                                ms.settings
                                    .lock()
                                    .map(|s| s.recording_mode.clone())
                                    .unwrap_or_else(|_| "hold".to_string())
                            };
                            if rec_mode == "toggle" || !is_recording {
                                continue;
                            }
                            is_recording = false;
                            if let Err(e) = do_stop_recording(&app_handle) {
                                log::error!("failed to stop recording: {}", e);
                                let murmur_state = app_handle.state::<MurmurState>();
                                reset_to_idle(&murmur_state, &app_handle);
                                let _ = app_handle.emit(events::RECORDING_ERROR, e.to_string());
                                hide_preview_window(&app_handle);
                                hide_main_window(&app_handle);
                            }
                        }
                        hotkey::HotkeyEvent::EscCancel => {
                            let murmur_state = app_handle.state::<MurmurState>();
                            if murmur_state.app_state.current()
                                != state::RecordingState::Recording
                            {
                                continue; // Only cancel during active recording
                            }

                            cancel_active_recording(&murmur_state);
                            is_recording = false;
                            recording_started_at = None;
                            reset_to_idle(&murmur_state, &app_handle);
                            let _ = app_handle.emit(events::RECORDING_CANCELLED, ());

                            // Hide windows after 2-second delay
                            let app_hide = app_handle.clone();
                            let gen =
                                murmur_state.preview_generation.load(Ordering::SeqCst);
                            std::thread::spawn(move || {
                                std::thread::sleep(std::time::Duration::from_secs(2));
                                let ms = app_hide.state::<MurmurState>();
                                if ms.preview_generation.load(Ordering::SeqCst) == gen {
                                    hide_main_window(&app_hide);
                                    hide_preview_window(&app_hide);
                                    ms.main_visible.store(false, Ordering::SeqCst);
                                }
                            });
                        }
                        hotkey::HotkeyEvent::EventTapFailed => unreachable!(),
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
