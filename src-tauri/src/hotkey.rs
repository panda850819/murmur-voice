#[cfg(target_os = "macos")]
#[path = "hotkey_macos.rs"]
mod platform;

#[cfg(target_os = "windows")]
#[path = "hotkey_windows.rs"]
mod platform;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "hotkey_linux.rs"]
mod platform;

pub(crate) use platform::*;
