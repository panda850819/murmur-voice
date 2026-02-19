mod audio;
mod clipboard;
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

    // Start live transcription thread (local engine only — Groq would be too expensive).
    // Requires GPU acceleration (Metal on macOS, CUDA on Windows) to be fast enough.
    let enable_live_preview = state
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
                let model_path = model::model_path(&state.app_data_dir);
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
        "transcription_engine_info",
        serde_json::json!({
            "engine": &engine_type,
            "local": engine_type != "groq",
        }),
    );

    // LLM post-processing via TextEnhancer trait
    let (enhancer, app_aware_style) = {
        let s = state
            .settings
            .lock()
            .map_err(|e| format!("settings mutex poisoned: {e}"))?;
        (llm::create_enhancer(&s), s.app_aware_style)
    };

    eprintln!("[whisper raw] {}", raw_text);

    let text = if let Some(enhancer) = enhancer {
        if raw_text.is_empty() {
            raw_text
        } else {
            let _ = state
                .app_state
                .transition(state::RecordingState::Processing);
            let _ = app.emit("recording_state_changed", "processing");

            let _ = app.emit(
                "enhancer_info",
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
                    eprintln!("[llm output] {}", processed);
                    processed
                }
                Err(e) => {
                    log::error!("LLM post-processing failed: {}", e);
                    let _ = app.emit(
                        "recording_error",
                        format!("LLM processing failed, using raw text: {e}"),
                    );
                    raw_text
                }
            }
        }
    } else {
        raw_text
    };

    // Detect if foreground app can accept paste (default: true, only false for Desktop/Finder)
    let has_input = if !text.is_empty() {
        std::panic::catch_unwind(frontapp::has_focused_text_input).unwrap_or(true)
    } else {
        false
    };
    let output_mode = if !text.is_empty() {
        if has_input {
            // Auto-paste mode: save clipboard → paste → restore
            if let Err(e) = clipboard::insert_text(&text) {
                let _ = app.emit("recording_error", format!("clipboard error: {e}"));
                log::error!("failed to insert text: {}", e);
            }
            "pasted"
        } else {
            // Clipboard-only mode: just copy, no paste simulation
            if let Err(e) = clipboard::copy_only(&text) {
                let _ = app.emit("recording_error", format!("clipboard error: {e}"));
                log::error!("failed to copy text: {}", e);
            }
            "clipboard"
        }
    } else {
        "pasted" // empty text, doesn't matter
    };

    let _ = state.app_state.transition(state::RecordingState::Idle);
    let _ = app.emit("recording_state_changed", "idle");
    let _ = app.emit(
        "transcription_complete",
        serde_json::json!({ "text": text, "mode": output_mode }),
    );

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
    model::is_model_ready(&state.app_data_dir)
}

#[tauri::command]
async fn download_model_cmd(app: tauri::AppHandle) -> Result<(), String> {
    let murmur_state = app.state::<MurmurState>();
    let base = murmur_state.app_data_dir.clone();

    let app_clone = app.clone();
    model::download_model(&base, move |downloaded, total| {
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

    // Spawn background engine init (don't block the command)
    {
        let init_done = &murmur_state.engine_init_done;
        if let Ok(mut done) = init_done.0.lock() {
            *done = false; // Mark as pending — background thread will set true
        }
    }
    let app_handle = app.clone();
    let model_path = model::model_path(&base);
    std::thread::spawn(move || {
        let model_path_str = match model_path.to_str() {
            Some(s) => s.to_string(),
            None => {
                log::error!("model path contains invalid UTF-8");
                signal_engine_init_done(&app_handle);
                return;
            }
        };
        match whisper::TranscriptionEngine::new(&model_path_str) {
            Ok(engine) => {
                let ms = app_handle.state::<MurmurState>();
                if let Ok(mut lock) = ms.engine.lock() {
                    *lock = Some(engine);
                }
                signal_engine_init_done(&app_handle);
                log::info!("whisper engine loaded after download");
            }
            Err(e) => {
                log::error!("engine init after download failed: {}", e);
                signal_engine_init_done(&app_handle);
            }
        }
    });

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
    let t = new_settings.ptt_key_target();
    hotkey::set_hotkey_target(t.modifier_mask, t.regular_key);

    // Apply window opacity
    if let Some(window) = app.get_webview_window("main") {
        let _ = app.emit("opacity_changed", new_settings.window_opacity);
        let _ = window.set_always_on_top(true);
    }

    // Persist
    settings::save_settings(&new_settings, &state.app_data_dir)?;

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
        settings::save_settings(&s, &state.app_data_dir)?;
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
fn hide_overlay_windows(app: tauri::AppHandle) {
    hide_preview_window(&app);
    hide_main_window(&app);
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    clipboard::copy_only(&text).map_err(|e| e.to_string())
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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            add_dictionary_term,
            check_for_updates,
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
            let t = initial_settings.ptt_key_target();
            hotkey::set_hotkey_target(t.modifier_mask, t.regular_key);

            // Register MurmurState with the resolved app_data_dir.
            // engine_init_done starts as `true` if model is not ready (nothing to wait for),
            // `false` if model exists (background thread will set it on completion).
            let model_ready = model::is_model_ready(&app_data_dir);
            app.manage(MurmurState {
                app_data_dir: app_data_dir.clone(),
                app_state: state::AppState::new(),
                recorder: Mutex::new(None),
                engine: Mutex::new(None),
                engine_init_done: (Mutex::new(!model_ready), std::sync::Condvar::new()),
                settings: Mutex::new(initial_settings),
                live_stop: AtomicBool::new(false),
                live_thread: Mutex::new(None),
                preview_generation: AtomicU64::new(0),
            });

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

            // Load whisper engine in background thread (non-blocking startup)
            if model_ready {
                let app_handle = app.handle().clone();
                let model_path = model::model_path(&app_data_dir);
                std::thread::spawn(move || {
                    let model_path_str = match model_path.to_str() {
                        Some(s) => s.to_string(),
                        None => {
                            log::error!("model path contains invalid UTF-8");
                            signal_engine_init_done(&app_handle);
                            return;
                        }
                    };
                    match whisper::TranscriptionEngine::new(&model_path_str) {
                        Ok(engine) => {
                            let ms = app_handle.state::<MurmurState>();
                            if let Ok(mut lock) = ms.engine.lock() {
                                *lock = Some(engine);
                            }
                            signal_engine_init_done(&app_handle);
                            log::info!("whisper engine loaded in background");
                        }
                        Err(e) => {
                            log::error!("background engine init failed: {}", e);
                            signal_engine_init_done(&app_handle);
                        }
                    }
                });
            }

            // Start hotkey listener
            let app_handle = app.handle().clone();
            let (sender, receiver) = std::sync::mpsc::channel();
            hotkey::start_listener(sender);

            std::thread::spawn(move || {
                let mut is_recording = false;
                let mut last_toggle: Option<Instant> = None;
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
                                            let _ = murmur_state
                                                .app_state
                                                .transition(state::RecordingState::Idle);
                                            let _ = app_handle
                                                .emit("recording_state_changed", "idle");
                                            let _ = app_handle
                                                .emit("recording_error", e.to_string());
                                            hide_preview_window(&app_handle);
                                            hide_main_window(&app_handle);
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
                                let _ = app_handle.emit("recording_error", e.to_string());
                                hide_preview_window(&app_handle);
                                hide_main_window(&app_handle);
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
