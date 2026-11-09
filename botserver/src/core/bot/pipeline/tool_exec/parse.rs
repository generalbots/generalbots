//! Reply dialects for model tool calls.
//!
//! Providers emit the call as JSON, as a DSML/XML `invoke` block, or as a
//! `<tool_use>` element; these helpers normalize every shape into the native
//! `{"__tool_call__": true, "name": …, "arguments": …}` wrapper.

/// Extracts the first balanced JSON object from a model reply.
///
/// Models routinely embed real newlines/tabs inside the `arguments` string
/// (generated code), which `serde_json` rejects as control characters. The
/// first attempt parses the object as-is; the second escapes raw control
/// characters inside string literals and retries.
/// Native tools every bot with an agent VM can call; used to recover a tool
/// name the model left out of its JSON call.
const RECOVERABLE_TOOL_NAMES: [&str; 11] = [
    "vm_exec",
    "drive_save",
    "save_to_external_drive",
    "drive_fetch",
    "vm_file",
    "vm_files",
    "classify_media",
    "ask_user",
    "propose_plan",
    "show_surface",
    "show_app",
];

/// Recover a missing tool name from a parsed call. Providers occasionally emit
/// `{"__tool_call__": true, "arguments": …}` with no `name`, or nest the name
/// under `tool`/`tool_name`/`function`. Falling back to a known native tool
/// name found in the call text keeps the request executable instead of
/// dropping it.
pub(super) fn recover_tool_name(call: &serde_json::Value, raw: &str) -> String {
    for key in ["tool", "tool_name", "function"] {
        if let Some(value) = call.get(key) {
            if let Some(name) = value.as_str() {
                let name = name.trim();
                if !name.is_empty() {
                    return name.to_string();
                }
            }
            if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
                let name = name.trim();
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    RECOVERABLE_TOOL_NAMES
        .iter()
        .find(|name| raw.contains(&format!("\"{name}\"")))
        .map(|name| (*name).to_string())
        .unwrap_or_default()
}

pub(super) fn extract_tool_call(raw: &str) -> Option<serde_json::Value> {
    let start = raw.find('{')?;
    let slice = &raw[start..];
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut end = None;
    for (index, ch) in slice.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = Some(index + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let object = &slice[..end?];
    if let Ok(value) = serde_json::from_str(object) {
        return Some(normalize_tool_call(value));
    }

    let mut repaired = String::with_capacity(object.len());
    let mut in_string = false;
    let mut escaped = false;
    for ch in object.chars() {
        if in_string {
            if escaped {
                repaired.push(ch);
                escaped = false;
                continue;
            }
            match ch {
                '\\' => {
                    repaired.push(ch);
                    escaped = true;
                }
                '"' => {
                    repaired.push(ch);
                    in_string = false;
                }
                '\n' => repaired.push_str("\\n"),
                '\r' => repaired.push_str("\\r"),
                '\t' => repaired.push_str("\\t"),
                other => repaired.push(other),
            }
        } else {
            if ch == '"' {
                in_string = true;
            }
            repaired.push(ch);
        }
    }
    serde_json::from_str(&repaired)
        .ok()
        .map(normalize_tool_call)
}

/// Providers do not always follow the documented envelope
/// `{"__tool_call__": true, "name": …, "arguments": …}`. Some wrap the call
/// inside the marker (`{"__tool_call__": {"name": …, "arguments": …}}`), which
/// previously failed extraction and left the reply stuck on "Thinking" with no
/// message. Unwrap that shape so the call still runs.
fn normalize_tool_call(value: serde_json::Value) -> serde_json::Value {
    for key in ["__tool_call__", "tool_call"] {
        if let Some(inner) = value.get(key) {
            if inner.is_object() {
                return inner.clone();
            }
        }
    }
    value
}

/// Lenient recovery for the malformed JSON providers emit when the program
/// inside `arguments` contains unescaped quotes (common with Python
/// `print("x")`): pull the sibling `name` field and the raw `arguments`
/// payload, decoding the escapes the model did apply. Without this the reply
/// stops at "Thinking" and the user never sees the generated files.
pub(super) fn salvage_tool_call(raw: &str) -> Option<serde_json::Value> {
    let name = {
        let at = raw.find("\"name\"")?;
        let rest = &raw[at + 6..];
        let colon = rest.find(':')?;
        let after = &rest[colon + 1..];
        let start = after.find('"')? + 1;
        let end = after[start..].find('"')? + start;
        after[start..end].trim().to_string()
    };
    if name.is_empty() {
        return None;
    }
    let args = {
        let at = raw.find("\"arguments\"")?;
        let rest = &raw[at + 11..];
        let colon = rest.find(':')?;
        let after = &rest[colon + 1..];
        let start = after.find('"')? + 1;
        let tail = &after[start..];
        let end = tail.rfind('"')?;
        let payload = &tail[..end];
        let mut out = String::with_capacity(payload.len());
        let mut chars = payload.chars();
        while let Some(ch) = chars.next() {
            if ch != '\\' {
                out.push(ch);
                continue;
            }
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        }
        out
    };
    Some(serde_json::json!({ "name": name, "arguments": args }))
}

/// Parses the DSML/XML tool-call dialect some providers emit:
/// `invoke name="..."` with `parameter name="arguments"` containing the JSON
/// arguments. Returns a wrapper object in the native protocol shape.
pub(super) fn parse_dsml_tool_call(raw: &str) -> Option<serde_json::Value> {
    let invoke = raw.find("invoke name=")?;
    let rest = &raw[invoke..];
    let name_start = rest.find("name=\"")? + 6;
    let name_end = dispatch_quote_end(rest, name_start)?;
    let mut name = rest[name_start..name_end].trim().to_string();
    if name.is_empty() {
        return None;
    }
    // The envelope is sometimes named instead of the tool.
    if name == "__tool_call__" || name == "tool_call" {
        name = "vm_exec".to_string();
    }
    let args_at = rest.find("parameter name=\"arguments\"")?;
    let after = &rest[args_at..];
    let brace = after.find('{')?;
    let value = extract_tool_call(&after[brace..])?;
    Some(serde_json::json!({
        "__tool_call__": true,
        "name": name,
        "arguments": value.to_string(),
    }))
}

fn dispatch_quote_end(text: &str, start: usize) -> Option<usize> {
    text[start..].find('"').map(|pos| start + pos)
}

/// Parses the `<tool_use>{"name": …, "arguments": …}</tool_use>` dialect
/// emitted by some Kilo-hosted models. Returns the native wrapper shape.
pub(super) fn parse_xml_tool_use(raw: &str) -> Option<serde_json::Value> {
    // Two shapes reach us: `<tool_use>{"name": …}</tool_use>` and the
    // attribute dialect `<tool_use name="vm_exec">{"language": …}</tool_use>`.
    // The latter used to leak into the chat as raw text ("vm_exec">…") because
    // only the JSON-named form was recognised.
    let tag = raw.find("<tool_use")?;
    let after_tag = tag + "<tool_use".len();
    let tag_end = raw[after_tag..].find('>')? + after_tag;
    let header = &raw[after_tag..tag_end];
    let attribute_name = header
        .split_once("name=")
        .map(|(_, rest)| {
            rest.trim_matches(|c| c == '"' || c == '\'' || c == ' ')
                .to_string()
        })
        .filter(|n| !n.is_empty());
    let body_start = tag_end + 1;
    let close = raw[body_start..].find("</tool_use>")?;
    let payload = raw[body_start..body_start + close].trim();

    if let Some(value) = extract_tool_call(payload).or_else(|| salvage_tool_call(payload)) {
        if let Some(name) = value.get("name").and_then(|n| n.as_str()) {
            let name = name.trim();
            if !name.is_empty() {
                let args = value.get("arguments")?.clone();
                let args = match args {
                    serde_json::Value::String(text) => text,
                    other => other.to_string(),
                };
                return Some(serde_json::json!({
                    "__tool_call__": true,
                    "name": name,
                    "arguments": args,
                }));
            }
        }
    }
    // Attribute dialect: the body IS the arguments object.
    let name = attribute_name?;
    Some(serde_json::json!({
        "__tool_call__": true,
        "name": name,
        "arguments": payload,
    }))
}
