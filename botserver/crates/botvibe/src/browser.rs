use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{VibeState, VibeToolResult};
use serde_json::{json, Value};
use std::sync::Arc;

fn cdp_base() -> String {
    std::env::var("BROWSER_CDP_URL").unwrap_or_else(|_| "http://127.0.0.1:9222".to_string())
}

fn ok(data: Value) -> VibeToolResult {
    VibeToolResult { success: true, data, error: None, latency_ms: 0 }
}

fn err(msg: String) -> VibeToolResult {
    VibeToolResult { success: false, data: Value::Null, error: Some(msg), latency_ms: 0 }
}

pub fn browser_tools() -> Vec<(String, ToolSchema, ToolHandler)> {
    vec![
        ("browser/navigate".into(), ToolSchema::new("browser/navigate", "Open a URL in a new Chrome tab via CDP").with_parameters(json!({
            "type": "object",
            "properties": {
                "url": {"type": "string", "description": "Full http(s) URL to open"}
            },
            "required": ["url"]
        })), browser_navigate()),
        ("browser/list".into(), ToolSchema::new("browser/list", "List open Chrome tabs via CDP"), browser_list()),
        ("browser/close".into(), ToolSchema::new("browser/close", "Close a Chrome tab by id").with_parameters(json!({
            "type": "object",
            "properties": {
                "tab_id": {"type": "string", "description": "Tab id from browser/list"}
            },
            "required": ["tab_id"]
        })), browser_close()),
        ("browser/screenshot".into(), ToolSchema::new("browser/screenshot", "Capture a tab screenshot via CDP and return its path").with_parameters(json!({
            "type": "object",
            "properties": {
                "tab_id": {"type": "string", "description": "Tab id from browser/list"}
            },
            "required": ["tab_id"]
        })), browser_screenshot()),
    ]
}

fn browser_navigate() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if url.is_empty() || !(url.starts_with("http://") || url.starts_with("https://")) {
                return err("url must be a full http(s) URL".into());
            }
            let client = reqwest::Client::new();
            let base = cdp_base();
            let target = format!("{base}/json/new?{url}");
            match client.put(&target).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Value>().await {
                        Ok(v) => ok(json!({
                            "tab_id": v.get("id").cloned().unwrap_or(Value::Null),
                            "url": v.get("url").cloned().unwrap_or(json!(url)),
                        })),
                        Err(e) => err(format!("CDP parse error: {e}")),
                    }
                }
                Ok(resp) => err(format!("CDP returned status {}", resp.status())),
                Err(e) => err(format!("CDP request failed: {e} (is Chrome running with --remote-debugging-port=9222?)")),
            }
        })
    })
}

fn browser_list() -> ToolHandler {
    Arc::new(move |_args: Value, _state: &dyn VibeState| {
        Box::pin(async move {
            let client = reqwest::Client::new();
            let base = cdp_base();
            match client.get(format!("{base}/json/list")).send().await {
                Ok(resp) if resp.status().is_success() => match resp.json::<Value>().await {
                    Ok(v) => {
                        let tabs = v
                            .as_array()
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .map(|t| json!({
                                "tab_id": t.get("id").cloned().unwrap_or(Value::Null),
                                "title": t.get("title").cloned().unwrap_or(Value::Null),
                                "url": t.get("url").cloned().unwrap_or(Value::Null),
                                "type": t.get("type").cloned().unwrap_or(Value::Null),
                            }))
                            .collect::<Vec<_>>();
                        ok(json!({ "tabs": tabs }))
                    }
                    Err(e) => err(format!("CDP parse error: {e}")),
                },
                Ok(resp) => err(format!("CDP returned status {}", resp.status())),
                Err(e) => err(format!("CDP request failed: {e}")),
            }
        })
    })
}

fn browser_close() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let tab_id = args.get("tab_id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if tab_id.is_empty() {
                return err("tab_id is required".into());
            }
            let client = reqwest::Client::new();
            let base = cdp_base();
            match client.get(format!("{base}/json/close/{tab_id}")).send().await {
                Ok(resp) if resp.status().is_success() => ok(json!({ "closed": tab_id })),
                Ok(resp) => err(format!("CDP returned status {}", resp.status())),
                Err(e) => err(format!("CDP request failed: {e}")),
            }
        })
    })
}

fn browser_screenshot() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let tab_id = args.get("tab_id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if tab_id.is_empty() {
                return err("tab_id is required".into());
            }
            let client = reqwest::Client::new();
            let base = cdp_base();
            let target = format!("{base}/json/screenshot/{tab_id}");
            match client.get(&target).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let bytes = match resp.bytes().await {
                        Ok(b) => b,
                        Err(e) => return err(format!("CDP body error: {e}")),
                    };
                    let path = format!("/tmp/browser_{tab_id}_{}.png", chrono::Utc::now().timestamp());
                    if let Err(e) = std::fs::write(&path, &bytes) {
                        return err(format!("Failed to write screenshot: {e}"));
                    }
                    ok(json!({ "screenshot": path, "bytes": bytes.len() }))
                }
                Ok(resp) => err(format!("CDP returned status {}", resp.status())),
                Err(e) => err(format!("CDP request failed: {e}")),
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdp_base_defaults_to_localhost_9222() {
        std::env::remove_var("BROWSER_CDP_URL");
        assert_eq!(cdp_base(), "http://127.0.0.1:9222");
        std::env::set_var("BROWSER_CDP_URL", "http://example.com:9223");
        assert_eq!(cdp_base(), "http://example.com:9223");
        std::env::remove_var("BROWSER_CDP_URL");
    }
}
