use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HotkeyEvent {
    Pressed,
    Released,
}

/// The active PTT virtual key code. Updated at runtime via settings.
/// Default: VK_LMENU (Left Alt) = 0xA4
static HOTKEY_MASK: AtomicU64 = AtomicU64::new(0xA4);

/// Track whether the hotkey is currently held down (edge detection).
static KEY_WAS_DOWN: AtomicBool = AtomicBool::new(false);

/// Global sender for the hook callback. Set once before installing the hook.
static mut GLOBAL_SENDER: Option<mpsc::Sender<HotkeyEvent>> = None;

/// Update the PTT key (virtual key code) at runtime.
pub(crate) fn set_hotkey_mask(mask: u64) {
    HOTKEY_MASK.store(mask, Ordering::SeqCst);
}

unsafe extern "system" fn keyboard_hook_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kb = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
        let target_vk = HOTKEY_MASK.load(Ordering::SeqCst) as u32;
        let msg = w_param.0 as u32;

        if kb.vkCode == target_vk {
            let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

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
            }
        }
    })
}
