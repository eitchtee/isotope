#![allow(dead_code)]

use isotope_core::model::{Config, HibernationConfig, SidebarItem, DEFAULT_PROFILE_ID};
use isotope_core::ops::NewApp;

pub fn new_app(name: &str) -> NewApp {
    NewApp {
        name: name.into(),
        url: format!("https://{}.example.com", name.to_lowercase()),
        icon: None,
        profile_id: DEFAULT_PROFILE_ID.into(),
        user_agent: None,
        allowed_domains: vec![],
        hibernation: HibernationConfig { enabled: true, timeout_minutes: 10, start_hibernated: false },
        notifications: true,
        badges: true,
    }
}

/// Sidebar rendered as names: apps by name, folders as "[Name]".
pub fn sidebar_names(c: &Config) -> Vec<String> {
    c.sidebar
        .iter()
        .map(|item| match item {
            SidebarItem::App { id } => c.app(id).unwrap().name.clone(),
            SidebarItem::Folder { id } => format!("[{}]", c.folder(id).unwrap().name),
        })
        .collect()
}

pub fn folder_names(c: &Config, folder_id: &str) -> Vec<String> {
    c.folder(folder_id)
        .unwrap()
        .app_ids
        .iter()
        .map(|id| c.app(id).unwrap().name.clone())
        .collect()
}
