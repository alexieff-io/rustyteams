use std::sync::OnceLock;

use cef::*;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder,
};

use crate::{browser, config};

wrap_task! {
    struct ShowMainTask;

    impl Task {
        fn execute(&self) {
            browser::with_main_window(|w| w.show());
        }
    }
}

wrap_task! {
    struct QuitAppTask;

    impl Task {
        fn execute(&self) {
            browser::set_quitting();
            if !browser::with_main_window(|w| w.close()) {
                quit_message_loop();
            }
        }
    }
}

fn post_show() {
    let mut task = ShowMainTask::new();
    post_task(ThreadId::UI, Some(&mut task));
}

fn post_quit() {
    let mut task = QuitAppTask::new();
    post_task(ThreadId::UI, Some(&mut task));
}

static QUIT_ID: OnceLock<String> = OnceLock::new();
static SHOW_ID: OnceLock<String> = OnceLock::new();
static PICK_BROWSER_ID: OnceLock<String> = OnceLock::new();
static CLEAR_BROWSER_ID: OnceLock<String> = OnceLock::new();

pub fn install() -> Option<TrayIcon> {
    let menu = Menu::new();
    let show = MenuItem::new("Show Teams", true, None);
    let pick_browser = MenuItem::new("Set external browser…", true, None);
    let clear_browser = MenuItem::new("Use system default browser", true, None);
    let quit = MenuItem::new("Quit", true, None);

    SHOW_ID.set(show.id().0.clone()).ok();
    PICK_BROWSER_ID.set(pick_browser.id().0.clone()).ok();
    CLEAR_BROWSER_ID.set(clear_browser.id().0.clone()).ok();
    QUIT_ID.set(quit.id().0.clone()).ok();

    menu.append(&show).ok()?;
    menu.append(&PredefinedMenuItem::separator()).ok()?;
    menu.append(&pick_browser).ok()?;
    menu.append(&clear_browser).ok()?;
    menu.append(&PredefinedMenuItem::separator()).ok()?;
    menu.append(&quit).ok()?;

    let icon = load_icon();
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Microsoft Teams")
        .with_icon(icon)
        .build()
        .ok()?;

    MenuEvent::set_event_handler(Some(|event: MenuEvent| {
        let id = &event.id.0;
        if QUIT_ID.get().is_some_and(|q| q == id) {
            post_quit();
        } else if SHOW_ID.get().is_some_and(|s| s == id) {
            post_show();
            focus_main_hwnd();
        } else if PICK_BROWSER_ID.get().is_some_and(|p| p == id) {
            pick_external_browser();
        } else if CLEAR_BROWSER_ID.get().is_some_and(|c| c == id) {
            let _ = config::set_external_browser(None);
        }
    }));

    Some(tray)
}

fn pick_external_browser() {
    let start = config::external_browser()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .or_else(|| Some(std::path::PathBuf::from("C:/Program Files")));
    let mut dialog = rfd::FileDialog::new()
        .add_filter("Executable", &["exe"])
        .set_title("Pick a browser to open external links with");
    if let Some(dir) = start {
        dialog = dialog.set_directory(dir);
    }
    if let Some(picked) = dialog.pick_file() {
        let _ = config::set_external_browser(Some(&picked));
    }
}

fn load_icon() -> tray_icon::Icon {
    // On Windows the ICO is embedded into the exe by winres (build.rs).
    // Pull it straight from our own resource table — no file I/O, and the
    // tray always matches the taskbar icon.
    #[cfg(target_os = "windows")]
    {
        if let Ok(icon) = tray_icon::Icon::from_resource(1, Some((32, 32))) {
            return icon;
        }
        if let Ok(icon) = tray_icon::Icon::from_resource(1, None) {
            return icon;
        }
    }
    placeholder_icon()
}

fn placeholder_icon() -> tray_icon::Icon {
    let mut rgba = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..(16 * 16) {
        rgba.extend_from_slice(&[0x8B, 0x45, 0x13, 0xFF]); // rust fallback
    }
    tray_icon::Icon::from_rgba(rgba, 16, 16).expect("valid icon")
}

/// CEF's `Window::show` un-hides the Views window, but doesn't necessarily
/// pull a minimized Win32 window to the foreground. Walk our own top-level
/// windows and restore+raise whichever one is visible.
#[cfg(target_os = "windows")]
fn focus_main_hwnd() {
    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible, SetForegroundWindow, ShowWindow,
        SW_RESTORE,
    };

    unsafe extern "system" fn cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == std::process::id() && IsWindowVisible(hwnd) != 0 {
            let out = lparam as *mut HWND;
            if (*out).is_null() {
                *out = hwnd;
            }
        }
        1
    }

    unsafe {
        let mut hwnd: HWND = std::ptr::null_mut();
        EnumWindows(Some(cb), &mut hwnd as *mut _ as LPARAM);
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn focus_main_hwnd() {}
