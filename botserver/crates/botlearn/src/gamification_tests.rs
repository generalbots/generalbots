//! Offline unit tests for the Learn gamification service (#2075).
//!
//! `GamificationService` is an in-memory state machine: XP is an append-only
//! ledger, badges are awarded once per user, and the leaderboard is a cache
//! that must be invalidated whenever XP changes. No database, no clock reads
//! beyond the stored timestamps.

use super::GamificationService;
use crate::models::calculate_level;
use uuid::Uuid;

fn user() -> Uuid {
    Uuid::from_u128(1)
}

#[test]
fn the_badge_catalog_is_well_formed() {
    let service = GamificationService::new();
    let badges = service.get_badge_definitions();
    assert!(!badges.is_empty());

    let mut types: Vec<&str> = badges.iter().map(|b| b.badge_type.as_str()).collect();
    let total = types.len();
    types.sort_unstable();
    types.dedup();
    assert_eq!(types.len(), total, "badge types must be unique");

    for badge in badges {
        assert!(!badge.name.is_empty(), "{} has no name", badge.badge_type);
        assert!(
            !badge.description.is_empty(),
            "{} has no description",
            badge.badge_type
        );
        assert!(
            badge.xp_reward > 0,
            "{} must reward XP",
            badge.badge_type
        );
        assert!(
            !badge.criteria.action.is_empty(),
            "{} has no trigger action",
            badge.badge_type
        );
    }
}

#[test]
fn xp_is_an_append_only_ledger() {
    let mut service = GamificationService::new();
    assert_eq!(service.get_user_xp(user()), 0);

    service.add_xp(user(), 100, "lesson");
    service.add_xp(user(), 50, "quiz");
    assert_eq!(service.get_user_xp(user()), 150);

    // Negative adjustments (corrections) are recorded, not clamped.
    service.add_xp(user(), -25, "correction");
    assert_eq!(service.get_user_xp(user()), 125);
}

#[test]
fn xp_is_tracked_per_user() {
    let mut service = GamificationService::new();
    let other = Uuid::from_u128(2);

    service.add_xp(user(), 100, "lesson");
    service.add_xp(other, 10, "lesson");

    assert_eq!(service.get_user_xp(user()), 100);
    assert_eq!(service.get_user_xp(other), 10);
}

#[test]
fn the_level_follows_the_xp_ladder() {
    let mut service = GamificationService::new();
    assert_eq!(service.get_user_level(user()).level, 0);

    service.add_xp(user(), 110, "first rung");
    let info = service.get_user_level(user());
    assert_eq!(info.level, 1);
    assert_eq!(info.total_xp, 110);
    assert_eq!(info.user_id, user());
    assert_eq!(info.level, calculate_level(110));
}

#[test]
fn awarding_a_badge_grants_its_xp_and_is_recorded() {
    let mut service = GamificationService::new();
    let reward = service
        .get_badge_definitions()
        .iter()
        .find(|b| b.badge_type == "first_course")
        .map(|b| b.xp_reward)
        .expect("first_course badge exists");

    let awarded = service
        .award_badge(user(), "first_course")
        .expect("known badge");
    assert_eq!(awarded.badge_type, "first_course");
    assert!(awarded.badge_name.is_some());
    assert_eq!(service.get_user_xp(user()), reward);
    assert_eq!(service.get_user_achievements(user()).len(), 1);

    // An unknown badge is a no-op, not a panic.
    assert!(service.award_badge(user(), "does-not-exist").is_none());
}

#[test]
fn achievements_are_listed_per_user() {
    let mut service = GamificationService::new();
    let other = Uuid::from_u128(2);
    service.award_badge(user(), "first_course");
    service.award_badge(other, "first_lesson");

    let achievements = service.get_user_achievements(user());
    assert_eq!(achievements.len(), 1);
    assert_eq!(achievements[0].badge_type, "first_course");
    assert!(service.get_user_achievements(Uuid::from_u128(9)).is_empty());
}

#[test]
fn check_and_award_respects_the_threshold_and_awards_once() {
    let mut service = GamificationService::new();

    // The badge with an 80 threshold only fires at or above it.
    let (action, threshold, badge_type) = service
        .get_badge_definitions()
        .iter()
        .find(|b| b.criteria.threshold == 80)
        .map(|b| {
            (
                b.criteria.action.clone(),
                b.criteria.threshold,
                b.badge_type.clone(),
            )
        })
        .expect("a badge with an 80 threshold");

    assert!(service
        .check_and_award(user(), &action, threshold - 1)
        .is_empty());
    assert_eq!(
        service.check_and_award(user(), &action, threshold).len(),
        1
    );
    // Re-running the same action must not duplicate the badge.
    assert!(service
        .check_and_award(user(), &action, threshold)
        .is_empty());
    assert_eq!(service.get_user_achievements(user()).len(), 1);
    assert!(!badge_type.is_empty());
}

#[test]
fn check_and_award_ignores_unknown_actions() {
    let mut service = GamificationService::new();
    assert!(service.check_and_award(user(), "no_such_action", 1000).is_empty());
    assert!(service.get_user_achievements(user()).is_empty());
}

#[test]
fn the_leaderboard_is_ranked_and_invalidated_on_xp_change() {
    let mut service = GamificationService::new();
    let second = Uuid::from_u128(2);

    service.add_xp(user(), 100, "lesson");
    service.add_xp(second, 250, "lesson");

    let board = service.get_leaderboard(10);
    assert_eq!(board.len(), 2);
    assert_eq!(board[0].user_id, second);
    assert_eq!(board[0].rank, 1);
    assert_eq!(board[0].total_xp, 250);
    assert_eq!(board[1].user_id, user());
    assert_eq!(board[1].rank, 2);
    assert!(board[0].user_name.starts_with("User "));

    // The cache is invalidated by a new transaction, so the ranking flips.
    service.add_xp(user(), 500, "bonus");
    let board = service.get_leaderboard(10);
    assert_eq!(board[0].user_id, user());
    assert_eq!(board[0].total_xp, 600);
    assert_eq!(board[1].user_id, second);
}

#[test]
fn the_leaderboard_honours_the_limit() {
    let mut service = GamificationService::new();
    for index in 0..5 {
        service.add_xp(Uuid::from_u128(index + 1), (index as i32 + 1) * 10, "xp");
    }
    assert_eq!(service.get_leaderboard(2).len(), 2);
    assert_eq!(service.get_leaderboard(0).len(), 0);
    // Requesting more than exists returns everyone, once.
    assert_eq!(service.get_leaderboard(99).len(), 5);
}

#[test]
fn the_leaderboard_counts_badges_and_completed_courses() {
    let mut service = GamificationService::new();
    service.award_badge(user(), "first_course");
    let board = service.get_leaderboard(10);
    assert_eq!(board.len(), 1);
    assert_eq!(board[0].badges_count, 1);
    assert_eq!(board[0].courses_completed, 1);
    assert_eq!(board[0].level, calculate_level(board[0].total_xp));
}
