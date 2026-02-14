## ADDED Requirements

### Requirement: Platform-conditional compilation
The system SHALL use `#[cfg(target_os = "macos")]` and `#[cfg(target_os = "windows")]` to select platform-specific implementations at compile time. Shared interfaces (trait definitions, type aliases, re-exports) SHALL reside in thin dispatcher modules (e.g., `hotkey.rs`, `frontapp.rs`) that re-export the platform-specific implementation.

#### Scenario: macOS build includes macOS modules only
- **WHEN** the project is compiled on macOS
- **THEN** only `hotkey_macos.rs` and `frontapp_macos.rs` are compiled; Windows modules are excluded

#### Scenario: Windows build includes Windows modules only
- **WHEN** the project is compiled on Windows
- **THEN** only `hotkey_windows.rs` and `frontapp_windows.rs` are compiled; macOS modules are excluded

### Requirement: Platform-conditional Cargo dependencies
The system SHALL use `[target.'cfg(target_os = "macos")'.dependencies]` and `[target.'cfg(target_os = "windows")'.dependencies]` in `Cargo.toml` to declare platform-specific dependencies. The `whisper-rs` `metal` feature SHALL only be enabled on macOS.

#### Scenario: Metal feature on macOS
- **WHEN** building on macOS
- **THEN** `whisper-rs` is compiled with the `metal` feature enabled

#### Scenario: No Metal feature on Windows
- **WHEN** building on Windows
- **THEN** `whisper-rs` is compiled without the `metal` feature (CPU-only)

#### Scenario: Windows crate only on Windows
- **WHEN** building on Windows
- **THEN** the `windows` crate is included as a dependency with the required feature flags

### Requirement: Windows bundle configuration
The system SHALL include Windows bundle configuration in `tauri.conf.json` to produce `.msi` and/or `.exe` installers via the Tauri bundler.

#### Scenario: Build Windows installer
- **WHEN** `pnpm tauri build` is run on Windows
- **THEN** a Windows installer (`.msi` or `.exe`) is produced in the build output directory

### Requirement: Consistent shared interface
The system SHALL expose the same public function signatures for hotkey and foreground app detection on both platforms, so that callers (e.g., `lib.rs`) do not need platform-conditional code.

#### Scenario: Caller uses same API on both platforms
- **WHEN** `lib.rs` calls the hotkey or frontapp module
- **THEN** the same function names and types are used regardless of platform
