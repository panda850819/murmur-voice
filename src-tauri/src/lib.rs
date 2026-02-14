mod audio;
mod clipboard;
mod frontapp;
mod hotkey;
mod llm;
mod model;
mod settings;
mod state;
mod whisper;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use tauri::{Emitter, Manager};

pub(crate) struct MurmurState {
    app_state: state::AppState,
    recorder: Mutex<Option<audio::AudioRecorder>>,
    engine: Mutex<Option<whisper::TranscriptionEngine>>,
    settings: Mutex<settings::Settings>,
    live_stop: AtomicBool,
    /// Join handle for the live transcription thread so stop_recording can wait for it.
    live_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// Generation counter for preview auto-hide timer cancellation.
    /// Incremented on each new recording; stale timers compare and bail out.
    preview_generation: AtomicU64,
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
                let preview_h = 280.0;
                let gap = 8.0;
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

fn do_start_recording(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<MurmurState>();

    // Cancel any pending preview auto-hide timer
    state.preview_generation.fetch_add(1, Ordering::SeqCst);

    show_main_window(app);
    show_preview_window(app);

    state
        .app_state
        .transition(state::RecordingState::Starting)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("recording_state_changed", "starting");

    let mut recorder = audio::AudioRecorder::new();
    if let Err(e) = recorder.start() {
        let _ = state.app_state.transition(state::RecordingState::Idle);
        let _ = app.emit("recording_state_changed", "idle");
        let _ = app.emit("recording_error", e.to_string());
        hide_main_window(app);
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
    let _ = app.emit("recording_state_changed", "recording");

    // Auto-stop after 5 minutes in toggle mode
    let mode = state
        .settings
        .lock()
        .map(|s| s.recording_mode.clone())
        .unwrap_or_default();
    if mode == "toggle" {
        let app_timeout = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(300));
            let ms = app_timeout.state::<MurmurState>();
            if ms.app_state.current() == state::RecordingState::Recording {
                let _ = do_stop_recording(&app_timeout);
            }
        });
    }

    // Start live transcription thread (local engine only — Groq would be too expensive)
    let use_local_engine = state
        .settings
        .lock()
        .map(|s| s.engine != "groq")
        .unwrap_or(true);

    state.live_stop.store(false, Ordering::SeqCst);
    if use_local_engine {
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
                        Some(rec) => rec.peek_samples(),
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
                        None => break,
                    }
                };

                if ms.live_stop.load(Ordering::SeqCst) {
                    break;
                }

                if !text.is_empty() {
                    let _ = app_clone.emit("partial_transcription", &text);
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
    let _ = app.emit("recording_state_changed", "stopping");

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
        let _ = state.app_state.transition(state::RecordingState::Idle);
        let _ = app.emit("recording_state_changed", "idle");
        hide_main_window(app);
        hide_preview_window(app);
        return Ok(String::new());
    }

    let _ = state
        .app_state
        .transition(state::RecordingState::Transcribing);
    let _ = app.emit("recording_state_changed", "transcribing");

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
                "foreground_app_info",
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
        // Local Whisper
        let engine_lock = state
            .engine
            .lock()
            .map_err(|e| format!("engine mutex poisoned: {e}"))?;
        match engine_lock.as_ref() {
            Some(engine) => engine.transcribe(&samples, &language, &initial_prompt).map_err(|e| e.to_string())?,
            None => return Err("whisper engine not loaded".to_string()),
        }
    };

    // LLM post-processing (if enabled and API key present)
    let (llm_enabled, api_key, llm_model, app_aware_style) = state
        .settings
        .lock()
        .map(|s| (
            s.llm_enabled,
            s.groq_api_key.clone(),
            s.llm_model.clone(),
            s.app_aware_style,
        ))
        .unwrap_or_else(|_| (false, String::new(), String::new(), false));

    let text = if llm_enabled && !api_key.is_empty() && !raw_text.is_empty() {
        let _ = state
            .app_state
            .transition(state::RecordingState::Processing);
        let _ = app.emit("recording_state_changed", "processing");

        let style = if app_aware_style {
            frontapp::foreground_app_bundle_id()
                .as_deref()
                .map(frontapp::style_for_app)
                .unwrap_or("default")
        } else {
            "default"
        };

        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        match rt.block_on(llm::process_text(&api_key, &llm_model, &raw_text, style)) {
            Ok(processed) => processed,
            Err(e) => {
                log::error!("LLM post-processing failed: {}", e);
                let _ = app.emit("recording_error", format!("LLM processing failed, using raw text: {e}"));
                raw_text
            }
        }
    } else {
        raw_text
    };

    if !text.is_empty() {
        if let Err(e) = clipboard::insert_text(&text) {
            let _ = app.emit("recording_error", format!("clipboard error: {e}"));
            log::error!("failed to insert text: {}", e);
        }
    }

    let _ = state.app_state.transition(state::RecordingState::Idle);
    let _ = app.emit("recording_state_changed", "idle");
    let _ = app.emit("transcription_complete", &text);

    // Auto-hide main + preview windows after 3 seconds (minimum 1s display).
    // The generation counter cancels this timer if a new recording starts.
    let generation = state.preview_generation.load(Ordering::SeqCst);
    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        let ms = app_clone.state::<MurmurState>();
        if ms.preview_generation.load(Ordering::SeqCst) == generation {
            hide_preview_window(&app_clone);
            hide_main_window(&app_clone);
        }
    });

    Ok(text)
}

// --- Tauri Commands ---

#[tauri::command]
fn get_recording_state(state: tauri::State<'_, MurmurState>) -> String {
    state.app_state.current().to_string()
}

#[tauri::command]
fn is_model_ready() -> bool {
    model::is_model_ready()
}

#[tauri::command]
async fn download_model_cmd(app: tauri::AppHandle) -> Result<(), String> {
    let app_clone = app.clone();
    model::download_model(move |downloaded, total| {
        let _ = app_clone.emit(
            "model_download_progress",
            serde_json::json!({
                "downloaded": downloaded,
                "total": total,
            }),
        );
    })
    .await
    .map_err(|e| e.to_string())?;

    let _ = app.emit("model_ready", ());

    let murmur_state = app.state::<MurmurState>();
    let model_path_str = model::model_path()
        .to_str()
        .ok_or("invalid model path")?
        .to_string();
    let engine =
        whisper::TranscriptionEngine::new(&model_path_str).map_err(|e| e.to_string())?;
    let mut engine_lock = murmur_state
        .engine
        .lock()
        .map_err(|e| format!("engine mutex poisoned: {e}"))?;
    *engine_lock = Some(engine);

    Ok(())
}

#[tauri::command]
fn start_recording(app: tauri::AppHandle) -> Result<(), String> {
    do_start_recording(&app)
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
    // Apply hotkey change
    hotkey::set_hotkey_mask(new_settings.ptt_key_mask());

    // Apply window opacity
    if let Some(window) = app.get_webview_window("main") {
        let _ = app.emit("opacity_changed", new_settings.window_opacity);
        let _ = window.set_always_on_top(true);
    }

    // Persist
    settings::save_settings(&new_settings)?;

    // Update in-memory state
    if let Ok(mut s) = state.settings.lock() {
        *s = new_settings;
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
        settings::save_settings(&s)?;
    }
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
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
    let initial_settings = settings::load_settings();
    log::info!(
        "loaded settings: onboarding_complete={}, recording_mode={}",
        initial_settings.onboarding_complete,
        initial_settings.recording_mode
    );
    hotkey::set_hotkey_mask(initial_settings.ptt_key_mask());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MurmurState {
            app_state: state::AppState::new(),
            recorder: Mutex::new(None),
            engine: Mutex::new(None),
            settings: Mutex::new(initial_settings),
            live_stop: AtomicBool::new(false),
            live_thread: Mutex::new(None),
            preview_generation: AtomicU64::new(0),
        })
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
            complete_onboarding,
        ])
        .setup(|app| {
            // Create system tray with Settings + Quit
            let settings_item =
                tauri::menu::MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let quit =
                tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&settings_item, &quit])?;
            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&menu)
                .tooltip("Murmur Voice")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "settings" => open_settings(app.clone()),
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
                .inner_size(560.0, 480.0)
                .resizable(false)
                .center()
                .build();
            }

            // Position main window at bottom center
            if let Some(window) = app.get_webview_window("main") {
                if let Some(monitor) = window.current_monitor().ok().flatten() {
                    let screen = monitor.size();
                    let scale = monitor.scale_factor();
                    let win_w = 420.0;
                    let win_h = 48.0;
                    let margin = 80.0;
                    let x = (screen.width as f64 / scale - win_w) / 2.0;
                    let y = screen.height as f64 / scale - win_h - margin;
                    use tauri::PhysicalPosition;
                    let _ = window.set_position(PhysicalPosition::new(
                        (x * scale) as i32,
                        (y * scale) as i32,
                    ));
                }
            }

            // Position preview window above main bar (hidden by default)
            if let (Some(main_win), Some(preview_win)) = (
                app.get_webview_window("main"),
                app.get_webview_window("preview"),
            ) {
                if let Ok(main_pos) = main_win.outer_position() {
                    let preview_h = 280.0;
                    let gap = 8.0;
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

            // Load whisper engine if model exists
            if model::is_model_ready() {
                let load_result = model::model_path()
                    .to_str()
                    .map(whisper::TranscriptionEngine::new);

                match load_result {
                    Some(Ok(engine)) => {
                        let murmur_state = app.state::<MurmurState>();
                        match murmur_state.engine.lock() {
                            Ok(mut lock) => {
                                *lock = Some(engine);
                                log::info!("whisper engine loaded successfully");
                            }
                            Err(e) => {
                                log::error!("engine mutex poisoned during setup: {}", e);
                            }
                        };
                    }
                    Some(Err(e)) => {
                        log::error!("failed to load whisper engine: {}", e);
                    }
                    None => {
                        log::error!("model path contains invalid UTF-8");
                    }
                }
            }

            // Start hotkey listener
            let app_handle = app.handle().clone();
            let (sender, receiver) = std::sync::mpsc::channel();
            hotkey::start_listener(sender);

            std::thread::spawn(move || {
                let mut is_recording = false;
                while let Ok(event) = receiver.recv() {
                    let mode = {
                        let ms = app_handle.state::<MurmurState>();
                        ms.settings
                            .lock()
                            .map(|s| s.recording_mode.clone())
                            .unwrap_or_else(|_| "hold".to_string())
                    };

                    match event {
                        hotkey::HotkeyEvent::Pressed => {
                            match mode.as_str() {
                                "toggle" => {
                                    let murmur_state = app_handle.state::<MurmurState>();
                                    let current = murmur_state.app_state.current();
                                    if current == state::RecordingState::Recording {
                                        is_recording = false;
                                        if let Err(e) = do_stop_recording(&app_handle) {
                                            log::error!("failed to stop recording: {}", e);
                                            let _ = murmur_state
                                                .app_state
                                                .transition(state::RecordingState::Idle);
                                            let _ = app_handle
                                                .emit("recording_state_changed", "idle");
                                        }
                                    } else if current == state::RecordingState::Idle {
                                        match do_start_recording(&app_handle) {
                                            Ok(()) => {
                                                is_recording = true;
                                            }
                                            Err(e) => {
                                                log::error!(
                                                    "failed to start recording: {}",
                                                    e
                                                );
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
                                    match do_start_recording(&app_handle) {
                                        Ok(()) => {
                                            is_recording = true;
                                        }
                                        Err(e) => {
                                            log::error!("failed to start recording: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        hotkey::HotkeyEvent::Released => {
                            if mode == "toggle" || !is_recording {
                                continue;
                            }
                            is_recording = false;
                            if let Err(e) = do_stop_recording(&app_handle) {
                                log::error!("failed to stop recording: {}", e);
                                let murmur_state = app_handle.state::<MurmurState>();
                                let _ = murmur_state
                                    .app_state
                                    .transition(state::RecordingState::Idle);
                                let _ = app_handle.emit("recording_state_changed", "idle");
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
