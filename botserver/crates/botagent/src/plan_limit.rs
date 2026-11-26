//! A spent subscription plan, as the agent CLIs report it, and when it comes back.
//!
//! Each CLI prints its own sentence when the plan's allowance is used up:
//! - Codex: "You've hit your usage limit. … or try again at 10:15 PM."
//!   (older builds: "… try again in 2 days 3 hours").
//! - Claude Code: "Claude AI usage limit reached|<epoch>", "5-hour limit
//!   reached ∙ resets 3pm", "You've hit your limit · resets 3pm".
//! - Antigravity: "You have exhausted your capacity on this model. Your quota
//!   will reset after 1h14m53s.", "You have reached your daily quota limit."
//!
//! A short-lived rate limit ("Rate limit reached … try again in 20ms",
//! "… requests per minute") is a different condition: it is not recognised
//! here and stays an ordinary, retryable rate limit.

use chrono::{DateTime, Duration, TimeZone, Timelike, Utc};

/// Prefix of the turn error when the plan's allowance is spent.
pub const MARKER: &str = "subscription plan limit:";

/// Phrases that only appear when the plan itself is spent.
const SPENT: &[&str] = &[
    "usage limit",
    "hit your limit",
    "limit reached",
    "exhausted your capacity",
    "quota will reset",
    "daily quota",
    "quota limit",
    "quota exceeded",
];

/// Phrases of a short-lived limit, which a retry can clear.
const SHORT_LIVED: &[&str] = &["rate limit reached", "per minute", "per second"];

fn is_spent(lower: &str) -> bool {
    SPENT.iter().any(|p| lower.contains(p)) && !SHORT_LIVED.iter().any(|p| lower.contains(p))
}

fn clean(line: &str) -> &str {
    let line = line.trim();
    let line = ["ERROR:", "Error:", "error:"]
        .iter()
        .find_map(|p| line.strip_prefix(p))
        .unwrap_or(line);
    line.trim_start_matches(|c: char| !c.is_alphanumeric()).trim_end()
}

/// The CLI's own line when the subscription's allowance is spent.
pub fn spent_line(stdout: &str, stderr: &str) -> Option<String> {
    stderr
        .lines()
        .chain(stdout.lines())
        .map(clean)
        .find(|l| is_spent(&l.to_ascii_lowercase()))
        .map(str::to_string)
}

/// Claude Code in print mode can exit cleanly with the limit line as its
/// whole answer; only its distinctive shapes count, never a sentence that
/// merely mentions a limit.
pub fn spent_answer(answer: &str) -> Option<String> {
    let line = clean(answer);
    if line.contains('\n') || line.len() > 300 {
        return None;
    }
    let lower = line.to_ascii_lowercase();
    let epoch_form = lower.contains("usage limit reached|");
    let reset_form =
        (lower.contains("limit reached") || lower.contains("hit your limit")) && lower.contains("resets");
    (epoch_form || reset_form).then(|| line.to_string())
}

/// What the CLI's sentence says about the return of the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetHint {
    /// An exact instant (an epoch the CLI printed, or a delay from now).
    At(DateTime<Utc>),
    /// A wall-clock time in the workspace's own time zone, which the host
    /// does not know: it can only point at a plan window read elsewhere.
    Clock { hour: u32, minute: u32 },
    /// Nothing usable.
    Unknown,
}

fn epoch_after_pipe(line: &str) -> Option<DateTime<Utc>> {
    let digits: String = line
        .split('|')
        .nth(1)?
        .trim()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    let value: i64 = digits.parse().ok()?;
    let secs = match digits.len() {
        9..=11 => value,
        12..=14 => value / 1000,
        _ => return None,
    };
    Utc.timestamp_opt(secs, 0).single()
}

fn unit_seconds(unit: &str) -> Option<i64> {
    match unit {
        "d" | "day" | "days" => Some(86_400),
        "h" | "hr" | "hrs" | "hour" | "hours" => Some(3_600),
        "m" | "min" | "mins" | "minute" | "minutes" => Some(60),
        "s" | "sec" | "secs" | "second" | "seconds" => Some(1),
        _ => None,
    }
}

/// "4 days 2 hours 3 minutes", "1h14m53s", "2 hours and 5 minutes".
fn duration(text: &str) -> Option<Duration> {
    let mut total: i64 = 0;
    let mut found = false;
    let mut rest = text.trim_start();
    loop {
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        if digits == 0 {
            break;
        }
        let number: i64 = rest[..digits].parse().ok()?;
        let after = rest[digits..].trim_start();
        let letters = after.chars().take_while(char::is_ascii_alphabetic).count();
        let Some(secs) = unit_seconds(&after[..letters]) else {
            break;
        };
        total = total.checked_add(number.checked_mul(secs)?)?;
        found = true;
        rest = after[letters..].trim_start_matches([' ', ',']);
        rest = rest.strip_prefix("and ").unwrap_or(rest);
    }
    (found && total > 0).then(|| Duration::seconds(total))
}

/// The first "10:15 pm", "3pm" or "22:15" in `text`, as a 24-hour time.
fn clock(text: &str) -> Option<(u32, u32)> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() || (i > 0 && bytes[i - 1].is_ascii_digit()) {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        let mut end = i;
        let mut minute = None;
        if bytes.get(end) == Some(&b':')
            && bytes.get(end + 1).is_some_and(u8::is_ascii_digit)
            && bytes.get(end + 2).is_some_and(u8::is_ascii_digit)
        {
            minute = match text[end + 1..end + 3].parse::<u32>() {
                Ok(value) => Some(value),
                Err(error) => {
                    tracing::debug!("plan usage time has an unreadable minute: {error}");
                    None
                }
            };
            end += 3;
        }
        let tail = text[end..].trim_start();
        let meridiem = ["am", "pm"].iter().position(|m| {
            tail.starts_with(m) && !tail[2..].starts_with(|c: char| c.is_ascii_alphabetic())
        });
        if i - start > 2 || (minute.is_none() && meridiem.is_none()) {
            continue;
        }
        let Ok(mut hour) = text[start..i].parse::<u32>() else {
            continue;
        };
        if let Some(pm) = meridiem {
            if hour == 0 || hour > 12 {
                continue;
            }
            hour = hour % 12 + if pm == 1 { 12 } else { 0 };
        }
        let minute = minute.unwrap_or(0);
        if hour < 24 && minute < 60 {
            return Some((hour, minute));
        }
    }
    None
}

/// Reads the CLI's sentence: an epoch, a delay ("in 2 hours", "after
/// 1h14m"), or a clock time ("at 10:15 PM", "resets 3pm").
pub fn reset_hint(line: &str, now: DateTime<Utc>) -> ResetHint {
    if let Some(at) = epoch_after_pipe(line) {
        return ResetHint::At(at);
    }
    let lower = line.to_ascii_lowercase();
    for key in ["again in ", "reset after ", "resets after ", "resets in ", "reset in "] {
        if let Some(delay) = lower.find(key).and_then(|at| duration(&lower[at + key.len()..])) {
            return ResetHint::At(now + delay);
        }
    }
    for key in ["again at ", "reset at ", "resets at ", "resets "] {
        if let Some((hour, minute)) = lower.find(key).and_then(|at| clock(&lower[at + key.len()..])) {
            return ResetHint::Clock { hour, minute };
        }
    }
    ResetHint::Unknown
}

/// When the spent plan comes back.
///
/// `windows` are the plan windows the CLI recorded itself (used percent and
/// reset instant; only Codex keeps them). In order:
/// 1. an exact instant the CLI printed;
/// 2. the one known window whose reset falls on the CLI's clock time read in
///    UTC (the CLI formats that same instant in the workspace clock, so the
///    match also proves which window ran out);
/// 3. the latest reset among the windows recorded at 100 %.
///
/// `None` when nothing pins it down: a clock time alone is never converted,
/// since the workspace's time zone is not known on the host.
pub fn reset_at(
    hint: ResetHint,
    windows: &[(f64, Option<DateTime<Utc>>)],
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    let ahead: Vec<(f64, DateTime<Utc>)> = windows
        .iter()
        .filter_map(|(used, reset)| reset.filter(|t| *t > now).map(|t| (*used, t)))
        .collect();
    match hint {
        ResetHint::At(at) if at > now => return Some(at),
        ResetHint::Clock { hour, minute } => {
            let mut hits = ahead
                .iter()
                .map(|(_, t)| *t)
                .filter(|t| t.hour() == hour && t.minute() == minute);
            if let Some(first) = hits.next() {
                if hits.all(|t| t == first) {
                    return Some(first);
                }
            }
        }
        _ => {}
    }
    ahead.iter().filter(|(used, _)| *used >= 100.0).map(|(_, t)| *t).max()
}

#[cfg(test)]
#[path = "plan_limit_tests.rs"]
mod tests;
