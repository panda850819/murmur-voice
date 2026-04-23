use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc;

use crate::state::RecordingMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HotkeyEvent {
    Pressed(RecordingMode),
    Released(RecordingMode),
    EscCancel,
    EventTapFailed,
}

/// A hotkey slot with atomic modifier mask and regular key.
struct HotkeySlot {
    modifier_mask: AtomicU64,
    regular_key: AtomicU32,
}

impl HotkeySlot {
    const fn new() -> Self {
        Self {
            modifier_mask: AtomicU64::new(0),
            regular_key: AtomicU32::new(0),
        }
    }
}

/// 4 slots indexed by RecordingMode as usize:
/// [0] = Dictation, [1] = Translate, [2] = VoiceCommand, [3] = ClipboardRewrite
static HOTKEY_SLOTS: [HotkeySlot; 4] = [
    HotkeySlot::new(),
    HotkeySlot::new(),
    HotkeySlot::new(),
    HotkeySlot::new(),
];

/// Update a hotkey slot for the given mode.
pub(crate) fn set_hotkey(mode: RecordingMode, modifier: u64, regular_key: u32) {
    let slot = &HOTKEY_SLOTS[mode as usize];
    slot.modifier_mask.store(modifier, Ordering::SeqCst);
    slot.regular_key.store(regular_key, Ordering::SeqCst);
}

/// Disable a specific hotkey slot (set mask to 0).
pub(crate) fn pause_hotkey(mode: RecordingMode) {
    let slot = &HOTKEY_SLOTS[mode as usize];
    slot.modifier_mask.store(0, Ordering::SeqCst);
    slot.regular_key.store(0, Ordering::SeqCst);
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

// Edge detection state for modifier-only mode (Dictation slot)
static KEY_WAS_DOWN: AtomicBool = AtomicBool::new(false);

// State for combo mode (per-slot edge detection)
// For simplicity, we track a single combo state — only one combo can be active at a time.
static COMBO_ACTIVE: AtomicBool = AtomicBool::new(false);
static COMBO_ACTIVE_SLOT: AtomicU32 = AtomicU32::new(u32::MAX);

const MODES: [RecordingMode; 4] = [
    RecordingMode::Dictation,
    RecordingMode::Translate,
    RecordingMode::VoiceCommand,
    RecordingMode::ClipboardRewrite,
];

/// Mode order for matching: combo-only modes first (more specific), modifier-only last.
/// VoiceCommand(2), ClipboardRewrite(3), Translate(1), Dictation(0)
const MATCH_ORDER: [usize; 4] = [2, 3, 1, 0];

/// Find which slot matches the given flags + keycode for a key-down event.
/// Returns (slot_index, RecordingMode) if found.
fn find_combo_match(keycode: u32, flags: u64) -> Option<(usize, RecordingMode)> {
    for &idx in &MATCH_ORDER {
        let slot = &HOTKEY_SLOTS[idx];
        let mask = slot.modifier_mask.load(Ordering::SeqCst);
        let rk = slot.regular_key.load(Ordering::SeqCst);
        if mask == 0 {
            continue; // disabled slot
        }
        if rk != 0 && keycode == rk && (flags & mask) == mask {
            return Some((idx, MODES[idx]));
        }
    }
    None
}

unsafe extern "C" fn event_tap_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    let sender = &*(user_info as *const mpsc::Sender<HotkeyEvent>);

    // ESC key detection — cancel recording regardless of hotkey mode
    if event_type == K_CG_EVENT_KEY_DOWN {
        let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u32;
        if keycode == 0x35 {
            let _ = sender.send(HotkeyEvent::EscCancel);
            return event; // pass through ESC to other apps
        }
    }

    // --- Combo hotkey detection (key-down) ---
    if event_type == K_CG_EVENT_KEY_DOWN {
        let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u32;
        let flags = CGEventGetFlags(event);

        // Don't match if a combo is already active
        if !COMBO_ACTIVE.load(Ordering::SeqCst) {
            if let Some((idx, mode)) = find_combo_match(keycode, flags) {
                COMBO_ACTIVE.store(true, Ordering::SeqCst);
                COMBO_ACTIVE_SLOT.store(idx as u32, Ordering::SeqCst);
                let _ = sender.send(HotkeyEvent::Pressed(mode));
                return std::ptr::null_mut(); // consume event
            }
        }
    }

    // --- Combo hotkey release (key-up for regular key) ---
    if event_type == K_CG_EVENT_KEY_UP && COMBO_ACTIVE.load(Ordering::SeqCst) {
        let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u32;
        let active_idx = COMBO_ACTIVE_SLOT.load(Ordering::SeqCst) as usize;
        if active_idx < 4 {
            let slot = &HOTKEY_SLOTS[active_idx];
            let rk = slot.regular_key.load(Ordering::SeqCst);
            if keycode == rk {
                COMBO_ACTIVE.store(false, Ordering::SeqCst);
                let _ = sender.send(HotkeyEvent::Released(MODES[active_idx]));
                return std::ptr::null_mut(); // consume event
            }
        }
    }

    // --- Modifier released while combo active → release ---
    if event_type == K_CG_EVENT_FLAGS_CHANGED && COMBO_ACTIVE.load(Ordering::SeqCst) {
        let active_idx = COMBO_ACTIVE_SLOT.load(Ordering::SeqCst) as usize;
        if active_idx < 4 {
            let slot = &HOTKEY_SLOTS[active_idx];
            let mask = slot.modifier_mask.load(Ordering::SeqCst);
            let flags = CGEventGetFlags(event);
            if (flags & mask) == 0 {
                COMBO_ACTIVE.store(false, Ordering::SeqCst);
                let _ = sender.send(HotkeyEvent::Released(MODES[active_idx]));
            }
        }
        return event; // combo is active, skip modifier-only check
    }

    // --- Modifier-only mode (Dictation slot only, when regular_key == 0) ---
    let dict_slot = &HOTKEY_SLOTS[0]; // Dictation
    let dict_rk = dict_slot.regular_key.load(Ordering::SeqCst);
    if dict_rk == 0
        && !COMBO_ACTIVE.load(Ordering::SeqCst)
        && event_type == K_CG_EVENT_FLAGS_CHANGED
    {
        let flags = CGEventGetFlags(event);
        let mask = dict_slot.modifier_mask.load(Ordering::SeqCst);
        if mask != 0 {
            let key_now = (flags & mask) != 0;
            let was_down = KEY_WAS_DOWN.swap(key_now, Ordering::SeqCst);
            if key_now && !was_down {
                let _ = sender.send(HotkeyEvent::Pressed(RecordingMode::Dictation));
            } else if !key_now && was_down {
                let _ = sender.send(HotkeyEvent::Released(RecordingMode::Dictation));
            }
        }
    }

    event
}

pub(crate) fn start_listener(sender: mpsc::Sender<HotkeyEvent>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || unsafe {
        let event_mask: u64 =
            (1 << K_CG_EVENT_KEY_DOWN) | (1 << K_CG_EVENT_KEY_UP) | (1 << K_CG_EVENT_FLAGS_CHANGED);

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
