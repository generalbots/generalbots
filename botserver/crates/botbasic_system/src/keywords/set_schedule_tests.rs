// Tests for the natural-language → cron parser (#2065), split out of
// `set_schedule.rs` to keep that file under the AGENTS.md size limit.

use super::parse_natural_schedule;

fn cron(input: &str) -> String {
    parse_natural_schedule(input).expect("schedule should parse")
}

#[test]
fn raw_cron_expressions_pass_through_unchanged() {
    assert_eq!(cron("0 * * * *"), "0 * * * *");
    assert_eq!(cron("*/5 9-17 * * 1-5"), "*/5 9-17 * * 1-5");
    assert_eq!(cron("0 0 1 1 *"), "0 0 1 1 *");
    // Six-field cron is accepted too.
    assert_eq!(cron("0 0 * * * 2026"), "0 0 * * * 2026");
}

#[test]
fn named_intervals_map_to_cron() {
    assert_eq!(cron("every minute"), "* * * * *");
    assert_eq!(cron("every hour"), "0 * * * *");
    assert_eq!(cron("every day"), "0 0 * * *");
    assert_eq!(cron("every week"), "0 0 * * 0");
    assert_eq!(cron("every month"), "0 0 1 * *");
    assert_eq!(cron("every year"), "0 0 1 1 *");
}

#[test]
fn bare_interval_words_are_accepted() {
    assert_eq!(cron("daily"), "0 0 * * *");
    assert_eq!(cron("weekly"), "0 0 * * 0");
    assert_eq!(cron("monthly"), "0 0 1 * *");
    assert_eq!(cron("yearly"), "0 0 1 1 *");
    assert_eq!(cron("annually"), "0 0 1 1 *");
    assert_eq!(cron("hourly"), "0 * * * *");
}

#[test]
fn numbered_intervals_are_bounded() {
    assert_eq!(cron("every 5 minutes"), "*/5 * * * *");
    assert_eq!(cron("every 2 hours"), "0 */2 * * *");
    // Out-of-range intervals fall through to the error path.
    assert!(parse_natural_schedule("every 90 minutes").is_err());
    assert!(parse_natural_schedule("every 30 hours").is_err());
    assert!(parse_natural_schedule("every 0 minutes").is_err());
}

#[test]
fn times_of_day_convert_to_cron() {
    assert_eq!(cron("at 9am"), "0 9 * * *");
    assert_eq!(cron("at 9pm"), "0 21 * * *");
    assert_eq!(cron("at 12am"), "0 0 * * *");
    assert_eq!(cron("at 12pm"), "0 12 * * *");
    assert_eq!(cron("at 14:30"), "30 14 * * *");
    assert_eq!(cron("at midnight"), "0 0 * * *");
    assert_eq!(cron("at noon"), "0 12 * * *");
}

#[test]
fn invalid_times_are_rejected() {
    assert!(parse_natural_schedule("at 25:00").is_err());
    assert!(parse_natural_schedule("at 10:75").is_err());
    assert!(parse_natural_schedule("at nonsense").is_err());
}

#[test]
fn weekdays_map_to_day_of_week_fields() {
    assert_eq!(cron("every monday"), "0 0 * * 1");
    assert_eq!(cron("every friday"), "0 0 * * 5");
    assert_eq!(cron("every sunday"), "0 0 * * 0");
    assert_eq!(cron("every wed"), "0 0 * * 3");
    assert_eq!(cron("every weekday"), "0 0 * * 1-5");
    assert_eq!(cron("every weekend"), "0 0 * * 0,6");
}

#[test]
fn day_and_time_combine() {
    assert_eq!(cron("every monday at 9am"), "0 9 * * 1");
    assert_eq!(cron("every friday at 17:30"), "30 17 * * 5");
    assert_eq!(cron("every day at 8am"), "0 8 * * *");
    assert_eq!(cron("weekdays at 8am"), "0 8 * * 1-5");
    assert_eq!(cron("every weekday at 6am"), "0 6 * * 1-5");
    assert_eq!(cron("weekends at 10am"), "0 10 * * 0,6");
}

#[test]
fn hourly_ranges_are_expressed_in_the_hour_field() {
    assert_eq!(cron("every hour from 9 to 17"), "0 9-17 * * *");
}

#[test]
fn business_hours_defaults_to_weekday_office_time() {
    assert_eq!(cron("business hours"), "0 9-17 * * 1-5");
    assert_eq!(cron("every business hours"), "0 9-17 * * 1-5");
    assert_eq!(cron("every 15 minutes business hours"), "*/15 9-17 * * 1-5");
}

#[test]
fn unknown_schedules_fail_with_guidance() {
    let error = parse_natural_schedule("whenever the mood strikes").unwrap_err();
    assert!(error.contains("Could not parse schedule"));
    assert!(error.contains("every hour"));
}

#[test]
fn input_is_case_and_whitespace_insensitive() {
    assert_eq!(cron("  EVERY  MONDAY  AT  9AM  "), "0 9 * * 1");
    assert_eq!(cron("Daily"), "0 0 * * *");
}
