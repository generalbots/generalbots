//! Kiro (kiro.dev) transport for the Vibe agent loop.
//!
//! When the central LLM config (`secret/gbo/llm`) names the Kiro provider,
//! the agent loop speaks Kiro's CodeWhisperer protocol instead of OpenAI
//! SSE: `X-Amz-Target:
//! AmazonCodeWhispererStreamingService.GenerateAssistantResponse` with
//! `tokentype: API_KEY` auth, tool specs under `toolSpecification`, and a
//! binary AWS Event Stream envelope with embedded JSON events
//! (`{"content":…}`, `{"toolUseId":…,"name":…,"input":…}`).

use serde_json::{json, Value};
use std::time::Duration;

/// Re-exported usage type from the agent loop (kept crate-private there).
use crate::agent_loop::LlmUsage;

const DEFAULT_ENDPOINT: &str = "https://q.us-east-1.amazonaws.com/";
const TARGET_STREAMING: &str = "AmazonCodeWhispererStreamingService.GenerateAssistantResponse";
const ORIGIN: &str = "AI_EDITOR";
const EVENT_PATTERNS: [&str; 11] = [
    "{\"content\":", "{\"name\":", "{\"input\":", "{\"stop\":",
    "{\"contextUsagePercentage\":", "{\"followupPrompt\":", "{\"usage\":",
    "{\"toolUseId\":", "{\"unit\":", "{\"error\":", "{\"Error\":",
];

/// True when the resolved api_url/model belongs to the Kiro provider.
pub fn is_kiro(api_url: &str) -> bool {
    let u = api_url.to_lowercase();
    u.contains("kiro") || u.contains("q.us-east-1.amazonaws.com")
        || u.contains("codewhisperer") || u.contains("q.eu-central-1.amazonaws.com")
}

/// Normalizes a model name to a Kiro catalog id (Claude family + open MoE).
fn kiro_model_id(model: &str) -> String {
    let m = model.to_lowercase();
    if m.contains("opus") {
        "claude-opus-5".to_string()
    } else if m.contains("haiku") {
        "claude-haiku-4-5".to_string()
    } else if m.contains("sonnet-4-5") || m.contains("sonnet-4.5") {
        "claude-sonnet-4-5".to_string()
    } else if m.contains("sonnet") {
        "claude-sonnet-4-6".to_string()
    } else if m.contains("glm") {
        "GLM-5".to_string()
    } else if m.contains("deepseek") {
        "DeepSeek-V3.2".to_string()
    } else if m.contains("qwen") {
        "Qwen3-Coder-Next".to_string()
    } else if m.contains("minimax") {
        "MiniMax-M2.5".to_string()
    } else {
        "auto".to_string()
    }
}

/// OpenAI `{"type":"function","function":{…}}` schema → Kiro toolSpecification.
fn convert_tools(tools: &[Value]) -> Vec<Value> {
    tools
        .iter()
        .filter_map(|t| {
            let f = t.get("function")?;
            Some(json!({
                "toolSpecification": {
                    "name": f.get("name")?,
                    "description": f.get("description").cloned().unwrap_or(json!("")),
                    "inputSchema": { "json": f.get("parameters").cloned().unwrap_or(json!({})) },
                }
            }))
        })
        .collect()
}

/// Builds the GenerateAssistantResponse request. System prompt is folded
/// into the current user message (Kiro has no system role); history keeps
/// strict user/assistant alternation.
fn build_request(system: &str, prompt: &str, tools: &[Value], model: &str) -> Value {
    let model_id = kiro_model_id(model);
    let mut current = String::new();
    if !system.is_empty() {
        current.push_str(system);
        current.push_str("\n\n");
    }
    current.push_str(prompt);

    let spec = json!({
        "conversationState": {
            "chatTriggerType": "MANUAL",
            "agentTaskType": "vibe",
            "currentMessage": {
                "userInputMessage": {
                    "content": current,
                    "modelId": model_id,
                    "origin": ORIGIN,
                    "userInputMessageContext": {
                        "tools": convert_tools(tools),
                    },
                }
            },
        },
        "agentMode": "vibe",
    });
    spec
}

/// Finds the matching `}` for the `{` at `start` (brace-balanced, string-aware).
fn find_json_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        let c = b as char;
        if escape {
            escape = false;
            continue;
        }
        match c {
            '\\' if in_string => escape = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Accumulates Kiro events into (assistant_text, tool_calls) matching the
/// canonical tool-call envelope the agent loop's `parse_tool_calls` expects.
#[derive(Default)]
struct EventSink {
    content: String,
    tool_calls: Vec<(String, String)>, // (name, raw input json)
    current_tool: Option<(String, String)>,
    error: Option<String>,
}

impl EventSink {
    fn feed(&mut self, raw: &str) {
        let Ok(parsed) = serde_json::from_str::<Value>(raw) else {
            return;
        };
        if let Some(c) = parsed.get("content").and_then(|c| c.as_str()) {
            // Flush a completed tool before more content arrives.
            self.flush_tool();
            self.content.push_str(c);
        } else if parsed.get("name").and_then(|n| n.as_str()).is_some() {
            // Tool start: {"toolUseId":…,"name":…,"input":…}. Input may be
            // absent on the start event and streamed in follow-up chunks.
            self.flush_tool();
            let name = parsed["name"].as_str().unwrap_or_default().to_string();
            let args = match parsed.get("input") {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => String::new(),
            };
            self.current_tool = Some((name, args));
        } else if let Some(input) = parsed.get("input").cloned() {
            // Continuation chunk of the current tool's arguments.
            if let Some((name, args)) = self.current_tool.as_mut() {
                let _ = name;
                match input {
                    Value::String(s) => args.push_str(&s),
                    v => args.push_str(&v.to_string()),
                }
            }
        } else if let Some(err) = parsed.get("error").or_else(|| parsed.get("Error")) {
            let msg = match err.as_str() {
                Some(s) => s.to_string(),
                None => err.to_string(),
            };
            // Capacity pressure is retryable upstream; surface everything.
            self.error = Some(msg);
        }
    }

    fn flush_tool(&mut self) {
        if let Some((name, args)) = self.current_tool.take() {
            self.tool_calls.push((name, args));
        }
    }

    /// True when the sink holds either content or a usable tool call.
    fn has_output(&self) -> bool {
        !self.content.is_empty() || !self.tool_calls.is_empty()
    }
}

/// Calls Kiro with the Vibe agent's system+prompt+tools and returns
/// `(content_or_canonical_tool_calls_json, usage)` — the same contract as
/// the OpenAI path in the agent loop.
pub async fn call_kiro(
    api_url: &str,
    api_key: &str,
    model: &str,
    system: &str,
    prompt: &str,
    tools: &[Value],
    max_retries: usize,
) -> Result<(String, Option<LlmUsage>), String> {
    let endpoint = if api_url.contains("amazonaws.com") || api_url.starts_with("http") {
        // Accept a bare service root; anything else falls back to the default.
        if api_url.ends_with('/') || api_url.contains("amazonaws.com") {
            api_url.to_string()
        } else {
            format!("{api_url}/")
        }
    } else {
        DEFAULT_ENDPOINT.to_string()
    };
    let body = build_request(system, prompt, tools, model);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .connect_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let mut last_error = String::new();
    for attempt in 0..=max_retries {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_millis(800 * 2u64.pow(attempt as u32 - 1))).await;
        }
        let ua = format!(
            "aws-sdk-rust/1.0.0 ua/2.1 os/other lang/rust api/codewhispererstreaming#1.28.3 m/E app/AmazonQ-For-CLI md/appVersion-1.28.3-{}",
            uuid::Uuid::new_v4().simple()
        );
        let response = client
            .post(&endpoint)
            .header("Content-Type", "application/x-amz-json-1.0")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {api_key}"))
            .header("tokentype", "API_KEY")
            .header("X-Amz-Target", TARGET_STREAMING)
            .header("x-amzn-codewhisperer-optout", "true")
            .header("amz-sdk-invocation-id", uuid::Uuid::new_v4().to_string())
            .header("amz-sdk-request", "attempt=1; max=1")
            .header("x-amzn-kiro-agent-mode", "vibe")
            .header("x-amz-user-agent", &ua)
            .header("user-agent", &ua)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Kiro HTTP request failed: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            if text.contains("INSUFFICIENT_MODEL_CAPACITY") && attempt < max_retries {
                last_error = format!("Kiro capacity error: {text}");
                continue;
            }
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(format!(
                    "Kiro API key rejected ({status}) — check the ksk_ key in secret/gbo/llm: {text}"
                ));
            }
            return Err(format!("Kiro returned status {status}: {text}"));
        }

        // Consume the full body (AWS event-stream envelope with JSON events).
        let mut text = String::new();
        let mut resp = response;
        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| format!("Kiro stream error: {e}"))?
        {
            text.push_str(&String::from_utf8_lossy(&chunk));
        }

        let mut sink = EventSink::default();
        let mut pos = 0usize;
        loop {
            let mut earliest: Option<usize> = None;
            for p in EVENT_PATTERNS.iter() {
                if let Some(idx) = text[pos..].find(p) {
                    let abs = pos + idx;
                    earliest = Some(match earliest {
                        Some(e) if e <= abs => e,
                        _ => abs,
                    });
                }
            }
            let Some(start) = earliest else { break };
            let Some(end) = find_json_end(&text, start) else { break };
            let raw = &text[start..=end];
            sink.feed(raw);
            pos = end + 1;
        }
        sink.flush_tool();

        if let Some(err) = sink.error.clone() {
            if !sink.has_output() {
                return Err(format!("Kiro stream error: {err}"));
            }
        }
        if !sink.has_output() {
            last_error = "Kiro returned empty response".to_string();
            continue;
        }

        // Tool calls take priority and use the canonical envelope.
        if !sink.tool_calls.is_empty() {
            let calls: Vec<Value> = sink
                .tool_calls
                .iter()
                .filter_map(|(name, args)| {
                    let arguments: Value = serde_json::from_str(args).unwrap_or_else(|_| json!({}));
                    Some(json!({"tool_name": name, "arguments": arguments}))
                })
                .collect();
            let canonical = json!({ "tool_calls": calls }).to_string();
            return Ok((canonical, None));
        }
        return Ok((sink.content, None));
    }
    Err(last_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_kiro_urls() {
        assert!(is_kiro("https://q.us-east-1.amazonaws.com/"));
        assert!(is_kiro("kiro"));
        assert!(!is_kiro("https://integrate.api.nvidia.com/v1/chat/completions"));
    }

    #[test]
    fn builds_request_with_tools() {
        let tools = vec![json!({
            "type": "function",
            "function": {
                "name": "file/write",
                "description": "Write a file",
                "parameters": {"type": "object", "properties": {"path": {"type": "string"}}}
            }
        })];
        let body = build_request("SYS", "do it", &tools, "claude-sonnet-4-5");
        let uim = &body["conversationState"]["currentMessage"]["userInputMessage"];
        assert_eq!(uim["modelId"], json!("claude-sonnet-4-5"));
        assert_eq!(uim["origin"], json!("AI_EDITOR"));
        assert!(uim["content"].as_str().unwrap_or("").starts_with("SYS\n\ndo it"));
        let spec = &uim["userInputMessageContext"]["tools"][0]["toolSpecification"];
        assert_eq!(spec["name"], json!("file/write"));
    }

    #[test]
    fn parses_content_and_tool_events() {
        let mut sink = EventSink::default();
        sink.feed("{\"content\": \"hello \"}");
        sink.feed("{\"toolUseId\": \"t1\", \"name\": \"file/write\"}");
        sink.feed("{\"input\": \"{\\\"path\\\": \\\"a.txt\\\"}\"}");
        sink.flush_tool();
        assert_eq!(sink.content, "hello ");
        assert_eq!(sink.tool_calls.len(), 1);
        assert_eq!(sink.tool_calls[0].0, "file/write");
        let args: Value = serde_json::from_str(&sink.tool_calls[0].1).unwrap();
        assert_eq!(args["path"], json!("a.txt"));
    }

    #[test]
    fn json_end_handles_nested_and_strings() {
        let s = "noise {\"content\": \"a}b{\\\"c\\\"\"} tail";
        let start = s.find('{').unwrap();
        let end = find_json_end(s, start).unwrap();
        assert_eq!(&s[start..=end], "{\"content\": \"a}b{\\\"c\\\"\"}");
    }
}
