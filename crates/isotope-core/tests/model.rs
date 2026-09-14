use isotope_core::model::*;

#[test]
fn default_config_has_only_default_profile() {
    let c = Config::default();
    assert_eq!(c.version, CURRENT_VERSION);
    assert_eq!(
        c.profiles,
        vec![Profile { id: DEFAULT_PROFILE_ID.into(), name: "Default".into() }]
    );
    assert!(c.apps.is_empty());
    assert!(c.folders.is_empty());
    assert!(c.sidebar.is_empty());
    assert_eq!(c.settings.default_hibernation_minutes, 10);
    assert_eq!(c.settings.last_active_app_id, None);
}

#[test]
fn serializes_camel_case_and_tagged_sidebar_items() {
    let mut c = Config::default();
    c.sidebar.push(SidebarItem::Folder { id: "f1".into() });
    c.sidebar.push(SidebarItem::App { id: "a1".into() });
    c.settings.last_active_app_id = Some("a1".into());

    let json = serde_json::to_value(&c).unwrap();

    assert_eq!(json["sidebar"][0], serde_json::json!({ "type": "folder", "id": "f1" }));
    assert_eq!(json["sidebar"][1], serde_json::json!({ "type": "app", "id": "a1" }));
    assert_eq!(json["settings"]["lastActiveAppId"], "a1");
    assert_eq!(json["settings"]["defaultHibernationMinutes"], 10);
}

#[test]
fn parses_and_round_trips_the_spec_example() {
    let text = r#"{
      "version": 1,
      "profiles": [ { "id": "default", "name": "Default" } ],
      "apps": [
        {
          "id": "a1",
          "name": "Gmail",
          "url": "https://mail.google.com",
          "icon": "icons/a1.png",
          "profileId": "default",
          "userAgent": null,
          "allowedDomains": ["login.microsoftonline.com"],
          "hibernation": { "enabled": true, "timeoutMinutes": 10, "startHibernated": false },
          "notifications": true,
          "badges": true
        }
      ],
      "folders": [ { "id": "f1", "name": "Work", "icon": null, "appIds": ["a1"] } ],
      "sidebar": [ { "type": "folder", "id": "f1" } ],
      "settings": { "defaultHibernationMinutes": 10, "lastActiveAppId": "a1" }
    }"#;

    let c: Config = serde_json::from_str(text).unwrap();
    assert_eq!(c.apps[0].profile_id, "default");
    assert_eq!(c.apps[0].allowed_domains, vec!["login.microsoftonline.com".to_string()]);
    assert_eq!(c.apps[0].hibernation.timeout_minutes, 10);
    assert_eq!(c.folders[0].app_ids, vec!["a1".to_string()]);

    let again: Config = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(c, again);
}
