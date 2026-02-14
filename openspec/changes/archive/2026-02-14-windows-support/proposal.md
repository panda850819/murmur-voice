## Why

Murmur Voice currently only runs on macOS due to platform-specific FFI code (CGEventTap for hotkey, NSWorkspace for foreground app detection, Cmd+V for paste). Adding Windows support doubles the potential user base and is straightforward since the core architecture (Tauri 2 + Rust) is already cross-platform. The main work is replacing 3 macOS-specific modules with platform-abstracted equivalents using `#[cfg(target_os)]` conditional compilation.

## What Changes

- **hotkey.rs**: Split into `hotkey_macos.rs` (existing CGEventTap) and `hotkey_windows.rs` (Windows low-level keyboard hook via `SetWindowsHookExW`). Shared `HotkeyEvent` enum and `set_hotkey_mask()` interface remain the same.
- **frontapp.rs**: Split into `frontapp_macos.rs` (existing NSWorkspace FFI) and `frontapp_windows.rs` (Windows `GetForegroundWindow` + `GetWindowThreadProcessId` to get executable name). Same `foreground_app_bundle_id()` and `style_for_app()` interface.
- **clipboard.rs**: Change `simulate_paste()` to use `Ctrl+V` on Windows instead of `Cmd+V`. The `rdev` crate already supports both platforms — only the key sequence changes.
- **whisper.rs**: Disable Metal feature on Windows, fall back to CPU (or CUDA if available). Feature flag `metal` becomes macOS-only in `Cargo.toml`.
- **settings.rs**: Windows PTT key mapping uses virtual key codes instead of CGEventFlags. Add Windows-specific mask values.
- **Cargo.toml**: Platform-conditional dependencies (`whisper-rs` features, potential `windows` crate for FFI).
- **tauri.conf.json**: Add Windows bundle configuration (`.msi` / `.exe` installer).

## Capabilities

### New Capabilities

- `windows-hotkey`: Global hotkey detection on Windows using low-level keyboard hooks (`SetWindowsHookExW` + `WH_KEYBOARD_LL`)
- `windows-foreground-app`: Foreground application detection on Windows for app-aware style feature
- `cross-platform-build`: Platform-conditional compilation setup and Windows build/bundle configuration

### Modified Capabilities

- `push-to-talk`: PTT key mask mapping extended with Windows virtual key codes
- `text-insertion`: Paste simulation changes from Cmd+V to Ctrl+V on Windows
- `speech-transcription`: Whisper backend switches from Metal to CPU/CUDA on Windows

## Impact

- **Files changed**: hotkey.rs (split), frontapp.rs (split), clipboard.rs (cfg gate), whisper.rs (feature gate), settings.rs (key mapping), Cargo.toml, tauri.conf.json
- **New files**: hotkey_macos.rs, hotkey_windows.rs, frontapp_macos.rs, frontapp_windows.rs
- **Dependencies**: Potential `windows` crate for Windows FFI (or raw FFI like macOS approach)
- **Build**: GitHub Actions CI needs Windows runner for cross-platform testing
- **No breaking changes** to macOS functionality — all changes are additive via cfg gates
