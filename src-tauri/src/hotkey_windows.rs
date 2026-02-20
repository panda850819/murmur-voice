use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HotkeyEvent {
    Pressed,
    Released,
    EscCancel,
    EventTapFailed,
}

/// The active PTT modifier virtual key code. Updated at runtime via settings.
/// Default: VK_LMENU (Left Alt) = 0xA4
static MODIFIER_MASK: AtomicU64 = AtomicU64::new(0xA4);

/// The regular key virtual key code for combo mode (0 = modifier-only mode).
static REGULAR_KEY: AtomicU32 = AtomicU32::new(0);

/// Track whether the hotkey is currently held down (edge detection, modifier-only mode).
static KEY_WAS_DOWN: AtomicBool = AtomicBool::new(false);

/// Track whether the modifier key is currently held (combo mode).
static MODIFIER_HELD: AtomicBool = AtomicBool::new(false);

/// Track whether the combo is currently active (combo mode).
static COMBO_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Global sender for the hook callback. Set once before installing the hook.
static mut GLOBAL_SENDER: Option<mpsc::Sender<HotkeyEvent>> = None;

/// Update the PTT hotkey target (modifier VK code + regular key VK code) at runtime.
pub(crate) fn set_hotkey_target(modifier: u64, regular_key: u32) {
    MODIFIER_MASK.store(modifier, Ordering::SeqCst);
    REGULAR_KEY.store(regular_key, Ordering::SeqCst);
}

/// Temporarily pause hotkey detection (set mask to 0 so nothing matches).
pub(crate) fn pause_hotkey() {
    MODIFIER_MASK.store(0, Ordering::SeqCst);
    REGULAR_KEY.store(0, Ordering::SeqCst);
}

unsafe extern "system" fn keyboard_hook_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kb = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
        let modifier_vk = MODIFIER_MASK.load(Ordering::SeqCst) as u32;
        let regular_vk = REGULAR_KEY.load(Ordering::SeqCst);
        let msg = w_param.0 as u32;
        let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

        // ESC key detection — cancel recording regardless of hotkey mode
        if kb.vkCode == 0x1B && is_down {
            if let Some(ref sender) = GLOBAL_SENDER {
                let _ = sender.send(HotkeyEvent::EscCancel);
            }
            // Pass through to other apps (don't return early with LRESULT(1))
        }

        if regular_vk != 0 {
            // Combo mode: modifier + regular key
            if let Some(ref sender) = GLOBAL_SENDER {
                if kb.vkCode == modifier_vk {
                    if is_down {
                        MODIFIER_HELD.store(true, Ordering::SeqCst);
                    } else if is_up {
                        MODIFIER_HELD.store(false, Ordering::SeqCst);
                        if COMBO_ACTIVE.load(Ordering::SeqCst) {
                            COMBO_ACTIVE.store(false, Ordering::SeqCst);
                            let _ = sender.send(HotkeyEvent::Released);
                        }
                    }
                } else if kb.vkCode == regular_vk {
                    if is_down
                        && MODIFIER_HELD.load(Ordering::SeqCst)
                        && !COMBO_ACTIVE.load(Ordering::SeqCst)
                    {
                        COMBO_ACTIVE.store(true, Ordering::SeqCst);
                        let _ = sender.send(HotkeyEvent::Pressed);
                        return LRESULT(1);
                    } else if is_up && COMBO_ACTIVE.load(Ordering::SeqCst) {
                        COMBO_ACTIVE.store(false, Ordering::SeqCst);
                        let _ = sender.send(HotkeyEvent::Released);
                        return LRESULT(1);
                    }
                }
            }
        } else {
            // Modifier-only mode (unchanged)
            if kb.vkCode == modifier_vk {
                if let Some(ref sender) = GLOBAL_SENDER {
                    let was_down = KEY_WAS_DOWN.load(Ordering::SeqCst);

                    if is_down && !was_down {
                        KEY_WAS_DOWN.store(true, Ordering::SeqCst);
                        let _ = sender.send(HotkeyEvent::Pressed);
                    } else if is_up && was_down {
                        KEY_WAS_DOWN.store(false, Ordering::SeqCst);
                        let _ = sender.send(HotkeyEvent::Released);
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
