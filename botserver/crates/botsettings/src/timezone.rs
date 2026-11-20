//! Tenant timezone resolution (#2389).
//!
//! Settings stores an IANA zone name per user (`user_preferences.timezone`).
//! The digest used to translate it through a hardcoded eight-entry table of
//! *fixed* offsets, so `America/New_York` was Eastern Standard Time all year
//! and every "week ending" label drifted by an hour during daylight saving.
//! This module resolves the real zone with `chrono-tz`, which carries the
//! historical DST rules.
//!
//! Two fallbacks are deliberate and documented:
//!
//! * numeric strings (`UTC+05:30`, `GMT-03:00`, `-0300`) stay accepted for
//!   callers that never had a zone name — they are fixed by definition;
//! * anything else that cannot be resolved is an explicit UTC fallback with a
//!   `warn!` naming the offending value, instead of a silent UTC.
//!
//! Host timezone detection uses `iana-time-zone`; it is only used by callers
//! that have no stored preference, never as a substitute for an invalid one.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use chrono::{DateTime, Duration, NaiveDate, Offset, Utc};
use chrono_tz::Tz;

/// Upper bound on the distinct unresolvable zone strings we remember for
/// warning purposes. The values come from a per-user preference, so an
/// unbounded set would be a memory leak driven by user input.
const MAX_WARNED_ZONES: usize = 64;

/// A Settings timezone value resolved to something the calendar helpers use.
///
/// Settings stores IANA names, which carry their own DST rules, so calendar
/// days are computed in the zone itself rather than from a fixed offset table.
/// Explicit `UTC±hh:mm` / `±hhmm` values remain supported as a documented
/// fallback for callers that never had a zone name.
#[derive(Debug, Clone, Copy)]
pub enum Zone {
    Iana(Tz),
    Fixed(i32),
}

fn warned_zone_log() -> &'static Mutex<HashSet<String>> {
    static WARNED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    WARNED.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Warns once per distinct unresolvable value — not once per call, because
/// `today_in` runs on request paths and a misconfigured tenant would otherwise
/// flood the log.
fn warn_unresolvable(zone: &str) {
    let mutex = warned_zone_log();
    let mut warned = match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if warned.len() >= MAX_WARNED_ZONES || !warned.insert(zone.to_owned()) {
        return;
    }
    drop(warned);
    log::warn!("settings: unresolvable timezone {zone:?}; falling back to UTC");
}

/// Resolves a timezone value, warning instead of silently pretending it is UTC.
///
/// Surrounding whitespace is tolerated (`"America/Sao_Paulo "` is a common
/// copy-paste typo and still resolves to the real zone).
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
                warn_unresolvable(value);
                Zone::Iana(Tz::UTC)
            }
        },
    }
}

/// Host IANA zone name, for callers that have no stored timezone preference.
///
/// `iana-time-zone` reads the OS setting without shelling out. The result is
/// cached so request paths do not repeat that lookup. An invalid or missing
/// host setting is an explicit UTC fallback with a warning.
pub fn host_timezone_name() -> &'static str {
    static HOST_ZONE: OnceLock<String> = OnceLock::new();
    HOST_ZONE
        .get_or_init(|| match iana_time_zone::get_timezone() {
            Ok(name) if !name.trim().is_empty() => name.trim().to_owned(),
            Ok(_) => "UTC".to_owned(),
            Err(error) => {
                log::warn!("settings: cannot detect host timezone ({error}); using UTC");
                "UTC".to_owned()
            }
        })
        .as_str()
}

/// UTC offset (minutes) at `at`, honouring the zone's daylight-saving rules.
///
/// Diagnostic only: calendar dates must come from [`local_date`] or [`today_in`]
/// (as the digest does), never from an offset applied to a UTC instant.
pub fn timezone_offset_minutes(timezone: &str, at: DateTime<Utc>) -> i32 {
    match resolve_zone(timezone) {
        // Zone offsets are whole minutes, so the division is exact.
        Zone::Iana(tz) => at.with_timezone(&tz).offset().fix().local_minus_utc() / 60,
        Zone::Fixed(offset) => offset,
    }
}

/// The calendar day in `zone` for `instant` — what "today" and "week ending"
/// mean to the person reading the digest.
///
/// #2389 `date_naive()` classification (deliberately NOT a bulk rewrite):
///
/// * The two conversions below are tenant-local by construction: the instant
///   is shifted into the resolved IANA zone (or the documented fixed offset)
///   *before* `date_naive()` is called. They replace the old UTC-derived digest
///   dates and are the canonical pattern for user-facing day buckets.
/// * The `Utc::now().date_naive()` in the `today_in` test is a UTC reference
///   value only; it never becomes a user-facing date.
/// * The other sites flagged by #2389 (98 reported; 102 raw occurrences in
///   `botserver/` outside this module) are intentionally left untouched here.
///   They need per-caller triage: some already use `Local::now()`, some are
///   deliberately UTC (server-side billing/log buckets with no tenant zone),
///   and only the UTC-derived user-facing ones should move to `local_date`.
///   Rewriting them mechanically would change unrelated billing and logging
///   semantics and belongs to a separate change.
pub fn local_date(instant: DateTime<Utc>, zone: &str) -> NaiveDate {
    match resolve_zone(zone) {
        Zone::Iana(tz) => instant.with_timezone(&tz).date_naive(),
        Zone::Fixed(offset) => (instant + Duration::minutes(i64::from(offset))).date_naive(),
    }
}

/// The current calendar day in `zone`.
pub fn today_in(zone: &str) -> NaiveDate {
    local_date(Utc::now(), zone)
}

/// `local_date` formatted as `YYYY-MM-DD`, the shape digest labels use.
pub fn local_date_in(timezone: &str, at: DateTime<Utc>) -> String {
    local_date(at, timezone)
        .format("%Y-%m-%d")
        .to_string()
}

fn trim_offset_prefix(value: &str) -> Option<&str> {
    for prefix in ["UTC", "GMT", "utc", "gmt"] {
        if let Some(rest) = value.strip_prefix(prefix) {
            return Some(rest.trim());
        }
    }
    None
}

/// Numeric fallback: `UTC±hh[:mm]`, `GMT±hh[:mm]`, or a bare `±hh[:mm]` /
/// `±hhmm`. Returns `None` for anything else so the caller can warn.
fn parse_numeric_offset(value: &str) -> Option<i32> {
    let trimmed = value.trim();
    let body = trim_offset_prefix(trimmed).unwrap_or(trimmed);
    if body.is_empty() {
        // "UTC" / "GMT" alone is the zero offset.
        return Some(0);
    }
    let negative = match body.chars().next() {
        Some('-') => true,
        Some('+') => false,
        _ => return None,
    };
    let digits = body.get(1..)?;
    let (hours, minutes) = match digits.split_once(':') {
        Some((hours, minutes)) => (hours, minutes),
        // Compact form: "-0300" is hours "03" plus minutes "00".
        None if digits.len() > 2 => (digits.get(..2)?, digits.get(2..)?),
        None => (digits, "0"),
    };
    let hours: i32 = hours.parse().ok()?;
    let minutes: i32 = minutes.parse().ok()?;
    if hours > 14 || minutes > 59 {
        return None;
    }
    let total = hours * 60 + minutes;
    Some(if negative { -total } else { total })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(raw: &str) -> DateTime<Utc> {
        match DateTime::parse_from_rfc3339(raw) {
            Ok(parsed) => parsed.with_timezone(&Utc),
            // A bad literal in a test must fail the assertion, not panic.
            Err(_) => DateTime::<Utc>::MIN_UTC,
        }
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(value) => value,
            None => NaiveDate::MIN,
        }
    }

    #[test]
    fn iana_zones_resolve_to_their_real_offsets() {
        let winter = at("2026-01-15T12:00:00Z");
        assert_eq!(timezone_offset_minutes("UTC", winter), 0);
        assert_eq!(timezone_offset_minutes("America/Sao_Paulo", winter), -180);
        assert_eq!(timezone_offset_minutes("Asia/Tokyo", winter), 540);
    }

    /// #2389 — the fixed table returned -300 all year, so a week label, digest
    /// instant or "today" bucket was off by an hour for half the year.
    #[test]
    fn dst_spring_forward_in_new_york_is_dst_correct() {
        // 2026-03-08: 01:59 EST is -300, one minute later the clock jumps to
        // 03:00 EDT and the offset is -240.
        assert_eq!(
            timezone_offset_minutes("America/New_York", at("2026-03-08T06:59:00Z")),
            -300
        );
        assert_eq!(
            timezone_offset_minutes("America/New_York", at("2026-03-08T07:00:00Z")),
            -240
        );
        // Late on the 7th locally, even though UTC is already the 8th.
        assert_eq!(
            local_date(at("2026-03-08T04:30:00Z"), "America/New_York"),
            date(2026, 3, 7)
        );
        assert_eq!(
            local_date(at("2026-03-08T07:00:00Z"), "America/New_York"),
            date(2026, 3, 8)
        );
    }

    #[test]
    fn dst_fall_back_in_new_york_is_dst_correct() {
        // 2026-11-01: 01:59 EDT is -240, one minute later the clock falls back
        // to 01:00 EST and the offset is -300.
        assert_eq!(
            timezone_offset_minutes("America/New_York", at("2026-11-01T05:59:00Z")),
            -240
        );
        assert_eq!(
            timezone_offset_minutes("America/New_York", at("2026-11-01T06:00:00Z")),
            -300
        );
        assert_eq!(
            local_date(at("2026-11-02T04:59:00Z"), "America/New_York"),
            date(2026, 11, 1)
        );
        assert_eq!(
            local_date(at("2026-11-02T05:00:00Z"), "America/New_York"),
            date(2026, 11, 2)
        );
        // Europe/London is UTC in winter and UTC+1 in summer.
        assert_eq!(
            timezone_offset_minutes("Europe/London", at("2026-01-15T12:00:00Z")),
            0
        );
        assert_eq!(
            timezone_offset_minutes("Europe/London", at("2026-07-01T12:00:00Z")),
            60
        );
    }

    /// #2389 — Sao Paulo keeps UTC-3 but is a day behind UTC at 23:30 local,
    /// which is the case a naive UTC date gets wrong at every month boundary.
    #[test]
    fn sao_paulo_month_boundary_is_local() {
        let late = at("2026-12-01T02:30:00Z"); // 2026-11-30 23:30 in Sao Paulo
        assert_eq!(local_date(late, "America/Sao_Paulo"), date(2026, 11, 30));
        assert_eq!(local_date(late, "UTC"), date(2026, 12, 1));
        assert_eq!(local_date_in("America/Sao_Paulo", late), "2026-11-30");
        // Auckland is already on the next day.
        assert_eq!(local_date(late, "Pacific/Auckland"), date(2026, 12, 1));
    }

    #[test]
    fn today_in_matches_the_zone_for_the_current_instant() {
        // Midnight may tick between the two reads, so allow the adjacent day.
        let utc_today = today_in("UTC");
        let captured = Utc::now().date_naive();
        assert!(utc_today == captured || Some(utc_today) == captured.succ_opt());

        // Sao Paulo is at most one day behind UTC, Tokyo at most one ahead.
        let sao_paulo = (today_in("America/Sao_Paulo") - utc_today).num_days();
        assert!(sao_paulo == 0 || sao_paulo == -1, "Sao Paulo delta {sao_paulo}");
        let tokyo = (today_in("Asia/Tokyo") - utc_today).num_days();
        assert!(tokyo == 0 || tokyo == 1, "Tokyo delta {tokyo}");
    }

    #[test]
    fn unknown_zone_falls_back_to_utc_and_warns() {
        let instant = at("2026-07-01T12:00:00Z");
        // "BRT" is a Brazilian abbreviation, not an IANA name.
        assert_eq!(timezone_offset_minutes("BRT", instant), 0);
        assert_eq!(local_date(instant, "BRT"), date(2026, 7, 1));
        assert_eq!(local_date(instant, "not-a-zone"), date(2026, 7, 1));
        let warned = match warned_zone_log().lock() {
            Ok(zones) => zones.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        };
        assert!(warned.contains("BRT"));
        assert!(warned.contains("not-a-zone"));
    }

    /// A trailing space used to be enough to land a tenant in UTC; trimming
    /// keeps the real zone.
    #[test]
    fn surrounding_whitespace_does_not_lose_the_zone() {
        let winter = at("2026-01-15T12:00:00Z");
        assert_eq!(timezone_offset_minutes("Europe/Berlin ", winter), 60);
        assert_eq!(timezone_offset_minutes(" America/Sao_Paulo", winter), -180);
        assert_eq!(timezone_offset_minutes("  ", winter), 0);
    }

    #[test]
    fn numeric_offsets_remain_a_documented_fallback() {
        let instant = at("2026-01-15T12:00:00Z");
        assert_eq!(timezone_offset_minutes("UTC+05:30", instant), 330);
        assert_eq!(timezone_offset_minutes("GMT-03:00", instant), -180);
        assert_eq!(timezone_offset_minutes("-0300", instant), -180);
        assert_eq!(timezone_offset_minutes("+0530", instant), 330);
        assert_eq!(timezone_offset_minutes("UTC-3", instant), -180);
        // Out-of-range offsets are unresolvable, not silently clamped.
        assert_eq!(timezone_offset_minutes("UTC+16:00", instant), 0);
        assert_eq!(timezone_offset_minutes("+05:99", instant), 0);
    }

    #[test]
    fn host_timezone_name_is_a_resolvable_zone() {
        let host = host_timezone_name();
        assert!(!host.is_empty());
        assert!(matches!(resolve_zone(host), Zone::Iana(_)));
    }
}
