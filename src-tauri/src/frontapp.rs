#[cfg(target_os = "macos")]
#[path = "frontapp_macos.rs"]
mod platform;

#[cfg(target_os = "windows")]
#[path = "frontapp_windows.rs"]
mod platform;

pub(crate) use platform::*;
