//! `agent_loop::parsing` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) const MAX_EMPTY_PARSE_RETRIES: u32 = 3;

impl AgentLoop {
    /// Conservative mapping of a JSON value to a tool call: the name must
    /// come with arguments/parameters, or look like a canonical vibe tool
    /// name (`/`-separated) — arbitrary `{"name": ...}` JSON is not a call.
    pub(crate) fn call_from_value(v: &serde_json::Value) -> Option<ExtractedToolCall> {
        let name = v
            .get("tool_name")
            .and_then(|x| x.as_str())
            .or_else(|| v.get("name").and_then(|x| x.as_str()))?;
        if name.is_empty() {
            return None;
        }
        let args = v
            .get("arguments")
            .cloned()
            .or_else(|| v.get("parameters").cloned());
        let looks_like_tool = args.is_some() || name.contains('/');
        if !looks_like_tool {
            return None;
        }
        Some(ExtractedToolCall {
            tool_name: restore_tool_name_from_llm(name),
            arguments: args.unwrap_or_else(|| serde_json::json!({})),
        })
    }
}

pub(crate) struct ExtractedToolCall {
    pub(crate) tool_name: String,
    pub(crate) arguments: serde_json::Value,
}

/// Serializes native tool calls back into the canonical JSON envelope that
/// `parse_tool_calls` consumes downstream.
pub(crate) fn canonical_tool_calls_json(calls: &[ExtractedToolCall]) -> String {
    let body = serde_json::json!({
        "tool_calls": calls.iter().map(|c| serde_json::json!({
            "tool_name": c.tool_name,
            "arguments": c.arguments,
        })).collect::<Vec<_>>(),
    });
    body.to_string()
}

/// Extract the first balanced bracketed block (`[ ... ]`) from `s`,
/// respecting strings and escapes — the array counterpart of
/// `extract_json_object`. Used for text-embedded tool calls (#1276).
pub(crate) fn extract_json_array(s: &str) -> Option<String> {
    let start = s.find('[')?;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;

    for (i, ch) in s[start..].char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(s[start..=start + i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn extract_json_object(s: &str) -> Option<String> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let mut start = None;

    for (i, ch) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = start {
                        return Some(s[start..=i].to_string());
                    }
                }
            }
            _ => {}
        }
    }
    None
}
