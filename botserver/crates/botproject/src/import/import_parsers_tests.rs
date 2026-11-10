//! Date and duration parsers shared by the MS Project and MPP importers.
//!
//! Both accept what the source files actually contain — `YYYY-MM-DDT00:00:00`
//! timestamps, plain dates and the two slash forms — and return `None` for
//! anything else so the caller can fall back to the project start date.
use super::*;

#[test]
fn an_ms_timestamp_keeps_its_date_and_drops_the_time() {
    assert_eq!(
        parse_ms_date("2026-09-01T00:00:00"),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 1)
    );
}

#[test]
fn a_plain_iso_date_parses() {
    assert_eq!(
        parse_ms_date("2026-09-01"),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 1)
    );
}

#[test]
fn an_unparsable_ms_date_yields_none() {
    assert_eq!(parse_ms_date(""), None);
    assert_eq!(parse_ms_date("01/09/2026"), None);
    assert_eq!(parse_ms_date("2026-13-45T00:00:00"), None);
}

#[test]
fn a_missing_duration_yields_none() {
    assert_eq!(parse_ms_duration(&None), None);
}

#[test]
fn a_duration_is_converted_to_eight_hour_days_rounded_up() {
    assert_eq!(parse_ms_duration(&Some("PT8H".to_string())), Some(1));
    assert_eq!(parse_ms_duration(&Some("PT16H".to_string())), Some(2));
    assert_eq!(parse_ms_duration(&Some("PT4H".to_string())), Some(1));
}

#[test]
fn a_non_time_duration_falls_back_to_one_day() {
    assert_eq!(parse_ms_duration(&Some("5".to_string())), Some(1));
}

#[test]
fn a_malformed_time_duration_yields_none() {
    assert_eq!(parse_ms_duration(&Some("PTXH".to_string())), None);
}

#[test]
fn work_is_read_as_hours() {
    assert_eq!(parse_ms_work(&Some("PT16H".to_string())), Some(16.0));
    assert_eq!(parse_ms_work(&Some("PT1.5H".to_string())), Some(1.5));
}

#[test]
fn work_outside_the_time_form_yields_none() {
    assert_eq!(parse_ms_work(&None), None);
    assert_eq!(parse_ms_work(&Some("5".to_string())), None);
    assert_eq!(parse_ms_work(&Some("PTXH".to_string())), None);
}

#[test]
fn the_flexible_parser_accepts_the_three_documented_forms() {
    assert_eq!(
        parse_date_flexible("2026-09-01"),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 1)
    );
    assert_eq!(
        parse_date_flexible("09/01/2026"),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 1)
    );
}

#[test]
fn a_slash_date_that_fits_both_orders_is_read_month_first() {
    // US order is tried before day-first, so "01/09/2026" is 9 January.
    assert_eq!(
        parse_date_flexible("01/09/2026"),
        chrono::NaiveDate::from_ymd_opt(2026, 1, 9)
    );
}

#[test]
fn the_flexible_parser_rejects_impossible_and_empty_dates() {
    assert_eq!(parse_date_flexible(""), None);
    assert_eq!(parse_date_flexible("13/45/2026"), None);
    assert_eq!(parse_date_flexible("not a date"), None);
}
