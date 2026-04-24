use cef::*;

wrap_display_handler! {
    pub struct TeamsDisplayHandler;

    impl DisplayHandler {
        fn on_title_change(&self, browser: Option<&mut Browser>, title: Option<&CefString>) {
            let Some(mut browser) = browser.cloned() else { return; };
            let Some(view) = browser_view_get_for_browser(Some(&mut browser)) else { return; };
            let Some(window) = view.window() else { return; };
            window.set_title(title);
        }
    }
}
