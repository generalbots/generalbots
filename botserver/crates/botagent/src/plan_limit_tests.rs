use chrono::{DateTime, Duration, TimeZone, Utc};

use super::{reset_at, reset_hint, spent_answer, spent_line, ResetHint};

/// The owner's Codex turn, word for word.
const CODEX: &str = "ERROR: You've hit your usage limit. Upgrade to Pro (https://chatgpt.com/explore/pro), visit https://chatgpt.com/settings/usage to purchase more credits or try again at 10:15 PM.";

fn at(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text)
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_default()
}

fn now() -> DateTime<Utc> {
    at("2026-11-22T15:00:00Z")
}

#[test]
fn the_codex_usage_limit_is_a_spent_plan() {
    let line = spent_line("OpenAI Codex v0.170.0\n--------", CODEX);
    let Some(line) = line else { panic!("not recognised") };
    assert!(line.starts_with("You've hit your usage limit."));
    assert!(line.ends_with("try again at 10:15 PM."));
}

#[test]
fn the_codex_clock_points_at_the_window_that_ran_out() {
    let hint = reset_hint(CODEX, now());
    assert_eq!(hint, ResetHint::Clock { hour: 22, minute: 15 });
    let windows = [
        (97.0, Some(at("2026-11-22T22:15:11Z"))),
        (68.0, Some(at("2026-11-26T20:54:03Z"))),
    ];
    assert_eq!(reset_at(hint, &windows, now()), Some(at("2026-11-22T22:15:11Z")));
}

#[test]
fn a_clock_time_alone_is_never_converted() {
    let hint = reset_hint(CODEX, now());
    assert_eq!(reset_at(hint, &[], now()), None);
    let unrelated = [(40.0, Some(at("2026-11-22T19:15:00Z")))];
    assert_eq!(reset_at(hint, &unrelated, now()), None);
}

#[test]
fn without_a_time_the_window_at_full_use_decides() {
    let old = "You've hit your usage limit. Try again at Nov 22nd.";
    assert!(spent_line(old, "").is_some());
    let hint = reset_hint(old, now());
    assert_eq!(hint, ResetHint::Unknown);
    let windows = [
        (100.0, Some(at("2026-11-22T22:15:11Z"))),
        (60.0, Some(at("2026-11-26T20:54:03Z"))),
        (100.0, Some(at("2026-11-22T10:00:00Z"))),
    ];
    assert_eq!(reset_at(hint, &windows, now()), Some(at("2026-11-22T22:15:11Z")));
}

#[test]
fn claude_code_limits_are_recognised() {
    let epoch = "Claude AI usage limit reached|1795400000";
    assert_eq!(spent_line(epoch, "").as_deref(), Some(epoch));
    let Some(exact) = Utc.timestamp_opt(1_795_400_000, 0).single() else { panic!("epoch") };
    assert_eq!(reset_hint(epoch, now()), ResetHint::At(exact));
    assert_eq!(reset_at(reset_hint(epoch, now()), &[], now()), Some(exact));

    let window = "5-hour limit reached \u{2219} resets 3pm";
    assert!(spent_line("", window).is_some());
    assert_eq!(reset_hint(window, now()), ResetHint::Clock { hour: 15, minute: 0 });
    assert_eq!(reset_at(reset_hint(window, now()), &[], now()), None);
    assert!(spent_line("You've hit your limit \u{b7} resets 10pm (UTC)", "").is_some());
}

#[test]
fn claude_code_answer_that_is_only_the_limit_line() {
    assert!(spent_answer("Claude AI usage limit reached|1795400000\n").is_some());
    assert!(spent_answer("Weekly limit reached \u{2219} resets Nov 25, 10am").is_some());
    assert!(spent_answer("A usage limit reached its peak in the report.").is_none());
    assert!(spent_answer("Here is the plan.\nThe 5-hour limit reached zero, it resets daily.").is_none());
}

#[test]
fn antigravity_quota_with_a_delay() {
    let line = "You have exhausted your capacity on this model. Your quota will reset after 1h14m53s.";
    assert!(spent_line("", line).is_some());
    let expected = now() + Duration::seconds(3_600 + 14 * 60 + 53);
    assert_eq!(reset_hint(line, now()), ResetHint::At(expected));
    assert!(spent_line("You have reached your daily quota limit.", "").is_some());
}

#[test]
fn a_spelled_out_delay_is_read() {
    let line = "You've hit your usage limit. Upgrade to Plus, or try again in 4 days 2 hours 3 minutes.";
    let expected = now() + Duration::seconds(4 * 86_400 + 2 * 3_600 + 3 * 60);
    assert_eq!(reset_hint(line, now()), ResetHint::At(expected));
}

#[test]
fn short_lived_limits_are_not_a_spent_plan() {
    let openai = "Rate limit reached for gpt-5 in organization org-x on tokens per min. Please try again in 20ms.";
    assert!(spent_line(openai, "").is_none());
    let google = "Quota exceeded for quota metric 'Generate Content API requests per minute'";
    assert!(spent_line("", google).is_none());
    assert!(spent_line("all fine", "").is_none());
}
