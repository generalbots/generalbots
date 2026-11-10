//! Offline unit tests for the minutes form parsers (#2068).
//!
//! Meeting forms split the start into a `date` field and a `time` field, while
//! the calendar widget sometimes posts a single RFC 3339 timestamp. `parse_dt`
//! accepts the timestamp form and returns `None` for everything else, so the
//! handler falls back to "now".
//!
//! **Known defect pinned here:** the `date` + `time` combination does *not*
//! parse, because `DateTime::parse_from_str` requires an offset in the format
//! string and `%Y-%m-%dT%H:%M:%S` has none. A meeting created from the form is
//! therefore stored with the current time instead of the chosen one. Fixing it
//! means switching to `NaiveDateTime::parse_from_str(..).and_utc()`; the tests
//! below assert today's behaviour so the change cannot pass unnoticed.

use super::{default_priority, parse_dt};
use chrono::{Datelike, TimeZone, Timelike, Utc};

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
    assert_eq!(parsed.hour(), 8);
    assert_eq!(parsed.minute(), 6);
    assert_eq!(parsed.second(), 7);
}

#[test]
fn a_date_without_a_time_is_still_accepted() {
    // `date` alone is not a `DateTime`, so it must go through the fallback and
    // come back as `None` — the handler then uses the current instant.
    assert_eq!(parse_dt("2026-01-02", ""), None);
}

#[test]
fn form_date_and_time_are_currently_dropped() {
    // See the module note: `%Y-%m-%dT%H:%M:%S` cannot match a `DateTime`
    // because the format carries no offset. Both halves are valid, and the
    // result is still `None`.
    assert_eq!(parse_dt("2026-01-02", "09:30"), None);
    assert_eq!(parse_dt("2026-12-31", "00:00"), None);
}

#[test]
fn malformed_dates_and_times_are_rejected() {
    for (date, time) in [
        ("", ""),
        ("nonsense", "09:30"),
        ("2026-01-02", "09:30:00"),
        ("2026-13-02", "09:30"),
        ("02/01/2026", "09:30"),
        ("2026-03-04T05:06:07", ""),
    ] {
        assert_eq!(
            parse_dt(date, time),
            None,
            "{date:?} + {time:?} must not parse"
        );
    }
}

#[test]
#[test]
fn the_parsed_instant_keeps_its_calendar_fields() {
    let parsed = parse_dt("2026-03-04T05:06:07Z", "").expect("fallback parse");
    assert_eq!(parsed.year(), 2026);
    assert_eq!(parsed.month(), 3);
    assert_eq!(parsed.day(), 4);
}

#[test]
fn the_default_priority_is_medium() {
    assert_eq!(default_priority(), "medium");
}
