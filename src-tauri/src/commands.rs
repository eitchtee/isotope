//! Commands callable from the shell webview only (see capabilities/shell.json).
//! All are async: commands that may create webviews must not run on the main thread.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use isotope_core::controller::{Controller, Outcome, Snapshot, Toast};
use isotope_core::model::SidebarItem;
use isotope_core::ops::{Container, NewApp, OpError};
use tauri::{AppHandle, Manager};

use crate::host::{app_label, apply, with_controller, Shared};
use crate::{icons, paths};

/// Runs a fallible controller call, applies its outcome and returns its value.
fn run<T>(app: &AppHandle, f: impl FnOnce(&mut Controller) -> Result<(T, Outcome), OpError>) -> Result<T, String> {
    let (value, outcome) = with_controller(app, f).map_err(|e| e.to_string())?;
    apply(app, outcome);
    Ok(value)
}

fn unit(result: Result<Outcome, OpError>) -> Result<((), Outcome), OpError> {
    result.map(|outcome| ((), outcome))
}

#[tauri::command]
pub async fn get_state(app: AppHandle) -> Snapshot {
    with_controller(&app, |c| c.snapshot())
}

#[tauri::command]
pub async fn take_pending_toasts(app: AppHandle) -> Vec<Toast> {
    std::mem::take(&mut *app.state::<Shared>().pending_toasts.lock().expect("toasts mutex poisoned"))
}

#[tauri::command]
pub async fn activate_app(app: AppHandle, id: String) -> Result<(), String> {
    run(&app, |c| unit(c.activate(&id, Instant::now())))
}

#[tauri::command]
pub async fn toggle_folder_panel(app: AppHandle, folder_id: String) -> Result<(), String> {
    run(&app, |c| unit(c.toggle_folder_panel(&folder_id)))
}

#[tauri::command]
pub async fn add_app(app: AppHandle, input: NewApp) -> Result<String, String> {
    let fetch_from = input.icon.is_none().then(|| input.url.clone());
    let id = run(&app, |c| c.add_app(input, Instant::now()))?;
    if let Some(url) = fetch_from {
        let (handle, app_id) = (app.clone(), id.clone());
        std::thread::spawn(move || {
            if let Ok(bytes) = icons::fetch_favicon(&url) {
                let _ = store_icon(&handle, &app_id, &bytes);
            }
        });
    }
    Ok(id)
}

#[tauri::command]
pub async fn update_app(app: AppHandle, id: String, input: NewApp) -> Result<(), String> {
    run(&app, |c| unit(c.update_app(&id, input)))
}

#[tauri::command]
pub async fn remove_app(app: AppHandle, id: String) -> Result<(), String> {
    run(&app, |c| unit(c.remove_app(&id)))?;
    if let Ok(app_data) = app.path().app_data_dir()
        && let Some(path) = paths::icon_path(&app_data, &id)
    {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

#[tauri::command]
pub async fn reload_app(app: AppHandle, id: String) -> Result<(), String> {
    match app.get_webview(&app_label(&id)) {
        Some(webview) => webview.reload().map_err(|e| e.to_string()),
        None => Ok(()),
    }
}

#[tauri::command]
pub async fn hibernate_app(app: AppHandle, id: String) -> Result<(), String> {
    run(&app, |c| unit(c.hibernate(&id)))
}

#[tauri::command]
pub async fn wake_app(app: AppHandle, id: String) -> Result<(), String> {
    run(&app, |c| unit(c.wake(&id, Instant::now())))
}

#[tauri::command]
pub async fn add_folder(app: AppHandle, name: String, app_ids: Vec<String>) -> Result<String, String> {
    run(&app, |c| c.add_folder(&name, &app_ids))
}

#[tauri::command]
pub async fn rename_folder(app: AppHandle, id: String, name: String) -> Result<(), String> {
    run(&app, |c| unit(c.rename_folder(&id, &name)))
}

#[tauri::command]
pub async fn remove_folder(app: AppHandle, id: String) -> Result<(), String> {
    run(&app, |c| unit(c.remove_folder(&id)))
}

#[tauri::command]
pub async fn move_item(app: AppHandle, item: SidebarItem, target: Container, index: usize) -> Result<(), String> {
    run(&app, |c| unit(c.move_item(&item, &target, index)))
}

#[tauri::command]
pub async fn add_profile(app: AppHandle, name: String) -> Result<String, String> {
    run(&app, |c| Ok(c.add_profile(&name)))
}

#[tauri::command]
pub async fn rename_profile(app: AppHandle, id: String, name: String) -> Result<(), String> {
    run(&app, |c| unit(c.rename_profile(&id, &name)))
}

#[tauri::command]
pub async fn remove_profile(app: AppHandle, id: String) -> Result<(), String> {
    run(&app, |c| unit(c.remove_profile(&id)))
}

#[tauri::command]
pub async fn set_default_hibernation_minutes(app: AppHandle, minutes: u32) {
    let outcome = with_controller(&app, |c| c.set_default_hibernation_minutes(minutes));
    apply(&app, outcome);
}

#[tauri::command]
pub async fn set_overlay_open(app: AppHandle, open: bool) {
    let outcome = with_controller(&app, |c| c.set_overlay_open(open));
    apply(&app, outcome);
}

#[tauri::command]
pub async fn set_toast_visible(app: AppHandle, visible: bool) {
    let outcome = with_controller(&app, |c| c.set_toast_visible(visible));
    apply(&app, outcome);
}

#[tauri::command]
pub async fn app_icon(app: AppHandle, id: String) -> Option<String> {
    let has_icon = with_controller(&app, |c| c.config().app(&id).is_some_and(|a| a.icon.is_some()));
    if !has_icon {
        return None;
    }
    let app_data = app.path().app_data_dir().ok()?;
    let bytes = std::fs::read(paths::icon_path(&app_data, &id)?).ok()?;
    icons::data_url(&bytes)
}

#[tauri::command]
pub async fn set_app_icon(app: AppHandle, id: String, bytes: Vec<u8>) -> Result<(), String> {
    if bytes.len() as u64 > icons::MAX_ICON_BYTES || icons::sniff_mime(&bytes).is_none() {
        return Err("Icon must be a PNG, ICO, JPEG, GIF or WebP image up to 1 MB".into());
    }
    store_icon(&app, &id, &bytes)
}

fn store_icon(app: &AppHandle, id: &str, bytes: &[u8]) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = paths::icon_path(&app_data, id).ok_or("invalid app id")?;
    icons::write_icon(&path, bytes).map_err(|e| e.to_string())?;
    let version = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    run(app, |c| unit(c.set_icon(id, Some(format!("icons/{id}.png#{version}")))))
}
