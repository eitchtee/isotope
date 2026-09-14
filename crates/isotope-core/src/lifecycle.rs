use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::model::HibernationConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    pub enabled: bool,
    pub timeout: Duration,
}

impl From<&HibernationConfig> for Policy {
    fn from(config: &HibernationConfig) -> Self {
        Self {
            enabled: config.enabled,
            timeout: Duration::from_secs(u64::from(config.timeout_minutes) * 60),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum AppState {
    Hibernated,
    /// Webview exists but is hidden.
    Running,
    /// Webview exists and is the visible app.
    Active,
    Error(String),
}

/// A webview operation the Tauri layer must perform, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Create { id: String, visible: bool },
    Show(String),
    Hide(String),
    Destroy(String),
}

/// Decides when app webviews are created, shown, hidden and destroyed (spec §2.2, §4.3).
/// Pure: time is always passed in.
#[derive(Debug, Default)]
pub struct Lifecycle {
    states: HashMap<String, AppState>,
    deadlines: HashMap<String, Instant>,
    active: Option<String>,
}

impl Lifecycle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self, id: &str) -> AppState {
        self.states.get(id).cloned().unwrap_or(AppState::Hibernated)
    }

    pub fn active(&self) -> Option<&str> {
        self.active.as_deref()
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        self.deadlines.values().min().copied()
    }

    /// Startup registration of an app.
    pub fn register(&mut self, id: &str, start_hibernated: bool, policy: Policy, now: Instant) -> Vec<Effect> {
        if start_hibernated {
            self.states.insert(id.into(), AppState::Hibernated);
            return vec![];
        }
        self.states.insert(id.into(), AppState::Running);
        self.schedule(id, policy, now);
        vec![Effect::Create { id: id.into(), visible: false }]
    }

    pub fn activate(&mut self, id: &str, policy_of: impl Fn(&str) -> Policy, now: Instant) -> Vec<Effect> {
        if self.active.as_deref() == Some(id) {
            return vec![];
        }
        let mut effects = vec![match self.state(id) {
            AppState::Running | AppState::Active => Effect::Show(id.into()),
            AppState::Hibernated | AppState::Error(_) => Effect::Create { id: id.into(), visible: true },
        }];
        self.states.insert(id.into(), AppState::Active);
        self.deadlines.remove(id);

        if let Some(previous) = self.active.replace(id.to_string())
            && self.state(&previous) == AppState::Active
        {
            self.states.insert(previous.clone(), AppState::Running);
            self.schedule(&previous, policy_of(&previous), now);
            effects.push(Effect::Hide(previous));
        }
        effects
    }

    /// Starts a hibernated app in the background without switching to it.
    pub fn wake(&mut self, id: &str, policy: Policy, now: Instant) -> Vec<Effect> {
        match self.state(id) {
            AppState::Hibernated | AppState::Error(_) => {
                self.states.insert(id.into(), AppState::Running);
                self.schedule(id, policy, now);
                vec![Effect::Create { id: id.into(), visible: false }]
            }
            AppState::Running | AppState::Active => vec![],
        }
    }

    pub fn hibernate(&mut self, id: &str) -> Vec<Effect> {
        self.deadlines.remove(id);
        if self.active.as_deref() == Some(id) {
            self.active = None;
        }
        match self.state(id) {
            AppState::Running | AppState::Active => {
                self.states.insert(id.into(), AppState::Hibernated);
                vec![Effect::Destroy(id.into())]
            }
            AppState::Hibernated | AppState::Error(_) => vec![],
        }
    }

    /// Records that applying `Create` for this app failed.
    pub fn failed(&mut self, id: &str, message: impl Into<String>) {
        self.deadlines.remove(id);
        self.states.insert(id.into(), AppState::Error(message.into()));
    }

    /// Hibernates every background app whose deadline is at or before `now`.
    pub fn tick(&mut self, now: Instant) -> Vec<Effect> {
        let mut due: Vec<String> = self
            .deadlines
            .iter()
            .filter(|(_, deadline)| **deadline <= now)
            .map(|(id, _)| id.clone())
            .collect();
        due.sort();

        let mut effects = Vec::new();
        for id in due {
            self.deadlines.remove(&id);
            if self.state(&id) == AppState::Running {
                self.states.insert(id.clone(), AppState::Hibernated);
                effects.push(Effect::Destroy(id));
            }
        }
        effects
    }

    /// Rebuilds a live webview after its profile or user agent changed.
    pub fn recreate(&mut self, id: &str) -> Vec<Effect> {
        match self.state(id) {
            AppState::Running => vec![Effect::Destroy(id.into()), Effect::Create { id: id.into(), visible: false }],
            AppState::Active => vec![Effect::Destroy(id.into()), Effect::Create { id: id.into(), visible: true }],
            AppState::Hibernated | AppState::Error(_) => vec![],
        }
    }

    pub fn remove(&mut self, id: &str) -> Vec<Effect> {
        let effects = match self.state(id) {
            AppState::Running | AppState::Active => vec![Effect::Destroy(id.into())],
            AppState::Hibernated | AppState::Error(_) => vec![],
        };
        self.states.remove(id);
        self.deadlines.remove(id);
        if self.active.as_deref() == Some(id) {
            self.active = None;
        }
        effects
    }

    fn schedule(&mut self, id: &str, policy: Policy, now: Instant) {
        if policy.enabled {
            self.deadlines.insert(id.into(), now + policy.timeout);
        } else {
            self.deadlines.remove(id);
        }
    }
}
