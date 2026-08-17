//! Wired implementations for tools that previously returned
//! `tool '...' is not wired up yet` (Issue #796).
//!
//! - Autotask tools (`classify_intent`, `compile_plan`, `execute_plan`,
//!   `create_and_execute`) delegate to `botautotask` (heuristic classifier +
//!   BASIC script generation).
//! - CRM tools (`search_contacts`, `get_deals`, `create_ticket`,
//!   `update_ticket`, `send_email`) operate on the CRM tables through the
//!   run's database pool.
//! - Analysis tools (`fetch_market_data`, `analyze_sentiment`,
//!   `generate_report`, `detect_anomalies`) are pure algorithms plus an
//!   optional live market feed.

pub mod analysis;
pub mod autotask;
pub mod crm;

use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::VibeToolResult;
use serde_json::Value;
use std::sync::Arc;

/// Success result for a tool handler.
pub fn ok(data: Value) -> VibeToolResult {
    VibeToolResult { success: true, data, error: None, latency_ms: 0 }
}

/// Failure result for a tool handler.
pub fn err(msg: String) -> VibeToolResult {
    VibeToolResult { success: false, data: Value::Null, error: Some(msg), latency_ms: 0 }
}

/// Reads a required string argument, returning an error message when missing.
pub fn require_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("missing required argument '{key}'"))
}

/// Reads an optional string argument with a default.
pub fn opt_str(args: &Value, key: &str, default: &str) -> String {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(default)
        .to_string()
}

/// Converts a `Result<Value, String>` into a tool result.
pub fn result_of(result: Result<Value, String>) -> VibeToolResult {
    match result {
        Ok(data) => ok(data),
        Err(e) => err(e),
    }
}

/// Builds a tool handler closure from an async function over (args, state).
pub fn handler<F, Fut>(f: F) -> ToolHandler
where
    F: Fn(Value, &dyn crate::types::VibeState) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = VibeToolResult> + Send + 'static,
{
    Arc::new(move |args, state| Box::pin(f(args.clone(), state)))
}

/// Serializes a compact ticket-style reference token from a UUID prefix.
pub fn short_token(prefix: &str, uuid: &uuid::Uuid) -> String {
    let hex = uuid.simple().to_string();
    format!("{prefix}-{}", hex[..8].to_uppercase())
}

/// Common `bot_id` argument used to scope CRM queries. Handlers must fail
/// closed on `nil` (no global fallback) — see `crm::require_bot_scope`.
pub fn bot_id_arg(args: &Value) -> uuid::Uuid {
    args.get("bot_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .unwrap_or(uuid::Uuid::nil())
}

/// Builds the (label, value) pairs of a series for anomaly/report helpers.
pub fn number_array(args: &Value, key: &str) -> Result<Vec<f64>, String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .map(|item| {
                    item.as_f64().or_else(|| {
                        item.as_str().and_then(|s| s.parse::<f64>().ok())
                    })
                })
                .collect::<Option<Vec<f64>>>()
        })
        .ok_or_else(|| format!("argument '{key}' must be an array of numbers"))
}

/// Combined tool registration for all wired groups.
pub fn all_wired_tools() -> Vec<(String, ToolSchema, ToolHandler)> {
    let mut out: Vec<(String, ToolSchema, ToolHandler)> = Vec::new();
    out.extend(autotask::autotask_tools());
    out.extend(crm::crm_tools());
    out.extend(analysis::analysis_tools());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn require_str_validates_arguments() {
        let args = json!({"name": "x", "empty": ""});
        assert_eq!(require_str(&args, "name").unwrap(), "x");
        assert!(require_str(&args, "empty").is_err());
        assert!(require_str(&args, "missing").is_err());
    }

    #[test]
    fn opt_str_falls_back() {
        let args = json!({});
        assert_eq!(opt_str(&args, "a", "def"), "def");
    }

    #[test]
    fn short_token_is_upper_prefixed() {
        let token = short_token("T", &uuid::Uuid::nil());
        assert_eq!(token.len(), 10);
        assert!(token.starts_with("T-"));
    }

    #[test]
    fn number_array_accepts_numbers_and_strings() {
        let args = json!({"series": [1, "2.5", 3]});
        assert_eq!(number_array(&args, "series").unwrap(), vec![1.0, 2.5, 3.0]);
        assert!(number_array(&json!({"series": ["x"]}), "series").is_err());
    }
}
