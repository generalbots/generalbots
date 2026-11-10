//! Tests for the recall ranking and the token-budgeted block assembly.

use super::*;
use crate::models::SCOPE_PRIVATE;
use chrono::{Duration, Utc};

fn memory(content: &str, kind: &str, pinned: bool, minutes_ago: i64) -> UserMemory {
    let updated_at = Utc::now() - Duration::minutes(minutes_ago);
    UserMemory {
        id: Uuid::new_v4(),
        org_id: None,
        branch_id: None,
        owner_user_id: Uuid::new_v4(),
        scope: SCOPE_PRIVATE.to_string(),
        kind: kind.to_string(),
        content: content.to_string(),
        source: "manual".to_string(),
        confidence: 0.8,
        pinned,
        superseded_by: None,
        embedding_ref: None,
        created_at: updated_at,
        updated_at,
    }
}

#[test]
fn lexical_score_counts_every_matching_word() {
    let words = vec!["invoice".to_string(), "paid".to_string()];
    assert_eq!(lexical_score("the invoice was paid", &words), 2);
    assert_eq!(lexical_score("invoice pending", &words), 1);
    assert_eq!(lexical_score("nothing here", &words), 0);
    assert_eq!(lexical_score("anything", &[]), 0);
}

#[test]
fn query_words_are_lowercased_stripped_and_filtered() {
    let rows = vec![
        memory("Invoice, paid.", "fact", false, 1),
        memory("INVOICE pending", "fact", false, 2),
        memory("unrelated", "fact", false, 3),
    ];
    // "a" and "I" are single characters and must not influence the ranking.
    let ranked = rank_rows(rows, "A Invoice, I");
    assert!(ranked[0].content.starts_with("Invoice"));
    assert!(ranked[1].content.starts_with("INVOICE"));
    assert_eq!(ranked[2].content, "unrelated");
}

#[test]
fn pinning_beats_a_better_lexical_match() {
    let pinned = memory("Owns a red bicycle", "fact", true, 500);
    let matching = memory("Invoice deadline is Friday", "task", false, 1);
    let ranked = rank_rows(vec![matching, pinned.clone()], "invoice");
    assert_eq!(ranked[0].id, pinned.id);
}

#[test]
fn recency_breaks_ties_between_unpinned_rows() {
    let older = memory("same text", "fact", false, 90);
    let newer = memory("same text", "fact", false, 5);
    let ranked = rank_rows(vec![older, newer.clone()], "");
    assert_eq!(ranked[0].id, newer.id);
}

#[test]
fn ranking_is_stable_for_an_empty_query() {
    let first = memory("alpha", "fact", false, 30);
    let second = memory("beta", "fact", false, 20);
    let third = memory("gamma", "fact", false, 10);
    let ranked = rank_rows(vec![first, second, third.clone()], "");
    assert_eq!(ranked[0].id, third.id);
}

#[test]
fn a_budget_below_the_header_yields_no_block() {
    let rows = vec![memory("anything", "fact", false, 1)];
    let header_len = HEADER.chars().count();
    assert_eq!(assemble_block(&rows, header_len), "");
    assert_eq!(assemble_block(&rows, header_len - 1), "");
    assert_eq!(assemble_block(&[], header_len * 4), "");
}

#[test]
fn the_block_starts_with_the_header_and_lists_kind_and_content() {
    let rows = vec![memory("prefers email contact", "preference", true, 1)];
    let block = assemble_block(&rows, 4000);
    assert!(block.starts_with(HEADER));
    assert!(block.contains("- [preference] prefers email contact"));
}

#[test]
fn lines_are_clipped_to_three_hundred_characters() {
    // 'z' avoids colliding with the 'x' inside the block header ("context").
    let long = "z".repeat(LINE_CONTENT_MAX_CHARS + 50);
    let block = assemble_block(&[memory(&long, "fact", false, 1)], 4000);
    let content_chars = block.chars().filter(|c| *c == 'z').count();
    assert_eq!(content_chars, LINE_CONTENT_MAX_CHARS);
}

#[test]
fn a_partial_line_is_appended_only_when_enough_room_remains() {
    let rows = vec![memory("a rather long memory that cannot fit", "fact", false, 1)];
    let header_len = HEADER.chars().count();

    // 20 characters of room after the header: more than the 16-character floor.
    let clipped = assemble_block(&rows, header_len + 20);
    assert_eq!(clipped.chars().count(), header_len + 20);
    assert!(clipped.contains("- [fact] a rather l"), "{clipped:?}");

    // Exactly 16 characters of room: below the floor, nothing is appended.
    assert_eq!(assemble_block(&rows, header_len + 16), "");
}

#[test]
fn the_budget_is_never_exceeded_across_many_rows() {
    let rows: Vec<UserMemory> = (0..RECALL_ROW_LIMIT)
        .map(|i| memory(&format!("memory number {i}"), "fact", false, i))
        .collect();
    for budget in [40usize, 80, 160, 320] {
        let block = assemble_block(&rows, budget);
        assert!(
            block.chars().count() <= budget,
            "budget {budget} exceeded by {block:?}"
        );
    }
}
