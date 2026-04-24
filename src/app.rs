use cef::*;

use crate::browser;

wrap_app! {
    pub struct TeamsApp;

    impl App {
        fn on_before_command_line_processing(
            &self,
            _process_type: Option<&CefString>,
            command_line: Option<&mut CommandLine>,
        ) {
            let Some(cmd) = command_line else { return; };
            let mut switches: Vec<String> = [
                "disable-extensions",
                "disable-sync",
                "disable-translate",
                "disable-background-networking",
                "disable-component-update",
                // Strip Chromium features that chat apps never need and that
                // otherwise spam the console with device-enumeration errors.
                "disable-features=TranslateUI,HardwareMediaKeyHandling,MediaRouter,WebUSB,WebBluetooth,HidBlocklist",
                "autoplay-policy=no-user-gesture-required",
                // Throttle background tabs hard when Teams is hidden.
                "enable-aggressive-background-gc",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();

            // Opt-in remote debugging: set RUSTYTEAMS_DEBUG_PORT=9222 and
            // browse to http://localhost:9222 from Chrome/Edge to get
            // DevTools externally (a workaround while in-app DevTools is off).
            if let Ok(port) = std::env::var("RUSTYTEAMS_DEBUG_PORT") {
                switches.push(format!("remote-debugging-port={port}"));
                switches.push("remote-allow-origins=*".to_string());
            }

            for switch in &switches {
                let switch = switch.as_str();
                let (name, value) = match switch.split_once('=') {
                    Some((n, v)) => (n, Some(v)),
                    None => (switch, None),
                };
                let name = CefString::from(name);
                if let Some(v) = value {
                    let value = CefString::from(v);
                    cmd.append_switch_with_value(Some(&name), Some(&value));
                } else {
                    cmd.append_switch(Some(&name));
                }
            }
        }

        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(TeamsBrowserProcessHandler::new())
        }

        fn render_process_handler(&self) -> Option<RenderProcessHandler> {
            Some(crate::handlers::render_process::TeamsRenderProcessHandler::new())
        }
    }
}

wrap_browser_process_handler! {
    pub struct TeamsBrowserProcessHandler;

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            browser::create_main_window();
        }
    }
}
