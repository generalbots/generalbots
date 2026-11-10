//! Tests for the dry-run projection of an import.
//!
//! A dry run must predict exactly what the live path would do, so the
//! projection is exercised against sequences where later items observe the
//! simulated inserts of earlier ones.

use super::*;

fn candidate(content: &str, confidence: f32) -> DedupeCandidate {
    DedupeCandidate::build(Uuid::new_v4(), content, confidence, Utc::now())
}

fn item(content: &str, confidence: f32) -> MemoryItem {
    MemoryItem {
        kind: "fact".to_string(),
        content: content.to_string(),
        scope: SCOPE_PRIVATE.to_string(),
        pinned: false,
        confidence,
        source: None,
    }
}

#[test]
fn an_empty_import_plans_nothing() {
    let report = plan_import(Vec::new(), &[]);
    assert_eq!(report.would_create, 0);
    assert_eq!(report.superseded, 0);
    assert_eq!(report.skipped, 0);
}

#[test]
fn repeated_items_are_projected_as_one_create_then_supersedes() {
    let items = vec![
        item("Likes tea", 0.4),
        item("Likes tea", 0.6),
        item("Likes tea", 0.8),
    ];
    let report = plan_import(Vec::new(), &items);
    assert_eq!(report.would_create, 1);
    assert_eq!(report.superseded, 2);
    assert_eq!(report.skipped, 0);
}

#[test]
fn a_lower_confidence_restatement_is_projected_as_skipped() {
    let items = vec![item("Likes tea", 0.9), item("Likes tea", 0.2)];
    let report = plan_import(Vec::new(), &items);
    assert_eq!(report.would_create, 1);
    assert_eq!(report.skipped, 1);
    assert_eq!(report.superseded, 0);
}

#[test]
fn independent_items_keep_their_own_simulated_candidates() {
    // Regression: simulated inserts used to share the nil id, so superseding
    // one item dropped every other simulated candidate and the projection
    // reported "would create" for an item the live run would supersede.
    let items = vec![
        item("Likes tea", 0.5),
        item("Likes coffee", 0.5),
        item("Likes tea", 0.9),
        item("Likes coffee", 0.9),
    ];
    let report = plan_import(Vec::new(), &items);
    assert_eq!(report.would_create, 2, "both items are created once");
    assert_eq!(report.superseded, 2, "both later items revise the first ones");
    assert_eq!(report.skipped, 0);
}

#[test]
fn existing_rows_are_counted_without_being_recreated() {
    let existing = vec![candidate("Allergic to peanuts", 0.9)];
    let items = vec![
        item("Allergic to peanuts", 0.95),
        item("Speaks Portuguese", 0.7),
    ];
    let report = plan_import(existing, &items);
    assert_eq!(report.superseded, 1);
    assert_eq!(report.would_create, 1);
    assert_eq!(report.skipped, 0);
}

#[test]
fn the_projection_never_reports_performed_work() {
    let items = vec![item("Likes tea", 0.5)];
    let report = plan_import(Vec::new(), &items);
    assert_eq!(report.created, 0, "dry runs write nothing");
    assert_eq!(report.would_create, 1);
}

#[test]
fn the_import_limit_is_five_hundred_items() {
    assert_eq!(MAX_IMPORT_ITEMS, 500);
}
