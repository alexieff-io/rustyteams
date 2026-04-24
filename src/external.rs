//! Route external URLs (anything outside the Teams allowlist) to either a
//! user-configured browser or the OS default.

use crate::config;

pub fn open_url(url: &str) {
    if let Some(browser) = config::external_browser() {
        if browser.exists()
            && std::process::Command::new(&browser)
                .arg(url)
                .spawn()
                .is_ok()
        {
            return;
        }
        // If the custom browser failed to launch, fall through to the OS
        // default so the user isn't stranded.
    }
    open_with_system_default(url);
}

#[cfg(target_os = "windows")]
fn open_with_system_default(url: &str) {
    use std::iter::once;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    let url_w: Vec<u16> = url.encode_utf16().chain(once(0)).collect();
    let op: Vec<u16> = "open".encode_utf16().chain(once(0)).collect();
    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            op.as_ptr(),
            url_w.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            5, // SW_SHOW
        );
    }
}

#[cfg(not(target_os = "windows"))]
fn open_with_system_default(_url: &str) {}
