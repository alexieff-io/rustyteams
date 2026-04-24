//! Bridge between Web Notifications and Windows toast notifications.
//!
//! The renderer injects a polyfill replacing `window.Notification`. When the
//! page constructs one, it hops a `ProcessMessage` to the browser process,
//! where `TeamsClient::on_process_message_received` calls `show()` here.

#[cfg(target_os = "windows")]
pub fn show(title: &str, body: &str) {
    use winrt_notification::{Duration, Sound, Toast};
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .sound(Some(Sound::Default))
        .duration(Duration::Short)
        .show();
}

#[cfg(not(target_os = "windows"))]
pub fn show(_title: &str, _body: &str) {}
