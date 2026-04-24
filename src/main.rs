#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod browser;
mod config;
mod external;
mod handlers;
mod notifications;
mod tray;

use cef::*;

fn main() -> Result<(), &'static str> {
    // Declare which CEF API version we were built against. Must be the very
    // first CEF call — every other entry point CHECKs that this has run and
    // aborts the process (STATUS_BREAKPOINT / 0x80000003) otherwise.
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    // CEF subprocesses re-invoke this same executable with --type=... .
    // `execute_process` returns non-negative for subprocesses (renderer, GPU, utility)
    // and -1 for the browser (main) process.
    let args = args::Args::new();
    let Some(cmd_line) = args.as_cmd_line() else {
        return Err("Failed to parse command line arguments");
    };

    // Tag the browser process with an explicit AppUserModelID so Windows
    // groups the taskbar icon, pins, and toast notifications as one app
    // instead of a generic "rustyteams.exe".
    #[cfg(target_os = "windows")]
    if cmd_line.has_switch(Some(&CefString::from("type"))) != 1 {
        set_app_user_model_id();
    }

    let type_switch = CefString::from("type");
    let is_browser_process = cmd_line.has_switch(Some(&type_switch)) != 1;

    let ret = execute_process(Some(args.as_main_args()), None, std::ptr::null_mut());
    if !is_browser_process {
        // Subprocess handled — exit.
        assert!(ret >= 0, "cannot execute non-browser process");
        return Ok(());
    }
    assert_eq!(ret, -1, "cannot execute browser process");

    // Distinct path so no real Chromium browser (Edge/Chrome) can mistake this
    // for one of its profiles and trigger the cross-process singleton handoff.
    let root_cache_dir = dirs::data_local_dir()
        .map(|p| p.join("io.alexieff.rustyteams"))
        .ok_or("no LOCALAPPDATA")?;
    let cache_dir = root_cache_dir.join("cache");
    let log_file = root_cache_dir.join("cef.log");
    std::fs::create_dir_all(&cache_dir).ok();

    let settings = Settings {
        no_sandbox: 1,
        persist_session_cookies: 1,
        cache_path: CefString::from(cache_dir.to_string_lossy().as_ref()),
        root_cache_path: CefString::from(root_cache_dir.to_string_lossy().as_ref()),
        user_agent: CefString::from(browser::EDGE_USER_AGENT),
        log_file: CefString::from(log_file.to_string_lossy().as_ref()),
        log_severity: LogSeverity::INFO,
        ..Default::default()
    };

    let mut app = app::TeamsApp::new();
    assert_eq!(
        initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(&mut app),
            std::ptr::null_mut(),
        ),
        1,
        "CEF initialize failed"
    );

    // Tray runs on the same thread and pumps its events through the Win32 queue,
    // which CEF's message loop also drives.
    let _tray = tray::install();

    run_message_loop();
    shutdown();
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_app_user_model_id() {
    use windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
    let id: Vec<u16> = "io.alexieff.rustyteams"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        let _ = SetCurrentProcessExplicitAppUserModelID(id.as_ptr());
    }
}
