use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HotkeyEvent {
    Pressed,
    Released,
}

/// The active PTT key mask. Updated at runtime via settings.
/// Default: Left Option (NX_DEVICELALTKEYMASK = 0x20)
static HOTKEY_MASK: AtomicU64 = AtomicU64::new(0x20);

/// Update the PTT key mask at runtime.
pub(crate) fn set_hotkey_mask(mask: u64) {
    HOTKEY_MASK.store(mask, Ordering::SeqCst);
}

// CGEvent constants
const K_CG_EVENT_FLAGS_CHANGED: u32 = 12;
const K_CG_HID_EVENT_TAP: u32 = 0;
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const K_CG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;

// FFI types
type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFRunLoopRef = *mut c_void;
type CFStringRef = *const c_void;
type CFAllocatorRef = *const c_void;

type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        port: CFMachPortRef,
        order: i64,
    ) -> CFRunLoopSourceRef;

    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopRun();
    fn CGEventGetFlags(event: CGEventRef) -> u64;

    static kCFRunLoopDefaultMode: CFStringRef;
}

static KEY_WAS_DOWN: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn event_tap_callback(
    _proxy: CGEventTapProxy,
    _event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    let sender = &*(user_info as *const mpsc::Sender<HotkeyEvent>);
    let flags = CGEventGetFlags(event);
    let mask = HOTKEY_MASK.load(Ordering::SeqCst);

    let key_now = (flags & mask) != 0;
    let was_down = KEY_WAS_DOWN.swap(key_now, Ordering::SeqCst);

    if key_now && !was_down {
        let _ = sender.send(HotkeyEvent::Pressed);
    } else if !key_now && was_down {
        let _ = sender.send(HotkeyEvent::Released);
    }

    event
}

pub(crate) fn start_listener(
    sender: mpsc::Sender<HotkeyEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || unsafe {
        let event_mask: u64 = 1 << K_CG_EVENT_FLAGS_CHANGED;

        let sender_box = Box::new(sender);
        let sender_ptr = Box::into_raw(sender_box) as *mut c_void;

        let tap = CGEventTapCreate(
            K_CG_HID_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
            event_mask,
            event_tap_callback,
            sender_ptr,
        );

        if tap.is_null() {
            log::error!(
                "failed to create event tap — grant Accessibility permission in \
                 System Settings > Privacy & Security > Accessibility"
            );
            let _ = Box::from_raw(sender_ptr as *mut mpsc::Sender<HotkeyEvent>);
            return;
        }

        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopDefaultMode);

        CFRunLoopRun();

        let _ = Box::from_raw(sender_ptr as *mut mpsc::Sender<HotkeyEvent>);
    })
}
