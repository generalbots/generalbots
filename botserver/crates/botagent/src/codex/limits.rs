//! Plan usage of the Codex subscription, read from the CLI's own session logs.
//!
//! Every Codex turn appends `token_count` events to
//! `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`; their `rate_limits` field
//! carries the plan windows the service returned (used percentage, window
//! length, reset time). The newest event is the latest state the CLI saw.
//! Reading is non-interactive and changes nothing in the workspace.

use axum::http::StatusCode;
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::AgentService;

use super::paths::CODEX_HOME;
use super::status::run_script;

/// One plan window (`primary` is the 5-hour window, `secondary` the weekly).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CodexWindow {
    pub name: String,
    pub used_percent: f64,
    pub window_minutes: Option<u64>,
    pub resets_at: Option<DateTime<Utc>>,
}

/// The plan state of the newest `rate_limits` event.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CodexLimits {
    pub observed_at: Option<DateTime<Utc>>,
    pub plan_type: Option<String>,
    pub windows: Vec<CodexWindow>,
}

pub(super) fn window_name(minutes: Option<u64>, slot: &str) -> String {
    match minutes {
        Some(300) => "5h".to_string(),
        Some(1440) => "daily".to_string(),
        Some(10080) => "weekly".to_string(),
        Some(m) if m % 60 == 0 => format!("{}h", m / 60),
        Some(m) => format!("{m}m"),
        None => slot.to_string(),
    }
}

pub(super) fn epoch(value: &Value) -> Option<DateTime<Utc>> {
    let secs = value.as_i64().or_else(|| value.as_f64().map(|v| v as i64))?;
    Utc.timestamp_opt(secs, 0).single()
}

fn window(raw: &Value, slot: &str, observed_at: Option<DateTime<Utc>>) -> Option<CodexWindow> {
    let used = raw.get("used_percent")?.as_f64()?;
    let minutes = raw.get("window_minutes").and_then(Value::as_u64);
    let resets_at = raw.get("resets_at").and_then(epoch).or_else(|| {
        let secs = raw.get("resets_in_seconds").and_then(Value::as_i64)?;
        observed_at.map(|at| at + Duration::seconds(secs))
    });
    Some(CodexWindow {
        name: window_name(minutes, slot),
        used_percent: used.clamp(0.0, 100.0),
        window_minutes: minutes,
        resets_at,
    })
}

/// Parses one session-log line; `None` unless it is an event with a
/// non-empty `rate_limits` object.
pub fn parse_rate_limits_line(line: &str) -> Option<CodexLimits> {
    let event: Value = serde_json::from_str(line.trim()).ok()?;
    let payload = event.get("payload").unwrap_or(&event);
    let limits = payload
        .get("rate_limits")
        .or_else(|| payload.get("info").and_then(|i| i.get("rate_limits")))?;
    if !limits.is_object() {
        return None;
    }
    let observed_at = event
        .get("timestamp")
        .and_then(Value::as_str)
        .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
        .map(|t| t.with_timezone(&Utc));
    let windows: Vec<CodexWindow> = ["primary", "secondary"]
        .iter()
        .filter_map(|slot| window(limits.get(*slot)?, slot, observed_at))
        .collect();
    if windows.is_empty() {
        return None;
    }
    Some(CodexLimits {
        observed_at,
        plan_type: limits
            .get("plan_type")
            .and_then(Value::as_str)
            .map(str::to_string),
        windows,
    })
}

/// Newest plan state from the Codex session logs in `vm_name`; `Ok(None)`
/// when no turn has recorded one yet.
pub async fn limits(
    state: &AgentService,
    vm_name: &str,
) -> Result<Option<CodexLimits>, (StatusCode, String)> {
    let script = format!(
        "find {home}/.codex/sessions -type f -name '*.jsonl' -printf '%T@ %p\\n' 2>/dev/null \
         | sort -rn | head -n 5 | cut -d' ' -f2- | while read -r f; do \
           line=$(grep -F '\"rate_limits\":{{' \"$f\" | tail -n 1); \
           if [ -n \"$line\" ]; then printf '%s\\n' \"$line\"; break; fi; \
         done",
        home = CODEX_HOME
    );
    let (_, stdout, _) = run_script(state, vm_name, &script).await?;
    Ok(stdout.lines().rev().find_map(parse_rate_limits_line))
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL: &str = r#"{"timestamp":"2026-11-21T17:15:20.277Z","ordinal":16,"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":21811,"cached_input_tokens":13696,"cache_write_input_tokens":0,"output_tokens":85,"reasoning_output_tokens":0,"total_tokens":21896},"last_token_usage":{"input_tokens":21811,"cached_input_tokens":13696,"cache_write_input_tokens":0,"output_tokens":85,"reasoning_output_tokens":0,"total_tokens":21896},"last_usage_includes_reasoning":true,"model_context_window":475000},"rate_limits":{"limit_id":"codex","limit_name":null,"primary":{"used_percent":81.0,"window_minutes":300,"resets_at":1795297151},"secondary":{"used_percent":68.0,"window_minutes":10080,"resets_at":1795726443},"credits":{"has_credits":false,"unlimited":false,"balance":"0"},"individual_limit":null,"spend_control_reached":null,"plan_type":"plus","rate_limit_reached_type":null}}}"#;

    #[test]
    fn the_real_event_gives_both_windows() {
        let parsed = parse_rate_limits_line(REAL);
        let Some(limits) = parsed else {
            panic!("event not parsed");
        };
        assert_eq!(limits.plan_type.as_deref(), Some("plus"));
        assert_eq!(
            limits.observed_at.map(|t| t.timestamp_millis()),
            Some(1_795_281_320_277)
        );
        assert_eq!(limits.windows.len(), 2);
        assert_eq!(limits.windows[0].name, "5h");
        assert_eq!(limits.windows[0].used_percent, 81.0);
        assert_eq!(limits.windows[0].resets_at.map(|t| t.timestamp()), Some(1_795_297_151));
        assert_eq!(limits.windows[1].name, "weekly");
        assert_eq!(limits.windows[1].used_percent, 68.0);
    }

    #[test]
    fn the_older_relative_reset_counts_from_the_event_time() {
        let line = r#"{"timestamp":"2026-11-21T17:00:00Z","type":"event_msg","payload":{"type":"token_count","info":null,"rate_limits":{"primary":{"used_percent":12.5,"window_minutes":300,"resets_in_seconds":600}}}}"#;
        let parsed = parse_rate_limits_line(line);
        let reset = parsed.and_then(|l| l.windows.first().and_then(|w| w.resets_at));
        assert_eq!(reset.map(|t| t.to_rfc3339()).as_deref(), Some("2026-11-21T17:10:00+00:00"));
    }

    #[test]
    fn events_without_limits_are_skipped() {
        assert!(parse_rate_limits_line(r#"{"type":"event_msg","payload":{"type":"token_count","rate_limits":null}}"#).is_none());
        assert!(parse_rate_limits_line(r#"{"type":"event_msg","payload":{"type":"agent_message"}}"#).is_none());
        assert!(parse_rate_limits_line("not json").is_none());
    }
}
