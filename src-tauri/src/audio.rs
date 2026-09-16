//! Per-app audio: reports which apps are playing sound, and mutes them.
//! Only WebView2 (Windows) exposes this; elsewhere these functions do nothing.

use tauri::{AppHandle, Webview};

/// Subscribes to the page's "playing audio" changes and forwards them to the controller.
#[cfg(windows)]
pub fn watch(handle: &AppHandle, webview: &Webview, app_id: &str) {
    use webview2_com::IsDocumentPlayingAudioChangedEventHandler;
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_8;
    use windows_core::{Interface, BOOL};

    use crate::host::{apply, with_controller};

    let handle = handle.clone();
    let app_id = app_id.to_owned();
    let _ = webview.with_webview(move |platform| unsafe {
        let Ok(core) = platform.controller().CoreWebView2() else { return };
        let Ok(core) = core.cast::<ICoreWebView2_8>() else { return };
        let handler = IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |sender, _| {
            let Some(sender) = sender.and_then(|s| s.cast::<ICoreWebView2_8>().ok()) else { return Ok(()) };
            let mut playing = BOOL::default();
            sender.IsDocumentPlayingAudio(&mut playing)?;
            let outcome = with_controller(&handle, |c| c.audio_changed(&app_id, playing.as_bool()));
            apply(&handle, outcome);
            Ok(())
        }));
        let mut token = 0i64;
        let _ = core.add_IsDocumentPlayingAudioChanged(&handler, &mut token);
    });
}

#[cfg(windows)]
pub fn set_muted(webview: &Webview, muted: bool) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_8;
    use windows_core::Interface;

    let _ = webview.with_webview(move |platform| unsafe {
        let Ok(core) = platform.controller().CoreWebView2() else { return };
        if let Ok(core) = core.cast::<ICoreWebView2_8>() {
            let _ = core.SetIsMuted(muted);
        }
    });
}

#[cfg(not(windows))]
pub fn watch(_handle: &AppHandle, _webview: &Webview, _app_id: &str) {}

#[cfg(not(windows))]
pub fn set_muted(_webview: &Webview, _muted: bool) {}
