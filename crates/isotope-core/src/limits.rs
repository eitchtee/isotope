use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

pub const NOTIFY_TITLE_MAX: usize = 200;
pub const NOTIFY_BODY_MAX: usize = 1000;
pub const PAGE_TITLE_MAX: usize = 500;
pub const NOTIFY_MAX_PER_WINDOW: usize = 5;
pub const NOTIFY_WINDOW: Duration = Duration::from_secs(10);

pub fn truncate_chars(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

/// Sliding-window limiter keyed by app id.
#[derive(Debug)]
pub struct RateLimiter {
    max: usize,
    window: Duration,
    hits: HashMap<String, VecDeque<Instant>>,
}

impl RateLimiter {
    pub fn new(max: usize, window: Duration) -> Self {
        Self { max, window, hits: HashMap::new() }
    }

    pub fn notifications() -> Self {
        Self::new(NOTIFY_MAX_PER_WINDOW, NOTIFY_WINDOW)
    }

    pub fn allow(&mut self, key: &str, now: Instant) -> bool {
        let hits = self.hits.entry(key.to_string()).or_default();
        while hits.front().is_some_and(|t| now.duration_since(*t) >= self.window) {
            hits.pop_front();
        }
        if hits.len() >= self.max {
            return false;
        }
        hits.push_back(now);
        true
    }
}
