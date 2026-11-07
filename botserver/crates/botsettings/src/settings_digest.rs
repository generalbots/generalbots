//! Weekly notification digest helpers (#1813).
//!
//! The digest used to be a stored-but-unused preference: the Settings checkbox
//! wrote `email_digest` and nothing ever read it. The job that sends the mail
//! lives in `botserver` (it needs the platform notification sender), while the
//! pure scheduling/formatting rules live here so both the job and the Settings
//! API share one definition and can be unit-tested without a database or SMTP.

use chrono::{DateTime, Datelike, Duration, NaiveDateTime, Utc, Weekday};
use serde_json::Value;

/// `user_preferences` key that stores the last digest run.
pub const DIGEST_STATUS_KEY: &str = "digest_status";

/// One digest per week.
pub const DIGEST_INTERVAL_DAYS: i64 = 7;

/// Failed deliveries are retried at most this often. Without the guard a
/// missing mail sender would retry every scheduler tick for every opted-in
/// user and flood the logs.
pub const DIGEST_RETRY_HOURS: i64 = 6;

/// Counts shown in the digest body.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DigestSummary {
    pub messages: i64,
    pub conversations: i64,
    pub drive_files: i64,
    pub active_tasks: i64,
    pub task_runs: i64,
    pub mentions: i64,
}

fn timestamp(value: Option<&Value>) -> Option<DateTime<Utc>> {
    value
        .and_then(Value::as_str)
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|parsed| parsed.with_timezone(&Utc))
}

/// The last successful delivery recorded for a user.
pub fn last_sent_at(status: &Value) -> Option<DateTime<Utc>> {
    timestamp(status.get("last_sent_at"))
}

fn last_attempt_at(status: &Value) -> Option<DateTime<Utc>> {
    timestamp(status.get("last_attempt_at"))
}

/// Whether a digest is due at `now`.
///
/// A user with no prior delivery gets one on the next scheduler pass. After a
/// successful send the next one is due on the following Monday at 09:00 in the
/// user's timezone. Failed attempts are retried after [`DIGEST_RETRY_HOURS`],
/// independently of the weekly cadence.
pub fn digest_due(status: &Value, now: DateTime<Utc>, timezone: &str) -> bool {
    if let Some(attempt) = last_attempt_at(status) {
        if attempt + Duration::hours(DIGEST_RETRY_HOURS) > now {
            return false;
        }
    }
    match next_run_at(status, timezone) {
        Some(next) => now >= next,
        None => true,
    }
}

fn next_monday_local(from: NaiveDateTime) -> Option<NaiveDateTime> {
    let mut date = from.date() + Duration::days(1);
    while date.weekday() != Weekday::Mon {
        date += Duration::days(1);
    }
    date.and_hms_opt(9, 0, 0)
}

/// Next scheduled delivery: the Monday 09:00 after the last send, in the
/// user's timezone. `None` means the first digest is sent on the next
/// scheduler pass.
pub fn next_run_at(status: &Value, timezone: &str) -> Option<DateTime<Utc>> {
    let sent = last_sent_at(status)?;
    let offset = i64::from(timezone_offset_minutes(timezone));
    let local = sent + Duration::minutes(offset);
    let next_local = next_monday_local(local.naive_utc())?;
    Some(next_local.and_utc() - Duration::minutes(offset))
}

/// UTC offset (minutes) for the timezone values offered by Settings.
///
/// The profile/language pickers store IANA names; the digest only needs the
/// local calendar day for the "week ending" label. Fixed standard offsets keep
/// the job dependency-free; daylight-saving shifts move the label by an hour at
/// most, never the weekly cadence.
pub fn timezone_offset_minutes(timezone: &str) -> i32 {
    let timezone = timezone.trim();
    if timezone.is_empty()
        || timezone.eq_ignore_ascii_case("utc")
        || timezone.eq_ignore_ascii_case("gmt")
    {
        return 0;
    }
    match timezone {
        "America/Sao_Paulo" => -180,
        "America/New_York" => -300,
        "America/Chicago" => -360,
        "America/Denver" => -420,
        "America/Los_Angeles" => -480,
        "Europe/London" => 0,
        "Europe/Paris" => 60,
        "Asia/Tokyo" => 540,
        _ => parse_numeric_offset(timezone).unwrap_or(0),
    }
}

fn parse_numeric_offset(value: &str) -> Option<i32> {
    let rest = value
        .strip_prefix("UTC")
        .or_else(|| value.strip_prefix("GMT"))
        .or_else(|| value.strip_prefix("utc"))
        .or_else(|| value.strip_prefix("gmt"))?;
    let rest = rest.trim();
    if rest.is_empty() {
        return Some(0);
    }
    let sign = if rest.starts_with('-') { -1 } else { 1 };
    let rest = rest.trim_start_matches(['+', '-']);
    let (hours, minutes) = match rest.split_once(':') {
        Some((hours, minutes)) => (hours, minutes),
        None => (rest, "0"),
    };
    let hours: i32 = hours.parse().ok()?;
    let minutes: i32 = minutes.parse().ok()?;
    if hours > 14 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 60 + minutes))
}

fn local_day(now: DateTime<Utc>, offset_minutes: i32) -> String {
    (now + Duration::minutes(i64::from(offset_minutes)))
        .format("%Y-%m-%d")
        .to_string()
}

/// Plain-text body sent through the platform notification sender.
pub fn build_digest_body(
    username: &str,
    since: DateTime<Utc>,
    now: DateTime<Utc>,
    timezone: &str,
    summary: DigestSummary,
) -> String {
    let offset = timezone_offset_minutes(timezone);
    let name = if username.trim().is_empty() {
        "there"
    } else {
        username.trim()
    };
    format!(
        "Hi {name},\n\n\
         Here is your General Bots weekly digest for the week ending {day} ({timezone}).\n\n\
         Activity since {since}:\n\
         - Messages: {messages}\n\
         - Conversations: {conversations}\n\
         - Drive files updated: {drive_files}\n\
         - Scheduled tasks active: {active_tasks}\n\
         - Scheduled task runs: {task_runs}\n\
         - Mentions received: {mentions}\n\n\
         Open General Bots to pick up where you left off.\n",
        day = local_day(now, offset),
        since = since.format("%Y-%m-%d %H:%M UTC"),
        timezone = timezone.trim(),
        messages = summary.messages,
        conversations = summary.conversations,
        drive_files = summary.drive_files,
        active_tasks = summary.active_tasks,
        task_runs = summary.task_runs,
        mentions = summary.mentions,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .map(|parsed| parsed.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    }

    #[test]
    fn first_digest_is_due_without_status() {
        assert!(digest_due(&json!({}), at("2026-11-07T12:00:00Z"), "UTC"));
    }

    #[test]
    fn digest_runs_on_monday_morning_in_the_users_timezone() {
        let status = json!({
            "last_sent_at": "2026-11-02T09:00:00Z",
            "last_attempt_at": "2026-11-02T09:00:00Z"
        });
        assert!(!digest_due(&status, at("2026-11-07T12:00:00Z"), "UTC"));
        assert!(digest_due(&status, at("2026-11-09T09:00:00Z"), "UTC"));
        assert_eq!(
            next_run_at(&status, "UTC"),
            Some(at("2026-11-09T09:00:00Z"))
        );
        // Tokyo's Monday 09:00 is Monday 00:00 UTC.
        assert_eq!(
            next_run_at(&status, "Asia/Tokyo"),
            Some(at("2026-11-09T00:00:00Z"))
        );
    }

    #[test]
    fn failed_attempt_is_retried_after_six_hours() {
        let status = json!({
            "last_error": "SMTP unavailable",
            "last_attempt_at": "2026-11-07T09:00:00Z"
        });
        assert!(!digest_due(&status, at("2026-11-07T12:00:00Z"), "UTC"));
        assert!(digest_due(&status, at("2026-11-07T15:01:00Z"), "UTC"));
    }

    #[test]
    fn timezone_values_map_to_expected_offsets() {
        assert_eq!(timezone_offset_minutes("UTC"), 0);
        assert_eq!(timezone_offset_minutes("America/Sao_Paulo"), -180);
        assert_eq!(timezone_offset_minutes("Asia/Tokyo"), 540);
        assert_eq!(timezone_offset_minutes("UTC+05:30"), 330);
        assert_eq!(timezone_offset_minutes("GMT-03:00"), -180);
        assert_eq!(timezone_offset_minutes("not-a-zone"), 0);
    }

    #[test]
    fn body_contains_counts_and_local_week_label() {
        let summary = DigestSummary {
            messages: 12,
            conversations: 3,
            drive_files: 4,
            active_tasks: 2,
            task_runs: 7,
            mentions: 1,
        };
        let body = build_digest_body(
            "maria",
            at("2026-11-01T00:00:00Z"),
            at("2026-11-07T23:30:00Z"),
            "Asia/Tokyo",
            summary,
        );
        assert!(body.contains("Hi maria"));
        assert!(body.contains("week ending 2026-11-08"));
        assert!(body.contains("Messages: 12"));
        assert!(body.contains("Conversations: 3"));
        assert!(body.contains("Drive files updated: 4"));
        assert!(body.contains("Scheduled task runs: 7"));
        assert!(body.contains("Mentions received: 1"));
    }
}
