use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use isotope_core::controller::{Controller, Outcome, Toast};
use isotope_core::lifecycle::Effect;
use isotope_core::links::{classify, LinkTarget};
use isotope_core::model::App;
use tauri::webview::{NewWindowFeatures, NewWindowResponse};
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Url, WebviewBuilder, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, Wry,
};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;

use crate::layout::content_bounds;
use crate::paths;
use crate::saver::Saver;

pub struct Shared {
    pub controller: Mutex<Controller>,
    pub saver: Saver,
    /// Toasts raised before the shell was listening (e.g. config recovery).
    pub pending_toasts: Mutex<Vec<Toast>>,
}

static POPUPS: AtomicUsize = AtomicUsize::new(0);

pub fn app_label(app_id: &str) -> String {
    format!("app-{app_id}")
}

pub fn with_controller<T>(handle: &AppHandle, f: impl FnOnce(&mut Controller) -> T) -> T {
    let shared = handle.state::<Shared>();
    let mut controller = shared.controller.lock().expect("controller mutex poisoned");
    f(&mut controller)
}

fn current_app(handle: &AppHandle, app_id: &str) -> Option<App> {
    with_controller(handle, |c| c.config().app(app_id).cloned())
}

/// Applies a controller outcome. Must be called without holding the controller lock.
pub fn apply(handle: &AppHandle, outcome: Outcome) {
    let mut needs_layout = outcome.layout;
    for effect in &outcome.effects {
        needs_layout = true;
        match effect {
            Effect::Create { id, .. } => {
                if let Err(message) = create_app_webview(handle, id) {
                    let failed = with_controller(handle, |c| c.webview_failed(id, &message));
                    apply(handle, failed);
                }
            }
            Effect::Destroy(id) => {
                if let Some(webview) = handle.get_webview(&app_label(id)) {
                    let _ = webview.close();
                }
            }
            // Visibility is derived from controller state in `layout`.
            Effect::Show(_) | Effect::Hide(_) => {}
        }
    }
    if needs_layout {
        layout(handle);
    }
    if outcome.save {
        let config = with_controller(handle, |c| (!c.read_only()).then(|| c.config().clone()));
        if let Some(config) = config {
            handle.state::<Shared>().saver.request(config);
        }
    }
    if let Some(profile_id) = outcome.delete_profile_data {
        delete_profile_data(handle, &profile_id);
    }
    if let Some(n) = outcome.notification {
        let _ = handle.notification().builder().title(n.title).body(n.body).show();
    }
    for toast in outcome.toasts {
        emit_toast(handle, toast);
    }
    if outcome.changed {
        let snapshot = with_controller(handle, |c| c.snapshot());
        let _ = handle.emit_to("shell", "state-changed", snapshot);
    }
}

pub fn emit_toast(handle: &AppHandle, toast: Toast) {
    let _ = handle.emit_to("shell", "toast", toast);
}

/// Sizes the shell to the window and every app webview to the content area.
/// Only the active app is visible, and not while a shell overlay is open.
pub fn layout(handle: &AppHandle) {
    let Some(window) = handle.get_window("main") else { return };
    let Ok(physical) = window.inner_size() else { return };
    let size = physical.to_logical::<f64>(window.scale_factor().unwrap_or(1.0));
    let (active, view) = with_controller(handle, |c| (c.active().map(str::to_owned), c.view()));
    let bounds = content_bounds(size.width, size.height, view);

    for webview in window.webviews() {
        if webview.label() == "shell" {
            let _ = webview.set_position(LogicalPosition::new(0.0, 0.0));
            let _ = webview.set_size(size);
            continue;
        }
        let Some(app_id) = webview.label().strip_prefix("app-") else { continue };
        let _ = webview.set_position(LogicalPosition::new(bounds.x, bounds.y));
        let _ = webview.set_size(LogicalSize::new(bounds.width, bounds.height));
        let visible = active.as_deref() == Some(app_id) && !view.overlay_open;
        let _ = if visible { webview.show() } else { webview.hide() };
    }
}

fn create_app_webview(handle: &AppHandle, app_id: &str) -> Result<(), String> {
    if handle.get_webview(&app_label(app_id)).is_some() {
        return Ok(());
    }
    let app = current_app(handle, app_id).ok_or("app no longer exists")?;
    let window = handle.get_window("main").ok_or("main window is missing")?;
    let builder = app_webview_builder(handle, &app)?;
    window
        .add_child(builder, LogicalPosition::new(0.0, 0.0), LogicalSize::new(0.0, 0.0))
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn app_webview_builder(handle: &AppHandle, app: &App) -> Result<WebviewBuilder<Wry>, String> {
    let url: Url = app.url.parse().map_err(|e| format!("invalid URL {}: {e}", app.url))?;
    let app_data = handle.path().app_data_dir().map_err(|e| e.to_string())?;

    let nav_handle = handle.clone();
    let nav_id = app.id.clone();
    let popup_handle = handle.clone();
    let popup_id = app.id.clone();

    let mut builder = WebviewBuilder::new(app_label(&app.id), WebviewUrl::External(url))
        .on_navigation(move |url| allow_navigation(&nav_handle, &nav_id, url.as_str()))
        .on_new_window(move |url, features| new_window(&popup_handle, &popup_id, url, features))
        .on_document_title_changed(|webview, title| {
            let Some(app_id) = webview.label().strip_prefix("app-") else { return };
            let handle = webview.app_handle().clone();
            let outcome = with_controller(&handle, |c| c.page_title(app_id, &title));
            apply(&handle, outcome);
        });
    if let Some(user_agent) = &app.user_agent {
        builder = builder.user_agent(user_agent);
    }
    with_profile(builder, &app_data, &app.profile_id)
}

#[cfg(not(target_os = "macos"))]
fn with_profile(builder: WebviewBuilder<Wry>, app_data: &Path, profile_id: &str) -> Result<WebviewBuilder<Wry>, String> {
    let dir = paths::profile_dir(app_data, profile_id).ok_or("invalid profile id")?;
    Ok(builder.data_directory(dir))
}

#[cfg(target_os = "macos")]
fn with_profile(builder: WebviewBuilder<Wry>, _app_data: &Path, profile_id: &str) -> Result<WebviewBuilder<Wry>, String> {
    Ok(builder.data_store_identifier(paths::profile_store_id(profile_id)))
}

/// `on_navigation`: `true` keeps the navigation inside the app webview (spec §4.4).
fn allow_navigation(handle: &AppHandle, app_id: &str, url: &str) -> bool {
    let Some(app) = current_app(handle, app_id) else { return true };
    match classify(&app.url, &app.allowed_domains, url) {
        LinkTarget::InApp => true,
        LinkTarget::External | LinkTarget::OsOpener => {
            open_outside(handle, url);
            false
        }
    }
}

fn new_window(handle: &AppHandle, app_id: &str, url: Url, features: NewWindowFeatures) -> NewWindowResponse<Wry> {
    let Some(app) = current_app(handle, app_id) else { return NewWindowResponse::Deny };
    match classify(&app.url, &app.allowed_domains, url.as_str()) {
        LinkTarget::InApp => match build_popup(handle, &app, url, features) {
            Ok(window) => NewWindowResponse::Create { window },
            Err(message) => {
                emit_toast(handle, Toast::Error(format!("Couldn't open popup: {message}")));
                NewWindowResponse::Deny
            }
        },
        LinkTarget::External | LinkTarget::OsOpener => {
            open_outside(handle, url.as_str());
            NewWindowResponse::Deny
        }
    }
}

fn build_popup(handle: &AppHandle, app: &App, url: Url, features: NewWindowFeatures) -> Result<WebviewWindow<Wry>, String> {
    let app_data = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let label = format!("popup-{}", POPUPS.fetch_add(1, Ordering::Relaxed));
    let mut builder = WebviewWindowBuilder::new(handle, label, WebviewUrl::External(url))
        .title(&app.name)
        .inner_size(520.0, 680.0)
        .window_features(features);
    if let Some(user_agent) = &app.user_agent {
        builder = builder.user_agent(user_agent);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let dir = paths::profile_dir(&app_data, &app.profile_id).ok_or("invalid profile id")?;
        builder = builder.data_directory(dir);
    }
    #[cfg(target_os = "macos")]
    {
        let _ = &app_data;
        builder = builder.data_store_identifier(paths::profile_store_id(&app.profile_id));
    }
    builder.build().map_err(|e| e.to_string())
}

fn open_outside(handle: &AppHandle, url: &str) {
    if handle.opener().open_url(url, None::<&str>).is_err() {
        emit_toast(handle, Toast::Error("Couldn't open link".into()));
    }
}

fn delete_profile_data(handle: &AppHandle, profile_id: &str) {
    #[cfg(target_os = "macos")]
    {
        // WKWebsiteDataStore removal isn't exposed by Tauri; the store is simply left unused.
        let _ = (handle, profile_id);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let Ok(app_data) = handle.path().app_data_dir() else { return };
        let Some(dir) = paths::profile_dir(&app_data, profile_id) else { return };
        if dir.exists() && std::fs::remove_dir_all(&dir).is_err() {
            emit_toast(handle, Toast::Warning("Couldn't delete the profile's browsing data".into()));
        }
    }
}

/// Checks hibernation deadlines every 5 seconds (timeouts are measured in minutes).
pub fn spawn_hibernation_ticker(handle: AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(5));
            let outcome = with_controller(&handle, |c| c.tick(Instant::now()));
            if !outcome.effects.is_empty() {
                apply(&handle, outcome);
            }
        }
    });
}
