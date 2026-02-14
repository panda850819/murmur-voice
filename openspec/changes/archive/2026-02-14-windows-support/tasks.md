## 1. Project Structure & Build Configuration

- [ ] 1.1 Update `Cargo.toml` with platform-conditional dependencies: `whisper-rs/metal` only on macOS, `windows` crate only on Windows (features: `Win32_UI_WindowsAndMessaging`, `Win32_System_Threading`, `Win32_Foundation`)
- [ ] 1.2 Add Windows bundle configuration to `tauri.conf.json` (`.msi` / `.exe` targets, Windows icon)
- [ ] 1.3 Create module dispatcher files: `hotkey.rs` re-exports from `hotkey_macos.rs` or `hotkey_windows.rs` via `#[cfg]`; same pattern for `frontapp.rs`

## 2. Hotkey Module Split

- [ ] 2.1 Create `hotkey_macos.rs` — move existing CGEventTap code from `hotkey.rs`, keep same public API (`HotkeyEvent`, `start_listener`, `set_hotkey_mask`)
- [ ] 2.2 Create `hotkey_windows.rs` — implement `SetWindowsHookExW` + `WH_KEYBOARD_LL` hook with `GetMessageW` loop, same public API as macOS
- [ ] 2.3 Update `hotkey.rs` to be a thin dispatcher that re-exports from the platform-specific module
- [ ] 2.4 Verify macOS build still compiles and hotkey works after split

## 3. Foreground App Module Split

- [ ] 3.1 Create `frontapp_macos.rs` — move existing NSWorkspace FFI code from `frontapp.rs`, keep same public API (`foreground_app_bundle_id`, `style_for_app`, `display_name_for_app`)
- [ ] 3.2 Create `frontapp_windows.rs` — implement `GetForegroundWindow` + `GetWindowThreadProcessId` + `QueryFullProcessImageNameW` to get exe name, map exe names to styles
- [ ] 3.3 Update `frontapp.rs` to be a thin dispatcher that re-exports from the platform-specific module
- [ ] 3.4 Verify macOS build still compiles after split

## 4. Text Insertion (Clipboard)

- [ ] 4.1 Add `#[cfg]` gate in `clipboard.rs` to switch paste simulation from `Key::MetaLeft` + `Key::KeyV` (macOS) to `Key::ControlLeft` + `Key::KeyV` (Windows)
- [ ] 4.2 Verify `rdev` and `arboard` crate compatibility on Windows target

## 5. Speech Transcription (Whisper)

- [ ] 5.1 Gate `whisper-rs` Metal feature behind `#[cfg(target_os = "macos")]` in `Cargo.toml`
- [ ] 5.2 Verify `whisper.rs` compiles on both platforms (no Metal-specific API calls outside cfg gates)

## 6. Settings & PTT Key Mapping

- [ ] 6.1 Add Windows virtual key code mapping in `settings.rs` `ptt_key_mask()` — map `AltLeft` → `VK_LMENU`, `MetaLeft` → `VK_LWIN`, etc. behind `#[cfg(target_os = "windows")]`
- [ ] 6.2 Ensure `set_hotkey_mask()` uses `AtomicU64` consistently on both platforms

## 7. Verification & CI

- [ ] 7.1 Run `cargo check` on macOS to verify zero regressions after all module splits
- [ ] 7.2 Run `cargo clippy` and `cargo test` on macOS to verify existing tests pass
- [ ] 7.3 Cross-compile check for Windows target: `cargo check --target x86_64-pc-windows-msvc` (if toolchain available) or verify all `#[cfg]` gates are consistent
- [ ] 7.4 Add GitHub Actions CI workflow for Windows build (`.github/workflows/ci.yml`)
