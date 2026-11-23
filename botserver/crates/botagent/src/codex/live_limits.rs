//! The Codex plan windows read now, from the service, through the CLI's
//! app server (`account/rateLimits/read`), without running a turn.
//!
//! The session logs (`limits.rs`) only change when a turn runs; this read
//! is what the limits page's refresh asks for.

use std::time::Duration;

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::duplex::{DuplexOptions, RpcClient, RpcOptions};
use crate::{vm, AgentService};

use super::bridge::handshake;
use super::limits::{epoch, window_name};
use super::paths::{CODEX_HOME, CODEX_USER};
use super::{CodexLimits, CodexWindow};

const READ_WAIT: Duration = Duration::from_secs(25);
const SESSION_DEADLINE: Duration = Duration::from_secs(45);

fn field<'a>(raw: &'a Value, camel: &str, snake: &str) -> Option<&'a Value> {
    raw.get(camel).or_else(|| raw.get(snake))
}

fn window(raw: &Value, slot: &str) -> Option<CodexWindow> {
    let used = field(raw, "usedPercent", "used_percent")?.as_f64()?;
    let minutes = field(raw, "windowDurationMins", "window_minutes").and_then(Value::as_u64);
    Some(CodexWindow {
        name: window_name(minutes, slot),
        used_percent: used.clamp(0.0, 100.0),
        window_minutes: minutes,
        resets_at: field(raw, "resetsAt", "resets_at").and_then(epoch),
    })
}

/// The plan state in an `account/rateLimits/read` result; `None` when the
/// service sent no window.
pub fn parse_snapshot(result: &Value, observed_at: DateTime<Utc>) -> Option<CodexLimits> {
    let snapshot = field(result, "rateLimits", "rate_limits").unwrap_or(result);
    let windows: Vec<CodexWindow> = ["primary", "secondary"]
        .iter()
        .filter_map(|slot| window(snapshot.get(*slot)?, slot))
        .collect();
    if windows.is_empty() {
        return None;
    }
    Some(CodexLimits {
        observed_at: Some(observed_at),
        plan_type: field(snapshot, "planType", "plan_type").and_then(Value::as_str).map(str::to_string),
        windows,
    })
}

/// Asks the Codex app server in `vm_name` for the plan windows now.
pub async fn live_limits(state: &AgentService, vm_name: &str) -> Result<Option<CodexLimits>, (StatusCode, String)> {
    let script = format!(
        "exec su {CODEX_USER} -s /bin/sh -c 'cd {CODEX_HOME} && export HOME={CODEX_HOME} && exec codex app-server'"
    );
    let options = DuplexOptions { deadline: SESSION_DEADLINE, ..DuplexOptions::default() };
    let session = vm::exec_in_vm_duplex(state, vm_name, &["sh".into(), "-c".into(), script], options).await?;
    let (client, _inbox) = RpcClient::start(session, RpcOptions::default());
    let outcome = read(&client).await;
    client.close().await;
    outcome
}

async fn read(client: &RpcClient) -> Result<Option<CodexLimits>, (StatusCode, String)> {
    let failed = |step: &str, error: String| (StatusCode::BAD_GATEWAY, format!("codex app-server {step}: {error}"));
    handshake(client).await.map_err(|e| failed("initialize", format!("{e:?}")))?;
    let result = client
        .request("account/rateLimits/read", json!({}), READ_WAIT)
        .await
        .map_err(|e| failed("account/rateLimits/read", e.to_string()))?;
    Ok(parse_snapshot(&result, Utc::now()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_app_server_snapshot_becomes_the_plan_windows() {
        let result = json!({"rateLimits": {"limitId": "codex", "primary": {"usedPercent": 81, "windowDurationMins": 300, "resetsAt": 1795297151},
            "secondary": {"usedPercent": 68, "windowDurationMins": 10080, "resetsAt": 1795726443}, "planType": "plus"}});
        let parsed = parse_snapshot(&result, Utc::now());
        let names: Vec<String> = parsed.iter().flat_map(|l| l.windows.iter().map(|w| w.name.clone())).collect();
        assert_eq!(names, ["5h", "weekly"]);
        assert_eq!(parsed.as_ref().map(|l| l.windows[0].used_percent), Some(81.0));
        assert_eq!(parsed.and_then(|l| l.plan_type).as_deref(), Some("plus"));
    }

    #[test]
    fn a_snapshot_without_windows_is_none() {
        assert!(parse_snapshot(&json!({"rateLimits": {"primary": null, "secondary": null}}), Utc::now()).is_none());
    }
}
