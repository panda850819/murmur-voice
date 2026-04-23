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
            std::mem::transmute(
                objc_msgSend as unsafe extern "C" fn(*mut Object, Sel) -> *mut Object,
            );
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

/// Returns true if the foreground app can likely accept pasted text.
///
/// Default is true (auto-paste). Returns false only when we can confirm the
/// foreground app is a context where paste makes no sense (e.g. Desktop/Finder
/// with no window, or AX query fails suggesting no active UI).
pub(crate) fn has_focused_text_input() -> bool {
    // Check the foreground app — skip paste only for known non-input contexts
    let bundle = foreground_app_bundle_id();
    match bundle.as_deref() {
        // Finder: only skip paste if there's no focused element (user is on Desktop)
        Some("com.apple.finder") => unsafe { finder_has_input_focus() },
        // No foreground app detected
        None => false,
        // All other apps: assume they can accept paste (terminals, editors, browsers, etc.)
        Some(_) => true,
    }
}

/// For Finder specifically, check if there's a text input focused (e.g. rename dialog).
/// Returns false when user is on the Desktop or browsing files without a text field.
unsafe fn finder_has_input_focus() -> bool {
    let system = AXUIElementCreateSystemWide();
    if system.is_null() {
        return false;
    }

    let attr_focused = cf_str(c"AXFocusedUIElement");
    if attr_focused.is_null() {
        CFRelease(system);
        return false;
    }

    let mut focused_element: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(system, attr_focused, &mut focused_element);
    CFRelease(attr_focused);
    CFRelease(system);

    if err != K_AX_ERROR_SUCCESS || focused_element.is_null() {
        return false;
    }

    // Check if the focused element is a text field
    let attr_role = cf_str(c"AXRole");
    if attr_role.is_null() {
        CFRelease(focused_element);
        return false;
    }
    let mut role_value: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(focused_element, attr_role, &mut role_value);
    CFRelease(attr_role);
    CFRelease(focused_element);

    if err != K_AX_ERROR_SUCCESS || role_value.is_null() {
        return false;
    }

    let role = cfstring_to_string(role_value);
    CFRelease(role_value);

    matches!(
        role.as_deref(),
        Some("AXTextField" | "AXTextArea" | "AXSearchField" | "AXComboBox")
    )
}

// --- Accessibility API helpers ---

type CFTypeRef = *const std::ffi::c_void;
type AXUIElementRef = CFTypeRef;
type CFStringRef = CFTypeRef;
type AXError = i32;
const K_AX_ERROR_SUCCESS: AXError = 0;

unsafe fn cf_str(s: &CStr) -> CFStringRef {
    extern "C" {
        fn CFStringCreateWithCString(
            alloc: CFTypeRef,
            c_str: *const std::ffi::c_char,
            encoding: u32,
        ) -> CFStringRef;
    }
    CFStringCreateWithCString(std::ptr::null(), s.as_ptr(), 0x0600_0100) // kCFStringEncodingUTF8
}

unsafe fn cfstring_to_string(cfstr: CFStringRef) -> Option<String> {
    let mut buf = [0i8; 256];
    if CFStringGetCString(cfstr, buf.as_mut_ptr(), buf.len() as i64, 0x0600_0100) {
        CStr::from_ptr(buf.as_ptr()).to_str().ok().map(String::from)
    } else {
        None
    }
}

/// Cached AVFoundation symbols loaded once via `dlopen`/`dlsym`.
/// Avoids re-loading the framework on every `is_microphone_authorized()` call
/// (which runs on a 3s polling loop during onboarding).
struct AvFoundationSymbols {
    av_capture_device_class: *const Object,
    av_media_type_audio: CFTypeRef,
}

// SAFETY: These are framework constants valid for the process lifetime.
unsafe impl Send for AvFoundationSymbols {}
unsafe impl Sync for AvFoundationSymbols {}

static AV_FOUNDATION: std::sync::OnceLock<AvFoundationSymbols> = std::sync::OnceLock::new();

fn load_av_foundation() -> Option<&'static AvFoundationSymbols> {
    let syms = AV_FOUNDATION.get_or_init(|| unsafe {
        let null_syms = AvFoundationSymbols {
            av_capture_device_class: std::ptr::null(),
            av_media_type_audio: std::ptr::null(),
        };

        let handle = dlopen(
            c"/System/Library/Frameworks/AVFoundation.framework/AVFoundation".as_ptr(),
            RTLD_LAZY,
        );
        if handle.is_null() {
            log::warn!("mic check: failed to dlopen AVFoundation");
            return null_syms;
        }

        let class = objc_getClass(c"AVCaptureDevice".as_ptr());
        if class.is_null() {
            log::warn!("mic check: AVCaptureDevice class not found");
            return null_syms;
        }

        let av_media_type_audio_ptr = dlsym(handle, c"AVMediaTypeAudio".as_ptr());
        if av_media_type_audio_ptr.is_null() {
            log::warn!("mic check: AVMediaTypeAudio symbol not found");
            return null_syms;
        }
        let audio_type: CFTypeRef = *(av_media_type_audio_ptr as *const CFTypeRef);

        AvFoundationSymbols {
            av_capture_device_class: class,
            av_media_type_audio: audio_type,
        }
    });

    if syms.av_capture_device_class.is_null() {
        None
    } else {
        Some(syms)
    }
}

/// Checks microphone permission status via AVCaptureDevice.
///
/// Returns one of: "granted", "denied", "restricted", "not_determined", "unknown".
pub(crate) fn is_microphone_authorized() -> &'static str {
    let syms = match load_av_foundation() {
        Some(s) => s,
        None => return "unknown",
    };

    unsafe {
        let sel = sel_registerName(c"authorizationStatusForMediaType:".as_ptr());
        let send: unsafe extern "C" fn(*const Object, Sel, CFTypeRef) -> i64 = std::mem::transmute(
            objc_msgSend as unsafe extern "C" fn(*mut Object, Sel) -> *mut Object,
        );
        let status = send(syms.av_capture_device_class, sel, syms.av_media_type_audio);
        log::info!("mic check: AVCaptureDevice authorizationStatus = {status}");
        match status {
            0 => "not_determined",
            1 => "restricted",
            2 => "denied",
            3 => "granted",
            _ => "unknown",
        }
    }
}

/// Requests microphone permission by briefly opening a Core Audio input stream.
/// On macOS, this triggers the TCC permission dialog if status is notDetermined.
/// No-op if permission is already granted or denied.
pub(crate) fn request_microphone_access() {
    if is_microphone_authorized() == "granted" {
        return;
    }

    use cpal::traits::{DeviceTrait, StreamTrait};

    std::thread::spawn(|| {
        let (device, config) = match crate::audio::open_default_input() {
            Some(d) => d,
            None => return,
        };
        let stream = device.build_input_stream(
            &config.into(),
            |_data: &[f32], _: &cpal::InputCallbackInfo| {},
            |_err| {},
            None,
        );
        if let Ok(s) = stream {
            let _ = s.play();
            // Keep stream alive briefly so macOS shows the permission dialog
            std::thread::sleep(std::time::Duration::from_millis(200));
            drop(s);
        }
    });
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
    fn dlopen(path: *const std::ffi::c_char, mode: i32) -> *const std::ffi::c_void;
    fn dlsym(
        handle: *const std::ffi::c_void,
        symbol: *const std::ffi::c_char,
    ) -> *const std::ffi::c_void;
}

const RTLD_LAZY: i32 = 0x1;

// --- Accessibility API FFI bindings ---

extern "C" {
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    fn CFRelease(cf: CFTypeRef);
    fn CFStringGetCString(
        the_string: CFStringRef,
        buffer: *mut std::ffi::c_char,
        buffer_size: i64,
        encoding: u32,
    ) -> bool;
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Sel {
    _ptr: *const std::ffi::c_void,
}
