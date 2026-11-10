//! Offline unit tests for the minutes form parsers (#2068).
//!
//! Meeting forms split the start into a `date` field and a `time` field, while
//! the calendar widget sometimes posts a single RFC 3339 timestamp. `parse_dt`
//! accepts both and returns `None` for anything else, so the handler can fall
//! back to "now" explicitly instead of storing a half-parsed date.

use super::{default_priority, parse_dt};
use chrono::{Datelike, TimeZone, Timelike, Utc};

#[test]
fn date_and_time_are_combined_as_utc() {
    let parsed = parse_dt("2026-01-02", "09:30").expect("valid form input");
    assert_eq!(parsed, Utc.with_ymd_and_hms(2026, 1, 2, 9, 30, 0).unwrap());
    assert_eq!(parsed.year(), 2026);
    assert_eq!(parsed.month(), 1);
    assert_eq!(parsed.day(), 2);
    assert_eq!(parsed.hour(), 9);
    assert_eq!(parsed.minute(), 30);
    assert_eq!(parsed.second(), 0);
}

#[test]
fn midnight_is_preserved() {
    let parsed = parse_dt("2026-12-31", "00:00").expect("valid form input");
    assert_eq!(parsed, Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap());
}

#[test]
fn a_full_rfc3339_date_is_accepted_in_the_date_field() {
    // Some clients post the whole timestamp in `date` and leave `time` empty;
    // the fallback branch parses it as an absolute instant.
    let parsed = parse_dt("2026-03-04T05:06:07Z", "").expect("fallback parse");
    assert_eq!(parsed, Utc.with_ymd_and_hms(2026, 3, 4, 5, 6, 7).unwrap());
}

#[test]
fn the_rfc3339_offset_is_normalised_to_utc() {
    let parsed = parse_dt("2026-03-04T05:06:07-03:00", "").expect("fallback parse");
    assert_eq!(parsed, Utc.with_ymd_and_hms(2026, 3, 4, 8, 6, 7).unwrap());
}

#[test]
fn malformed_dates_and_times_are_rejected() {
    for (date, time) in [
        ("", ""),
        ("nonsense", "09:30"),
        ("2026-01-02", "09:30:00"),
        ("2026-13-02", "09:30"),
        ("02/01/2026", "09:30"),
    ] {
        assert_eq!(
            parse_dt(date, time),
            None,
            "{date:?} + {time:?} must not parse"
        );
    }
}

#[test]
fn the_default_priority_is_medium() {
    assert_eq!(default_priority(), "medium");
}
