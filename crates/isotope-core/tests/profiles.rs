mod common;

use common::new_app;
use isotope_core::model::{Config, DEFAULT_PROFILE_ID};
use isotope_core::ops::OpError;

#[test]
fn add_and_rename_profile() {
    let mut c = Config::default();
    let id = c.add_profile("Work");
    assert_eq!(c.profiles.len(), 2);
    assert_ne!(id, DEFAULT_PROFILE_ID);

    c.rename_profile(&id, "Office").unwrap();
    assert_eq!(c.profiles[1].name, "Office");
}

#[test]
fn rename_unknown_profile_errors() {
    let mut c = Config::default();
    assert_eq!(c.rename_profile("nope", "X"), Err(OpError::ProfileNotFound("nope".into())));
}

#[test]
fn default_profile_cannot_be_removed() {
    let mut c = Config::default();
    assert_eq!(c.remove_profile(DEFAULT_PROFILE_ID), Err(OpError::DefaultProfile));
    assert_eq!(c.profiles.len(), 1);
}

#[test]
fn profile_in_use_cannot_be_removed() {
    let mut c = Config::default();
    let work = c.add_profile("Work");
    let mut input = new_app("Gmail");
    input.profile_id = work.clone();
    c.add_app(input).unwrap();

    assert_eq!(c.remove_profile(&work), Err(OpError::ProfileInUse(work.clone())));
    assert_eq!(c.profiles.len(), 2);
}

#[test]
fn unused_profile_is_removed() {
    let mut c = Config::default();
    let work = c.add_profile("Work");
    c.remove_profile(&work).unwrap();
    assert_eq!(c.profiles.len(), 1);
    assert_eq!(c.remove_profile(&work), Err(OpError::ProfileNotFound(work)));
    c.check_invariants().unwrap();
}
