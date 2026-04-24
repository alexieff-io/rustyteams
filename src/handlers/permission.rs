use cef::*;

use crate::browser;

wrap_permission_handler! {
    pub struct TeamsPermissionHandler;

    impl PermissionHandler {
        fn on_request_media_access_permission(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            requesting_origin: Option<&CefString>,
            requested_permissions: u32,
            callback: Option<&mut MediaAccessCallback>,
        ) -> i32 {
            let Some(callback) = callback else { return 0; };
            if origin_is_teams(requesting_origin) {
                // Grant camera, mic, screen capture — whatever Teams asked for.
                callback.cont(requested_permissions);
                1
            } else {
                callback.cancel();
                1
            }
        }

        fn on_show_permission_prompt(
            &self,
            _browser: Option<&mut Browser>,
            _prompt_id: u64,
            requesting_origin: Option<&CefString>,
            requested_permissions: u32,
            callback: Option<&mut PermissionPromptCallback>,
        ) -> i32 {
            let Some(callback) = callback else { return 0; };
            if origin_is_teams(requesting_origin) {
                // PermissionRequestResult::ACCEPT = 0 in the CEF enum.
                callback.cont(PermissionRequestResult::ACCEPT);
                let _ = requested_permissions;
                1
            } else {
                callback.cont(PermissionRequestResult::DISMISS);
                1
            }
        }
    }
}

fn origin_is_teams(origin: Option<&CefString>) -> bool {
    let Some(origin) = origin else {
        return false;
    };
    let s = origin.to_string();
    host_from_origin(&s)
        .map(|h| browser::host_is_allowed(&h))
        .unwrap_or(false)
}

fn host_from_origin(origin: &str) -> Option<String> {
    let rest = origin.split_once("://").map(|(_, r)| r).unwrap_or(origin);
    let host = rest.split(['/', '?', '#']).next()?;
    let host = host.rsplit_once(':').map(|(h, _)| h).unwrap_or(host);
    if host.is_empty() {
        None
    } else {
        Some(host.to_ascii_lowercase())
    }
}
