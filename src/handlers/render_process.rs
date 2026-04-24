//! Renderer-process handler. Runs inside CEF's child "renderer" process,
//! one instance per page. Its job is to replace the page's Notification API
//! with a polyfill that forwards every `new Notification(...)` call back to
//! the browser process via a CEF process-message, where
//! `notifications::show` turns it into a native Windows toast.

use cef::*;

pub const IPC_NOTIFY: &str = "rustyteams.notify";

/// Injected polyfill. Runs once per V8 context (per frame). Keeps the real
/// constructor available on `window.__OriginalNotification__` in case Teams
/// feature-detects it, but uses our bridge for anything it actually fires.
const NOTIFICATION_SHIM_JS: &str = r#"
(function() {
    if (window.__rustyTeamsNotifyInstalled) return;
    window.__rustyTeamsNotifyInstalled = true;

    const bridge = function(title, body) {
        try {
            if (typeof window.__rustyTeamsNotify__ === 'function') {
                window.__rustyTeamsNotify__(String(title || ''), String(body || ''));
            }
        } catch (e) { /* swallow */ }
    };

    const Original = window.Notification;
    function RTNotification(title, options) {
        options = options || {};
        bridge(title, options.body);
        this.title = title;
        this.body = options.body || '';
        this.onclick = null;
    }
    RTNotification.prototype = Object.create(EventTarget.prototype);
    RTNotification.prototype.close = function() {};
    RTNotification.prototype.addEventListener = function() {};
    RTNotification.prototype.removeEventListener = function() {};
    RTNotification.prototype.dispatchEvent = function() { return true; };

    Object.defineProperty(RTNotification, 'permission', {
        get: function() { return 'granted'; }
    });
    RTNotification.requestPermission = function(cb) {
        const p = Promise.resolve('granted');
        if (typeof cb === 'function') {
            try { cb('granted'); } catch (_) {}
        }
        return p;
    };
    RTNotification.maxActions = 0;

    if (Original) window.__OriginalNotification__ = Original;
    window.Notification = RTNotification;
})();
"#;

wrap_v8_handler! {
    struct NotifyV8Handler;

    impl V8Handler {
        fn execute(
            &self,
            _name: Option<&CefString>,
            _object: Option<&mut V8Value>,
            arguments: Option<&[Option<V8Value>]>,
            _retval: Option<&mut Option<V8Value>>,
            _exception: Option<&mut CefString>,
        ) -> i32 {
            let title = arguments
                .and_then(|a| a.first().and_then(|v| v.as_ref()))
                .and_then(read_v8_string)
                .unwrap_or_default();
            let body = arguments
                .and_then(|a| a.get(1).and_then(|v| v.as_ref()))
                .and_then(read_v8_string)
                .unwrap_or_default();

            // Resolve the current frame via V8 so we don't need to stash
            // browser/frame refs in this handler.
            let Some(ctx) = v8_context_get_current_context() else { return 1; };
            let Some(frame) = ctx.frame() else { return 1; };

            let Some(mut msg) = process_message_create(Some(&CefString::from(IPC_NOTIFY))) else {
                return 1;
            };
            if let Some(args) = msg.argument_list() {
                let _ = args.set_size(2);
                let _ = args.set_string(0, Some(&CefString::from(title.as_str())));
                let _ = args.set_string(1, Some(&CefString::from(body.as_str())));
            }
            frame.send_process_message(ProcessId::BROWSER, Some(&mut msg));
            1
        }
    }
}

fn read_v8_string(v: &V8Value) -> Option<String> {
    if v.is_string() == 0 {
        return None;
    }
    Some(CefString::from(&v.string_value()).to_string())
}

wrap_render_process_handler! {
    pub struct TeamsRenderProcessHandler;

    impl RenderProcessHandler {
        fn on_context_created(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            context: Option<&mut V8Context>,
        ) {
            // Skip CEF-internal pages (devtools://, chrome://, about:blank, etc.).
            // Injecting into them has historically crashed Chromium's own UI.
            let is_web_page = match frame.as_deref() {
                Some(f) => {
                    let u = CefString::from(&f.url()).to_string();
                    u.starts_with("http://") || u.starts_with("https://")
                }
                None => false,
            };
            if !is_web_page {
                return;
            }

            let Some(ctx) = context else { return; };
            let Some(global) = ctx.global() else { return; };

            // Expose the native bridge as window.__rustyTeamsNotify__.
            let mut handler = NotifyV8Handler::new();
            let name = CefString::from("__rustyTeamsNotify__");
            if let Some(mut func) = v8_value_create_function(Some(&name), Some(&mut handler)) {
                let _ = global.set_value_bykey(
                    Some(&name),
                    Some(&mut func),
                    V8Propertyattribute::default(),
                );
            }

            // Install the Notification polyfill.
            let code = CefString::from(NOTIFICATION_SHIM_JS);
            let url = CefString::from("rustyteams://notification-shim.js");
            let mut retval: Option<V8Value> = None;
            let mut exception: Option<V8Exception> = None;
            let _ = ctx.eval(Some(&code), Some(&url), 0, Some(&mut retval), Some(&mut exception));
        }
    }
}
