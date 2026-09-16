//! Reduces the cost of app webviews that aren't visible.
//!
//! Every background app asks the engine to use less memory. Apps with
//! `suspend_in_background` are also frozen until they're shown again.
//! Only WebView2 (Windows) exposes these controls; elsewhere this is a no-op.

use std::collections::HashSet;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, Webview};

use crate::host::Shared;

/// App ids currently in background mode, so each transition is applied once.
#[derive(Debug, Default)]
pub struct BackgroundApps(Mutex<HashSet<String>>);

impl BackgroundApps {
    /// Records the app's mode; returns `true` when this is a change.
    fn mark(&self, app_id: &str, background: bool) -> bool {
        let mut apps = self.0.lock().expect("background mutex poisoned");
        if background { apps.insert(app_id.into()) } else { apps.remove(app_id) }
    }

    /// Called when a webview is destroyed, so a recreated one starts fresh.
    pub fn forget(&self, app_id: &str) {
        self.0.lock().expect("background mutex poisoned").remove(app_id);
    }
}

/// Call after hiding the webview (WebView2 only suspends hidden webviews).
pub fn enter(handle: &AppHandle, webview: &Webview, app_id: &str, suspend: bool) {
    if handle.state::<Shared>().background.mark(app_id, true) {
        set_background(webview, true, suspend);
    }
}

/// Call before showing the webview.
pub fn leave(handle: &AppHandle, webview: &Webview, app_id: &str) {
    if handle.state::<Shared>().background.mark(app_id, false) {
        set_background(webview, false, false);
    }
}

#[cfg(windows)]
fn set_background(webview: &Webview, background: bool, suspend: bool) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL, ICoreWebView2_3,
        ICoreWebView2_19,
    };
    use webview2_com::TrySuspendCompletedHandler;
    use windows_core::Interface;

    let _ = webview.with_webview(move |platform| unsafe {
        let Ok(core) = platform.controller().CoreWebView2() else { return };
        // Needs WebView2 Runtime 1.0.2210+; older runtimes simply skip this.
        if let Ok(core) = core.cast::<ICoreWebView2_19>() {
            let level = if background {
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
            } else {
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
            };
            let _ = core.SetMemoryUsageTargetLevel(level);
        }
        if let Ok(core) = core.cast::<ICoreWebView2_3>() {
            if background && suspend {
                let handler = TrySuspendCompletedHandler::create(Box::new(|_, _| Ok(())));
                let _ = core.TrySuspend(&handler);
            } else if !background {
                // Showing resumes automatically; this also covers a suspend still in flight.
                let _ = core.Resume();
            }
        }
    });
}

#[cfg(not(windows))]
fn set_background(_webview: &Webview, _background: bool, _suspend: bool) {}
