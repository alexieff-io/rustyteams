use cef::*;

use crate::browser;

// Windows virtual-key codes we care about.
const VK_Q: i32 = 0x51;
const VK_0: i32 = 0x30;
// F12/DevTools is disabled: show_dev_tools crashes under Alloy runtime with
// a BrowserView parent. Revisit once CEF gets a safe path for that combo.

// EVENTFLAG_* masks (see cef_event_flags_t).
const EF_CTRL: u32 = 1 << 2;
const EF_SHIFT: u32 = 1 << 1;

wrap_keyboard_handler! {
    pub struct TeamsKeyboardHandler;

    impl KeyboardHandler {
        fn on_pre_key_event(
            &self,
            browser: Option<&mut Browser>,
            event: Option<&KeyEvent>,
            _os_event: Option<&mut sys::MSG>,
            _is_keyboard_shortcut: Option<&mut i32>,
        ) -> i32 {
            let Some(event) = event else { return 0; };
            if event.type_ != KeyEventType::RAWKEYDOWN {
                return 0;
            }
            let ctrl = (event.modifiers & EF_CTRL) != 0;
            let shift = (event.modifiers & EF_SHIFT) != 0;
            let vk = event.windows_key_code;

            // Ctrl+Q -> graceful quit (honors close-to-quit path).
            if ctrl && !shift && vk == VK_Q {
                browser::set_quitting();
                if let Some(b) = browser.cloned() {
                    if let Some(host) = b.host() {
                        host.close_browser(0);
                    }
                } else {
                    quit_message_loop();
                }
                return 1;
            }

            // Ctrl+0 -> reset zoom (mirrors browser muscle memory).
            if ctrl && !shift && vk == VK_0 {
                if let Some(b) = browser.cloned() {
                    if let Some(host) = b.host() {
                        host.set_zoom_level(0.0);
                    }
                }
                return 1;
            }

            0
        }
    }
}
