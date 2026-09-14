use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::badge::Badge;
use crate::lifecycle::{AppState, Effect, Lifecycle, Policy};
use crate::model::{Config, SidebarItem};
use crate::ops::OpError;

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

    pub fn hibernate(&mut self, id: &str) -> Result<Outcome, OpError> {
        if self.config.app(id).is_none() {
            return Err(OpError::AppNotFound(id.into()));
        }
        let effects = self.lifecycle.hibernate(id);
        self.badges.remove(id);
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
}
