use isotope_core::links::{classify, LinkTarget::{self, External, InApp, OsOpener}};

const GMAIL: &str = "https://mail.google.com/mail/u/0/";

fn gmail(target: &str) -> LinkTarget {
    classify(GMAIL, &["login.microsoftonline.com".to_string(), "*.Okta.com".to_string()], target)
}

#[test]
fn same_registrable_domain_stays_in_app() {
    assert_eq!(gmail("https://mail.google.com/mail/u/1/"), InApp);
    assert_eq!(gmail("https://accounts.google.com/signin"), InApp);
    assert_eq!(gmail("http://google.com"), InApp);
}

#[test]
fn other_sites_are_external() {
    assert_eq!(gmail("https://example.com"), External);
    assert_eq!(gmail("https://google.co.uk"), External);
    assert_eq!(gmail("https://microsoftonline.com"), External);
    assert_eq!(gmail("https://evil-login.microsoftonline.com"), External);
}

#[test]
fn allowlisted_domains_and_subdomains_stay_in_app() {
    assert_eq!(gmail("https://login.microsoftonline.com/common/oauth2"), InApp);
    assert_eq!(gmail("https://eu.login.microsoftonline.com"), InApp);
    assert_eq!(gmail("https://acme.okta.com/app"), InApp);
    assert_eq!(gmail("https://okta.com"), InApp);
}

#[test]
fn special_schemes() {
    assert_eq!(gmail("javascript:void(0)"), InApp);
    assert_eq!(gmail("data:text/html,hello"), InApp);
    assert_eq!(gmail("blob:https://mail.google.com/1234-5678"), InApp);
    assert_eq!(gmail("about:blank"), InApp);
    assert_eq!(gmail("mailto:someone@example.com"), OsOpener);
    assert_eq!(gmail("tel:+15551234567"), OsOpener);
    assert_eq!(gmail("zoommtg://zoom.us/join?confno=1"), OsOpener);
    assert_eq!(gmail("not a url"), OsOpener);
}

#[test]
fn public_suffix_edge_cases() {
    assert_eq!(classify("https://alice.github.io", &[], "https://alice.github.io/page"), InApp);
    assert_eq!(classify("https://alice.github.io", &[], "https://bob.github.io"), External);
    assert_eq!(classify("https://www.bbc.co.uk", &[], "https://news.bbc.co.uk"), InApp);
    assert_eq!(classify("https://www.bbc.co.uk", &[], "https://other.co.uk"), External);
}

#[test]
fn ip_and_localhost_hosts() {
    assert_eq!(classify("http://127.0.0.1:8080", &[], "http://127.0.0.1:9000/x"), InApp);
    assert_eq!(classify("http://127.0.0.1:8080", &[], "http://127.0.0.2"), External);
    assert_eq!(classify("http://10.0.0.1", &[], "http://20.0.0.1"), External);
    assert_eq!(classify("http://localhost:3000", &[], "http://localhost:5173/x"), InApp);
}
