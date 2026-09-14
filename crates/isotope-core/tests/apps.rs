mod common;

use common::{new_app, sidebar_names};
use isotope_core::model::{Config, Folder, Profile, SidebarItem};
use isotope_core::ops::OpError;

#[test]
fn add_app_appends_to_sidebar() {
    let mut c = Config::default();
    let a = c.add_app(new_app("Gmail")).unwrap();
    c.add_app(new_app("Slack")).unwrap();

    assert_eq!(sidebar_names(&c), vec!["Gmail", "Slack"]);
    assert_eq!(c.app(&a).unwrap().url, "https://gmail.example.com");
    c.check_invariants().unwrap();
}

#[test]
fn add_app_rejects_non_http_urls() {
    let mut c = Config::default();
    for url in ["ftp://example.com", "not a url", "file:///C:/x"] {
        let mut input = new_app("Bad");
        input.url = url.into();
        assert_eq!(c.add_app(input), Err(OpError::InvalidUrl(url.into())));
    }
    assert!(c.apps.is_empty());
}

#[test]
fn add_app_rejects_unknown_profile() {
    let mut c = Config::default();
    let mut input = new_app("Gmail");
    input.profile_id = "nope".into();
    assert_eq!(c.add_app(input), Err(OpError::ProfileNotFound("nope".into())));
}

#[test]
fn blank_user_agent_is_stored_as_none() {
    let mut c = Config::default();
    let mut input = new_app("Gmail");
    input.user_agent = Some("   ".into());
    let id = c.add_app(input).unwrap();
    assert_eq!(c.app(&id).unwrap().user_agent, None);
}

#[test]
fn update_app_requests_recreate_only_for_profile_or_user_agent_changes() {
    let mut c = Config::default();
    c.profiles.push(Profile { id: "work".into(), name: "Work".into() });
    let id = c.add_app(new_app("Gmail")).unwrap();

    let mut renamed = new_app("Mail");
    assert_eq!(c.update_app(&id, renamed.clone()), Ok(false));
    assert_eq!(c.app(&id).unwrap().name, "Mail");

    renamed.profile_id = "work".into();
    assert_eq!(c.update_app(&id, renamed.clone()), Ok(true));

    renamed.user_agent = Some("Custom UA".into());
    assert_eq!(c.update_app(&id, renamed.clone()), Ok(true));

    assert_eq!(c.update_app(&id, renamed), Ok(false));
    c.check_invariants().unwrap();
}

#[test]
fn update_unknown_app_errors() {
    let mut c = Config::default();
    assert_eq!(c.update_app("x", new_app("X")), Err(OpError::AppNotFound("x".into())));
}

#[test]
fn remove_app_clears_sidebar_folders_and_last_active() {
    let mut c = Config::default();
    let top = c.add_app(new_app("Top")).unwrap();
    let nested = c.add_app(new_app("Nested")).unwrap();
    // Put `nested` in a folder by hand (folder ops arrive in Task 3).
    c.sidebar.retain(|i| *i != SidebarItem::App { id: nested.clone() });
    c.folders.push(Folder { id: "f".into(), name: "F".into(), icon: None, app_ids: vec![nested.clone()] });
    c.sidebar.push(SidebarItem::Folder { id: "f".into() });
    c.settings.last_active_app_id = Some(nested.clone());
    c.check_invariants().unwrap();

    c.remove_app(&nested).unwrap();
    c.remove_app(&top).unwrap();

    assert!(c.apps.is_empty());
    assert_eq!(c.sidebar, vec![SidebarItem::Folder { id: "f".into() }]);
    assert!(c.folders[0].app_ids.is_empty());
    assert_eq!(c.settings.last_active_app_id, None);
    c.check_invariants().unwrap();
}

#[test]
fn remove_unknown_app_errors() {
    let mut c = Config::default();
    assert_eq!(c.remove_app("x"), Err(OpError::AppNotFound("x".into())));
}

#[test]
fn check_invariants_detects_duplicate_placement() {
    let mut c = Config::default();
    let id = c.add_app(new_app("Gmail")).unwrap();
    c.folders.push(Folder { id: "f".into(), name: "F".into(), icon: None, app_ids: vec![id] });
    c.sidebar.push(SidebarItem::Folder { id: "f".into() });
    assert!(c.check_invariants().is_err());
}
