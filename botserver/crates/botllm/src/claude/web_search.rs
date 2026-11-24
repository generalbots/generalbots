//! Anthropic's server-side web search for a chat answer, and the steps it
//! leaves in the stream.
//!
//! The search runs on Anthropic's side (`server_tool_use` named
//! `web_search`, then a `web_search_tool_result`), so there is no client
//! loop. Each finished search becomes one marker chunk
//! `{"__web_step__": {"tool": "web_search", "text": <query>, "urls": [...],
//! "ok": bool}}` that the chat pipeline turns into a visible step; it never
//! reaches the answer text.

use std::collections::HashMap;

use serde_json::{json, Value};

/// Key of the step marker chunk (`{"__web_step__": {...}}`).
pub const WEB_STEP_MARKER: &str = "__web_step__";
/// Searches one answer may run.
const MAX_USES: u32 = 5;

/// The search tool for `model` on `base_url`, or `None` where the endpoint
/// is not Anthropic's own API or the model has no web search. Models with
/// dynamic filtering get the newer type.
#[must_use]
pub fn tool_for(model: &str, base_url: &str) -> Option<Value> {
    if !base_url.contains("anthropic.com") {
        return None;
    }
    let model = model.to_ascii_lowercase();
    let filtering = ["fable", "mythos", "opus-5", "opus-4-8", "opus-4-7", "opus-4-6", "sonnet-5", "sonnet-4-6"];
    let basic = ["opus-4", "sonnet-4", "haiku-4", "haiku-5", "3-7-sonnet", "3-5-sonnet", "3-5-haiku"];
    let kind = if filtering.iter().any(|m| model.contains(m)) {
        "web_search_20260209"
    } else if basic.iter().any(|m| model.contains(m)) {
        "web_search_20250305"
    } else {
        return None;
    };
    Some(json!({ "type": kind, "name": "web_search", "max_uses": MAX_USES }))
}

/// Follows the stream's search blocks: the query of each `server_tool_use`
/// (built from its `input_json_delta`s) until its result arrives.
#[derive(Debug, Default)]
pub struct SearchSteps {
    /// Open `server_tool_use` blocks: stream index → (tool use id, input JSON so far).
    open: HashMap<u64, (String, String)>,
    /// Finished queries by tool use id, waiting for their result.
    queries: HashMap<String, String>,
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

impl SearchSteps {
    /// Reads one SSE `data:` payload; returns the marker chunk when a search
    /// finished.
    pub fn on_event(&mut self, data: &str) -> Option<String> {
        if !data.contains("server_tool_use") && !data.contains("web_search_tool_result") && self.open.is_empty() {
            return None;
        }
        let event: Value = serde_json::from_str(data).ok()?;
        let index = event.get("index").and_then(Value::as_u64);
        match text(&event, "type") {
            "content_block_start" => {
                let block = event.get("content_block")?;
                match text(block, "type") {
                    "server_tool_use" if text(block, "name") == "web_search" => {
                        let given = block.get("input").and_then(|input| input.get("query")).and_then(Value::as_str);
                        let id = text(block, "id").to_string();
                        match given.filter(|query| !query.is_empty()) {
                            Some(query) => {
                                self.queries.insert(id, query.to_string());
                            }
                            None => {
                                self.open.insert(index?, (id, String::new()));
                            }
                        }
                        None
                    }
                    "web_search_tool_result" => Some(self.result(block)),
                    _ => None,
                }
            }
            "content_block_delta" => {
                let delta = event.get("delta")?;
                if text(delta, "type") == "input_json_delta" {
                    if let Some((_, json)) = self.open.get_mut(&index?) {
                        json.push_str(text(delta, "partial_json"));
                    }
                }
                None
            }
            "content_block_stop" => {
                let (id, input) = self.open.remove(&index?)?;
                let query = serde_json::from_str::<Value>(&input)
                    .ok()
                    .map(|value| text(&value, "query").to_string())
                    .unwrap_or_default();
                self.queries.insert(id, query);
                None
            }
            _ => None,
        }
    }

    /// The marker for one `web_search_tool_result`: a list of results, or an
    /// error object (HTTP 200 with `error_code`).
    fn result(&mut self, block: &Value) -> String {
        let query = self.queries.remove(text(block, "tool_use_id")).unwrap_or_default();
        let content = block.get("content");
        let urls: Vec<&str> = content
            .and_then(Value::as_array)
            .map(|items| items.iter().map(|item| text(item, "url")).filter(|url| !url.is_empty()).collect())
            .unwrap_or_default();
        let ok = content.is_some_and(Value::is_array);
        json!({ WEB_STEP_MARKER: { "tool": "web_search", "text": query, "urls": urls, "ok": ok } }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const API: &str = "https://api.anthropic.com";

    #[test]
    fn the_tool_type_follows_the_model_and_only_anthropic_gets_it() {
        assert_eq!(tool_for("claude-opus-5-5", API).map(|t| t["type"].clone()), Some(json!("web_search_20260209")));
        assert_eq!(tool_for("claude-sonnet-4-6", API).map(|t| t["type"].clone()), Some(json!("web_search_20260209")));
        assert_eq!(tool_for("claude-haiku-4-5", API).map(|t| t["type"].clone()), Some(json!("web_search_20250305")));
        assert_eq!(tool_for("claude-opus-5-5", API).map(|t| t["max_uses"].clone()), Some(json!(MAX_USES)));
        assert!(tool_for("claude-3-haiku-20240307", API).is_none());
        assert!(tool_for("claude-opus-5-5", "https://example.openai.azure.com").is_none());
    }

    #[test]
    fn a_streamed_search_becomes_one_marker_with_query_and_urls() {
        let mut steps = SearchSteps::default();
        let events = [
            r#"{"type":"content_block_start","index":1,"content_block":{"type":"server_tool_use","id":"srvtoolu_1","name":"web_search","input":{}}}"#,
            r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"query\": \"preço"}}"#,
            r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":" do café\"}"}}"#,
            r#"{"type":"content_block_stop","index":1}"#,
        ];
        for event in events {
            assert!(steps.on_event(event).is_none());
        }
        let result = r#"{"type":"content_block_start","index":2,"content_block":{"type":"web_search_tool_result","tool_use_id":"srvtoolu_1","content":[{"type":"web_search_result","url":"https://example.org/a","title":"A"},{"type":"web_search_result","url":"https://example.net/b","title":"B"}]}}"#;
        let marker: Value = steps.on_event(result).and_then(|m| serde_json::from_str(&m).ok()).unwrap_or_default();
        assert_eq!(marker[WEB_STEP_MARKER]["text"], "preço do café");
        assert_eq!(marker[WEB_STEP_MARKER]["urls"], json!(["https://example.org/a", "https://example.net/b"]));
        assert_eq!(marker[WEB_STEP_MARKER]["ok"], true);
    }

    #[test]
    fn a_failed_search_is_a_failed_step_and_text_is_ignored() {
        let mut steps = SearchSteps::default();
        let text_delta = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"oi"}}"#;
        assert!(steps.on_event(text_delta).is_none());
        let error = r#"{"type":"content_block_start","index":3,"content_block":{"type":"web_search_tool_result","tool_use_id":"x","content":{"type":"web_search_tool_result_error","error_code":"max_uses_exceeded"}}}"#;
        let marker: Value = steps.on_event(error).and_then(|m| serde_json::from_str(&m).ok()).unwrap_or_default();
        assert_eq!(marker[WEB_STEP_MARKER]["ok"], false);
    }
}
