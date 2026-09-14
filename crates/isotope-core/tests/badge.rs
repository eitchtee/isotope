use isotope_core::badge::{parse_badge, Badge};

#[test]
fn leading_counts() {
    assert_eq!(parse_badge("(3) Inbox - Gmail"), Some(Badge::Count(3)));
    assert_eq!(parse_badge("[12] Slack"), Some(Badge::Count(12)));
    assert_eq!(parse_badge("  (7)Chat"), Some(Badge::Count(7)));
    assert_eq!(parse_badge("(99+) Inbox"), Some(Badge::Count(99)));
}

#[test]
fn trailing_counts() {
    assert_eq!(parse_badge("Inbox (5)"), Some(Badge::Count(5)));
    assert_eq!(parse_badge("Inbox (5) "), Some(Badge::Count(5)));
}

#[test]
fn dot_markers() {
    assert_eq!(parse_badge("• Chat"), Some(Badge::Dot));
    assert_eq!(parse_badge("●Discord"), Some(Badge::Dot));
}

#[test]
fn no_badge() {
    assert_eq!(parse_badge("Inbox"), None);
    assert_eq!(parse_badge(""), None);
    assert_eq!(parse_badge("(0) Inbox"), None);
    assert_eq!(parse_badge("(Draft) Document"), None);
    assert_eq!(parse_badge("Meeting (notes)"), None);
    assert_eq!(parse_badge("(3"), None);
}

#[test]
fn huge_numbers_saturate() {
    assert_eq!(parse_badge("(99999999999999) x"), Some(Badge::Count(u32::MAX)));
}

#[test]
fn labels() {
    assert_eq!(Badge::Count(5).label(), "5");
    assert_eq!(Badge::Count(99).label(), "99");
    assert_eq!(Badge::Count(100).label(), "99+");
    assert_eq!(Badge::Dot.label(), "•");
}
