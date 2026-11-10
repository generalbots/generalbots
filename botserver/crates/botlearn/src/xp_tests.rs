//! Offline unit tests for the Learn XP ladder (#2068).
//!
//! `calculate_level` and `xp_progress` are pure integer arithmetic: level `n`
//! costs `n * 100 + n^2 * 10` XP, and the level is the highest ladder rung the
//! total has already paid for. The tests pin the boundaries, not the formula
//! implementation, so a cost-curve change fails loudly.

use super::{calculate_level, xp_progress};

/// XP required to *reach* `level` from zero (sum of every rung up to it).
fn cumulative(level: i32) -> i32 {
    (1..=level).map(|n| n * 100 + n * n * 10).sum()
}

#[test]
fn level_zero_starts_at_zero_and_survives_negative_input() {
    assert_eq!(calculate_level(0), 0);
    // A negative balance must not produce a negative level or loop forever.
    assert_eq!(calculate_level(-1), 0);
    assert_eq!(calculate_level(-10_000), 0);
}

#[test]
fn first_rung_costs_one_hundred_ten_xp() {
    assert_eq!(calculate_level(109), 0);
    assert_eq!(calculate_level(110), 1);
    // Level 2 costs 240 more, so 349 is still level 1.
    assert_eq!(calculate_level(349), 1);
    assert_eq!(calculate_level(350), 2);
}

#[test]
fn every_ladder_boundary_is_exact() {
    for level in 0..25 {
        let reached = cumulative(level);
        assert_eq!(
            calculate_level(reached),
            level,
            "total {reached} must be exactly level {level}"
        );
        if level > 0 {
            assert_eq!(
                calculate_level(reached - 1),
                level - 1,
                "one XP short of {reached} must stay at level {}",
                level - 1
            );
        }
    }
}

#[test]
fn level_is_monotonic_in_total_xp() {
    let mut previous = 0;
    for total in (0..5000).step_by(37) {
        let level = calculate_level(total);
        assert!(level >= previous, "level dropped at {total}");
        previous = level;
    }
}

#[test]
fn progress_reports_the_xp_inside_the_current_level() {
    let info = xp_progress(0);
    assert_eq!(info.total_xp, 0);
    assert_eq!(info.level, 0);
    assert_eq!(info.xp_in_current_level, 0);
    assert_eq!(info.xp_to_next_level, 110);

    let info = xp_progress(110);
    assert_eq!(info.level, 1);
    assert_eq!(info.xp_in_current_level, 0);
    assert_eq!(info.xp_to_next_level, 240);

    let info = xp_progress(150);
    assert_eq!(info.level, 1);
    assert_eq!(info.xp_in_current_level, 40);
    assert_eq!(info.xp_to_next_level, 240);
}

#[test]
fn progress_never_exceeds_the_next_rung() {
    for total in (0..3000).step_by(53) {
        let info = xp_progress(total);
        assert!(info.xp_in_current_level >= 0, "negative progress at {total}");
        assert!(
            info.xp_in_current_level < info.xp_to_next_level,
            "progress {}/{} at total {total} overflows the level",
            info.xp_in_current_level,
            info.xp_to_next_level
        );
        assert_eq!(
            info.total_xp,
            cumulative(info.level) + info.xp_in_current_level,
            "progress must reconstruct the total at {total}"
        );
    }
}

#[test]
fn progress_carries_a_nil_user_until_the_caller_fills_it_in() {
    let info = xp_progress(500);
    assert_eq!(info.user_id, uuid::Uuid::nil());
    assert!(info.level >= 2);
}
