//! Contract tests for the iCalendar recurrence rendering (#2064).
//!
//! `calendar_events.recurrence_rule` stores an RFC 5545 `RRULE` value. The
//! renderer is pure and its output is consumed by external calendar clients, so
//! the field order and the omission rules are pinned here.

use super::{recurrence_rule_to_string, RecurrenceRule};
use chrono::{TimeZone, Utc};

fn rule(frequency: &str) -> RecurrenceRule {
    RecurrenceRule {
        frequency: frequency.to_string(),
        interval: 0,
        count: None,
        until: None,
        by_day: None,
    }
}

#[test]
fn a_minimal_rule_emits_only_the_frequency() {
    assert_eq!(recurrence_rule_to_string(&rule("daily")), "FREQ=DAILY");
    // The frequency is upper-cased: iCalendar keywords are case-insensitive but
    // clients and tests compare the canonical form.
    assert_eq!(recurrence_rule_to_string(&rule("Weekly")), "FREQ=WEEKLY");
}

#[test]
fn interval_is_omitted_when_it_is_not_positive() {
    let mut weekly = rule("weekly");
    assert_eq!(recurrence_rule_to_string(&weekly), "FREQ=WEEKLY");
    weekly.interval = -3;
    assert_eq!(recurrence_rule_to_string(&weekly), "FREQ=WEEKLY");
    weekly.interval = 2;
    assert_eq!(recurrence_rule_to_string(&weekly), "FREQ=WEEKLY;INTERVAL=2");
}

#[test]
fn count_and_until_are_rendered_when_present() {
    let mut monthly = rule("monthly");
    monthly.count = Some(10);
    assert_eq!(recurrence_rule_to_string(&monthly), "FREQ=MONTHLY;COUNT=10");

    let mut daily = rule("daily");
    daily.until = Some(
        Utc.with_ymd_and_hms(2026, 12, 31, 23, 59, 59)
            .single()
            .expect("valid fixture"),
    );
    assert_eq!(
        recurrence_rule_to_string(&daily),
        "FREQ=DAILY;UNTIL=20261231T235959Z"
    );
}

#[test]
fn by_day_lists_are_joined_and_empty_lists_are_dropped() {
    let mut weekly = rule("weekly");
    weekly.by_day = Some(vec!["MO".to_string(), "WE".to_string(), "FR".to_string()]);
    assert_eq!(
        recurrence_rule_to_string(&weekly),
        "FREQ=WEEKLY;BYDAY=MO,WE,FR"
    );

    weekly.by_day = Some(Vec::new());
    assert_eq!(recurrence_rule_to_string(&weekly), "FREQ=WEEKLY");
}

#[test]
fn the_full_rule_uses_the_documented_field_order() {
    let mut rule = rule("weekly");
    rule.interval = 2;
    rule.count = Some(5);
    rule.until = Some(
        Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0)
            .single()
            .expect("valid fixture"),
    );
    rule.by_day = Some(vec!["TU".to_string()]);

    assert_eq!(
        recurrence_rule_to_string(&rule),
        "FREQ=WEEKLY;INTERVAL=2;COUNT=5;UNTIL=20261231T000000Z;BYDAY=TU"
    );
}
