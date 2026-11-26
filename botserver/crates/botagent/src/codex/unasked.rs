//! What Codex does without an approval request: web searches and MCP tool
//! calls. Neither has an approval in the app-server protocol (v0.171).
//!
//! Web search is live for every bridged thread (`ConfigToml::web_search`,
//! `WebSearchMode` in Codex 0.171; the default `cached` answers from an
//! index): searching is always allowed, and each search or page it opens is
//! a step of the answer, never hidden. The ChatGPT apps connector (the
//! `codex_apps` MCP server, feature `apps`, on by default) and plugins
//! (feature `plugins`, which can bring their own MCP servers) are turned off
//! too: their tools run without an approval request. The keys are dotted
//! overrides, applied per thread like `-c features.apps=false`. The
//! workspace configures no MCP server. Should an MCP call still appear, it
//! becomes a step row too.

use serde_json::{json, Value};

/// Config overrides sent with `thread/start` and `thread/resume`.
#[must_use]
pub fn thread_config() -> Value {
    json!({
        "web_search": "live",
        "features.apps": false,
        "features.plugins": false,
    })
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty())
}

/// A finished item that ran without asking, as its step: `("web_search",
/// query)`, `("web_fetch", url)` for a page it opened, or `("mcp_tool",
/// "server/tool")`, and whether it worked.
#[must_use]
pub fn noted_step(item: &Value) -> Option<(&'static str, String, bool)> {
    match item.get("type").and_then(Value::as_str)? {
        "webSearch" => {
            let action = item.get("action").unwrap_or(&Value::Null);
            if let Some(query) = text(item, "query").or_else(|| text(action, "query")) {
                return Some(("web_search", query.to_string(), true));
            }
            match text(action, "url") {
                Some(url) => Some(("web_fetch", url.to_string(), true)),
                None => Some(("web_search", String::new(), true)),
            }
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
    fn every_bridged_thread_searches_the_live_web() {
        assert_eq!(thread_start_params("/w", "", false)["config"]["web_search"], "live");
        assert_eq!(thread_resume_params("t1", "/w", "", None)["config"]["web_search"], "live");
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
        assert_eq!(noted_step(&open), Some(("web_fetch", "https://example.org/".to_string(), true)));
        let blank = json!({"type": "webSearch", "id": "w3", "query": "", "action": {"type": "other"}});
        assert_eq!(noted_step(&blank), Some(("web_search", String::new(), true)));
        let mcp = json!({"type": "mcpToolCall", "id": "m1", "server": "docs", "tool": "lookup", "status": "failed"});
        assert_eq!(noted_step(&mcp), Some(("mcp_tool", "docs/lookup".to_string(), false)));
        assert_eq!(noted_step(&json!({"type": "commandExecution"})), None);
    }
}
