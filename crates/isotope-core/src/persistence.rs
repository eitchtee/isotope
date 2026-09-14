use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::{Config, CURRENT_VERSION};

#[derive(Debug)]
pub enum LoadOutcome {
    /// File parsed and passed invariant checks.
    Loaded(Config),
    /// No file yet (first run).
    Missing(Config),
    /// File was unreadable as a valid config; it was renamed to `backup`.
    Recovered { config: Config, backup: PathBuf },
    /// File was written by a newer Isotope. Best-effort parse; caller must not save.
    NewerVersion(Config),
}

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn load(path: &Path) -> io::Result<LoadOutcome> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(LoadOutcome::Missing(Config::default()));
        }
        Err(e) => return Err(e),
    };

    // Check the version before strict parsing so a newer file is never
    // mistaken for a corrupt one and moved aside.
    let version = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.get("version").and_then(serde_json::Value::as_u64));
    if version.is_some_and(|v| v > u64::from(CURRENT_VERSION)) {
        let config = serde_json::from_str(&text).unwrap_or_default();
        return Ok(LoadOutcome::NewerVersion(config));
    }

    match serde_json::from_str::<Config>(&text) {
        Ok(config) if config.check_invariants().is_ok() => Ok(LoadOutcome::Loaded(config)),
        _ => {
            let backup = path.with_extension("json.bak");
            fs::rename(path, &backup)?;
            Ok(LoadOutcome::Recovered { config: Config::default(), backup })
        }
    }
}

/// Writes to a sibling temp file, then renames over the target so a crash
/// mid-write never leaves a truncated config.
pub fn save(path: &Path, config: &Config) -> Result<(), PersistError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
