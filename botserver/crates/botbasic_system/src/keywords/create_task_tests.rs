// Tests for CREATE_TASK / ASSIGN_SMART (#2065), split out of
// `create_task.rs` to keep that file under the AGENTS.md size limit.

use super::*;

#[test]
fn a_null_or_blank_due_date_means_no_deadline() {
    assert!(parse_due_date("null").unwrap().is_none());
    assert!(parse_due_date("").unwrap().is_none());
    assert!(parse_due_date("NULL").unwrap().is_none());
}

#[test]
fn relative_offsets_are_added_to_now() {
    let before = Utc::now();
    let parsed = parse_due_date("+3").unwrap().expect("date");
    let delta = (parsed - before).num_days();
    assert!((2..=3).contains(&delta), "expected ~3 days, got {delta}");
}

#[test]
fn named_days_resolve_to_concrete_dates() {
    let today = parse_due_date("today").unwrap().expect("today");
    assert_eq!(today.time(), chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());

    let tomorrow = parse_due_date("tomorrow").unwrap().expect("tomorrow");
    assert!(tomorrow > today);

    let next_week = parse_due_date("next week").unwrap().expect("next week");
    // `num_days` truncates, so a few microseconds past the boundary read 6.
    assert!((6..=7).contains(&(next_week - Utc::now()).num_days()));

    let next_month = parse_due_date("next month").unwrap().expect("next month");
    assert!((29..=30).contains(&(next_month - Utc::now()).num_days()));
}

#[test]
fn iso_dates_are_parsed_at_midnight() {
    let parsed = parse_due_date("2026-12-25").unwrap().expect("date");
    assert_eq!(
        parsed.date_naive(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 25).unwrap()
    );
}

#[test]
fn unrecognised_input_falls_back_to_three_days() {
    let parsed = parse_due_date("sometime soon").unwrap().expect("date");
    // `num_days` truncates fractional days, so 3 days reads as 2 or 3.
    assert!((2..=3).contains(&(parsed - Utc::now()).num_days()));
}

#[test]
fn priority_tracks_the_deadline_distance() {
    assert_eq!(determine_priority(None), "medium");
    assert_eq!(determine_priority(Some(Utc::now())), "high");
    assert_eq!(determine_priority(Some(Utc::now() + Duration::days(1))), "high");
    assert_eq!(determine_priority(Some(Utc::now() + Duration::days(5))), "medium");
    assert_eq!(determine_priority(Some(Utc::now() + Duration::days(30))), "low");
}
