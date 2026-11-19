//! Weekly notification digest helpers (#1813).
//!
//! The digest used to be a stored-but-unused preference: the Settings checkbox
//! wrote `email_digest` and nothing ever read it. The job that sends the mail
//! lives in `botserver` (it needs the platform notification sender), while the
//! pure scheduling/formatting rules live here so both the job and the Settings
//! API share one definition and can be unit-tested without a database or SMTP.

use chrono::{
    DateTime, Datelike, Duration, LocalResult, NaiveDateTime, Offset, TimeZone, Utc, Weekday,
};
use chrono_tz::Tz;
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
    match resolve_zone(timezone) {
        Zone::Iana(tz) => {
            let local_sent = sent.with_timezone(&tz).naive_local();
            let next_local = next_monday_local(local_sent)?;
            // #2389 — the wall-clock Monday 09:00 is resolved by the zone, so a
            // digest due across a DST transition lands on the right instant.
            match tz.from_local_datetime(&next_local) {
                LocalResult::Single(dt) => Some(dt.with_timezone(&Utc)),
                LocalResult::Ambiguous(earliest, _) => Some(earliest.with_timezone(&Utc)),
                LocalResult::None => None,
            }
        }
        Zone::Fixed(offset) => {
            let minutes = i64::from(offset);
            let local = sent + Duration::minutes(minutes);
            let next_local = next_monday_local(local.naive_utc())?;
            Some(next_local.and_utc() - Duration::minutes(minutes))
        }
    }
}

/// A Settings timezone value resolved to something the digest can use.
///
/// `#2389` — Settings stores IANA names, which carry their own DST rules, so the
/// calendar day and the Monday 09:00 instant are computed in the zone itself
/// rather than from a fixed offset table. Explicit `UTC±hh:mm` values remain
/// supported as a documented fallback for callers that never had a zone name.
#[derive(Debug, Clone, Copy)]
pub enum Zone {
    Iana(Tz),
    Fixed(i32),
}

/// Resolves a timezone value, warning instead of silently pretending it is UTC.
pub fn resolve_zone(timezone: &str) -> Zone {
    let value = timezone.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("utc") || value.eq_ignore_ascii_case("gmt") {
        return Zone::Iana(Tz::UTC);
    }
    match value.parse::<Tz>() {
        Ok(tz) => Zone::Iana(tz),
        Err(_) => match parse_numeric_offset(value) {
            Some(offset) => Zone::Fixed(offset),
            None => {
                log::warn!(
                    "settings: unresolvable timezone {value:?}; falling back to UTC"
                );
                Zone::Iana(Tz::UTC)
            }
        },
    }
}

/// UTC offset (minutes) at `at`, honouring the zone's daylight-saving rules.
pub fn timezone_offset_minutes(timezone: &str, at: DateTime<Utc>) -> i32 {
    match resolve_zone(timezone) {
        // Zone offsets are whole minutes, so the division is exact.
        Zone::Iana(tz) => at.with_timezone(&tz).offset().fix().local_minus_utc() / 60,
        Zone::Fixed(offset) => offset,
    }
}

/// The calendar day in the user's zone, which is what a "week ending" label
/// means to the person receiving it (`#2389`).
pub fn local_date_in(timezone: &str, at: DateTime<Utc>) -> String {
    match resolve_zone(timezone) {
        Zone::Iana(tz) => at.with_timezone(&tz).format("%Y-%m-%d").to_string(),
        Zone::Fixed(offset) => (at + Duration::minutes(i64::from(offset)))
            .format("%Y-%m-%d")
            .to_string(),
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

/// Plain-text body sent through the platform notification sender.
pub fn build_digest_body(
    username: &str,
    since: DateTime<Utc>,
    now: DateTime<Utc>,
    timezone: &str,
    summary: DigestSummary,
) -> String {
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
        day = local_date_in(timezone, now),
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
        let winter = at("2026-01-15T12:00:00Z");
        assert_eq!(timezone_offset_minutes("UTC", winter), 0);
        assert_eq!(timezone_offset_minutes("America/Sao_Paulo", winter), -180);
        assert_eq!(timezone_offset_minutes("Asia/Tokyo", winter), 540);
        assert_eq!(timezone_offset_minutes("UTC+05:30", winter), 330);
        assert_eq!(timezone_offset_minutes("GMT-03:00", winter), -180);
        // #2389 — an unknown zone is an explicit UTC fallback, never a typo
        // that quietly shifts every date by hours.
        assert_eq!(timezone_offset_minutes("not-a-zone", winter), 0);
    }

    /// #2389 — the fixed table returned -300 all year, so a week label,
    /// digest instant or "today" bucket was off by an hour for half the year.
    #[test]
    fn daylight_saving_transitions_are_honoured() {
        // US DST starts 2026-03-08 and ends 2026-11-01.
        let before_dst = at("2026-03-01T12:00:00Z");
        let during_dst = at("2026-07-01T12:00:00Z");
        let after_dst = at("2026-11-10T12:00:00Z");
        assert_eq!(timezone_offset_minutes("America/New_York", before_dst), -300);
        assert_eq!(timezone_offset_minutes("America/New_York", during_dst), -240);
        assert_eq!(timezone_offset_minutes("America/New_York", after_dst), -300);
        // Europe/London is UTC in winter and UTC+1 in summer.
        assert_eq!(timezone_offset_minutes("Europe/London", before_dst), 0);
        assert_eq!(timezone_offset_minutes("Europe/London", during_dst), 60);
    }

    #[test]
    fn next_run_across_a_dst_transition_keeps_local_monday_nine() {
        // Sent on Thursday 2026-10-29 14:00 UTC (EDT, UTC-4). The next Monday
        // 09:00 local is 2026-11-02, which is EST (UTC-5) after the change.
        let status = json!({"last_sent_at": "2026-10-29T14:00:00Z"});
        assert_eq!(
            next_run_at(&status, "America/New_York"),
            Some(at("2026-11-02T14:00:00Z"))
        );
    }

    /// #2389 — Sao Paulo keeps UTC-3 but is a day ahead of UTC at 23:30 local,
    /// which is the case a naive UTC date gets wrong at every month boundary.
    #[test]
    fn local_date_follows_the_zone_at_month_boundaries() {
        let late = at("2026-12-01T02:30:00Z"); // 2026-11-30 23:30 in Sao Paulo
        assert_eq!(local_date_in("America/Sao_Paulo", late), "2026-11-30");
        assert_eq!(local_date_in("UTC", late), "2026-12-01");
        // Auckland is already on the next day.
        assert_eq!(local_date_in("Pacific/Auckland", late), "2026-12-01");
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
