//! `agent_loop::llm_6` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Accumulates an SSE body (`data: {...}` lines) into a single JSON payload
/// with the assistant `content`/`tool_calls` merged (Issue #794). Returns
/// `None` when the body contains no SSE events.
pub(crate) fn parse_sse(body: &str) -> Option<serde_json::Value> {
    let mut message = serde_json::json!({"role": "assistant", "content": "", "tool_calls": []});
    let mut first_seen = false;
    let mut usage: Option<serde_json::Value> = None;
    for line in body.lines() {
        let Some(data) = line.trim().strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data == "[DONE]" {
            first_seen = true;
            break;
        }
        let Ok(event) = serde_json::from_str::<serde_json::Value>(data) else {
            continue;
        };
        first_seen = true;
        if let Some(u) = event.get("usage").filter(|u| !u.is_null()) {
            usage = Some(u.clone());
        }
        let delta = &event["choices"][0]["delta"];
        if let Some(text) = delta["content"].as_str() {
            message["content"] = serde_json::Value::String(
                message["content"].as_str().unwrap_or_default().to_string() + text,
            );
        }
        if let Some(deltas) = delta["tool_calls"].as_array() {
            let Some(calls) = message["tool_calls"].as_array_mut() else {
                continue;
            };
            for delta_call in deltas {
                let index = delta_call["index"].as_u64().unwrap_or(0) as usize;
                while calls.len() <= index {
                    calls.push(serde_json::json!({
                        "id": "", "type": "function",
                        "function": {"name": "", "arguments": ""}
                    }));
                }
                let entry = &mut calls[index];
                if let Some(id) = delta_call["id"].as_str() {
                    if !id.is_empty() {
                        entry["id"] = serde_json::Value::String(id.to_string());
                    }
                }
                if let Some(name) = delta_call["function"]["name"].as_str() {
                    if !name.is_empty() {
                        let current = entry["function"]["name"].as_str().unwrap_or_default();
                        entry["function"]["name"] =
                            serde_json::Value::String(current.to_string() + name);
                    }
                }
                if let Some(args) = delta_call["function"]["arguments"].as_str() {
                    let current = entry["function"]["arguments"].as_str().unwrap_or_default();
                    entry["function"]["arguments"] =
                        serde_json::Value::String(current.to_string() + args);
                }
            }
        }
    }
    if !first_seen {
        return None;
    }
    let mut accumulated = serde_json::Map::new();
    accumulated.insert(
        "choices".to_string(),
        serde_json::json!([{"message": message}]),
    );
    if let Some(u) = usage {
        accumulated.insert("usage".to_string(), u);
    }
    Some(serde_json::Value::Object(accumulated))
}

/// Token usage reported by an LLM provider for a single completion attempt.
#[derive(Debug, Clone, Copy, Default)]
pub struct LlmUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// Extracts provider-reported token usage from an OpenAI-style completion
/// payload (`usage.prompt_tokens` / `usage.completion_tokens`), if present.
pub(crate) fn usage_from_payload(payload: &serde_json::Value) -> Option<LlmUsage> {
    let usage = payload.get("usage")?;
    Some(LlmUsage {
        prompt_tokens: usage
            .get("prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        completion_tokens: usage
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
    })
}

/// USD cost estimate for a model from a small built-in price table (per 1M
/// tokens). Unknown models fall back to `VIBE_LLM_COST_PER_1M_TOKENS` (default
/// 0.0) rather than fabricating a price, so a budget is never under-charged.
pub(crate) fn estimate_llm_cost(model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
    let m = model.to_ascii_lowercase();
    let (input, output): (f64, f64) = if m.contains("gpt-4o-mini") {
        (0.15, 0.60)
    } else if m.contains("gpt-4o") {
        (2.50, 10.0)
    } else if m.contains("gpt-3.5") {
        (0.50, 1.50)
    } else if m.contains("claude-3-5")
        || m.contains("claude-3-7")
        || m.contains("claude-3.5")
        || m.contains("claude-3.7")
    {
        (3.0, 15.0)
    } else if m.contains("claude") {
        (15.0, 75.0)
    } else if m.contains("llama-3.3") || m.contains("llama-3.1-70b") {
        (0.59, 0.79)
    } else if m.contains("llama") {
        (0.10, 0.30)
    } else {
        let fallback = std::env::var("VIBE_LLM_COST_PER_1M_TOKENS")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        return (prompt_tokens as f64 + completion_tokens as f64) / 1_000_000.0 * fallback;
    };
    (prompt_tokens as f64) / 1_000_000.0 * input + (completion_tokens as f64) / 1_000_000.0 * output
}

/// #1276 — provider-agnostic tool-name encoding for the LLM boundary.
///
/// Internal Vibe tool names use `/` (`file/write`, `backup/list`, …), which
/// several OpenAI-compatible providers reject outright (e.g. NVIDIA returns
/// `400 Bad Request: Function at index 0 has an invalid name`). Names are
/// sanitized to `[A-Za-z0-9_-]` on the way OUT (schemas) and restored on the
/// way IN (parsed tool calls) so the executor keeps seeing canonical names.
/// `/` is the only character used by internal names; `__` cannot collide
/// because no internal name contains a literal `__`.
pub(crate) fn sanitize_tool_name_for_llm(name: &str) -> String {
    if name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
        return name.to_string();
    }
    name.replace('/', "__")
}

pub(crate) fn restore_tool_name_from_llm(name: &str) -> String {
    if name.contains("__") {
        name.replace("__", "/")
    } else {
        name.to_string()
    }
}

/// Extracts native OpenAI-format tool calls (`message.tool_calls[].function`)
/// from a completion payload, if present.
pub(crate) fn native_tool_calls_from_value(payload: &serde_json::Value) -> Option<Vec<ExtractedToolCall>> {
    let calls = payload["choices"][0]["message"]["tool_calls"].as_array()?;
    let parsed: Vec<ExtractedToolCall> = calls
        .iter()
        .filter_map(|tc| {
            let name = tc["function"]["name"].as_str()?.to_string();
            // SSE deltas can leave a placeholder entry with no name — skip
            // it instead of forwarding an empty tool name to the executor.
            if name.is_empty() {
                return None;
            }
            let arguments = tc["function"]["arguments"]
                .as_str()
                .and_then(|a| serde_json::from_str::<serde_json::Value>(a).ok())
                .unwrap_or_else(|| serde_json::json!({}));
            Some(ExtractedToolCall {
                tool_name: restore_tool_name_from_llm(&name),
                arguments,
            })
        })
        .collect();
    if parsed.is_empty() {
        None
    } else {
        Some(parsed)
    }
}

#[cfg(test)]
pub(crate) mod tool_name_encoding_tests {
    use super::{restore_tool_name_from_llm, sanitize_tool_name_for_llm};

    #[test]
    fn nemotron_bracketed_call_block_parses() {
        // Exact shape observed from NVIDIA nemotron on dev (#1276): the call
        // is embedded in content with `name` + `parameters`, never native.
        let response = "[[ \n{\n  \"name\": \"file__write\",\n  \"parameters\": {\n    \"path\": \"index.html\",\n    \"content\": \"<html>hi</html>\"\n  }\n}\n]]";
        let calls = super::AgentLoop::parse_embedded_tool_calls(response)
            .expect("bracketed block should parse");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_name, "file/write");
        assert_eq!(calls[0].arguments["path"], "index.html");
    }

    #[test]
    fn bare_array_of_calls_parses() {
        let response =
            "[{\"tool_name\":\"file/list\",\"arguments\":{\"project\":\"p\"}}]";
        let calls = super::AgentLoop::parse_embedded_tool_calls(response)
            .expect("bare array should parse");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_name, "file/list");
    }

    #[test]
    fn plain_name_without_arguments_is_not_a_call() {
        // Arbitrary `{"name": ...}` JSON must not be mistaken for a call.
        assert!(super::AgentLoop::parse_embedded_tool_calls(
            "{\"name\": \"John\", \"age\": 30}"
        )
        .is_none());
    }

    #[test]
    fn prose_without_json_is_not_a_call() {
        assert!(super::AgentLoop::parse_embedded_tool_calls(
            "I will now write the file for you."
        )
        .is_none());
    }

    #[test]
    fn creation_intents_require_mutation() {
        // #1276 — create/build intents must be treated as mutations so a
        // prose-only reply cannot complete the run without writing anything.
        for intent in [
            "create",
            "Create a small bakery landing page",
            "build me a units converter",
            "generate an index.html",
            "scaffold a python service",
        ] {
            assert!(
                super::AgentLoop::intent_requires_mutation(intent),
                "intent `{intent}` should require mutation"
            );
        }
    }

    #[test]
    fn read_only_intents_still_do_not_require_mutation() {
        for intent in ["list the files", "explain the project", "review my code"] {
            assert!(
                !super::AgentLoop::intent_requires_mutation(intent),
                "intent `{intent}` should not require mutation"
            );
        }
    }

    #[test]
    fn slashes_are_encoded_for_the_wire() {
        assert_eq!(sanitize_tool_name_for_llm("file/write"), "file__write");
        assert_eq!(sanitize_tool_name_for_llm("backup/list"), "backup__list");
        assert_eq!(sanitize_tool_name_for_llm("publish/project"), "publish__project");
    }

    #[test]
    fn plain_names_pass_through_unchanged() {
        assert_eq!(sanitize_tool_name_for_llm("shell_run"), "shell_run");
        assert_eq!(sanitize_tool_name_for_llm("git-status"), "git-status");
    }

    #[test]
    fn round_trip_restores_canonical_name() {
        for canonical in ["file/write", "file/replace", "backup/list", "domain/bind"] {
            let wire = sanitize_tool_name_for_llm(canonical);
            assert!(wire.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'));
            assert_eq!(restore_tool_name_from_llm(&wire), canonical);
        }
    }

    #[test]
    fn restore_ignores_plain_names() {
        assert_eq!(restore_tool_name_from_llm("write_file"), "write_file");
    }
}
