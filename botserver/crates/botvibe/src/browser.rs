use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{VibeState, VibeToolResult};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

fn cdp_base() -> String {
    std::env::var("BROWSER_CDP_URL").unwrap_or_else(|_| "http://127.0.0.1:9222".to_string())
}

fn ok(data: Value) -> VibeToolResult {
    VibeToolResult { success: true, data, error: None, latency_ms: 0 }
}

fn err(msg: String) -> VibeToolResult {
    VibeToolResult { success: false, data: Value::Null, error: Some(msg), latency_ms: 0 }
}

/// Resolves the tab's `webSocketDebuggerUrl` from the CDP HTTP endpoint so the
/// tool can drive it over WebSocket (eval/fill/click/screenshot).
async fn tab_ws_url(tab_id: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let base = cdp_base();
    let resp = client
        .get(format!("{base}/json/list"))
        .send()
        .await
        .map_err(|e| format!("CDP list failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("CDP list returned status {}", resp.status()));
    }
    let tabs: Value = resp.json().await.map_err(|e| format!("CDP parse error: {e}"))?;
    for tab in tabs.as_array().cloned().unwrap_or_default() {
        if tab.get("id").and_then(|v| v.as_str()) == Some(tab_id) {
            return tab
                .get("webSocketDebuggerUrl")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| "tab has no webSocketDebuggerUrl".to_string());
        }
    }
    Err(format!("tab {tab_id} not found in CDP list"))
}

/// Sends one CDP command over the tab's WebSocket and returns the `result`
/// object, or the `error` message on protocol failure.
async fn cdp_call(tab_id: &str, method: &str, params: Value) -> Result<Value, String> {
    let ws_url = tab_ws_url(tab_id).await?;
    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .map_err(|e| format!("CDP websocket connect failed: {e}"))?;
    let id = 1u64;
    let command = json!({ "id": id, "method": method, "params": params });
    ws.send(tokio_tungstenite::tungstenite::Message::Text(
        command.to_string().into(),
    ))
    .await
    .map_err(|e| format!("CDP send failed: {e}"))?;

    loop {
        let msg = tokio::time::timeout(Duration::from_secs(15), ws.next())
            .await
            .map_err(|_| format!("CDP {method} timed out"))?
            .ok_or_else(|| "CDP connection closed".to_string())?;
        let text = msg
            .map_err(|e| format!("CDP read error: {e}"))?
            .into_text()
            .map_err(|_| "CDP non-text frame".to_string())?;
        let parsed: Value = serde_json::from_str(&text).unwrap_or_default();
        if parsed.get("id").and_then(|v| v.as_u64()) == Some(id) {
            if let Some(error) = parsed.get("error") {
                return Err(format!("CDP {method} error: {error}"));
            }
            return Ok(parsed.get("result").cloned().unwrap_or(Value::Null));
        }
    }
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
        ("browser/screenshot".into(), ToolSchema::new("browser/screenshot", "Capture a tab screenshot via CDP and return it as a base64 PNG").with_parameters(json!({
            "type": "object",
            "properties": {
                "tab_id": {"type": "string", "description": "Tab id from browser/list"}
            },
            "required": ["tab_id"]
        })), browser_screenshot()),
        ("browser/eval".into(), ToolSchema::new("browser/eval", "Run JavaScript in the tab and return the serialized result").with_parameters(json!({
            "type": "object",
            "properties": {
                "tab_id": {"type": "string", "description": "Tab id from browser/list"},
                "expression": {"type": "string", "description": "JavaScript expression to evaluate (return a JSON-serializable value)"}
            },
            "required": ["tab_id", "expression"]
        })), browser_eval()),
        ("browser/fill".into(), ToolSchema::new("browser/fill", "Set the value of an input matched by CSS selector").with_parameters(json!({
            "type": "object",
            "properties": {
                "tab_id": {"type": "string", "description": "Tab id from browser/list"},
                "selector": {"type": "string", "description": "CSS selector of the input"},
                "value": {"type": "string", "description": "Text to type into the input"}
            },
            "required": ["tab_id", "selector", "value"]
        })), browser_fill()),
        ("browser/click".into(), ToolSchema::new("browser/click", "Click the first element matched by a CSS selector").with_parameters(json!({
            "type": "object",
            "properties": {
                "tab_id": {"type": "string", "description": "Tab id from browser/list"},
                "selector": {"type": "string", "description": "CSS selector of the element to click"}
            },
            "required": ["tab_id", "selector"]
        })), browser_click()),
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
            // Real CDP: Page.captureScreenshot over the tab's WebSocket. The
            // result `data` field is the base64 PNG, returned inline so the
            // Vibe chat can render it directly.
            match cdp_call(&tab_id, "Page.captureScreenshot", json!({ "format": "png" })).await {
                Ok(result) => {
                    let b64 = result
                        .get("data")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    if b64.is_empty() {
                        return err("Page.captureScreenshot returned no data".into());
                    }
                    ok(json!({
                        "image_base64": b64,
                        "mime_type": "image/png",
                        "size_bytes": b64.len() * 3 / 4,
                        "tab_id": tab_id,
                    }))
                }
                Err(e) => err(format!("screenshot failed: {e}")),
            }
        })
    })
}

fn browser_eval() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let tab_id = args.get("tab_id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let expression = args.get("expression").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if tab_id.is_empty() || expression.is_empty() {
                return err("tab_id and expression are required".into());
            }
            match cdp_call(
                &tab_id,
                "Runtime.evaluate",
                json!({ "expression": expression, "returnByValue": true }),
            )
            .await
            {
                Ok(result) => {
                    let value = result
                        .get("result")
                        .and_then(|v| v.get("value"))
                        .cloned()
                        .unwrap_or(Value::Null);
                    ok(json!({ "tab_id": tab_id, "result": value }))
                }
                Err(e) => err(format!("eval failed: {e}")),
            }
        })
    })
}

fn browser_fill() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let tab_id = args.get("tab_id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let value = args.get("value").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if tab_id.is_empty() || selector.is_empty() {
                return err("tab_id and selector are required".into());
            }
            // Escape for a JS string literal inside the expression.
            let value_js = serde_json::to_string(&value).unwrap_or_else(|_| "\"\"".to_string());
            let selector_js = serde_json::to_string(&selector).unwrap_or_else(|_| "\"\"".to_string());
            let expression = format!(
                "(() => {{ const el = document.querySelector({selector_js}); if (!el) return {{ found: false }}; \
                 el.value = {value_js}; el.dispatchEvent(new Event('input', {{ bubbles: true }})); \
                 el.dispatchEvent(new Event('change', {{ bubbles: true }})); return {{ found: true, value: el.value }}; }})()"
            );
            match cdp_call(
                &tab_id,
                "Runtime.evaluate",
                json!({ "expression": expression, "returnByValue": true }),
            )
            .await
            {
                Ok(result) => {
                    let value = result
                        .get("result")
                        .and_then(|v| v.get("value"))
                        .cloned()
                        .unwrap_or(Value::Null);
                    ok(json!({ "tab_id": tab_id, "filled": value }))
                }
                Err(e) => err(format!("fill failed: {e}")),
            }
        })
    })
}

fn browser_click() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let tab_id = args.get("tab_id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if tab_id.is_empty() || selector.is_empty() {
                return err("tab_id and selector are required".into());
            }
            let selector_js = serde_json::to_string(&selector).unwrap_or_else(|_| "\"\"".to_string());
            let expression = format!(
                "(() => {{ const el = document.querySelector({selector_js}); if (!el) return {{ found: false }}; \
                 el.click(); return {{ found: true, tag: el.tagName, text: (el.innerText || '').slice(0, 120) }}; }})()"
            );
            match cdp_call(
                &tab_id,
                "Runtime.evaluate",
                json!({ "expression": expression, "returnByValue": true }),
            )
            .await
            {
                Ok(result) => {
                    let value = result
                        .get("result")
                        .and_then(|v| v.get("value"))
                        .cloned()
                        .unwrap_or(Value::Null);
                    ok(json!({ "tab_id": tab_id, "clicked": value }))
                }
                Err(e) => err(format!("click failed: {e}")),
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
