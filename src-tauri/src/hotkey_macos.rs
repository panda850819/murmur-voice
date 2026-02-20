use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HotkeyEvent {
    Pressed,
    Released,
    EscCancel,
    EventTapFailed,
}

/// The active PTT modifier mask. Updated at runtime via settings.
/// Default: Left Option (NX_DEVICELALTKEYMASK = 0x20)
static MODIFIER_MASK: AtomicU64 = AtomicU64::new(0x20);

/// The regular key CGKeyCode for combo mode (0 = modifier-only mode).
static REGULAR_KEY: AtomicU32 = AtomicU32::new(0);

/// Update the PTT hotkey target at runtime.
pub(crate) fn set_hotkey_target(modifier: u64, regular_key: u32) {
    MODIFIER_MASK.store(modifier, Ordering::SeqCst);
    REGULAR_KEY.store(regular_key, Ordering::SeqCst);
}

/// Temporarily pause hotkey detection (set mask to 0 so nothing matches).
pub(crate) fn pause_hotkey() {
    MODIFIER_MASK.store(0, Ordering::SeqCst);
    REGULAR_KEY.store(0, Ordering::SeqCst);
}

// CGEvent type constants
const K_CG_EVENT_KEY_DOWN: u32 = 10;
const K_CG_EVENT_KEY_UP: u32 = 11;
const K_CG_EVENT_FLAGS_CHANGED: u32 = 12;

// CGEventTap creation constants
const K_CG_HID_EVENT_TAP: u32 = 0;
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const K_CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;

// CGEventField constant for keyboard keycode
const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;

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
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;

    static kCFRunLoopDefaultMode: CFStringRef;
}

// Edge detection state for modifier-only mode
static KEY_WAS_DOWN: AtomicBool = AtomicBool::new(false);

// State for combo mode
static MODIFIER_HELD: AtomicBool = AtomicBool::new(false);
static COMBO_ACTIVE: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn event_tap_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    let sender = &*(user_info as *const mpsc::Sender<HotkeyEvent>);
    let regular_key = REGULAR_KEY.load(Ordering::SeqCst);

    // ESC key detection — cancel recording regardless of hotkey mode
    if event_type == K_CG_EVENT_KEY_DOWN {
        let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u32;
        if keycode == 0x35 {
            let _ = sender.send(HotkeyEvent::EscCancel);
            return event; // pass through ESC to other apps
        }
    }

    if regular_key == 0 {
        // Modifier-only mode — original edge detection logic (unchanged)
        if event_type == K_CG_EVENT_FLAGS_CHANGED {
            let flags = CGEventGetFlags(event);
            let mask = MODIFIER_MASK.load(Ordering::SeqCst);
            let key_now = (flags & mask) != 0;
            let was_down = KEY_WAS_DOWN.swap(key_now, Ordering::SeqCst);
            if key_now && !was_down {
                let _ = sender.send(HotkeyEvent::Pressed);
            } else if !key_now && was_down {
                let _ = sender.send(HotkeyEvent::Released);
            }
        }
        return event;
    }

    // Combo mode: modifier + regular key
    match event_type {
        K_CG_EVENT_FLAGS_CHANGED => {
            let modifier = MODIFIER_MASK.load(Ordering::SeqCst);
            let flags = CGEventGetFlags(event);
            let mod_held = (flags & modifier) != 0;
            MODIFIER_HELD.store(mod_held, Ordering::SeqCst);
            // Modifier released while combo was active → emit Released
            if !mod_held && COMBO_ACTIVE.swap(false, Ordering::SeqCst) {
                let _ = sender.send(HotkeyEvent::Released);
            }
        }
        K_CG_EVENT_KEY_DOWN => {
            let keycode =
                CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u32;
            if keycode == regular_key
                && MODIFIER_HELD.load(Ordering::SeqCst)
                && !COMBO_ACTIVE.load(Ordering::SeqCst)
            {
                COMBO_ACTIVE.store(true, Ordering::SeqCst);
                let _ = sender.send(HotkeyEvent::Pressed);
                return std::ptr::null_mut(); // consume event
            }
        }
        K_CG_EVENT_KEY_UP => {
            let keycode =
                CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u32;
            if keycode == regular_key && COMBO_ACTIVE.swap(false, Ordering::SeqCst) {
                let _ = sender.send(HotkeyEvent::Released);
                return std::ptr::null_mut(); // consume event
            }
        }
        _ => {}
    }

    event
}

pub(crate) fn start_listener(
    sender: mpsc::Sender<HotkeyEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || unsafe {
        let event_mask: u64 = (1 << K_CG_EVENT_KEY_DOWN)
            | (1 << K_CG_EVENT_KEY_UP)
            | (1 << K_CG_EVENT_FLAGS_CHANGED);

        let sender_box = Box::new(sender);
        let sender_ptr = Box::into_raw(sender_box) as *mut c_void;

        let tap = CGEventTapCreate(
            K_CG_HID_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_DEFAULT,
            event_mask,
            event_tap_callback,
            sender_ptr,
        );

        if tap.is_null() {
            log::error!(
                "failed to create event tap — grant Accessibility permission in \
                 System Settings > Privacy & Security > Accessibility"
            );
            let sender = Box::from_raw(sender_ptr as *mut mpsc::Sender<HotkeyEvent>);
            let _ = sender.send(HotkeyEvent::EventTapFailed);
            return;
        }

        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopDefaultMode);

        CFRunLoopRun();

        let _ = Box::from_raw(sender_ptr as *mut mpsc::Sender<HotkeyEvent>);
    })
}
