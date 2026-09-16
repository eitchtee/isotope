use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::badge::{parse_badge, Badge};
use crate::lifecycle::{AppState, Effect, Lifecycle, Policy};
use crate::limits::{truncate_chars, RateLimiter, NOTIFY_BODY_MAX, NOTIFY_TITLE_MAX, PAGE_TITLE_MAX};
use crate::model::{Config, SidebarItem, WindowTitleMode};

pub const APP_TITLE: &str = "Isotope";

/// Engines report the page URL as its title until the page sets one.
pub fn is_placeholder_title(title: &str, url: &str) -> bool {
    let title = title.trim().trim_end_matches('/');
    if title.is_empty() {
        return true;
    }
    let without_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    title == url.trim_end_matches('/') || title == without_scheme.trim_end_matches('/')
}
use crate::ops::{Container, NewApp, OpError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "level", content = "message", rename_all = "camelCase")]
pub enum Toast {
    Info(String),
    Warning(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub app_id: String,
    pub title: String,
    pub body: String,
}

/// What the Tauri layer must do after a controller call.
#[derive(Debug, Default, PartialEq)]
pub struct Outcome {
    /// Webview operations, applied in order.
    pub effects: Vec<Effect>,
    /// Emit a fresh `Snapshot` to the shell.
    pub changed: bool,
    /// Persist the config (the Tauri layer skips this when read-only).
    pub save: bool,
    /// Recompute webview bounds and visibility from `Controller::view`.
    pub layout: bool,
    pub toasts: Vec<Toast>,
    pub notification: Option<Notification>,
    /// Profile id whose on-disk data must be deleted.
    pub delete_profile_data: Option<String>,
}

impl Outcome {
    fn merge(&mut self, other: Outcome) {
        self.effects.extend(other.effects);
        self.changed |= other.changed;
        self.save |= other.save;
        self.layout |= other.layout;
        self.toasts.extend(other.toasts);
        self.notification = other.notification.or(self.notification.take());
        self.delete_profile_data = other.delete_profile_data.or(self.delete_profile_data.take());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct View {
    pub folder_panel_open: bool,
    pub overlay_open: bool,
    pub toast_visible: bool,
}

/// Everything the shell renders, sent with the `state-changed` event.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub config: Config,
    pub states: HashMap<String, AppState>,
    pub badges: HashMap<String, Badge>,
    pub active_app_id: Option<String>,
    pub folder_panel: Option<String>,
    pub read_only: bool,
}

#[derive(Debug)]
pub struct Controller {
    config: Config,
    read_only: bool,
    lifecycle: Lifecycle,
    badges: HashMap<String, Badge>,
    folder_panel: Option<String>,
    overlay_open: bool,
    toast_visible: bool,
    limiter: RateLimiter,
    /// Latest real page title per live app (placeholders are stored as empty).
    titles: HashMap<String, String>,
}

fn policy(config: &Config, id: &str) -> Policy {
    config
        .app(id)
        .map(|app| Policy::from(&app.hibernation))
        .unwrap_or(Policy { enabled: false, timeout: Duration::ZERO })
}

/// App ids in sidebar order, with folder contents expanded in place.
pub fn ordered_app_ids(config: &Config) -> Vec<String> {
    config
        .sidebar
        .iter()
        .flat_map(|item| match item {
            SidebarItem::App { id } => vec![id.clone()],
            SidebarItem::Folder { id } => config.folder(id).map(|f| f.app_ids.clone()).unwrap_or_default(),
        })
        .collect()
}

fn folder_of(config: &Config, app_id: &str) -> Option<String> {
    config
        .folders
        .iter()
        .find(|f| f.app_ids.iter().any(|a| a == app_id))
        .map(|f| f.id.clone())
}

impl Controller {
    pub fn new(config: Config, read_only: bool) -> Self {
        Self {
            config,
            read_only,
            lifecycle: Lifecycle::new(),
            badges: HashMap::new(),
            folder_panel: None,
            overlay_open: false,
            toast_visible: false,
            limiter: RateLimiter::notifications(),
            titles: HashMap::new(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn read_only(&self) -> bool {
        self.read_only
    }

    pub fn active(&self) -> Option<&str> {
        self.lifecycle.active()
    }

    pub fn app_state(&self, id: &str) -> AppState {
        self.lifecycle.state(id)
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        self.lifecycle.next_deadline()
    }

    pub fn view(&self) -> View {
        View {
            folder_panel_open: self.folder_panel.is_some(),
            overlay_open: self.overlay_open,
            toast_visible: self.toast_visible,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            config: self.config.clone(),
            states: self.config.apps.iter().map(|a| (a.id.clone(), self.lifecycle.state(&a.id))).collect(),
            badges: self.badges.clone(),
            active_app_id: self.active().map(str::to_owned),
            folder_panel: self.folder_panel.clone(),
            read_only: self.read_only,
        }
    }

    /// Registers every app (spec §4.3 startup) and activates the last active app,
    /// falling back to the first app in sidebar order.
    pub fn startup(&mut self, now: Instant) -> Outcome {
        let ids = ordered_app_ids(&self.config);
        let mut out = Outcome { changed: true, layout: true, ..Outcome::default() };
        for id in &ids {
            let hibernation = self.config.app(id).expect("ids come from config").hibernation;
            out.effects.extend(self.lifecycle.register(
                id,
                hibernation.start_hibernated,
                Policy::from(&hibernation),
                now,
            ));
        }
        let last = self.config.settings.last_active_app_id.clone().filter(|id| self.config.app(id).is_some());
        if let Some(target) = last.or_else(|| ids.first().cloned())
            && let Ok(activated) = self.activate(&target, now)
        {
            out.merge(activated);
        }
        out
    }

    pub fn activate(&mut self, id: &str, now: Instant) -> Result<Outcome, OpError> {
        if self.config.app(id).is_none() {
            return Err(OpError::AppNotFound(id.into()));
        }
        let config = &self.config;
        let effects = self.lifecycle.activate(id, |other| policy(config, other), now);
        self.folder_panel = folder_of(&self.config, id);
        let mut out = Outcome { effects, changed: true, layout: true, ..Outcome::default() };
        if self.config.settings.last_active_app_id.as_deref() != Some(id) {
            self.config.settings.last_active_app_id = Some(id.into());
            out.save = true;
        }
        Ok(out)
    }

    pub fn toggle_folder_panel(&mut self, folder_id: &str) -> Result<Outcome, OpError> {
        if self.config.folder(folder_id).is_none() {
            return Err(OpError::FolderNotFound(folder_id.into()));
        }
        self.folder_panel = if self.folder_panel.as_deref() == Some(folder_id) {
            None
        } else {
            Some(folder_id.into())
        };
        Ok(Outcome { changed: true, layout: true, ..Outcome::default() })
    }

    /// Folder icon click. If this folder's panel is open, close it. Otherwise open the panel and
    /// activate an app from the folder: the active app if it's already inside, else the first
    /// app that isn't hibernated, else the first app. An empty folder just opens its panel.
    pub fn open_folder(&mut self, folder_id: &str, now: Instant) -> Result<Outcome, OpError> {
        let folder = self
            .config
            .folder(folder_id)
            .ok_or_else(|| OpError::FolderNotFound(folder_id.into()))?;
        if self.folder_panel.as_deref() == Some(folder_id) {
            self.folder_panel = None;
            return Ok(Outcome { changed: true, layout: true, ..Outcome::default() });
        }

        let active_inside = self.active().filter(|active| folder.app_ids.iter().any(|id| id == active));
        let first_live = folder
            .app_ids
            .iter()
            .find(|id| matches!(self.lifecycle.state(id), AppState::Running | AppState::Active))
            .map(String::as_str);
        let target = active_inside.or(first_live).or(folder.app_ids.first().map(String::as_str)).map(str::to_owned);

        let mut out = match target {
            Some(id) => self.activate(&id, now)?,
            None => Outcome { changed: true, ..Outcome::default() },
        };
        self.folder_panel = Some(folder_id.into());
        out.layout = true;
        Ok(out)
    }

    pub fn hibernate(&mut self, id: &str) -> Result<Outcome, OpError> {
        if self.config.app(id).is_none() {
            return Err(OpError::AppNotFound(id.into()));
        }
        let effects = self.lifecycle.hibernate(id);
        self.badges.remove(id);
        self.titles.remove(id);
        Ok(Outcome { effects, changed: true, layout: true, ..Outcome::default() })
    }

    pub fn wake(&mut self, id: &str, now: Instant) -> Result<Outcome, OpError> {
        if self.config.app(id).is_none() {
            return Err(OpError::AppNotFound(id.into()));
        }
        let effects = self.lifecycle.wake(id, policy(&self.config, id), now);
        Ok(Outcome { effects, changed: true, ..Outcome::default() })
    }

    pub fn tick(&mut self, now: Instant) -> Outcome {
        let effects = self.lifecycle.tick(now);
        for effect in &effects {
            if let Effect::Destroy(id) = effect {
                self.badges.remove(id);
                self.titles.remove(id);
            }
        }
        Outcome { changed: !effects.is_empty(), effects, ..Outcome::default() }
    }

    pub fn webview_failed(&mut self, id: &str, message: &str) -> Outcome {
        self.lifecycle.failed(id, message);
        Outcome { changed: true, ..Outcome::default() }
    }

    pub fn set_overlay_open(&mut self, open: bool) -> Outcome {
        self.overlay_open = open;
        Outcome { layout: true, ..Outcome::default() }
    }

    pub fn set_toast_visible(&mut self, visible: bool) -> Outcome {
        self.toast_visible = visible;
        Outcome { layout: true, ..Outcome::default() }
    }

    pub fn add_app(&mut self, input: NewApp, now: Instant) -> Result<(String, Outcome), OpError> {
        let id = self.config.add_app(input)?;
        let mut out = self.activate(&id, now)?;
        out.save = true;
        Ok((id, out))
    }

    pub fn update_app(&mut self, id: &str, input: NewApp) -> Result<Outcome, OpError> {
        let recreate = self.config.update_app(id, input)?;
        let effects = if recreate { self.lifecycle.recreate(id) } else { Vec::new() };
        if recreate {
            self.titles.remove(id);
        }
        if self.config.app(id).is_some_and(|app| !app.badges) {
            self.badges.remove(id);
        }
        Ok(Outcome { effects, changed: true, save: true, layout: recreate, ..Outcome::default() })
    }

    pub fn remove_app(&mut self, id: &str) -> Result<Outcome, OpError> {
        self.config.remove_app(id)?;
        let effects = self.lifecycle.remove(id);
        self.badges.remove(id);
        self.titles.remove(id);
        Ok(Outcome { effects, ..self.structure_changed() })
    }

    pub fn add_folder(&mut self, name: &str, app_ids: &[String]) -> Result<(String, Outcome), OpError> {
        let id = self.config.add_folder(name, app_ids)?;
        Ok((id, self.structure_changed()))
    }

    pub fn rename_folder(&mut self, id: &str, name: &str) -> Result<Outcome, OpError> {
        self.config.rename_folder(id, name)?;
        Ok(self.structure_changed())
    }

    pub fn remove_folder(&mut self, id: &str) -> Result<Outcome, OpError> {
        self.config.remove_folder(id)?;
        Ok(self.structure_changed())
    }

    pub fn move_item(&mut self, item: &SidebarItem, target: &Container, index: usize) -> Result<Outcome, OpError> {
        self.config.move_item(item, target, index)?;
        Ok(self.structure_changed())
    }

    pub fn add_profile(&mut self, name: &str) -> (String, Outcome) {
        let id = self.config.add_profile(name);
        (id, Outcome { changed: true, save: true, ..Outcome::default() })
    }

    pub fn rename_profile(&mut self, id: &str, name: &str) -> Result<Outcome, OpError> {
        self.config.rename_profile(id, name)?;
        Ok(Outcome { changed: true, save: true, ..Outcome::default() })
    }

    pub fn remove_profile(&mut self, id: &str) -> Result<Outcome, OpError> {
        self.config.remove_profile(id)?;
        Ok(Outcome { changed: true, save: true, delete_profile_data: Some(id.into()), ..Outcome::default() })
    }

    pub fn set_default_hibernation_minutes(&mut self, minutes: u32) -> Outcome {
        self.config.settings.default_hibernation_minutes = minutes.max(1);
        Outcome { changed: true, save: true, ..Outcome::default() }
    }

    pub fn set_icon(&mut self, id: &str, icon: Option<String>) -> Result<Outcome, OpError> {
        let app = self
            .config
            .apps
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| OpError::AppNotFound(id.into()))?;
        app.icon = icon;
        Ok(Outcome { changed: true, save: true, ..Outcome::default() })
    }

    /// Updates the app's unread badge from its page title (spec §4.5).
    pub fn page_title(&mut self, id: &str, title: &str) -> Outcome {
        let Some(app) = self.config.app(id) else {
            return Outcome::default();
        };
        let title = truncate_chars(title, PAGE_TITLE_MAX);
        let badge = if app.badges { parse_badge(&title) } else { None };
        self.titles.insert(id.into(), title);
        let previous = match badge {
            Some(b) => self.badges.insert(id.into(), b),
            None => self.badges.remove(id),
        };
        Outcome { changed: previous != badge, ..Outcome::default() }
    }

    /// A navigation started: forget the old title so the window falls back to the app name.
    pub fn page_loading(&mut self, id: &str) {
        self.titles.remove(id);
    }

    pub fn set_window_title_mode(&mut self, mode: WindowTitleMode) -> Outcome {
        self.config.settings.window_title = mode;
        Outcome { changed: true, save: true, ..Outcome::default() }
    }

    /// Title for the main window, per the window title setting and the active app.
    pub fn window_title(&self) -> String {
        let Some(app) = self.active().and_then(|id| self.config.app(id)) else {
            return APP_TITLE.into();
        };
        let label = match self.config.settings.window_title {
            WindowTitleMode::Fixed => return APP_TITLE.into(),
            WindowTitleMode::AppName => app.name.clone(),
            WindowTitleMode::PageTitle => self
                .titles
                .get(&app.id)
                .map(|t| t.trim())
                .filter(|t| !t.is_empty())
                .map_or_else(|| app.name.clone(), str::to_owned),
        };
        format!("{label} - {APP_TITLE}")
    }

    /// Turns a web Notification into an OS notification request (spec §4.6).
    pub fn notify(&mut self, id: &str, title: &str, body: &str, now: Instant) -> Outcome {
        let Some(app) = self.config.app(id) else {
            return Outcome::default();
        };
        if !app.notifications || !self.limiter.allow(id, now) {
            return Outcome::default();
        }
        let notification = Notification {
            app_id: id.into(),
            title: format!("{}: {}", app.name, truncate_chars(title, NOTIFY_TITLE_MAX)),
            body: truncate_chars(body, NOTIFY_BODY_MAX),
        };
        Outcome { notification: Some(notification), ..Outcome::default() }
    }

    /// Sidebar structure changed: close the folder panel if its folder is gone.
    fn structure_changed(&mut self) -> Outcome {
        if let Some(folder) = &self.folder_panel
            && self.config.folder(folder).is_none()
        {
            self.folder_panel = None;
        }
        Outcome { changed: true, save: true, layout: true, ..Outcome::default() }
    }
}
