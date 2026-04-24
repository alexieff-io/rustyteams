use cef::*;

use crate::{browser, external};

wrap_request_handler! {
    pub struct TeamsRequestHandler;

    impl RequestHandler {
        fn on_before_browse(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _user_gesture: i32,
            _is_redirect: i32,
        ) -> i32 {
            let Some(request) = request else { return 0; };
            let url = CefString::from(&request.url()).to_string();
            if url_is_allowed(&url) { 0 } else {
                external::open_url(&url);
                1 // cancel
            }
        }

        fn on_open_urlfrom_tab(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            target_url: Option<&CefString>,
            _target_disposition: WindowOpenDisposition,
            _user_gesture: i32,
        ) -> i32 {
            let url = target_url.map(CefString::to_string).unwrap_or_default();
            if url_is_allowed(&url) { 0 } else {
                external::open_url(&url);
                1
            }
        }
    }
}

fn url_is_allowed(url: &str) -> bool {
    // Non-http schemes (data:, blob:, chrome:, devtools:) are internal; allow them.
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return true;
    }
    url_host(url)
        .map(|h| browser::host_is_allowed(&h))
        .unwrap_or(false)
}

fn url_host(url: &str) -> Option<String> {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let host = rest.split(['/', '?', '#']).next()?;
    let host = host.split('@').next_back()?;
    let host = host.rsplit_once(':').map(|(h, _)| h).unwrap_or(host);
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}
