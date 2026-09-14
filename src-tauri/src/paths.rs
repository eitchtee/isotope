use std::path::{Path, PathBuf};

use uuid::Uuid;

pub const CONFIG_FILE: &str = "isotope.json";

pub fn config_path(app_data: &Path) -> PathBuf {
    app_data.join(CONFIG_FILE)
}

/// Ids come from the config file, so only plain segments may become paths.
fn is_safe_segment(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn profile_dir(app_data: &Path, profile_id: &str) -> Option<PathBuf> {
    is_safe_segment(profile_id).then(|| app_data.join("profiles").join(profile_id))
}

/// Stable per-profile identifier for WKWebView data stores (macOS 14+).
pub fn profile_store_id(profile_id: &str) -> [u8; 16] {
    *Uuid::new_v5(&Uuid::NAMESPACE_URL, format!("isotope-profile:{profile_id}").as_bytes()).as_bytes()
}

pub fn icon_path(app_data: &Path, app_id: &str) -> Option<PathBuf> {
    is_safe_segment(app_id).then(|| app_data.join("icons").join(format!("{app_id}.png")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_paths_inside_app_data() {
        let base = Path::new("data");
        assert_eq!(config_path(base), base.join("isotope.json"));
        assert_eq!(profile_dir(base, "default"), Some(base.join("profiles").join("default")));
        assert_eq!(
            profile_dir(base, "6f1c2b9e-0d1a-4c8e-9a51-3f0b7d2e4a10"),
            Some(base.join("profiles").join("6f1c2b9e-0d1a-4c8e-9a51-3f0b7d2e4a10"))
        );
        assert_eq!(icon_path(base, "a1"), Some(base.join("icons").join("a1.png")));
    }

    #[test]
    fn rejects_unsafe_ids() {
        let base = Path::new("data");
        for id in ["", "..", "../evil", "a/b", "a\\b", "C:"] {
            assert_eq!(profile_dir(base, id), None, "{id}");
            assert_eq!(icon_path(base, id), None, "{id}");
        }
    }

    #[test]
    fn store_ids_are_stable_and_distinct() {
        assert_eq!(profile_store_id("default"), profile_store_id("default"));
        assert_ne!(profile_store_id("default"), profile_store_id("work"));
    }
}
