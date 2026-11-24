//! What Codex does without an approval request: web searches and MCP tool
//! calls. Neither has an approval in the app-server protocol (v0.171).
//!
//! Web search is turned off for every bridged thread: its default mode is
//! `cached` (`ConfigToml::web_search`, `WebSearchMode` in Codex 0.171), so it
//! would reach the web without asking. The ChatGPT apps connector (the
//! `codex_apps` MCP server, feature `apps`, on by default) and plugins
//! (feature `plugins`, which can bring their own MCP servers) are turned off
//! too: their tools run without an approval request. The keys are dotted
//! overrides, applied per thread like `-c features.apps=false`. The
//! workspace configures no MCP server. Either item, should one still appear,
//! becomes a step row.

use serde_json::{json, Value};

/// Config overrides sent with `thread/start` and `thread/resume`.
#[must_use]
pub fn thread_config() -> Value {
    json!({
        "web_search": "disabled",
        "features.apps": false,
        "features.plugins": false,
    })
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty())
}

/// A finished item that ran without asking, as its step: `("web_search",
/// query)` or `("mcp_tool", "server/tool")`, and whether it worked.
#[must_use]
pub fn noted_step(item: &Value) -> Option<(&'static str, String, bool)> {
    match item.get("type").and_then(Value::as_str)? {
        "webSearch" => {
            let action = item.get("action").unwrap_or(&Value::Null);
            let query = text(item, "query")
                .or_else(|| text(action, "query"))
                .or_else(|| text(action, "url"))
                .unwrap_or("");
            Some(("web_search", query.to_string(), true))
        }
        "mcpToolCall" => {
            let name = format!("{}/{}", text(item, "server").unwrap_or("?"), text(item, "tool").unwrap_or("?"));
            Some(("mcp_tool", name, item.get("status").and_then(Value::as_str) == Some("completed")))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::app_server::{thread_resume_params, thread_start_params};
    use super::*;

    #[test]
    fn every_bridged_thread_has_web_search_off() {
        assert_eq!(thread_start_params("/w", "", false)["config"]["web_search"], "disabled");
        assert_eq!(thread_resume_params("t1", "/w", "", None)["config"]["web_search"], "disabled");
    }

    #[test]
    fn every_bridged_thread_has_apps_and_plugins_off() {
        for params in [thread_start_params("/w", "", false), thread_resume_params("t1", "/w", "", None)] {
            assert_eq!(params["config"]["features.apps"], false);
            assert_eq!(params["config"]["features.plugins"], false);
        }
    }

    #[test]
    fn a_resumed_thread_gets_the_current_system_prompt() {
        let params = thread_resume_params("t1", "/w", "", Some("Answer in Portuguese."));
        assert_eq!(params["developerInstructions"], "Answer in Portuguese.");
        assert!(thread_resume_params("t1", "/w", "", Some("  ")).get("developerInstructions").is_none());
        assert!(thread_resume_params("t1", "/w", "", None).get("developerInstructions").is_none());
    }

    #[test]
    fn searches_and_mcp_calls_become_steps() {
        let search = json!({"type": "webSearch", "id": "w1", "query": "rust 1.90 release", "action": null});
        assert_eq!(noted_step(&search), Some(("web_search", "rust 1.90 release".to_string(), true)));
        let open = json!({"type": "webSearch", "id": "w2", "query": "", "action": {"type": "openPage", "url": "https://example.org/"}});
        assert_eq!(noted_step(&open).map(|s| s.1), Some("https://example.org/".to_string()));
        let mcp = json!({"type": "mcpToolCall", "id": "m1", "server": "docs", "tool": "lookup", "status": "failed"});
        assert_eq!(noted_step(&mcp), Some(("mcp_tool", "docs/lookup".to_string(), false)));
        assert_eq!(noted_step(&json!({"type": "commandExecution"})), None);
    }
}
