//! Offline tests for the social ranking helpers (issue #2075, CRM wave).
//!
//! `rank_hashtags` and `summarise_content` are the pure cores of the trending
//! and activity endpoints: they take already-loaded posts and never touch the
//! database.

use super::*;

#[test]
fn hashtags_are_ranked_by_usage_count() {
    let ranked = rank_hashtags(["rust", "rust", "bots", "rust", "bots", "news"], 10);

    let published: Vec<(&str, i64)> = ranked.iter().map(|t| (t.tag.as_str(), t.count)).collect();
    assert_eq!(published, vec![("rust", 3), ("bots", 2), ("news", 1)]);
}

#[test]
fn ties_are_broken_alphabetically_so_the_response_is_deterministic() {
    let ranked = rank_hashtags(["zeta", "alpha", "mike", "alpha", "zeta", "mike"], 10);

    let tags: Vec<&str> = ranked.iter().map(|t| t.tag.as_str()).collect();
    assert_eq!(tags, vec!["alpha", "mike", "zeta"]);
    assert!(ranked.iter().all(|t| t.count == 2));
}

#[test]
fn blank_tags_are_ignored() {
    let ranked = rank_hashtags(["", "bots", "   "], 10);

    let tags: Vec<&str> = ranked.iter().map(|t| t.tag.as_str()).collect();
    assert_eq!(tags, vec!["bots"], "blank tags are not trending topics");
}

#[test]
fn padded_and_unpadded_tags_count_as_the_same_topic() {
    let ranked = rank_hashtags([" rust ", "rust", "bots"], 10);

    let published: Vec<(&str, i64)> = ranked.iter().map(|t| (t.tag.as_str(), t.count)).collect();
    assert_eq!(published, vec![("rust", 2), ("bots", 1)]);
}

#[test]
fn no_tags_means_no_topics() {
    assert!(rank_hashtags(std::iter::empty(), 10).is_empty());
}

#[test]
fn the_result_is_capped_at_the_requested_limit() {
    let tags: Vec<&str> = vec!["a", "b", "c", "d", "e"];
    let ranked = rank_hashtags(tags, 3);

    assert_eq!(ranked.len(), 3);
    let published: Vec<&str> = ranked.iter().map(|t| t.tag.as_str()).collect();
    assert_eq!(published, vec!["a", "b", "c"], "equal counts sort alphabetically");
}

#[test]
fn a_zero_limit_returns_nothing() {
    assert!(rank_hashtags(["bots"], 0).is_empty());
}

#[test]
fn short_content_is_published_verbatim() {
    assert_eq!(summarise_content("hello", 80), "hello");
    assert_eq!(summarise_content("", 80), "");
}

#[test]
fn content_at_the_limit_is_not_elided() {
    let content = "a".repeat(80);
    assert_eq!(summarise_content(&content, 80), content);
}

#[test]
fn longer_content_is_cut_and_marked_with_one_ellipsis() {
    let content = "a".repeat(81);
    let summary = summarise_content(&content, 80);

    assert_eq!(summary.chars().count(), 81, "80 characters plus the ellipsis");
    assert!(summary.ends_with('…'));
    assert_eq!(summary, format!("{}…", "a".repeat(80)));
}

#[test]
fn multibyte_content_is_cut_on_a_character_boundary() {
    let content = "ção ".repeat(40); // 160 characters, 3-4 bytes each
    let summary = summarise_content(&content, 80);

    assert_eq!(summary.chars().count(), 81);
    assert!(summary.ends_with('…'));
    assert_eq!(summary.chars().take(80).collect::<String>(), content.chars().take(80).collect::<String>());
}
