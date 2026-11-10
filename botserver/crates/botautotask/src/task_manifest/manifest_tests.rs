//! Offline tests for the manifest presentation helpers (issue #2075):
//! section labels and the item-group lifecycle.

use super::*;

#[test]
fn section_labels_are_the_published_titles() {
    let cases = [
        (SectionType::DatabaseModels, "Database & Models"),
        (SectionType::SchemaDesign, "Database Schema Design"),
        (SectionType::Tables, "Tables"),
        (SectionType::Files, "Files"),
        (SectionType::Pages, "Pages"),
        (SectionType::Tools, "Tools"),
        (SectionType::Schedulers, "Schedulers"),
        (SectionType::Monitors, "Monitors"),
        (SectionType::Validation, "Validation"),
        (SectionType::Deployment, "Deployment"),
    ];
    for (section, label) in cases {
        assert_eq!(section.to_string(), label);
    }
}

#[test]
fn a_new_item_group_is_pending_and_has_no_timestamps() {
    let group = ItemGroup::new(vec!["orders".to_string(), "customers".to_string()]);

    assert_eq!(group.items, vec!["orders", "customers"]);
    assert_eq!(group.status, ItemStatus::Pending);
    assert!(group.duration_seconds.is_none());
    assert!(group.started_at.is_none());
    assert!(group.completed_at.is_none());
    assert!(!group.id.is_empty());
}

#[test]
fn item_group_ids_are_unique_per_instance() {
    let first = ItemGroup::new(vec!["a".to_string()]);
    let second = ItemGroup::new(vec!["a".to_string()]);
    assert_ne!(first.id, second.id);
}

#[test]
fn the_display_name_joins_the_items_with_commas() {
    let group = ItemGroup::new(vec![
        "orders".to_string(),
        "customers".to_string(),
        "invoices".to_string(),
    ]);
    assert_eq!(group.display_name(), "orders, customers, invoices");

    let empty = ItemGroup::new(Vec::new());
    assert_eq!(empty.display_name(), "");
}

#[test]
fn starting_a_group_marks_it_running() {
    let mut group = ItemGroup::new(vec!["a".to_string()]);
    group.start();

    assert_eq!(group.status, ItemStatus::Running);
    assert!(group.started_at.is_some());
    assert!(group.completed_at.is_none());
}

#[test]
fn completing_a_started_group_records_a_duration() {
    let mut group = ItemGroup::new(vec!["a".to_string()]);
    group.start();
    group.complete();

    assert_eq!(group.status, ItemStatus::Completed);
    assert!(group.completed_at.is_some());
    assert!(group.duration_seconds.is_some());
}

#[test]
fn completing_a_group_that_never_started_leaves_the_duration_empty() {
    let mut group = ItemGroup::new(vec!["a".to_string()]);
    group.complete();

    assert_eq!(group.status, ItemStatus::Completed);
    assert!(group.duration_seconds.is_none());
}

#[test]
fn section_and_item_statuses_default_to_pending() {
    assert_eq!(SectionStatus::default(), SectionStatus::Pending);
    assert_eq!(ItemStatus::default(), ItemStatus::Pending);
}
