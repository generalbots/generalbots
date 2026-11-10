//! Contract tests for the BOOK natural-language time parsers (#2064).
//!
//! The parsers turn what a user typed ("tomorrow 3:30pm", "12/03/2026") into
//! the `TIMESTAMPTZ` written to `calendar_events`. They are pure, and a wrong
//! hour silently books the wrong slot.

use super::{extract_hour_from_string, parse_date_string, parse_time_string};
use chrono::{Datelike, TimeZone, Timelike, Utc};

#[test]
fn absolute_times_accept_the_documented_formats() {
    let cases = [
        ("2026-11-09 14:30", (2026, 11, 9, 14, 30)),
        ("2026-11-09 14:30:15", (2026, 11, 9, 14, 30)),
        ("2026/11/09 14:30", (2026, 11, 9, 14, 30)),
        ("09/11/2026 14:30", (2026, 11, 9, 14, 30)),
        ("2026-11-09T14:30:00", (2026, 11, 9, 14, 30)),
    ];

    for (input, (year, month, day, hour, minute)) in cases {
        let parsed = parse_time_string(input).expect("format is documented");
        assert_eq!(
            (parsed.year(), parsed.month(), parsed.day()),
            (year, month, day),
            "{input}"
        );
        assert_eq!((parsed.hour(), parsed.minute()), (hour, minute), "{input}");
    }
}

#[test]
fn a_time_zone_less_string_is_read_as_utc() {
    let parsed = parse_time_string("2026-11-09 14:30").expect("valid");
    let expected = Utc
        .with_ymd_and_hms(2026, 11, 9, 14, 30, 0)
        .single()
        .expect("valid fixture");
    assert_eq!(parsed, expected);
}

#[test]
fn relative_hours_are_added_to_now() {
    let before = Utc::now();
    let parsed = parse_time_string("in 3 hours").expect("relative hours");
    let after = Utc::now();

    assert!(parsed >= before + chrono::Duration::hours(3));
    assert!(parsed <= after + chrono::Duration::hours(3));
}

#[test]
fn unparseable_times_are_an_error_not_a_guess() {
    for input in ["", "next tuesday", "in three hours", "2026-13-45 10:00"] {
        assert!(
            parse_time_string(input).is_err(),
            "{input:?} must not be silently scheduled"
        );
    }
}

#[test]
fn dates_accept_the_documented_formats_at_midnight() {
    let cases = [
        ("2026-11-09", (2026, 11, 9)),
        ("2026/11/09", (2026, 11, 9)),
        ("09/11/2026", (2026, 11, 9)),
    ];
    for (input, (year, month, day)) in cases {
        let parsed = parse_date_string(input).expect("format is documented");
        assert_eq!((parsed.year(), parsed.month(), parsed.day()), (year, month, day));
        assert_eq!((parsed.hour(), parsed.minute(), parsed.second()), (0, 0, 0));
    }

    assert!(parse_date_string("09-11-2026").is_err());
    assert!(parse_date_string("2026-11-31").is_err());
}

#[test]
fn today_and_tomorrow_are_relative_dates() {
    let today = parse_date_string("today").expect("today");
    let tomorrow = parse_date_string("tomorrow").expect("tomorrow");
    assert!((today - Utc::now()).num_seconds().abs() < 5);
    assert!((tomorrow - (Utc::now() + chrono::Duration::days(1))).num_seconds().abs() < 5);
}

#[test]
fn hours_are_extracted_from_the_hour_component_only() {
    assert_eq!(extract_hour_from_string("10 am"), Some(10));
    assert_eq!(extract_hour_from_string("3pm"), Some(15));
    assert_eq!(extract_hour_from_string("11 PM"), Some(23));
    assert_eq!(extract_hour_from_string("12 pm"), Some(12));
    assert_eq!(extract_hour_from_string("12 am"), Some(0));

    // The minutes must not be concatenated into the hour: "3:30pm" used to
    // yield 330, which was then clamped to 23:00 by the caller.
    assert_eq!(extract_hour_from_string("3:30pm"), Some(15));
    assert_eq!(extract_hour_from_string("12:30 am"), Some(0));

    // A prefix ("tomorrow 3pm") used to make the whole parse fail.
    assert_eq!(extract_hour_from_string("tomorrow 3pm"), Some(15));
    assert_eq!(extract_hour_from_string("team 10am"), Some(10));
}

#[test]
fn hours_outside_the_clock_are_rejected() {
    assert_eq!(extract_hour_from_string("25pm"), None);
    assert_eq!(extract_hour_from_string("no time here"), None);
    assert_eq!(extract_hour_from_string(""), None);
}

#[test]
fn tomorrow_with_a_meridiem_lands_on_that_hour() {
    let parsed = parse_time_string("tomorrow 3:30pm").expect("relative day + hour");
    let expected = (Utc::now() + chrono::Duration::days(1)).date_naive();
    assert_eq!(parsed.date_naive(), expected);
    assert_eq!(parsed.hour(), 15);
    assert_eq!(parsed.minute(), 0);
}
