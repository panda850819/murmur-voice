## Context

Murmur Voice is a Tauri 2 desktop app with a Rust backend. The core pipeline (audio capture → transcription → LLM processing → paste) is platform-agnostic, but three modules use macOS-specific FFI:

1. **hotkey.rs** — CGEventTap (CoreGraphics) for global modifier key detection
2. **frontapp.rs** — NSWorkspace (Objective-C) for foreground app bundle ID
3. **clipboard.rs** — rdev `Key::MetaLeft` for Cmd+V paste simulation

Additionally, `whisper-rs` uses the `metal` feature for GPU acceleration, which is macOS-only.

The app uses Tauri 2 which already supports Windows builds. The frontend (vanilla HTML/JS/CSS) requires zero changes.

## Goals / Non-Goals

**Goals:**
- Windows 10/11 support with full feature parity (hotkey, transcription, LLM, paste, app-aware style)
- Zero regression on macOS — existing code wrapped in `#[cfg(target_os = "macos")]`, not rewritten
- Single codebase, no forks — platform differences handled via conditional compilation
- Windows installer output (`.msi` or `.exe` via Tauri bundler)

**Non-Goals:**
- Linux support (future work, similar pattern)
- CUDA GPU acceleration on Windows (CPU-only Whisper for now, CUDA can be added later)
- Windows ARM support
- Code signing / Microsoft Store distribution (future work)
- Rewriting macOS code to use cross-platform abstractions — keep raw FFI approach for both platforms

## Decisions

### Decision 1: Module split strategy — separate files per platform

**Choice**: Split `hotkey.rs` → `hotkey_macos.rs` + `hotkey_windows.rs`, same for `frontapp.rs`. Keep a thin `hotkey.rs` that re-exports via `#[cfg]`.

**Why**: Cleaner than `#[cfg]` blocks scattered within a single file. Each platform file is self-contained and testable independently. Matches how Rust ecosystem handles platform code (e.g., `std::sys`).

**Alternative rejected**: Single file with inline `#[cfg]` — gets messy fast with 100+ lines of FFI per platform.

### Decision 2: Windows hotkey — raw `SetWindowsHookExW` via `windows` crate

**Choice**: Use the `windows` crate (Microsoft's official Rust bindings) with `SetWindowsHookExW` + `WH_KEYBOARD_LL` for low-level keyboard hook.

**Why**:
- `SetWindowsHookExW` is the direct equivalent of macOS CGEventTap — system-wide, low-level, works with modifier keys
- The `windows` crate provides safe-ish typed bindings without manual FFI
- Same event model: callback fires on every key event, we filter for our PTT modifier
- Requires a message pump (`GetMessageW` loop) similar to macOS `CFRunLoopRun`

**Alternative rejected**: `rdev` crate for cross-platform listening — would simplify code but `rdev` on Windows uses `SetWindowsHookEx` internally anyway, and we need fine-grained control over modifier key flags (left vs right). Going direct gives us the same specificity as the macOS implementation.

### Decision 3: Windows foreground app — `GetForegroundWindow` + process name

**Choice**: Use `GetForegroundWindow` → `GetWindowThreadProcessId` → `OpenProcess` → `QueryFullProcessImageNameW` to get the executable path, then extract the exe name.

**Why**: Windows doesn't have bundle IDs like macOS. The executable name (e.g., `Code.exe`, `slack.exe`) is the closest equivalent. We map exe names to style presets in `style_for_app()`.

**Alternative rejected**: Window class names — inconsistent across app versions and not human-readable.

### Decision 4: Windows PTT key mapping — virtual key codes

**Choice**: Map PTT keys to Windows virtual key codes (`VK_LMENU`, `VK_RMENU`, `VK_LWIN`, `VK_RWIN`, etc.) in `settings.rs`. The JS `event.code` format (`AltLeft`, `ShiftRight`) remains the canonical storage format — backend translates to platform-specific values.

**Why**: Settings UI already captures `event.code` which is cross-platform. Only the backend mask conversion differs per platform.

### Decision 5: Whisper on Windows — CPU-only initially

**Choice**: Disable `metal` feature on Windows via platform-conditional Cargo features. Whisper runs on CPU.

**Why**:
- Metal is macOS-only
- CUDA support requires users to have NVIDIA GPU + CUDA toolkit installed — too much friction for v1
- CPU Whisper on modern hardware (i7/Ryzen 5+) handles ~10s audio in 2-5s, acceptable for dictation
- Groq cloud engine is available as the fast alternative on any platform

### Decision 6: Clipboard paste — Ctrl+V on Windows

**Choice**: In `clipboard.rs`, use `#[cfg]` to switch between `Key::MetaLeft` + `Key::KeyV` (macOS) and `Key::ControlLeft` + `Key::KeyV` (Windows).

**Why**: Smallest change. `rdev` and `arboard` both support Windows already. Only the paste key combo differs.

## Risks / Trade-offs

- **[No GPU on Windows]** → CPU Whisper is slower (2-5s vs <1s with Metal). Mitigation: Groq cloud engine works identically on both platforms, recommend it for Windows users.
- **[Testing on Windows]** → Primary development is on macOS. Mitigation: GitHub Actions Windows runner for CI; manual testing in Windows VM/machine.
- **[`windows` crate size]** → Microsoft's `windows` crate can bloat compile times. Mitigation: Import only the specific features needed (`Win32_UI_WindowsAndMessaging`, `Win32_System_Threading`, etc.).
- **[Anti-virus false positives]** → `SetWindowsHookEx` and process querying may trigger AV alerts. Mitigation: Code signing (future), documentation for users.
- **[Accessibility not needed on Windows]** → Unlike macOS, Windows low-level keyboard hooks don't require special permissions. One less setup step.
