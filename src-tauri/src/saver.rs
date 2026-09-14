use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use isotope_core::model::Config;
use isotope_core::persistence::{save, PersistError};

/// Writes the config on a background thread, collapsing bursts of requests
/// that arrive within `debounce` of each other into one write.
pub struct Saver {
    tx: Sender<Config>,
}

impl Saver {
    pub fn spawn(path: PathBuf, debounce: Duration, on_error: impl Fn(PersistError) + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel::<Config>();
        thread::spawn(move || {
            while let Ok(mut latest) = rx.recv() {
                // Keep taking newer configs until the channel is quiet for `debounce`.
                while let Ok(newer) = rx.recv_timeout(debounce) {
                    latest = newer;
                }
                if let Err(e) = save(&path, &latest) {
                    on_error(e);
                }
            }
        });
        Self { tx }
    }

    pub fn request(&self, config: Config) {
        // The thread only stops when the Saver is dropped, so sending cannot fail.
        let _ = self.tx.send(config);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread::sleep;

    use isotope_core::persistence::{load, LoadOutcome};

    use super::*;

    #[test]
    fn bursts_collapse_into_one_write_of_the_latest_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("isotope.json");
        let saver = Saver::spawn(path.clone(), Duration::from_millis(150), |e| panic!("{e}"));

        let mut first = Config::default();
        first.add_profile("First");
        let mut latest = first.clone();
        latest.add_profile("Second");

        saver.request(first);
        saver.request(latest.clone());
        sleep(Duration::from_millis(50));
        assert!(!path.exists(), "must not write before the debounce elapses");

        sleep(Duration::from_millis(500));
        assert!(matches!(load(&path).unwrap(), LoadOutcome::Loaded(c) if c == latest));
    }

    #[test]
    fn write_errors_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        let blocker = dir.path().join("not-a-dir");
        std::fs::write(&blocker, "x").unwrap();
        let (tx, rx) = mpsc::channel();
        let saver = Saver::spawn(blocker.join("isotope.json"), Duration::from_millis(10), move |e| {
            let _ = tx.send(e.to_string());
        });

        saver.request(Config::default());

        assert!(rx.recv_timeout(Duration::from_secs(2)).is_ok());
    }
}
