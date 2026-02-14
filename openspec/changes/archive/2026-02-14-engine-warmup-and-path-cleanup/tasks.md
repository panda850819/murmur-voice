## 1. Dynamic thread count
- [x] 1.1 Add `fn optimal_threads() -> i32` helper in `whisper.rs` using `std::thread::available_parallelism()` with fallback to 4
- [x] 1.2 Replace hardcoded `set_n_threads(4)` in `warmup()` with `optimal_threads()`
- [x] 1.3 Replace hardcoded `set_n_threads(4)` in `transcribe()` with `optimal_threads()`

## 2. Tauri app data dir for paths
- [x] 2.1 Add `app_data_dir: PathBuf` field to `MurmurState` in `lib.rs`
- [x] 2.2 Resolve `app.path().app_data_dir()` in `setup()` and store in `MurmurState`
- [x] 2.3 Rewrite `model_dir()` in `model.rs` to accept a `base: &Path` parameter instead of using `#[cfg]` + env vars; return `Result<PathBuf>`
- [x] 2.4 Rewrite `model_path()`, `is_model_ready()`, `download_model()` to accept `base: &Path` and propagate errors
- [x] 2.5 Rewrite `settings_path()` in `settings.rs` to accept a `base: &Path` parameter; return `Result<PathBuf>`
- [x] 2.6 Update `load_settings()` and `save_settings()` to accept `base: &Path` and propagate path errors
- [x] 2.7 Update all callers in `lib.rs` to pass `app_data_dir` from `MurmurState` to model/settings functions
- [x] 2.8 Remove all `#[cfg]` path assembly and env var lookups from `model.rs` and `settings.rs`

## 3. Migration Once guard
- [x] 3.1 Add `static MIGRATE: std::sync::Once = Once::new()` in `model.rs`
- [x] 3.2 Wrap `migrate_model_from_old_path()` call inside `MIGRATE.call_once(|| ...)` in `is_model_ready()`

## 4. Background engine initialization
- [x] 4.1 Move engine loading from synchronous `setup()` block to `std::thread::spawn` in `setup()`
- [x] 4.2 Use `(Mutex<bool>, Condvar)` to signal when engine is ready
- [x] 4.3 In `do_start_recording()` and live transcription thread, wait for engine readiness instead of assuming `engine` is Some
- [x] 4.4 On background init failure, log error and retry on first recording attempt
- [x] 4.5 Remove synchronous engine init from `download_model_cmd()` — it signals the background init path instead
