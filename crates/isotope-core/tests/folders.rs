mod common;

use common::{folder_names, new_app, sidebar_names};
use isotope_core::model::{Config, SidebarItem};
use isotope_core::ops::{Container, OpError};

/// Config with top-level apps named A, B, C, D (in that order). Returns their ids.
fn setup() -> (Config, Vec<String>) {
    let mut c = Config::default();
    let ids = ["A", "B", "C", "D"].iter().map(|n| c.add_app(new_app(n)).unwrap()).collect();
    (c, ids)
}

#[test]
fn add_folder_takes_the_first_apps_slot() {
    let (mut c, ids) = setup();
    let f = c.add_folder("Work", &[ids[1].clone(), ids[3].clone()]).unwrap();

    assert_eq!(sidebar_names(&c), vec!["A", "[Work]", "C"]);
    assert_eq!(folder_names(&c, &f), vec!["B", "D"]);
    c.check_invariants().unwrap();
}

#[test]
fn add_folder_moves_apps_out_of_other_folders() {
    let (mut c, ids) = setup();
    let first = c.add_folder("One", &[ids[0].clone(), ids[1].clone()]).unwrap();
    let second = c.add_folder("Two", &[ids[1].clone()]).unwrap();

    assert_eq!(folder_names(&c, &first), vec!["A"]);
    assert_eq!(folder_names(&c, &second), vec!["B"]);
    assert_eq!(sidebar_names(&c), vec!["[One]", "C", "D", "[Two]"]);
    c.check_invariants().unwrap();
}

#[test]
fn add_folder_ignores_duplicate_ids() {
    let (mut c, ids) = setup();
    let f = c.add_folder("Work", &[ids[0].clone(), ids[0].clone()]).unwrap();
    assert_eq!(folder_names(&c, &f), vec!["A"]);
    c.check_invariants().unwrap();
}

#[test]
fn add_folder_with_unknown_app_changes_nothing() {
    let (mut c, ids) = setup();
    let before = c.clone();
    assert_eq!(
        c.add_folder("Work", &[ids[0].clone(), "nope".into()]),
        Err(OpError::AppNotFound("nope".into()))
    );
    assert_eq!(c, before);
}

#[test]
fn remove_folder_puts_apps_back_at_its_position() {
    let (mut c, ids) = setup();
    let f = c.add_folder("Work", &[ids[1].clone(), ids[2].clone()]).unwrap();
    assert_eq!(sidebar_names(&c), vec!["A", "[Work]", "D"]);

    c.remove_folder(&f).unwrap();

    assert_eq!(sidebar_names(&c), vec!["A", "B", "C", "D"]);
    assert!(c.folders.is_empty());
    c.check_invariants().unwrap();
}

#[test]
fn rename_folder() {
    let (mut c, ids) = setup();
    let f = c.add_folder("Work", &[ids[0].clone()]).unwrap();
    c.rename_folder(&f, "Office").unwrap();
    assert_eq!(c.folder(&f).unwrap().name, "Office");
    assert_eq!(c.rename_folder("nope", "X"), Err(OpError::FolderNotFound("nope".into())));
}

#[test]
fn move_item_reorders_the_sidebar() {
    let (mut c, ids) = setup();
    c.move_item(&SidebarItem::App { id: ids[0].clone() }, &Container::Sidebar, 2).unwrap();
    assert_eq!(sidebar_names(&c), vec!["B", "C", "A", "D"]);

    c.move_item(&SidebarItem::App { id: ids[3].clone() }, &Container::Sidebar, 0).unwrap();
    assert_eq!(sidebar_names(&c), vec!["D", "B", "C", "A"]);
    c.check_invariants().unwrap();
}

#[test]
fn move_item_clamps_index_to_the_end() {
    let (mut c, ids) = setup();
    c.move_item(&SidebarItem::App { id: ids[0].clone() }, &Container::Sidebar, 99).unwrap();
    assert_eq!(sidebar_names(&c), vec!["B", "C", "D", "A"]);
}

#[test]
fn move_app_into_and_out_of_a_folder() {
    let (mut c, ids) = setup();
    let f = c.add_folder("Work", &[ids[0].clone()]).unwrap();
    let into = Container::Folder { id: f.clone() };

    c.move_item(&SidebarItem::App { id: ids[2].clone() }, &into, 0).unwrap();
    assert_eq!(folder_names(&c, &f), vec!["C", "A"]);
    assert_eq!(sidebar_names(&c), vec!["[Work]", "B", "D"]);

    c.move_item(&SidebarItem::App { id: ids[0].clone() }, &Container::Sidebar, 1).unwrap();
    assert_eq!(folder_names(&c, &f), vec!["C"]);
    assert_eq!(sidebar_names(&c), vec!["[Work]", "A", "B", "D"]);
    c.check_invariants().unwrap();
}

#[test]
fn move_folder_within_sidebar() {
    let (mut c, ids) = setup();
    let f = c.add_folder("Work", &[ids[0].clone()]).unwrap();
    c.move_item(&SidebarItem::Folder { id: f }, &Container::Sidebar, 3).unwrap();
    assert_eq!(sidebar_names(&c), vec!["B", "C", "D", "[Work]"]);
    c.check_invariants().unwrap();
}

#[test]
fn folders_cannot_be_moved_into_folders() {
    let (mut c, ids) = setup();
    let a = c.add_folder("A", &[ids[0].clone()]).unwrap();
    let b = c.add_folder("B", &[ids[1].clone()]).unwrap();
    let before = c.clone();
    assert_eq!(
        c.move_item(&SidebarItem::Folder { id: a }, &Container::Folder { id: b }, 0),
        Err(OpError::NestedFolder)
    );
    assert_eq!(c, before);
}

#[test]
fn move_into_unknown_folder_changes_nothing() {
    let (mut c, ids) = setup();
    let before = c.clone();
    assert_eq!(
        c.move_item(&SidebarItem::App { id: ids[0].clone() }, &Container::Folder { id: "nope".into() }, 0),
        Err(OpError::FolderNotFound("nope".into()))
    );
    assert_eq!(c, before);
}

#[test]
fn container_serializes_with_type_tag() {
    assert_eq!(serde_json::to_value(Container::Sidebar).unwrap(), serde_json::json!({ "type": "sidebar" }));
    assert_eq!(
        serde_json::to_value(Container::Folder { id: "f".into() }).unwrap(),
        serde_json::json!({ "type": "folder", "id": "f" })
    );
}
