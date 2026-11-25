//! Anthropic's server-side web search and page reading for a chat answer,
//! and the steps they leave in the stream.
//!
//! The search runs on Anthropic's side (`server_tool_use` named
//! `web_search`, then a `web_search_tool_result`), so there is no client
//! loop. Each finished search becomes one marker chunk
//! `{"__web_step__": {"tool": "web_search", "text": <query>, "urls": [...],
//! "ok": bool}}` that the chat pipeline turns into a visible step; it never
//! reaches the answer text.

use std::collections::HashMap;

use log::warn;
use serde_json::{json, Value};

/// Key of the step marker chunk (`{"__web_step__": {...}}`).
pub const WEB_STEP_MARKER: &str = "__web_step__";
/// Searches one answer may run.
const MAX_USES: u32 = 5;
/// Pages one answer may read through Anthropic's fetch.
const MAX_FETCHES: u32 = 5;
/// Text kept from one fetched page.
const MAX_FETCH_TOKENS: u32 = 20_000;
/// Server tools that run code in Anthropic's sandbox. They are never offered:
/// file work goes through the person's own workspace with its permission
/// cards. The dynamic-filtering web tools (`*_20260209` and later) bring code
/// execution with them, so only the basic versions are used.
const SANDBOX_TOOLS: [&str; 3] = ["code_execution", "bash_code_execution", "text_editor_code_execution"];

/// Whether `model` has Anthropic's server web tools.
fn has_web_tools(model: &str) -> bool {
    let model = model.to_ascii_lowercase();
    let known = [
        "fable", "mythos", "opus-5", "opus-4", "sonnet-5", "sonnet-4", "haiku-4", "haiku-5",
        "3-7-sonnet", "3-5-sonnet", "3-5-haiku",
    ];
    known.iter().any(|m| model.contains(m))
}

/// The server web tools for `model` on `base_url`: search and page reading,
/// or nothing where the endpoint is not Anthropic's own API or the model has
/// no web tools. The basic versions run without code execution.
#[must_use]
pub fn tools_for(model: &str, base_url: &str) -> Vec<Value> {
    if !base_url.contains("anthropic.com") || !has_web_tools(model) {
        return Vec::new();
    }
    vec![
        json!({ "type": "web_search_20250305", "name": "web_search", "max_uses": MAX_USES }),
        json!({
            "type": "web_fetch_20250910",
            "name": "web_fetch",
            "max_uses": MAX_FETCHES,
            "max_content_tokens": MAX_FETCH_TOKENS,
        }),
    ]
}

/// Follows the stream's web blocks: the input of each `server_tool_use`
/// (built from its `input_json_delta`s) until its result arrives.
#[derive(Debug, Default)]
pub struct SearchSteps {
    /// Open `server_tool_use` blocks: stream index → (tool use id, tool, input JSON so far).
    open: HashMap<u64, (String, String, String)>,
    /// Finished inputs by tool use id (the query or the address), waiting for their result.
    queries: HashMap<String, String>,
    /// A text block was already streamed in this answer.
    text_seen: bool,
    /// A tool block came after the last text block.
    tool_since_text: bool,
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

/// The part of a tool's input a step shows: the query or the address.
fn input_text(tool: &str, input: &Value) -> String {
    let key = if tool == "web_fetch" { "url" } else { "query" };
    text(input, key).to_string()
}

impl SearchSteps {
    /// The paragraph break owed before a text block that follows a tool
    /// block: the narration before and after a tool round otherwise ran
    /// together ("…descrição.Ambiente…"). Text blocks split only by
    /// citations stay joined.
    pub fn paragraph_break(&mut self, data: &str) -> Option<String> {
        if !data.contains("content_block_start") {
            return None;
        }
        let event: Value = serde_json::from_str(data).ok()?;
        if text(&event, "type") != "content_block_start" {
            return None;
        }
        match text(event.get("content_block")?, "type") {
            "text" => {
                let owed = self.text_seen && self.tool_since_text;
                self.text_seen = true;
                self.tool_since_text = false;
                owed.then(|| "

".to_string())
            }
            "thinking" | "redacted_thinking" => None,
            _ => {
                self.tool_since_text = true;
                None
            }
        }
    }

    /// Reads one SSE `data:` payload; returns the marker chunk when a search
    /// or a page read finished.
    pub fn on_event(&mut self, data: &str) -> Option<String> {
        if !data.contains("server_tool_use") && !data.contains("_tool_result") && self.open.is_empty() {
            return None;
        }
        let event: Value = serde_json::from_str(data).ok()?;
        let index = event.get("index").and_then(Value::as_u64);
        match text(&event, "type") {
            "content_block_start" => {
                let block = event.get("content_block")?;
                match text(block, "type") {
                    "server_tool_use" => {
                        let tool = text(block, "name");
                        if SANDBOX_TOOLS.contains(&tool) {
                            warn!("claude: the model ran {tool} in Anthropic's sandbox; it is not offered");
                            return None;
                        }
                        if tool != "web_search" && tool != "web_fetch" {
                            return None;
                        }
                        let id = text(block, "id").to_string();
                        let given = block.get("input").map(|input| input_text(tool, input)).unwrap_or_default();
                        if given.is_empty() {
                            self.open.insert(index?, (id, tool.to_string(), String::new()));
                        } else {
                            self.queries.insert(id, given);
                        }
                        None
                    }
                    "web_search_tool_result" => Some(self.search_result(block)),
                    "web_fetch_tool_result" => Some(self.fetch_result(block)),
                    _ => None,
                }
            }
            "content_block_delta" => {
                let delta = event.get("delta")?;
                if text(delta, "type") == "input_json_delta" {
                    if let Some((_, _, json)) = self.open.get_mut(&index?) {
                        json.push_str(text(delta, "partial_json"));
                    }
                }
                None
            }
            "content_block_stop" => {
                let (id, tool, input) = self.open.remove(&index?)?;
                let given = serde_json::from_str::<Value>(&input)
                    .ok()
                    .map(|value| input_text(&tool, &value))
                    .unwrap_or_default();
                self.queries.insert(id, given);
                None
            }
            _ => None,
        }
    }

    /// The marker for one `web_search_tool_result`: a list of results, or an
    /// error object (HTTP 200 with `error_code`).
    fn search_result(&mut self, block: &Value) -> String {
        let query = self.queries.remove(text(block, "tool_use_id")).unwrap_or_default();
        let content = block.get("content");
        let urls: Vec<&str> = content
            .and_then(Value::as_array)
            .map(|items| items.iter().map(|item| text(item, "url")).filter(|url| !url.is_empty()).collect())
            .unwrap_or_default();
        let ok = content.is_some_and(Value::is_array);
        json!({ WEB_STEP_MARKER: { "tool": "web_search", "text": query, "urls": urls, "ok": ok } }).to_string()
    }

    /// The marker for one `web_fetch_tool_result`: a `web_fetch_result`, or a
    /// `web_fetch_tool_result_error`.
    fn fetch_result(&mut self, block: &Value) -> String {
        let asked = self.queries.remove(text(block, "tool_use_id")).unwrap_or_default();
        let content = block.get("content").cloned().unwrap_or_default();
        let ok = text(&content, "type") == "web_fetch_result";
        let read = text(&content, "url");
        let url = if read.is_empty() { asked } else { read.to_string() };
        let urls: Vec<&str> = if url.is_empty() { Vec::new() } else { vec![url.as_str()] };
        json!({ WEB_STEP_MARKER: { "tool": "web_fetch", "text": url, "urls": urls, "ok": ok } }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const API: &str = "https://api.anthropic.com";

    #[test]
    fn only_the_basic_web_tools_are_offered_and_only_on_anthropic() {
        let tools = tools_for("claude-fable-5-2", API);
        let types: Vec<Value> = tools.iter().map(|t| t["type"].clone()).collect();
        assert_eq!(types, vec![json!("web_search_20250305"), json!("web_fetch_20250910")]);
        assert_eq!(tools[0]["max_uses"], json!(MAX_USES));
        assert_eq!(tools[1]["max_uses"], json!(MAX_FETCHES));
        assert_eq!(tools_for("claude-opus-5-5", API).len(), 2);
        assert_eq!(tools_for("claude-haiku-4-5", API).len(), 2);
        assert!(tools_for("claude-3-haiku-20240307", API).is_empty());
        assert!(tools_for("claude-opus-5-5", "https://example.openai.azure.com").is_empty());
        let offered = serde_json::to_string(&tools_for("claude-sonnet-5-5", API)).unwrap_or_default();
        assert!(!offered.contains("20260209"));
        assert!(!offered.contains("code_execution"));
    }

    #[test]
    fn a_streamed_page_read_becomes_one_fetch_marker() {
        let mut steps = SearchSteps::default();
        let events = [
            r#"{"type":"content_block_start","index":1,"content_block":{"type":"server_tool_use","id":"srvtoolu_9","name":"web_fetch","input":{}}}"#,
            r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"url\": \"https://example.com/a\"}"}}"#,
            r#"{"type":"content_block_stop","index":1}"#,
        ];
        for event in events {
            assert!(steps.on_event(event).is_none());
        }
        let result = r#"{"type":"content_block_start","index":2,"content_block":{"type":"web_fetch_tool_result","tool_use_id":"srvtoolu_9","content":{"type":"web_fetch_result","url":"https://example.com/a","content":{"type":"document","source":{"type":"text","media_type":"text/plain","data":"x"}}}}}"#;
        let marker: Value = steps.on_event(result).and_then(|m| serde_json::from_str(&m).ok()).unwrap_or_default();
        assert_eq!(marker[WEB_STEP_MARKER]["tool"], "web_fetch");
        assert_eq!(marker[WEB_STEP_MARKER]["text"], "https://example.com/a");
        assert_eq!(marker[WEB_STEP_MARKER]["ok"], true);
    }

    #[test]
    fn a_failed_page_read_keeps_the_asked_address() {
        let mut steps = SearchSteps::default();
        let start = r#"{"type":"content_block_start","index":1,"content_block":{"type":"server_tool_use","id":"t2","name":"web_fetch","input":{"url":"https://example.org/x"}}}"#;
        assert!(steps.on_event(start).is_none());
        let error = r#"{"type":"content_block_start","index":2,"content_block":{"type":"web_fetch_tool_result","tool_use_id":"t2","content":{"type":"web_fetch_tool_result_error","error_code":"url_not_accessible"}}}"#;
        let marker: Value = steps.on_event(error).and_then(|m| serde_json::from_str(&m).ok()).unwrap_or_default();
        assert_eq!(marker[WEB_STEP_MARKER]["text"], "https://example.org/x");
        assert_eq!(marker[WEB_STEP_MARKER]["ok"], false);
    }

    #[test]
    fn text_after_a_tool_round_starts_a_new_paragraph() {
        let mut steps = SearchSteps::default();
        let text0 = r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#;
        let cited = r#"{"type":"content_block_start","index":1,"content_block":{"type":"text","text":""}}"#;
        let tool = r#"{"type":"content_block_start","index":2,"content_block":{"type":"server_tool_use","id":"t","name":"web_search","input":{}}}"#;
        let text3 = r#"{"type":"content_block_start","index":3,"content_block":{"type":"text","text":""}}"#;
        assert!(steps.paragraph_break(text0).is_none());
        assert!(steps.paragraph_break(cited).is_none());
        assert!(steps.paragraph_break(tool).is_none());
        assert_eq!(steps.paragraph_break(text3).as_deref(), Some("

"));
        let delta = r#"{"type":"content_block_delta","index":3,"delta":{"type":"text_delta","text":"x"}}"#;
        assert!(steps.paragraph_break(delta).is_none());
    }

    #[test]
    fn sandbox_code_blocks_leave_no_step() {
        let mut steps = SearchSteps::default();
        let start = r#"{"type":"content_block_start","index":4,"content_block":{"type":"server_tool_use","id":"t3","name":"bash_code_execution","input":{}}}"#;
        assert!(steps.on_event(start).is_none());
        assert!(steps.open.is_empty());
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
