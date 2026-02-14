## Context

PR #1 added GPU kernel warmup and Windows model path migration to the codebase. While functional, the implementation has quality issues: synchronous warmup blocks first recording, model paths use manual `#[cfg]` assembly with unsafe fallbacks, hardcoded thread count, and migration runs on every model check. This change addresses all five issues in one pass.

Current state:
- `TranscriptionEngine::new()` runs warmup synchronously — blocks caller
- `model_dir()` in `model.rs` uses `std::env::var_os("HOME"/"APPDATA")` with fallback to `/tmp` or `C:\Users\Default\AppData\Roaming`
- `settings_path()` in `settings.rs` uses similar manual path assembly
- `warmup()` hardcodes `set_n_threads(4)`
- `migrate_model_from_old_path()` runs filesystem checks on every `is_model_ready()` call

## Goals / Non-Goals

**Goals:**
- First recording has zero warmup delay (engine pre-initialized)
- Model and settings paths use a single Tauri-provided source of truth
- Path resolution failures are surfaced as errors, not silently swallowed
- Migration check runs once per process

**Non-Goals:**
- Changing the warmup strategy itself (1s dummy inference is fine)
- Adding CUDA support on Windows (out of scope, separate concern)
- Changing the model download flow

## Decisions

### D1: Background engine init via `tokio::spawn`

Engine initialization (model load + warmup) moves to a background task spawned during `setup()`. Store the engine in `MurmurState` wrapped in `tokio::watch` or `Option<Arc<TranscriptionEngine>>` behind a `Mutex`, set once init completes.

**Alternative**: `std::thread::spawn` — rejected because we already have tokio runtime and `spawn_blocking` integrates better with the async model download flow.

### D2: Tauri `app.path().app_data_dir()` for all paths

Replace `model_dir()` and `settings_path()` with a single helper that takes `AppHandle` and returns the platform-correct base directory. This requires passing `AppHandle` (or the resolved path) to `model.rs` and `settings.rs` functions that currently compute paths independently.

**Approach**: Add `pub(crate) fn app_data_base(app: &AppHandle) -> Result<PathBuf, _>` that calls `app.path().app_data_dir()`. Store the resolved path in `MurmurState` at startup so it's available without `AppHandle` in contexts like `is_model_ready()`.

**Alternative**: Keep `#[cfg]` paths but fix fallbacks — rejected because Tauri already handles this correctly and we're doubling the logic.

### D3: Error propagation instead of fallbacks

`model_dir()` and `settings_path()` return `Result<PathBuf>` instead of `PathBuf`. Callers handle the error (show UI message or log). No more silent `/tmp` fallbacks.

### D4: `Once` guard for migration

Wrap `migrate_model_from_old_path()` in `static MIGRATE: std::sync::Once = Once::new();` inside `is_model_ready()`.

### D5: `available_parallelism()` for thread count

Extract a helper `fn optimal_threads() -> i32` used by both `warmup()` and `transcribe()`:

```rust
fn optimal_threads() -> i32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(4)
}
```

## Risks / Trade-offs

- **[Risk] Engine not ready when user records immediately after launch** → Mitigation: Block on the engine future in `do_start_recording()` if not yet ready. Warmup is ~1s, model load is ~2s — acceptable one-time wait on cold start.
- **[Risk] `app_data_dir()` changes path format across Tauri versions** → Mitigation: Low risk; Tauri 2 path API is stable. Pin Tauri version in Cargo.toml.
- **[Risk] Existing settings.json at old path not migrated** → Out of scope for this change. Settings file is small and recreated on first launch if missing.
