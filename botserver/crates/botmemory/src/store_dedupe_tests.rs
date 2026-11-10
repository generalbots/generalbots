//! Tests for the dedupe contract: normalization and decision resolution.
//!
//! `store_tests.rs` covers the three basic outcomes; this file pins the
//! tie-breaking rules that decide *which* incumbent wins and documents the
//! two confidence comparisons the contract uses (`>` on the normalized path,
//! `>=` on the raw-lowercase path).

use super::*;

fn candidate(content: &str, confidence: f32, minutes_ago: i64) -> DedupeCandidate {
    let updated_at = Utc::now() - chrono::Duration::minutes(minutes_ago);
    DedupeCandidate::build(Uuid::new_v4(), content, confidence, updated_at)
}

#[test]
fn normalization_collapses_spacing_case_and_ascii_punctuation() {
    let cases = [
        ("Prefers   dark mode.", "prefers dark mode"),
        ("  prefers dark mode  ", "prefers dark mode"),
        ("Prefers\tdark\nmode", "prefers dark mode"),
        ("PREFERS DARK MODE!", "prefers dark mode"),
        ("prefers, dark; mode?", "prefers dark mode"),
        ("Café com leite", "café com leite"),
        ("", ""),
        ("...", ""),
    ];
    for (raw, expected) in cases {
        assert_eq!(normalize_content(raw), expected, "normalize {raw:?}");
    }
}

#[test]
fn an_empty_candidate_set_is_always_fresh() {
    assert_eq!(decide_dedupe(&[], "likes tea", "likes tea", 0.5), DedupeDecision::Fresh);
}

#[test]
fn normalized_equality_skips_even_when_confidence_is_higher() {
    let existing = vec![candidate("Prefers dark mode.", 0.5, 10)];
    // Different wording: a restatement, not a revision.
    let decision = decide_dedupe(
        &existing,
        &normalize_content("prefers dark mode"),
        "prefers dark mode",
        0.99,
    );
    assert_eq!(decision, DedupeDecision::SkipDuplicate { candidate: existing[0].id });
}

#[test]
fn identical_wording_supersedes_only_above_the_incumbent() {
    let existing = vec![candidate("Likes tea", 0.5, 10)];
    let id = existing[0].id;

    assert_eq!(
        decide_dedupe(&existing, "likes tea", "Likes Tea", 0.9),
        DedupeDecision::Supersede { candidate: id }
    );
    // Equal confidence is not an improvement on this path.
    assert_eq!(
        decide_dedupe(&existing, "likes tea", "Likes Tea", 0.5),
        DedupeDecision::SkipLowerConfidence { candidate: id }
    );
}

#[test]
fn the_strongest_incumbent_wins_among_duplicates() {
    let weak = candidate("Likes tea", 0.4, 5);
    let strong = candidate("Likes tea", 0.9, 60);
    let fresh_but_weaker = candidate("Likes tea", 0.6, 1);
    let existing = vec![weak.clone(), strong.clone(), fresh_but_weaker.clone()];

    assert_eq!(
        decide_dedupe(&existing, "likes tea", "Likes Tea", 0.95),
        DedupeDecision::Supersede { candidate: strong.id }
    );
    assert_eq!(
        decide_dedupe(&existing, "likes tea", "Likes Tea", 0.7),
        DedupeDecision::SkipLowerConfidence { candidate: strong.id }
    );
}

#[test]
fn equal_confidence_breaks_the_tie_by_recency() {
    let older = candidate("Likes tea", 0.8, 120);
    let newer = candidate("Likes tea", 0.8, 1);
    let existing = vec![older, newer.clone()];

    assert_eq!(
        decide_dedupe(&existing, "likes tea", "Likes Tea", 0.8),
        DedupeDecision::SkipLowerConfidence { candidate: newer.id }
    );
}

#[test]
fn a_reworded_incumbent_is_skipped_not_superseded() {
    // Extra spacing normalizes onto the incumbent, but the raw wording differs,
    // so the contract treats it as the same memory restated: skip, do not
    // revise — even with a much higher confidence.
    let existing = vec![candidate("Prefers dark mode", 0.8, 5)];
    let decision = decide_dedupe(
        &existing,
        &normalize_content("Prefers  dark   mode"),
        "prefers  dark   mode",
        0.99,
    );
    assert_eq!(
        decision,
        DedupeDecision::SkipDuplicate { candidate: existing[0].id }
    );
}

#[test]
fn a_hyphen_joins_words_instead_of_separating_them() {
    // Punctuation is dropped, not turned into a space: "dark-mode" normalizes
    // to "darkmode", which is a different memory from "dark mode". Documented
    // so a future normalizer change is a deliberate decision.
    assert_eq!(normalize_content("Prefers dark-mode"), "prefers darkmode");
    assert_eq!(normalize_content("Prefers dark mode"), "prefers dark mode");
}

#[test]
fn unrelated_content_never_matches() {
    let existing = vec![
        candidate("Drives a truck", 0.9, 5),
        candidate("Lives in Lisbon", 0.9, 5),
    ];
    assert_eq!(
        decide_dedupe(&existing, "flies planes", "flies planes", 0.9),
        DedupeDecision::Fresh
    );
}

#[test]
fn candidate_of_row_mirrors_the_row_columns() {
    let row = UserMemory {
        id: Uuid::new_v4(),
        org_id: None,
        branch_id: None,
        owner_user_id: Uuid::new_v4(),
        scope: crate::models::SCOPE_PRIVATE.to_string(),
        kind: "fact".to_string(),
        content: "Prefers  Dark Mode.".to_string(),
        source: "manual".to_string(),
        confidence: 0.75,
        pinned: false,
        superseded_by: None,
        embedding_ref: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let candidate = candidate_of_row(&row);
    assert_eq!(candidate.id, row.id);
    assert_eq!(candidate.confidence, 0.75);
    assert_eq!(candidate.normalized, "prefers dark mode");
    assert_eq!(candidate.raw_lowercase, "prefers  dark mode.");
}
