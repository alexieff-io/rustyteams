use cef::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{browser, external};

static OPEN_BROWSERS: AtomicUsize = AtomicUsize::new(0);

wrap_life_span_handler! {
    pub struct TeamsLifeSpanHandler;

    impl LifeSpanHandler {
        fn on_before_popup(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _popup_id: i32,
            target_url: Option<&CefString>,
            _target_frame_name: Option<&CefString>,
            _target_disposition: WindowOpenDisposition,
            _user_gesture: i32,
            _popup_features: Option<&PopupFeatures>,
            _window_info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<DictionaryValue>>,
            _no_javascript_access: Option<&mut i32>,
        ) -> i32 {
            let url = target_url.map(CefString::to_string).unwrap_or_default();
            if is_url_allowed(&url) {
                // Allow CEF to create the popup (OAuth, meeting pop-out, etc.).
                0
            } else {
                external::open_url(&url);
                // Block the popup.
                1
            }
        }

        fn on_after_created(&self, _browser: Option<&mut Browser>) {
            OPEN_BROWSERS.fetch_add(1, Ordering::SeqCst);
        }

        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            if OPEN_BROWSERS.fetch_sub(1, Ordering::SeqCst) == 1 {
                quit_message_loop();
            }
        }
    }
}

fn is_url_allowed(url: &str) -> bool {
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
