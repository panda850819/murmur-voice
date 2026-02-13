use std::ffi::CStr;

/// Returns the bundle identifier of the current foreground application on macOS.
///
/// Uses NSWorkspace.sharedWorkspace.frontmostApplication.bundleIdentifier via Objective-C FFI.
pub(crate) fn foreground_app_bundle_id() -> Option<String> {
    unsafe {
        // Get NSWorkspace class
        let class_name = c"NSWorkspace";
        let ns_workspace_class = objc_getClass(class_name.as_ptr());
        if ns_workspace_class.is_null() {
            return None;
        }

        // [NSWorkspace sharedWorkspace]
        let sel_shared = sel_registerName(c"sharedWorkspace".as_ptr());
        let workspace: *mut Object = objc_msgSend(ns_workspace_class as *mut Object, sel_shared);
        if workspace.is_null() {
            return None;
        }

        // [workspace frontmostApplication]
        let sel_frontmost = sel_registerName(c"frontmostApplication".as_ptr());
        let app: *mut Object = objc_msgSend(workspace, sel_frontmost);
        if app.is_null() {
            return None;
        }

        // [app bundleIdentifier]
        let sel_bundle_id = sel_registerName(c"bundleIdentifier".as_ptr());
        let ns_string: *mut Object = objc_msgSend(app, sel_bundle_id);
        if ns_string.is_null() {
            return None;
        }

        // [nsString UTF8String]
        let sel_utf8 = sel_registerName(c"UTF8String".as_ptr());
        let msg_send_str: unsafe extern "C" fn(*mut Object, Sel) -> *const std::ffi::c_char =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn(*mut Object, Sel) -> *mut Object);
        let utf8_ptr: *const std::ffi::c_char = msg_send_str(ns_string, sel_utf8);
        if utf8_ptr.is_null() {
            return None;
        }

        let c_str = CStr::from_ptr(utf8_ptr);
        c_str.to_str().ok().map(String::from)
    }
}

/// Maps a foreground app's bundle ID to a human-readable display name.
pub(crate) fn display_name_for_app(bundle_id: &str) -> &'static str {
    match bundle_id {
        "com.apple.mail" => "Mail",
        "com.microsoft.Outlook" => "Outlook",
        "com.google.Gmail" => "Gmail",
        "com.tinyspeck.slackmacgap" => "Slack",
        "com.apple.MobileSMS" => "Messages",
        "com.facebook.archon" => "Messenger",
        "ru.keepcoder.Telegram" => "Telegram",
        "net.whatsapp.WhatsApp" => "WhatsApp",
        "com.hnc.Discord" => "Discord",
        "com.microsoft.VSCode" => "VS Code",
        "com.apple.dt.Xcode" => "Xcode",
        "com.jetbrains.intellij" => "IntelliJ",
        "dev.zed.Zed" => "Zed",
        "com.sublimetext.4" => "Sublime Text",
        "com.todesktop.230313mzl4w4u92" => "Cursor",
        "com.googlecode.iterm2" => "iTerm2",
        "com.apple.Terminal" => "Terminal",
        _ => "Unknown",
    }
}

/// Maps a foreground app's bundle ID to a writing style preset.
pub(crate) fn style_for_app(bundle_id: &str) -> &'static str {
    match bundle_id {
        // Email
        "com.apple.mail" | "com.microsoft.Outlook" | "com.google.Gmail" => "formal",

        // Chat / Messaging
        "com.tinyspeck.slackmacgap"
        | "com.apple.MobileSMS"
        | "com.facebook.archon"       // Messenger
        | "ru.keepcoder.Telegram"
        | "net.whatsapp.WhatsApp"
        | "com.hnc.Discord" => "casual",

        // Code editors
        "com.microsoft.VSCode"
        | "com.apple.dt.Xcode"
        | "com.jetbrains.intellij"
        | "dev.zed.Zed"
        | "com.sublimetext.4"
        | "com.todesktop.230313mzl4w4u92" // Cursor
        | "com.googlecode.iterm2"
        | "com.apple.Terminal" => "technical",

        _ => "default",
    }
}

// --- Raw Objective-C FFI bindings ---

#[repr(C)]
struct Object {
    _private: [u8; 0],
}

extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *const Object;
    fn sel_registerName(name: *const std::ffi::c_char) -> Sel;
    fn objc_msgSend(obj: *mut Object, sel: Sel) -> *mut Object;
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Sel {
    _ptr: *const std::ffi::c_void,
}
