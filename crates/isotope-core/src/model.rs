use serde::{Deserialize, Serialize};

pub const CURRENT_VERSION: u32 = 1;
pub const DEFAULT_PROFILE_ID: &str = "default";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub version: u32,
    pub profiles: Vec<Profile>,
    pub apps: Vec<App>,
    pub folders: Vec<Folder>,
    pub sidebar: Vec<SidebarItem>,
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            profiles: vec![Profile { id: DEFAULT_PROFILE_ID.into(), name: "Default".into() }],
            apps: Vec::new(),
            folders: Vec::new(),
            sidebar: Vec::new(),
            settings: Settings::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub id: String,
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
    pub profile_id: String,
    pub user_agent: Option<String>,
    pub allowed_domains: Vec<String>,
    pub hibernation: HibernationConfig,
    pub notifications: bool,
    pub badges: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HibernationConfig {
    pub enabled: bool,
    pub timeout_minutes: u32,
    pub start_hibernated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub app_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SidebarItem {
    App { id: String },
    Folder { id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub default_hibernation_minutes: u32,
    pub last_active_app_id: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self { default_hibernation_minutes: 10, last_active_app_id: None }
    }
}
