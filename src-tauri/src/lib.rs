pub mod layout;
pub mod paths;
pub mod saver;

use tauri::window::Color;
use tauri::{LogicalPosition, LogicalSize, Manager, WebviewBuilder, WebviewUrl, Window, WindowBuilder, WindowEvent};

/// Window background, matched by the shell CSS so hidden app areas never flash white.
pub const BACKGROUND: Color = Color(27, 27, 31, 255);

fn fit_shell(window: &Window) {
    let scale = window.scale_factor().unwrap_or(1.0);
    if let (Ok(size), Some(shell)) = (window.inner_size(), window.app_handle().get_webview("shell")) {
        let _ = shell.set_size(size.to_logical::<f64>(scale));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
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
            fit_shell(&window);

            let resized = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } = event {
                    fit_shell(&resized);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
