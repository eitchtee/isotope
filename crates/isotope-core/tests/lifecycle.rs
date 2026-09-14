use std::time::{Duration, Instant};

use isotope_core::lifecycle::{AppState, Effect, Lifecycle, Policy};
use isotope_core::model::HibernationConfig;

const TEN_MIN: Duration = Duration::from_secs(600);

fn on(_: &str) -> Policy {
    Policy { enabled: true, timeout: TEN_MIN }
}

fn off(_: &str) -> Policy {
    Policy { enabled: false, timeout: TEN_MIN }
}

fn create(id: &str, visible: bool) -> Effect {
    Effect::Create { id: id.into(), visible }
}

#[test]
fn policy_from_config() {
    let cfg = HibernationConfig { enabled: true, timeout_minutes: 3, start_hibernated: false };
    assert_eq!(Policy::from(&cfg), Policy { enabled: true, timeout: Duration::from_secs(180) });
}

#[test]
fn register_running_creates_hidden_and_schedules() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    assert_eq!(lc.register("a", false, on("a"), now), vec![create("a", false)]);
    assert_eq!(lc.state("a"), AppState::Running);
    assert_eq!(lc.next_deadline(), Some(now + TEN_MIN));
}

#[test]
fn register_start_hibernated_does_nothing() {
    let mut lc = Lifecycle::new();
    assert_eq!(lc.register("a", true, on("a"), Instant::now()), vec![]);
    assert_eq!(lc.state("a"), AppState::Hibernated);
    assert_eq!(lc.next_deadline(), None);
}

#[test]
fn activating_hibernated_app_creates_it_visible() {
    let mut lc = Lifecycle::new();
    lc.register("a", true, on("a"), Instant::now());
    assert_eq!(lc.activate("a", on, Instant::now()), vec![create("a", true)]);
    assert_eq!(lc.state("a"), AppState::Active);
    assert_eq!(lc.active(), Some("a"));
}

#[test]
fn activating_running_app_shows_it_and_cancels_its_timer() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    lc.register("a", false, on("a"), now);
    assert_eq!(lc.activate("a", on, now), vec![Effect::Show("a".into())]);
    assert_eq!(lc.next_deadline(), None);
    assert_eq!(lc.tick(now + TEN_MIN * 2), vec![]);
}

#[test]
fn activating_the_active_app_is_a_no_op() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    lc.activate("a", on, now);
    assert_eq!(lc.activate("a", on, now), vec![]);
}

#[test]
fn switching_hides_previous_app_and_hibernates_it_after_timeout() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    lc.activate("a", on, now);

    assert_eq!(lc.activate("b", on, now), vec![create("b", true), Effect::Hide("a".into())]);
    assert_eq!(lc.state("a"), AppState::Running);

    assert_eq!(lc.tick(now + TEN_MIN - Duration::from_secs(1)), vec![]);
    assert_eq!(lc.tick(now + TEN_MIN), vec![Effect::Destroy("a".into())]);
    assert_eq!(lc.state("a"), AppState::Hibernated);
    assert_eq!(lc.state("b"), AppState::Active);
}

#[test]
fn reactivating_before_timeout_cancels_hibernation() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    lc.activate("a", on, now);
    lc.activate("b", on, now);
    lc.activate("a", on, now + Duration::from_secs(60));

    assert_eq!(lc.tick(now + TEN_MIN * 2), vec![Effect::Destroy("b".into())]);
    assert_eq!(lc.state("a"), AppState::Active);
}

#[test]
fn disabled_policy_never_hibernates() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    lc.activate("a", off, now);
    lc.activate("b", off, now);
    assert_eq!(lc.next_deadline(), None);
    assert_eq!(lc.tick(now + TEN_MIN * 100), vec![]);
    assert_eq!(lc.state("a"), AppState::Running);
}

#[test]
fn hibernate_now_destroys_and_clears_active() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    lc.activate("a", on, now);
    assert_eq!(lc.hibernate("a"), vec![Effect::Destroy("a".into())]);
    assert_eq!(lc.state("a"), AppState::Hibernated);
    assert_eq!(lc.active(), None);
    assert_eq!(lc.hibernate("a"), vec![]);
}

#[test]
fn wake_creates_hidden_and_schedules() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    assert_eq!(lc.wake("a", on("a"), now), vec![create("a", false)]);
    assert_eq!(lc.state("a"), AppState::Running);
    assert_eq!(lc.next_deadline(), Some(now + TEN_MIN));
    assert_eq!(lc.wake("a", on("a"), now), vec![]);
}

#[test]
fn failed_app_is_retried_on_activate() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    lc.register("a", false, on("a"), now);
    lc.failed("a", "boom");
    assert_eq!(lc.state("a"), AppState::Error("boom".into()));
    assert_eq!(lc.next_deadline(), None);
    assert_eq!(lc.activate("a", on, now), vec![create("a", true)]);
}

#[test]
fn recreate_only_affects_live_webviews() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    lc.register("bg", false, on("bg"), now);
    lc.activate("fg", on, now);
    lc.register("zz", true, on("zz"), now);

    assert_eq!(lc.recreate("bg"), vec![Effect::Destroy("bg".into()), create("bg", false)]);
    assert_eq!(lc.recreate("fg"), vec![Effect::Destroy("fg".into()), create("fg", true)]);
    assert_eq!(lc.recreate("zz"), vec![]);
}

#[test]
fn remove_destroys_and_forgets() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    lc.activate("a", on, now);
    assert_eq!(lc.remove("a"), vec![Effect::Destroy("a".into())]);
    assert_eq!(lc.active(), None);
    assert_eq!(lc.remove("a"), vec![]);
}

#[test]
fn next_deadline_is_the_earliest() {
    let now = Instant::now();
    let mut lc = Lifecycle::new();
    lc.register("late", false, Policy { enabled: true, timeout: TEN_MIN * 2 }, now);
    lc.register("soon", false, Policy { enabled: true, timeout: TEN_MIN }, now);
    assert_eq!(lc.next_deadline(), Some(now + TEN_MIN));
}
