// Declaring an app manifest makes every command deny-by-default, so remote app
// pages can only call what capabilities/apps.json grants them.
const COMMANDS: &[&str] = &[
    "get_state",
    "take_pending_toasts",
    "activate_app",
    "toggle_folder_panel",
    "add_app",
    "update_app",
    "remove_app",
    "reload_app",
    "hibernate_app",
    "wake_app",
    "add_folder",
    "rename_folder",
    "remove_folder",
    "move_item",
    "add_profile",
    "rename_profile",
    "remove_profile",
    "set_default_hibernation_minutes",
    "set_overlay_open",
    "set_toast_visible",
    "app_icon",
    "set_app_icon",
    "bridge_notify",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("failed to run tauri-build");
}
