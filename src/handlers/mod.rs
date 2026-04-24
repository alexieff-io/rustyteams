use cef::*;

mod display;
mod download;
#[cfg(target_os = "windows")]
mod keyboard;
mod life_span;
mod permission;
pub mod render_process;
mod request;

use crate::notifications;

wrap_client! {
    pub struct TeamsClient;

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(display::TeamsDisplayHandler::new())
        }
        fn download_handler(&self) -> Option<DownloadHandler> {
            Some(download::TeamsDownloadHandler::new())
        }
        fn keyboard_handler(&self) -> Option<KeyboardHandler> {
            #[cfg(target_os = "windows")]
            { Some(keyboard::TeamsKeyboardHandler::new()) }
            #[cfg(not(target_os = "windows"))]
            { None }
        }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(life_span::TeamsLifeSpanHandler::new())
        }
        fn permission_handler(&self) -> Option<PermissionHandler> {
            Some(permission::TeamsPermissionHandler::new())
        }
        fn request_handler(&self) -> Option<RequestHandler> {
            Some(request::TeamsRequestHandler::new())
        }

        fn on_process_message_received(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _source_process: ProcessId,
            message: Option<&mut ProcessMessage>,
        ) -> i32 {
            let Some(msg) = message else { return 0; };
            let name = CefString::from(&msg.name()).to_string();
            if name != render_process::IPC_NOTIFY {
                return 0;
            }
            let Some(args) = msg.argument_list() else { return 1; };
            let title = CefString::from(&args.string(0)).to_string();
            let body = CefString::from(&args.string(1)).to_string();
            notifications::show(&title, &body);
            1
        }
    }
}
