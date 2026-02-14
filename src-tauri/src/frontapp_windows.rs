use windows::core::PWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// Returns the executable name (e.g. "Code.exe") of the current foreground application on Windows.
///
/// Uses GetForegroundWindow → GetWindowThreadProcessId → OpenProcess → QueryFullProcessImageNameW.
pub(crate) fn foreground_app_bundle_id() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_FORMAT(0), PWSTR(buf.as_mut_ptr()), &mut len);
        let _ = CloseHandle(process);

        if ok.is_err() {
            return None;
        }

        let full_path = String::from_utf16_lossy(&buf[..len as usize]);
        // Extract just the filename from the full path
        full_path
            .rsplit('\\')
            .next()
            .map(|s| s.to_string())
    }
}

/// Maps a foreground app's exe name to a human-readable display name.
pub(crate) fn display_name_for_app(exe_name: &str) -> &'static str {
    match exe_name.to_lowercase().as_str() {
        // Email
        "outlook.exe" => "Outlook",
        // Chat / Messaging
        "slack.exe" => "Slack",
        "telegram.exe" => "Telegram",
        "whatsapp.exe" => "WhatsApp",
        "discord.exe" => "Discord",
        "teams.exe" | "ms-teams.exe" => "Teams",
        // Code editors
        "code.exe" => "VS Code",
        "devenv.exe" => "Visual Studio",
        "idea64.exe" | "idea.exe" => "IntelliJ",
        "cursor.exe" => "Cursor",
        "sublime_text.exe" => "Sublime Text",
        "windowsterminal.exe" | "wt.exe" => "Windows Terminal",
        "cmd.exe" => "Command Prompt",
        "powershell.exe" | "pwsh.exe" => "PowerShell",
        // Browsers
        "chrome.exe" => "Chrome",
        "firefox.exe" => "Firefox",
        "msedge.exe" => "Edge",
        _ => "Unknown",
    }
}

/// Maps a foreground app's exe name to a writing style preset.
pub(crate) fn style_for_app(exe_name: &str) -> &'static str {
    match exe_name.to_lowercase().as_str() {
        // Email
        "outlook.exe" => "formal",

        // Chat / Messaging
        "slack.exe" | "telegram.exe" | "whatsapp.exe" | "discord.exe" | "teams.exe"
        | "ms-teams.exe" => "casual",

        // Code editors / terminals
        "code.exe" | "devenv.exe" | "idea64.exe" | "idea.exe" | "cursor.exe"
        | "sublime_text.exe" | "windowsterminal.exe" | "wt.exe" | "cmd.exe"
        | "powershell.exe" | "pwsh.exe" => "technical",

        _ => "default",
    }
}
