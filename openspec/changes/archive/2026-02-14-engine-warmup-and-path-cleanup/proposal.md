## Why

PR #1 introduced GPU kernel warmup and model path improvements, but the implementation has several issues: warmup blocks the first recording synchronously, model path fallbacks silently use incorrect directories (`/tmp`, `C:\Users\Default`), warmup thread count is hardcoded to 4, model migration runs on every `is_model_ready()` call, and platform paths are manually assembled with `#[cfg]` instead of using Tauri's built-in path resolver.

## What Changes

- Move `TranscriptionEngine` initialization (including GPU warmup) to app startup in a background thread, so the engine is ready before the user's first recording
- Replace silent model path fallbacks with proper error propagation — if `HOME`/`APPDATA` is missing, report it instead of writing to volatile/inaccessible paths
- Use `std::thread::available_parallelism()` for warmup thread count instead of hardcoded 4
- Add `std::sync::Once` guard to `migrate_model_from_old_path()` so filesystem checks run only once per process
- Unify settings and model path resolution using Tauri's `app.path().app_data_dir()` to eliminate manual `#[cfg]` path assembly

## Capabilities

### New Capabilities

- `engine-lifecycle`: Covers background engine initialization, warmup strategy, and engine readiness signaling

### Modified Capabilities

- `model-management`: Model path resolution changes from manual `#[cfg]` + env vars to Tauri app data dir; fallback behavior changes from silent to error; migration gets `Once` guard
- `speech-transcription`: Warmup thread count changes from hardcoded to dynamic

## Impact

- `src-tauri/src/whisper.rs` — warmup thread count, engine init moved out of first-use
- `src-tauri/src/model.rs` — path resolution rewrite, migration guard, error handling
- `src-tauri/src/settings.rs` — path resolution rewrite to use Tauri app data dir
- `src-tauri/src/lib.rs` — background engine init on app startup, engine readiness state
