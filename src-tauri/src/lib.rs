pub mod host;
pub mod layout;
pub mod paths;
pub mod saver;

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use isotope_core::controller::{Controller, Toast};
use isotope_core::model::Config;
use isotope_core::persistence::{load, save, LoadOutcome};
use saver::Saver;
use tauri::window::Color;
use tauri::{LogicalPosition, LogicalSize, Manager, RunEvent, WebviewBuilder, WebviewUrl, WindowBuilder, WindowEvent};

/// Window background, matched by the shell CSS so hidden app areas never flash white.
pub const BACKGROUND: Color = Color(27, 27, 31, 255);

/// Returns the config, whether it is read-only, and toasts to show once the shell is ready.
fn load_config(app_data: &Path) -> (Config, bool, Vec<Toast>) {
    match load(&paths::config_path(app_data)) {
        Ok(LoadOutcome::Loaded(config) | LoadOutcome::Missing(config)) => (config, false, Vec::new()),
        Ok(LoadOutcome::Recovered { config, backup }) => (
            config,
            false,
            vec![Toast::Warning(format!(
                "Settings were unreadable and have been reset. The old file was kept at {}",
                backup.display()
            ))],
        ),
        Ok(LoadOutcome::NewerVersion(config)) => (
            config,
            true,
            vec![Toast::Error("Settings were created by a newer Isotope version. Changes won't be saved.".into())],
        ),
        Err(e) => (
            Config::default(),
            true,
            vec![Toast::Error(format!("Couldn't read settings ({e}). Changes won't be saved."))],
        ),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let app_data = app.path().app_data_dir()?;
            let (config, read_only, toasts) = load_config(&app_data);

            let error_handle = handle.clone();
            let saver = Saver::spawn(paths::config_path(&app_data), Duration::from_millis(500), move |e| {
                host::emit_toast(&error_handle, Toast::Error(format!("Couldn't save settings: {e}")));
            });
            app.manage(host::Shared {
                controller: Mutex::new(Controller::new(config, read_only)),
                saver,
                pending_toasts: Mutex::new(toasts),
            });

            let window = WindowBuilder::new(app, "main")
                .title("Isotope")
                .inner_size(1280.0, 800.0)
                .min_inner_size(640.0, 400.0)
                .background_color(BACKGROUND)
                .build()?;
            window.add_child(
                WebviewBuilder::new("shell", WebviewUrl::App("index.html".into())),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(1280.0, 800.0),
            )?;
            let layout_handle = handle.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } = event {
                    host::layout(&layout_handle);
                }
            });

            let outcome = host::with_controller(&handle, |c| c.startup(Instant::now()));
            host::apply(&handle, outcome);
            host::spawn_hibernation_ticker(handle);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            // Flush changes still waiting in the debounced saver.
            let config = host::with_controller(handle, |c| (!c.read_only()).then(|| c.config().clone()));
            if let (Some(config), Ok(app_data)) = (config, handle.path().app_data_dir()) {
                let _ = save(&paths::config_path(&app_data), &config);
            }
        }
    });
}
