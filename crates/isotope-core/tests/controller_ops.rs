mod common;

use std::time::{Duration, Instant};

use common::new_app;
use isotope_core::badge::Badge;
use isotope_core::controller::{Controller, Notification};
use isotope_core::lifecycle::Effect;
use isotope_core::model::{Config, SidebarItem};
use isotope_core::ops::{Container, OpError};

/// Started controller with top-level apps A and B; A is active. Returns ids [A, B].
fn started() -> (Controller, Vec<String>, Instant) {
    let mut config = Config::default();
    let ids: Vec<String> = ["A", "B"].iter().map(|n| config.add_app(new_app(n)).unwrap()).collect();
    let mut c = Controller::new(config, false);
    let now = Instant::now();
    c.startup(now);
    (c, ids, now)
}

#[test]
fn add_app_activates_the_new_app() {
    let (mut c, ids, now) = started();
    let (id, out) = c.add_app(new_app("D"), now).unwrap();

    assert_eq!(
        out.effects,
        vec![Effect::Create { id: id.clone(), visible: true }, Effect::Hide(ids[0].clone())]
    );
    assert!(out.save && out.changed && out.layout);
    assert_eq!(c.active(), Some(id.as_str()));
}

#[test]
fn add_app_with_invalid_url_errors() {
    let (mut c, _, now) = started();
    let mut input = new_app("Bad");
    input.url = "nope".into();
    assert_eq!(c.add_app(input, now), Err(OpError::InvalidUrl("nope".into())));
}

#[test]
fn update_app_recreates_live_webview_when_profile_changes() {
    let (mut c, ids, _) = started();
    let (work, _) = c.add_profile("Work");
    let mut input = new_app("A");
    input.profile_id = work;

    let out = c.update_app(&ids[0], input).unwrap();

    assert_eq!(
        out.effects,
        vec![Effect::Destroy(ids[0].clone()), Effect::Create { id: ids[0].clone(), visible: true }]
    );
    assert!(out.save && out.layout);
}

#[test]
fn update_app_rename_has_no_effects() {
    let (mut c, ids, _) = started();
    let out = c.update_app(&ids[1], new_app("Renamed")).unwrap();
    assert!(out.effects.is_empty());
    assert!(out.save && out.changed);
}

#[test]
fn disabling_badges_clears_the_badge() {
    let (mut c, ids, _) = started();
    c.page_title(&ids[0], "(2) Inbox");
    let mut input = new_app("A");
    input.badges = false;

    c.update_app(&ids[0], input).unwrap();

    assert!(c.snapshot().badges.is_empty());
    assert!(!c.page_title(&ids[0], "(5) Inbox").changed);
}

#[test]
fn remove_app_destroys_its_webview() {
    let (mut c, ids, _) = started();
    let out = c.remove_app(&ids[0]).unwrap();
    assert_eq!(out.effects, vec![Effect::Destroy(ids[0].clone())]);
    assert!(out.save);
    assert_eq!(c.active(), None);
}

#[test]
fn folder_operations_save_and_close_a_deleted_panel() {
    let (mut c, ids, _) = started();
    let (folder, out) = c.add_folder("Work", &[ids[1].clone()]).unwrap();
    assert!(out.save && out.changed);

    c.toggle_folder_panel(&folder).unwrap();
    assert!(c.rename_folder(&folder, "Office").unwrap().save);
    assert!(c.view().folder_panel_open);

    let out = c.remove_folder(&folder).unwrap();
    assert!(out.save && out.layout);
    assert!(!c.view().folder_panel_open);
}

#[test]
fn move_item_saves() {
    let (mut c, ids, _) = started();
    let out = c.move_item(&SidebarItem::App { id: ids[1].clone() }, &Container::Sidebar, 0).unwrap();
    assert!(out.save && out.changed);
    assert_eq!(c.config().sidebar[0], SidebarItem::App { id: ids[1].clone() });
}

#[test]
fn remove_profile_requests_data_deletion() {
    let (mut c, ids, _) = started();
    let (work, _) = c.add_profile("Work");
    assert!(c.rename_profile(&work, "Office").unwrap().save);

    let mut input = new_app("A");
    input.profile_id = work.clone();
    c.update_app(&ids[0], input).unwrap();
    assert_eq!(c.remove_profile(&work), Err(OpError::ProfileInUse(work.clone())));

    c.update_app(&ids[0], new_app("A")).unwrap();
    let out = c.remove_profile(&work).unwrap();
    assert_eq!(out.delete_profile_data, Some(work));
    assert!(out.save);
}

#[test]
fn default_hibernation_minutes_has_a_floor_of_one() {
    let (mut c, _, _) = started();
    assert!(c.set_default_hibernation_minutes(0).save);
    assert_eq!(c.config().settings.default_hibernation_minutes, 1);
    c.set_default_hibernation_minutes(30);
    assert_eq!(c.config().settings.default_hibernation_minutes, 30);
}

#[test]
fn page_title_sets_updates_and_clears_badges() {
    let (mut c, ids, _) = started();

    assert!(c.page_title(&ids[0], "(3) Inbox").changed);
    assert_eq!(c.snapshot().badges[&ids[0]], Badge::Count(3));
    assert!(!c.page_title(&ids[0], "(3) Inbox - reloaded").changed);
    assert!(c.page_title(&ids[0], "• Chat").changed);
    assert_eq!(c.snapshot().badges[&ids[0]], Badge::Dot);
    assert!(c.page_title(&ids[0], "Inbox").changed);
    assert!(c.snapshot().badges.is_empty());
    assert!(!c.page_title("unknown", "(1) x").changed);
}

#[test]
fn hibernating_clears_badges() {
    let (mut c, ids, _) = started();
    c.page_title(&ids[1], "(1) x");
    c.hibernate(&ids[1]).unwrap();
    assert!(c.snapshot().badges.is_empty());
}

#[test]
fn notify_formats_caps_and_rate_limits() {
    let (mut c, ids, now) = started();

    let out = c.notify(&ids[0], "New mail", &"b".repeat(2000), now);
    let n: Notification = out.notification.unwrap();
    assert_eq!(n.app_id, ids[0]);
    assert_eq!(n.title, "A: New mail");
    assert_eq!(n.body.chars().count(), 1000);

    for _ in 0..4 {
        assert!(c.notify(&ids[0], "x", "y", now).notification.is_some());
    }
    assert!(c.notify(&ids[0], "x", "y", now).notification.is_none());
    assert!(c.notify(&ids[0], "x", "y", now + Duration::from_secs(10)).notification.is_some());
}

#[test]
fn notify_respects_the_app_setting() {
    let (mut c, ids, now) = started();
    let mut input = new_app("B");
    input.notifications = false;
    c.update_app(&ids[1], input).unwrap();
    assert!(c.notify(&ids[1], "x", "y", now).notification.is_none());
    assert!(c.notify("unknown", "x", "y", now).notification.is_none());
}

#[test]
fn set_icon_saves() {
    let (mut c, ids, _) = started();
    let out = c.set_icon(&ids[0], Some("icons/a.png#1".into())).unwrap();
    assert!(out.save && out.changed);
    assert_eq!(c.config().app(&ids[0]).unwrap().icon.as_deref(), Some("icons/a.png#1"));
    assert_eq!(c.set_icon("nope", None), Err(OpError::AppNotFound("nope".into())));
}
