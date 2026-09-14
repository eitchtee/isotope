mod common;

use std::fs;

use common::new_app;
use isotope_core::model::Config;
use isotope_core::persistence::{load, save, LoadOutcome};

#[test]
fn missing_file_yields_default_config() {
    let dir = tempfile::tempdir().unwrap();
    let outcome = load(&dir.path().join("isotope.json")).unwrap();
    assert!(matches!(outcome, LoadOutcome::Missing(c) if c == Config::default()));
}

#[test]
fn save_then_load_round_trips_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("isotope.json");
    let mut config = Config::default();
    config.add_app(new_app("Gmail")).unwrap();

    save(&path, &config).unwrap();

    assert!(matches!(load(&path).unwrap(), LoadOutcome::Loaded(c) if c == config));
    assert!(!path.with_extension("json.tmp").exists());
}

#[test]
fn save_overwrites_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("isotope.json");
    save(&path, &Config::default()).unwrap();

    let mut config = Config::default();
    config.add_profile("Work");
    save(&path, &config).unwrap();

    assert!(matches!(load(&path).unwrap(), LoadOutcome::Loaded(c) if c.profiles.len() == 2));
}

#[test]
fn corrupt_file_is_moved_to_backup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("isotope.json");
    fs::write(&path, "{ not json").unwrap();

    match load(&path).unwrap() {
        LoadOutcome::Recovered { config, backup } => {
            assert_eq!(config, Config::default());
            assert_eq!(backup, dir.path().join("isotope.json.bak"));
            assert_eq!(fs::read_to_string(&backup).unwrap(), "{ not json");
            assert!(!path.exists());
        }
        other => panic!("expected Recovered, got {other:?}"),
    }
}

#[test]
fn file_violating_invariants_is_treated_as_corrupt() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("isotope.json");
    let mut config = Config::default();
    config.sidebar.push(isotope_core::model::SidebarItem::App { id: "ghost".into() });
    fs::write(&path, serde_json::to_string(&config).unwrap()).unwrap();

    assert!(matches!(load(&path).unwrap(), LoadOutcome::Recovered { .. }));
}

#[test]
fn newer_version_is_left_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("isotope.json");
    fs::write(&path, r#"{ "version": 99, "somethingNew": true }"#).unwrap();

    assert!(matches!(load(&path).unwrap(), LoadOutcome::NewerVersion(_)));
    assert!(path.exists());
    assert!(!dir.path().join("isotope.json.bak").exists());
}
