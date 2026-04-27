use cef::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::handlers;

// --- shared UI state -------------------------------------------------------
//
// The main window lives on the CEF UI thread. We cache a Window handle in
// thread-local storage so UI-thread tasks (tray menu, keyboard) can show/
// hide/close it. `QUITTING` flips on when the user chose Quit — from that
// point on, `can_close` actually lets the window close instead of hiding it.

thread_local! {
    static MAIN_WINDOW: RefCell<Option<Window>> = const { RefCell::new(None) };
}

static QUITTING: AtomicBool = AtomicBool::new(false);

fn set_main_window(w: Window) {
    MAIN_WINDOW.with(|c| *c.borrow_mut() = Some(w));
}

fn clear_main_window() {
    MAIN_WINDOW.with(|c| *c.borrow_mut() = None);
}

pub fn with_main_window<F: FnOnce(&Window)>(f: F) -> bool {
    MAIN_WINDOW.with(|c| match c.borrow().as_ref() {
        Some(w) => {
            f(w);
            true
        }
        None => false,
    })
}

pub fn set_quitting() {
    QUITTING.store(true, Ordering::SeqCst);
}

pub fn is_quitting() -> bool {
    QUITTING.load(Ordering::SeqCst)
}

pub const TEAMS_URL: &str = "https://teams.microsoft.com";

pub const EDGE_USER_AGENT: &str = concat!(
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ",
    "AppleWebKit/537.36 (KHTML, like Gecko) ",
    "Chrome/146.0.0.0 Safari/537.36 Edg/146.0.0.0",
);

/// Allowlisted domain suffixes. Navigation outside this set is blocked and
/// handed to the user's default browser.
///
/// Entries starting with `.` match the apex *and* any subdomain
/// (`.microsoft.com` -> both `microsoft.com` and `foo.microsoft.com`).
/// Entries without a leading dot must match exactly.
pub const ALLOWED_SUFFIXES: &[&str] = &[
    // Teams itself (classic + cloud migration)
    ".teams.microsoft.com",
    ".cloud.microsoft",
    // Microsoft auth
    ".microsoftonline.com",
    ".microsoftonline-p.com",
    ".live.com",
    ".msauth.net",
    ".msftauth.net",
    ".gfx.ms",
    // Office / Teams infrastructure + CDNs
    ".microsoft.com",
    ".office.com",
    ".office.net",
    ".office365.com",
    ".msocdn.com",
    ".sharepoint.com",
    ".onedrive.com",
    ".skype.com",
    ".lync.com",
    ".azureedge.net",
];

pub fn host_is_allowed(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    ALLOWED_SUFFIXES.iter().any(|s| {
        if let Some(apex) = s.strip_prefix('.') {
            // Apex or any subdomain — never a naked suffix match, so
            // "evilmicrosoft.com" does NOT match ".microsoft.com".
            host == apex || host.ends_with(s)
        } else {
            host == *s
        }
    })
}

wrap_window_delegate! {
    pub struct TeamsWindowDelegate {
        browser_view: RefCell<Option<BrowserView>>,
        initial_show_state: ShowState,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1280, height: 800 }
        }

        fn minimum_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 800, height: 600 }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            let browser_view = self.browser_view.borrow();
            let (Some(window), Some(browser_view)) = (window, browser_view.as_ref()) else {
                return;
            };
            let mut view = View::from(browser_view);
            window.add_child_view(Some(&mut view));
            let title = CefString::from("Microsoft Teams");
            window.set_title(Some(&title));
            // Wire our embedded PNGs into both the title-bar (small) and
            // taskbar/alt-tab (large) icon slots. Without this CEF Views
            // creates a fresh HWND that doesn't inherit the EXE's icon
            // resource, so the taskbar shows the generic Chromium glyph.
            if let Some(mut icon) = build_window_icon() {
                window.set_window_icon(Some(&mut icon));
            }
            if let Some(mut icon) = build_app_icon() {
                window.set_window_app_icon(Some(&mut icon));
            }
            if self.initial_show_state != ShowState::HIDDEN {
                window.show();
            }
            set_main_window(window.clone());
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            *self.browser_view.borrow_mut() = None;
            clear_main_window();
        }

        fn can_close(&self, window: Option<&mut Window>) -> i32 {
            // Closing while not quitting = hide-to-tray. Actual close only
            // proceeds once the user picked Quit (tray / Ctrl+Q).
            if !is_quitting() {
                if let Some(w) = window {
                    w.hide();
                }
                return 0;
            }
            let browser_view = self.browser_view.borrow();
            let Some(browser_view) = browser_view.as_ref() else { return 1; };
            if let Some(browser) = browser_view.browser() {
                browser.host().expect("no host").try_close_browser()
            } else {
                1
            }
        }

        // CEF Views defaults all of these to false, which strips the
        // resize grip and the min/max title-bar buttons. Re-enable them
        // so the window behaves like a normal top-level app window.
        fn can_resize(&self, _window: Option<&mut Window>) -> i32 { 1 }
        fn can_maximize(&self, _window: Option<&mut Window>) -> i32 { 1 }
        fn can_minimize(&self, _window: Option<&mut Window>) -> i32 { 1 }
        fn with_standard_window_buttons(&self, _window: Option<&mut Window>) -> i32 { 1 }

        fn initial_show_state(&self, _window: Option<&mut Window>) -> ShowState {
            self.initial_show_state
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

fn build_window_icon() -> Option<Image> {
    // Small icon — title bar / window list.
    static ICON_16: &[u8] = include_bytes!("../resources/icon-16.png");
    static ICON_32: &[u8] = include_bytes!("../resources/icon-32.png");
    static ICON_48: &[u8] = include_bytes!("../resources/icon-48.png");
    let img = image_create()?;
    img.add_png(1.0, Some(ICON_16));
    img.add_png(2.0, Some(ICON_32));
    img.add_png(3.0, Some(ICON_48));
    Some(img)
}

fn build_app_icon() -> Option<Image> {
    // Large icon — taskbar / alt-tab. Multiple reps so HiDPI displays
    // can pick a crisp size instead of upscaling 32×32.
    static ICON_32: &[u8] = include_bytes!("../resources/icon-32.png");
    static ICON_64: &[u8] = include_bytes!("../resources/icon-64.png");
    static ICON_128: &[u8] = include_bytes!("../resources/icon-128.png");
    static ICON_256: &[u8] = include_bytes!("../resources/icon-256.png");
    let img = image_create()?;
    img.add_png(1.0, Some(ICON_32));
    img.add_png(2.0, Some(ICON_64));
    img.add_png(4.0, Some(ICON_128));
    img.add_png(8.0, Some(ICON_256));
    Some(img)
}

wrap_browser_view_delegate! {
    pub struct TeamsBrowserViewDelegate {}

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn on_popup_browser_view_created(
            &self,
            _browser_view: Option<&mut BrowserView>,
            popup_browser_view: Option<&mut BrowserView>,
            _is_devtools: i32,
        ) -> i32 {
            let mut delegate = TeamsWindowDelegate::new(
                RefCell::new(popup_browser_view.cloned()),
                ShowState::NORMAL,
            );
            window_create_top_level(Some(&mut delegate));
            1
        }

        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

/// Called once CEF context is initialized. Creates the Teams window.
pub fn create_main_window() {
    debug_assert_ne!(currently_on(ThreadId::UI), 0);

    let settings = BrowserSettings {
        background_color: 0xFF_1F_1F_1F,
        ..Default::default()
    };
    let url = CefString::from(TEAMS_URL);

    let mut client = handlers::TeamsClient::new();
    let mut delegate = TeamsBrowserViewDelegate::new();
    let browser_view = browser_view_create(
        Some(&mut client),
        Some(&url),
        Some(&settings),
        None,
        None,
        Some(&mut delegate),
    );

    let mut window_delegate =
        TeamsWindowDelegate::new(RefCell::new(browser_view), ShowState::NORMAL);
    window_create_top_level(Some(&mut window_delegate));
}
