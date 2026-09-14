use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::model::{App, Config, Folder, HibernationConfig, Profile, SidebarItem, DEFAULT_PROFILE_ID};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OpError {
    #[error("app not found: {0}")]
    AppNotFound(String),
    #[error("folder not found: {0}")]
    FolderNotFound(String),
    #[error("profile not found: {0}")]
    ProfileNotFound(String),
    #[error("profile is used by at least one app: {0}")]
    ProfileInUse(String),
    #[error("the Default profile cannot be removed")]
    DefaultProfile,
    #[error("URL must be an http or https address: {0}")]
    InvalidUrl(String),
    #[error("folders cannot contain folders")]
    NestedFolder,
}

/// Editable fields of an app, as sent by the Add/Edit App modal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewApp {
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

impl NewApp {
    fn into_app(self, id: String) -> App {
        App {
            id,
            name: self.name,
            url: self.url,
            icon: self.icon,
            profile_id: self.profile_id,
            user_agent: self.user_agent.filter(|ua| !ua.trim().is_empty()),
            allowed_domains: self.allowed_domains,
            hibernation: self.hibernation,
            notifications: self.notifications,
            badges: self.badges,
        }
    }
}

/// Where an item is dropped: the top-level sidebar or inside a folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Container {
    Sidebar,
    Folder { id: String },
}

fn validate_url(url: &str) -> Result<(), OpError> {
    match Url::parse(url) {
        Ok(u) if u.scheme() == "http" || u.scheme() == "https" => Ok(()),
        _ => Err(OpError::InvalidUrl(url.into())),
    }
}

fn is_app_item(item: &SidebarItem, app_id: &str) -> bool {
    matches!(item, SidebarItem::App { id } if id == app_id)
}

fn is_folder_item(item: &SidebarItem, folder_id: &str) -> bool {
    matches!(item, SidebarItem::Folder { id } if id == folder_id)
}

impl Config {
    pub fn app(&self, id: &str) -> Option<&App> {
        self.apps.iter().find(|a| a.id == id)
    }

    pub fn folder(&self, id: &str) -> Option<&Folder> {
        self.folders.iter().find(|f| f.id == id)
    }

    pub(crate) fn has_profile(&self, id: &str) -> bool {
        self.profiles.iter().any(|p| p.id == id)
    }

    pub fn add_app(&mut self, input: NewApp) -> Result<String, OpError> {
        validate_url(&input.url)?;
        if !self.has_profile(&input.profile_id) {
            return Err(OpError::ProfileNotFound(input.profile_id));
        }
        let id = Uuid::new_v4().to_string();
        self.apps.push(input.into_app(id.clone()));
        self.sidebar.push(SidebarItem::App { id: id.clone() });
        Ok(id)
    }

    /// Replaces an app's editable fields. Returns `true` when the profile or
    /// user agent changed, meaning a running webview must be recreated.
    pub fn update_app(&mut self, id: &str, input: NewApp) -> Result<bool, OpError> {
        validate_url(&input.url)?;
        if !self.has_profile(&input.profile_id) {
            return Err(OpError::ProfileNotFound(input.profile_id));
        }
        let app = self
            .apps
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| OpError::AppNotFound(id.into()))?;
        let updated = input.into_app(id.to_string());
        let recreate = app.profile_id != updated.profile_id || app.user_agent != updated.user_agent;
        *app = updated;
        Ok(recreate)
    }

    pub fn remove_app(&mut self, id: &str) -> Result<(), OpError> {
        let pos = self
            .apps
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| OpError::AppNotFound(id.into()))?;
        self.apps.remove(pos);
        self.detach_app(id);
        if self.settings.last_active_app_id.as_deref() == Some(id) {
            self.settings.last_active_app_id = None;
        }
        Ok(())
    }

    /// Creates a folder containing `app_ids` (duplicates ignored). The folder takes
    /// the sidebar slot of the first app if that app is top-level; otherwise it is
    /// appended. Apps are moved out of wherever they were.
    pub fn add_folder(&mut self, name: &str, app_ids: &[String]) -> Result<String, OpError> {
        let mut unique: Vec<String> = Vec::new();
        for id in app_ids {
            if self.app(id).is_none() {
                return Err(OpError::AppNotFound(id.clone()));
            }
            if !unique.contains(id) {
                unique.push(id.clone());
            }
        }

        let id = Uuid::new_v4().to_string();
        let folder_item = SidebarItem::Folder { id: id.clone() };
        let slot = unique
            .first()
            .and_then(|first| self.sidebar.iter().position(|i| is_app_item(i, first)));
        match slot {
            Some(i) => self.sidebar[i] = folder_item,
            None => self.sidebar.push(folder_item),
        }
        for app_id in &unique {
            self.detach_app(app_id);
        }
        self.folders.push(Folder { id: id.clone(), name: name.into(), icon: None, app_ids: unique });
        Ok(id)
    }

    pub fn rename_folder(&mut self, id: &str, name: &str) -> Result<(), OpError> {
        let folder = self
            .folders
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| OpError::FolderNotFound(id.into()))?;
        folder.name = name.into();
        Ok(())
    }

    /// Deletes a folder; its apps return to the top-level sidebar at the folder's position.
    pub fn remove_folder(&mut self, id: &str) -> Result<(), OpError> {
        let pos = self
            .folders
            .iter()
            .position(|f| f.id == id)
            .ok_or_else(|| OpError::FolderNotFound(id.into()))?;
        let folder = self.folders.remove(pos);
        let slot = match self.sidebar.iter().position(|i| is_folder_item(i, id)) {
            Some(i) => {
                self.sidebar.remove(i);
                i
            }
            None => self.sidebar.len(),
        };
        for (offset, app_id) in folder.app_ids.into_iter().enumerate() {
            self.sidebar.insert(slot + offset, SidebarItem::App { id: app_id });
        }
        Ok(())
    }

    /// Moves an item to `index` within `target`. `index` is the position after the
    /// item is removed from its current place, clamped to the end of the target list.
    pub fn move_item(&mut self, item: &SidebarItem, target: &Container, index: usize) -> Result<(), OpError> {
        if let Container::Folder { id } = target
            && self.folder(id).is_none()
        {
            return Err(OpError::FolderNotFound(id.clone()));
        }
        match item {
            SidebarItem::App { id } => {
                if self.app(id).is_none() {
                    return Err(OpError::AppNotFound(id.clone()));
                }
                self.detach_app(id);
                match target {
                    Container::Sidebar => {
                        let at = index.min(self.sidebar.len());
                        self.sidebar.insert(at, item.clone());
                    }
                    Container::Folder { id: folder_id } => {
                        let folder = self
                            .folders
                            .iter_mut()
                            .find(|f| f.id == *folder_id)
                            .expect("folder existence checked above");
                        let at = index.min(folder.app_ids.len());
                        folder.app_ids.insert(at, id.clone());
                    }
                }
            }
            SidebarItem::Folder { id } => {
                if !matches!(target, Container::Sidebar) {
                    return Err(OpError::NestedFolder);
                }
                if self.folder(id).is_none() {
                    return Err(OpError::FolderNotFound(id.clone()));
                }
                self.sidebar.retain(|i| !is_folder_item(i, id));
                let at = index.min(self.sidebar.len());
                self.sidebar.insert(at, item.clone());
            }
        }
        Ok(())
    }

    pub fn add_profile(&mut self, name: &str) -> String {
        let id = Uuid::new_v4().to_string();
        self.profiles.push(Profile { id: id.clone(), name: name.into() });
        id
    }

    pub fn rename_profile(&mut self, id: &str, name: &str) -> Result<(), OpError> {
        let profile = self
            .profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| OpError::ProfileNotFound(id.into()))?;
        profile.name = name.into();
        Ok(())
    }

    pub fn remove_profile(&mut self, id: &str) -> Result<(), OpError> {
        if id == DEFAULT_PROFILE_ID {
            return Err(OpError::DefaultProfile);
        }
        let pos = self
            .profiles
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| OpError::ProfileNotFound(id.into()))?;
        if self.apps.iter().any(|a| a.profile_id == id) {
            return Err(OpError::ProfileInUse(id.into()));
        }
        self.profiles.remove(pos);
        Ok(())
    }

    /// Removes an app from wherever it is placed (top-level sidebar or a folder).
    pub(crate) fn detach_app(&mut self, id: &str) {
        self.sidebar.retain(|item| !is_app_item(item, id));
        for folder in &mut self.folders {
            folder.app_ids.retain(|x| x != id);
        }
    }

    /// Verifies the invariants from spec §3. Used by tests and by `persistence::load`.
    pub fn check_invariants(&self) -> Result<(), String> {
        if !self.has_profile(DEFAULT_PROFILE_ID) {
            return Err("default profile missing".into());
        }
        for app in &self.apps {
            let top = self.sidebar.iter().filter(|i| is_app_item(i, &app.id)).count();
            let nested: usize = self
                .folders
                .iter()
                .map(|f| f.app_ids.iter().filter(|x| **x == app.id).count())
                .sum();
            if top + nested != 1 {
                return Err(format!("app {} is placed {} times", app.id, top + nested));
            }
            if !self.has_profile(&app.profile_id) {
                return Err(format!("app {} references unknown profile {}", app.id, app.profile_id));
            }
        }
        for folder in &self.folders {
            let count = self.sidebar.iter().filter(|i| is_folder_item(i, &folder.id)).count();
            if count != 1 {
                return Err(format!("folder {} is placed {} times", folder.id, count));
            }
            if let Some(missing) = folder.app_ids.iter().find(|id| self.app(id).is_none()) {
                return Err(format!("folder {} references unknown app {}", folder.id, missing));
            }
        }
        for item in &self.sidebar {
            let known = match item {
                SidebarItem::App { id } => self.app(id).is_some(),
                SidebarItem::Folder { id } => self.folder(id).is_some(),
            };
            if !known {
                return Err(format!("sidebar references unknown item {item:?}"));
            }
        }
        Ok(())
    }
}
