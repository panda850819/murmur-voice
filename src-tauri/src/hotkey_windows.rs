use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

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

/// Track whether the hotkey is currently held down (edge detection, modifier-only mode).
static KEY_WAS_DOWN: AtomicBool = AtomicBool::new(false);

/// Track whether a combo is currently active.
static COMBO_ACTIVE: AtomicBool = AtomicBool::new(false);
static COMBO_ACTIVE_SLOT: AtomicU32 = AtomicU32::new(u32::MAX);

/// Global sender for the hook callback. Set once before installing the hook.
static mut GLOBAL_SENDER: Option<mpsc::Sender<HotkeyEvent>> = None;

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

const MODES: [RecordingMode; 4] = [
    RecordingMode::Dictation,
    RecordingMode::Translate,
    RecordingMode::VoiceCommand,
    RecordingMode::ClipboardRewrite,
];

/// Match order: combo-only modes first (more specific), modifier-only last.
const MATCH_ORDER: [usize; 4] = [2, 3, 1, 0];

/// Check if all packed modifier VK codes are currently held.
fn all_modifiers_held(packed: u64) -> bool {
    for i in 0..4u32 {
        let vk = ((packed >> (i * 16)) & 0xFFFF) as i32;
        if vk == 0 {
            break;
        }
        if unsafe { GetAsyncKeyState(vk) } >= 0 {
            return false;
        }
    }
    true
}

unsafe extern "system" fn keyboard_hook_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kb = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
        let msg = w_param.0 as u32;
        let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

        // ESC key detection
        if kb.vkCode == 0x1B && is_down {
            if let Some(ref sender) = GLOBAL_SENDER {
                let _ = sender.send(HotkeyEvent::EscCancel);
            }
        }

        // --- Combo hotkey detection (key-down) ---
        if is_down && !COMBO_ACTIVE.load(Ordering::SeqCst) {
            if let Some(ref sender) = GLOBAL_SENDER {
                for &idx in &MATCH_ORDER {
                    let slot = &HOTKEY_SLOTS[idx];
                    let mask = slot.modifier_mask.load(Ordering::SeqCst);
                    let rk = slot.regular_key.load(Ordering::SeqCst);
                    if mask == 0 || rk == 0 {
                        continue;
                    }
                    if kb.vkCode == rk && all_modifiers_held(mask) {
                        COMBO_ACTIVE.store(true, Ordering::SeqCst);
                        COMBO_ACTIVE_SLOT.store(idx as u32, Ordering::SeqCst);
                        let _ = sender.send(HotkeyEvent::Pressed(MODES[idx]));
                        return LRESULT(1); // consume
                    }
                }
            }
        }

        // --- Combo release (regular key up) ---
        if is_up && COMBO_ACTIVE.load(Ordering::SeqCst) {
            let active_idx = COMBO_ACTIVE_SLOT.load(Ordering::SeqCst) as usize;
            if active_idx < 4 {
                let slot = &HOTKEY_SLOTS[active_idx];
                let rk = slot.regular_key.load(Ordering::SeqCst);
                if kb.vkCode == rk {
                    COMBO_ACTIVE.store(false, Ordering::SeqCst);
                    if let Some(ref sender) = GLOBAL_SENDER {
                        let _ = sender.send(HotkeyEvent::Released(MODES[active_idx]));
                    }
                    return LRESULT(1); // consume
                }
                // Modifier released while combo active
                let mask = slot.modifier_mask.load(Ordering::SeqCst);
                let first_vk = (mask & 0xFFFF) as u32;
                if kb.vkCode == first_vk {
                    COMBO_ACTIVE.store(false, Ordering::SeqCst);
                    if let Some(ref sender) = GLOBAL_SENDER {
                        let _ = sender.send(HotkeyEvent::Released(MODES[active_idx]));
                    }
                }
            }
        }

        // --- Modifier-only mode (Dictation slot, when regular_key == 0) ---
        let dict_slot = &HOTKEY_SLOTS[0];
        let dict_mask = dict_slot.modifier_mask.load(Ordering::SeqCst);
        let dict_rk = dict_slot.regular_key.load(Ordering::SeqCst);
        if dict_rk == 0 && dict_mask != 0 && !COMBO_ACTIVE.load(Ordering::SeqCst) {
            let dict_vk = (dict_mask & 0xFFFF) as u32;
            if kb.vkCode == dict_vk {
                if let Some(ref sender) = GLOBAL_SENDER {
                    let was_down = KEY_WAS_DOWN.load(Ordering::SeqCst);
                    if is_down && !was_down {
                        KEY_WAS_DOWN.store(true, Ordering::SeqCst);
                        let _ = sender.send(HotkeyEvent::Pressed(RecordingMode::Dictation));
                    } else if is_up && was_down {
                        KEY_WAS_DOWN.store(false, Ordering::SeqCst);
                        let _ = sender.send(HotkeyEvent::Released(RecordingMode::Dictation));
                    }
                }
            }
        }
    }

    CallNextHookEx(None, n_code, w_param, l_param)
}

pub(crate) fn start_listener(
    sender: mpsc::Sender<HotkeyEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || unsafe {
        GLOBAL_SENDER = Some(sender);

        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0);

        match hook {
            Ok(_h) => {
                // Message pump required for low-level hooks to receive callbacks
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {}
            }
            Err(e) => {
                log::error!("failed to install keyboard hook: {}", e);
                if let Some(ref sender) = GLOBAL_SENDER {
                    let _ = sender.send(HotkeyEvent::EventTapFailed);
                }
            }
        }
    })
}
