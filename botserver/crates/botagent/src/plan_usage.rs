//! Plan usage the subscription CLIs reported while they ran, kept in memory
//! per workspace and agent so the limits page can show it between turns.
//!
//! - Claude Code prints a `rate_limit_event` line in its stream-json output
//!   for each plan window it was checked against (status, reset time and,
//!   when the service sends it, the share used).
//! - Every agent ends a turn with its own sentence when the plan is spent;
//!   that state and its reset time are kept too.
//!
//! Nothing is estimated: a window keeps only what the CLI said, with the
//! instant it said it.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

/// Provider id of Claude Code, as the account rows name it.
pub const CLAUDE_CODE: &str = "claude-code";

/// One plan window as the CLI reported it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanWindow {
    pub name: String,
    pub used_percent: Option<f64>,
    pub resets_at: Option<DateTime<Utc>>,
    /// `allowed`, `allowed_warning` or `rejected`.
    pub status: Option<String>,
    pub observed_at: DateTime<Utc>,
}

/// What one agent in one workspace last reported.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlanUsage {
    pub windows: Vec<PlanWindow>,
    /// When a turn ended on a spent plan, and when the plan comes back.
    pub spent_at: Option<DateTime<Utc>>,
    pub spent_until: Option<DateTime<Utc>>,
}

impl PlanUsage {
    /// The newest instant anything in this entry was observed.
    pub fn observed_at(&self) -> Option<DateTime<Utc>> {
        self.windows.iter().map(|w| w.observed_at).chain(self.spent_at).max()
    }
}

type Store = HashMap<(String, String), PlanUsage>;

fn store() -> std::sync::MutexGuard<'static, Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The last report of `agent` in `vm_name`.
pub fn latest(vm_name: &str, agent: &str) -> Option<PlanUsage> {
    store().get(&(vm_name.to_string(), agent.to_string())).cloned()
}

/// Keeps `windows`, replacing earlier reports of the same windows.
pub fn record_windows(vm_name: &str, agent: &str, windows: Vec<PlanWindow>) {
    if windows.is_empty() {
        return;
    }
    let mut map = store();
    let entry = map.entry((vm_name.to_string(), agent.to_string())).or_default();
    for window in windows {
        entry.windows.retain(|w| w.name != window.name);
        entry.windows.push(window);
    }
}

/// A turn of `agent` ended on a spent plan; `until` is when it comes back.
pub fn record_spent(vm_name: &str, agent: &str, until: Option<DateTime<Utc>>) {
    let mut map = store();
    let entry = map.entry((vm_name.to_string(), agent.to_string())).or_default();
    entry.spent_at = Some(Utc::now());
    entry.spent_until = until;
}

fn claude_window_name(kind: &str) -> String {
    match kind {
        "five_hour" => "5h".to_string(),
        "seven_day" => "weekly".to_string(),
        "seven_day_opus" => "weekly_opus".to_string(),
        "seven_day_sonnet" => "weekly_sonnet".to_string(),
        other => other.to_string(),
    }
}

/// The `rate_limit_info` of a Claude Code stream-json `rate_limit_event`
/// line, or `None` for any other line.
pub fn claude_rate_limit(line: &str) -> Option<Value> {
    if !line.contains("\"rate_limit_event\"") {
        return None;
    }
    let message: Value = serde_json::from_str(line).ok()?;
    if message.get("type").and_then(Value::as_str) != Some("rate_limit_event") {
        return None;
    }
    message.get("rate_limit_info").filter(|info| info.is_object()).cloned()
}

/// One plan window from a Claude Code `rate_limit_info`. `utilization` is
/// a fraction of the window; `resetsAt` is in epoch seconds.
pub fn claude_window(info: &Value, observed_at: DateTime<Utc>) -> Option<PlanWindow> {
    let kind = info.get("rateLimitType").and_then(Value::as_str)?;
    let status = info.get("status").and_then(Value::as_str).map(str::to_string);
    let used_percent = info
        .get("utilization")
        .and_then(Value::as_f64)
        .map(|share| (share * 100.0).clamp(0.0, 100.0))
        .or_else(|| (status.as_deref() == Some("rejected")).then_some(100.0));
    let resets_at = info
        .get("resetsAt")
        .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
        .and_then(|secs| Utc.timestamp_opt(secs, 0).single());
    Some(PlanWindow { name: claude_window_name(kind), used_percent, resets_at, status, observed_at })
}

/// Keeps the windows of the `rate_limit_info` objects one Claude Code turn printed.
pub fn record_claude(vm_name: &str, infos: &[Value]) {
    let now = Utc::now();
    let windows = infos.iter().filter_map(|info| claude_window(info, now)).collect();
    record_windows(vm_name, CLAUDE_CODE, windows);
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVENT: &str = r#"{"type":"rate_limit_event","rate_limit_info":{"status":"allowed_warning","resetsAt":1795297151,"rateLimitType":"five_hour","utilization":0.82,"isUsingOverage":false},"uuid":"u","session_id":"s"}"#;

    #[test]
    fn a_claude_rate_limit_event_becomes_a_window() {
        let info = claude_rate_limit(EVENT).unwrap_or_default();
        let at = Utc::now();
        let window = claude_window(&info, at);
        assert_eq!(window.as_ref().map(|w| w.name.as_str()), Some("5h"));
        assert_eq!(window.as_ref().and_then(|w| w.used_percent).map(f64::round), Some(82.0));
        assert_eq!(window.as_ref().and_then(|w| w.resets_at).map(|t| t.timestamp()), Some(1795297151));
        assert_eq!(window.and_then(|w| w.status).as_deref(), Some("allowed_warning"));
    }

    #[test]
    fn a_window_without_utilization_keeps_only_its_status() {
        let info = serde_json::json!({"status": "allowed", "rateLimitType": "seven_day"});
        let window = claude_window(&info, Utc::now());
        assert_eq!(window.as_ref().map(|w| w.name.as_str()), Some("weekly"));
        assert_eq!(window.and_then(|w| w.used_percent), None);
        let rejected = serde_json::json!({"status": "rejected", "rateLimitType": "five_hour"});
        assert_eq!(claude_window(&rejected, Utc::now()).and_then(|w| w.used_percent), Some(100.0));
    }

    #[test]
    fn other_lines_are_not_rate_limits() {
        assert!(claude_rate_limit(r#"{"type":"assistant","message":{}}"#).is_none());
        assert!(claude_rate_limit("not json \"rate_limit_event\"").is_none());
    }

    #[test]
    fn a_newer_report_replaces_the_same_window_and_spent_is_kept() {
        let at = Utc::now();
        let window = |pct| PlanWindow { name: "5h".into(), used_percent: Some(pct), resets_at: None, status: None, observed_at: at };
        record_windows("vm-test-usage", "agent", vec![window(10.0)]);
        record_windows("vm-test-usage", "agent", vec![window(30.0)]);
        record_spent("vm-test-usage", "agent", None);
        let seen = latest("vm-test-usage", "agent").unwrap_or_default();
        assert_eq!(seen.windows.len(), 1);
        assert_eq!(seen.windows[0].used_percent, Some(30.0));
        assert!(seen.spent_at.is_some());
    }
}
