use std::time::Instant;

use tauri::{Manager, Webview};

use crate::host::{apply, with_controller};

/// Injected into every app webview: routes the web Notification API to `bridge_notify` (spec §4.6).
/// The permission reflects the app's setting at webview creation; Rust re-checks on every call.
pub fn init_script(notifications_enabled: bool) -> String {
    let permission = if notifications_enabled { "granted" } else { "denied" };
    TEMPLATE.replace("__PERMISSION__", permission)
}

const TEMPLATE: &str = r#"(() => {
  const permission = "__PERMISSION__";
  class IsotopeNotification extends EventTarget {
    constructor(title, options = {}) {
      super();
      this.title = String(title);
      this.body = options && options.body ? String(options.body) : "";
      this.onclick = null;
      this.onclose = null;
      this.onerror = null;
      this.onshow = null;
      if (permission === "granted") {
        window.__TAURI_INTERNALS__
          .invoke("bridge_notify", { title: this.title, body: this.body })
          .catch(() => {});
      }
    }
    close() {}
    static get permission() {
      return permission;
    }
    static requestPermission(callback) {
      if (typeof callback === "function") callback(permission);
      return Promise.resolve(permission);
    }
  }
  Object.defineProperty(window, "Notification", { value: IsotopeNotification, writable: true, configurable: true });
})();"#;

#[tauri::command]
pub async fn bridge_notify(webview: Webview, title: String, body: String) {
    // The app is identified by the calling webview's label, never by the payload.
    let Some(app_id) = webview.label().strip_prefix("app-").map(str::to_owned) else { return };
    let handle = webview.app_handle().clone();
    let outcome = with_controller(&handle, |c| c.notify(&app_id, &title, &body, Instant::now()));
    apply(&handle, outcome);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_embeds_the_permission() {
        assert!(init_script(true).contains(r#"const permission = "granted";"#));
        assert!(init_script(false).contains(r#"const permission = "denied";"#));
        assert!(!init_script(true).contains("__PERMISSION__"));
    }
}
