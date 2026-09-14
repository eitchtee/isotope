mod common;

use std::time::{Duration, Instant};

use common::new_app;
use isotope_core::badge::Badge;
use isotope_core::controller::{Controller, View};
use isotope_core::lifecycle::{AppState, Effect};
use isotope_core::model::Config;
use isotope_core::ops::OpError;

const TEN_MIN: Duration = Duration::from_secs(600);

fn create(id: &str, visible: bool) -> Effect {
    Effect::Create { id: id.into(), visible }
}

/// Apps A and B at the top level, app C inside folder "F".
/// Returns the config, the app ids [A, B, C] and the folder id.
fn setup_config() -> (Config, Vec<String>, String) {
    let mut config = Config::default();
    let ids: Vec<String> = ["A", "B", "C"].iter().map(|n| config.add_app(new_app(n)).unwrap()).collect();
    let folder = config.add_folder("F", &[ids[2].clone()]).unwrap();
    (config, ids, folder)
}

#[test]
fn startup_creates_background_apps_and_activates_the_first() {
    let (config, ids, _) = setup_config();
    let mut c = Controller::new(config, false);

    let out = c.startup(Instant::now());

    assert_eq!(
        out.effects,
        vec![
            create(&ids[0], false),
            create(&ids[1], false),
            create(&ids[2], false),
            Effect::Show(ids[0].clone()),
        ]
    );
    assert!(out.changed && out.layout && out.save);
    assert_eq!(c.active(), Some(ids[0].as_str()));
    assert_eq!(c.config().settings.last_active_app_id.as_deref(), Some(ids[0].as_str()));
}

#[test]
fn startup_respects_start_hibernated_and_last_active_app_in_a_folder() {
    let (mut config, ids, folder) = setup_config();
    config.apps[1].hibernation.start_hibernated = true;
    config.settings.last_active_app_id = Some(ids[2].clone());
    let mut c = Controller::new(config, false);

    let out = c.startup(Instant::now());

    assert_eq!(
        out.effects,
        vec![create(&ids[0], false), create(&ids[2], false), Effect::Show(ids[2].clone())]
    );
    assert!(!out.save);
    assert_eq!(c.app_state(&ids[1]), AppState::Hibernated);
    assert_eq!(c.snapshot().folder_panel, Some(folder));
    assert!(c.view().folder_panel_open);
}

#[test]
fn startup_with_missing_last_active_falls_back_to_first_app() {
    let (mut config, ids, _) = setup_config();
    config.settings.last_active_app_id = Some("gone".into());
    let mut c = Controller::new(config, false);
    c.startup(Instant::now());
    assert_eq!(c.active(), Some(ids[0].as_str()));
}

#[test]
fn startup_with_no_apps_does_nothing() {
    let mut c = Controller::new(Config::default(), false);
    let out = c.startup(Instant::now());
    assert!(out.effects.is_empty());
    assert_eq!(c.active(), None);
}

#[test]
fn activate_unknown_app_errors() {
    let (config, _, _) = setup_config();
    let mut c = Controller::new(config, false);
    assert_eq!(c.activate("nope", Instant::now()), Err(OpError::AppNotFound("nope".into())));
}

#[test]
fn activating_an_app_outside_the_folder_closes_the_panel() {
    let (config, ids, folder) = setup_config();
    let mut c = Controller::new(config, false);
    let now = Instant::now();
    c.startup(now);

    c.activate(&ids[2], now).unwrap();
    assert_eq!(c.snapshot().folder_panel, Some(folder));

    c.activate(&ids[1], now).unwrap();
    assert_eq!(c.snapshot().folder_panel, None);
}

#[test]
fn toggle_folder_panel() {
    let (config, _, folder) = setup_config();
    let mut c = Controller::new(config, false);

    let out = c.toggle_folder_panel(&folder).unwrap();
    assert!(out.changed && out.layout);
    assert!(c.view().folder_panel_open);

    c.toggle_folder_panel(&folder).unwrap();
    assert!(!c.view().folder_panel_open);

    assert_eq!(c.toggle_folder_panel("nope"), Err(OpError::FolderNotFound("nope".into())));
}

#[test]
fn background_apps_hibernate_after_their_timeout() {
    let (config, ids, _) = setup_config();
    let mut c = Controller::new(config, false);
    let now = Instant::now();
    c.startup(now);
    c.activate(&ids[1], now).unwrap();

    assert!(c.tick(now + TEN_MIN - Duration::from_secs(1)).effects.is_empty());
    let out = c.tick(now + TEN_MIN);

    assert_eq!(out.effects.len(), 2);
    assert!(out.effects.contains(&Effect::Destroy(ids[0].clone())));
    assert!(out.effects.contains(&Effect::Destroy(ids[2].clone())));
    assert!(out.changed);
    assert_eq!(c.app_state(&ids[1]), AppState::Active);
}

#[test]
fn hibernate_and_wake() {
    let (config, ids, _) = setup_config();
    let mut c = Controller::new(config, false);
    let now = Instant::now();
    c.startup(now);

    let out = c.hibernate(&ids[1]).unwrap();
    assert_eq!(out.effects, vec![Effect::Destroy(ids[1].clone())]);

    let out = c.wake(&ids[1], now).unwrap();
    assert_eq!(out.effects, vec![create(&ids[1], false)]);
    assert_eq!(c.app_state(&ids[1]), AppState::Running);

    assert_eq!(c.hibernate("nope"), Err(OpError::AppNotFound("nope".into())));
}

#[test]
fn webview_failure_is_visible_in_snapshot() {
    let (config, ids, _) = setup_config();
    let mut c = Controller::new(config, false);
    c.startup(Instant::now());

    let out = c.webview_failed(&ids[0], "boom");

    assert!(out.changed);
    assert_eq!(c.snapshot().states[&ids[0]], AppState::Error("boom".into()));
}

#[test]
fn overlay_and_toast_flags_only_affect_layout() {
    let mut c = Controller::new(Config::default(), false);

    let out = c.set_overlay_open(true);
    assert!(out.layout && !out.changed);
    let out = c.set_toast_visible(true);
    assert!(out.layout && !out.changed);

    assert_eq!(c.view(), View { folder_panel_open: false, overlay_open: true, toast_visible: true });
}

#[test]
fn snapshot_serializes_for_the_shell() {
    let (config, ids, _) = setup_config();
    let mut c = Controller::new(config, true);
    c.startup(Instant::now());

    let json = serde_json::to_value(c.snapshot()).unwrap();

    assert_eq!(json["activeAppId"], ids[0].as_str());
    assert_eq!(json["readOnly"], true);
    assert_eq!(json["states"][&ids[0]], serde_json::json!({ "kind": "active" }));
    assert_eq!(json["states"][&ids[1]], serde_json::json!({ "kind": "running" }));
    assert_eq!(json["config"]["apps"][0]["name"], "A");
    assert_eq!(
        serde_json::to_value(AppState::Error("x".into())).unwrap(),
        serde_json::json!({ "kind": "error", "message": "x" })
    );
    assert_eq!(serde_json::to_value(Badge::Count(3)).unwrap(), serde_json::json!(3));
    assert_eq!(serde_json::to_value(Badge::Dot).unwrap(), serde_json::json!("dot"));
}
