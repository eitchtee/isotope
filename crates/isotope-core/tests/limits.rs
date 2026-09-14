use std::time::{Duration, Instant};

use isotope_core::limits::{truncate_chars, RateLimiter, NOTIFY_BODY_MAX, NOTIFY_TITLE_MAX};

#[test]
fn truncates_by_characters_not_bytes() {
    assert_eq!(truncate_chars("héllo", 2), "hé");
    assert_eq!(truncate_chars("hi", 10), "hi");
    assert_eq!(truncate_chars(&"x".repeat(5000), NOTIFY_BODY_MAX).chars().count(), 1000);
    assert_eq!(NOTIFY_TITLE_MAX, 200);
}

#[test]
fn allows_up_to_max_within_window_per_key() {
    let now = Instant::now();
    let mut limiter = RateLimiter::notifications();
    for _ in 0..5 {
        assert!(limiter.allow("a", now));
    }
    assert!(!limiter.allow("a", now + Duration::from_secs(9)));
    assert!(limiter.allow("b", now));
}

#[test]
fn window_slides() {
    let now = Instant::now();
    let mut limiter = RateLimiter::new(2, Duration::from_secs(10));
    assert!(limiter.allow("a", now));
    assert!(limiter.allow("a", now + Duration::from_secs(5)));
    assert!(!limiter.allow("a", now + Duration::from_secs(9)));
    assert!(limiter.allow("a", now + Duration::from_secs(10)));
    assert!(!limiter.allow("a", now + Duration::from_secs(11)));
    assert!(limiter.allow("a", now + Duration::from_secs(15)));
}
