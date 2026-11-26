//! Reads the model's JSON tool call as a whole.
//!
//! A program inside `arguments` carries JSON of its own
//! (`print('@@GB_TRACE@@ {"task": "x"}')`). When the model misses one escape
//! there, the first balanced object closes inside the program and can still be
//! valid JSON, so the call ran a program cut at that line. The prompt asks for
//! the call on its own line: an object followed on the same line by more text,
//! with the envelope's closing quote and brace further on, closed early, and
//! the call is read up to its last brace instead.

use super::parse::{first_object_span, parse_object, salvage_tool_call};
use serde_json::Value;

/// The tool call in `raw` (text starting at the call's opening brace).
pub(super) fn read_call(raw: &str) -> Option<Value> {
    let Some((start, end)) = first_object_span(raw) else {
        return lenient_call(raw);
    };
    if let Some(whole) = closed_early(raw, start, end) {
        if let Some(call) = parse_object(whole).or_else(|| lenient_call(whole)) {
            log::warn!(
                "tool_call: the call closed inside its arguments after {} bytes; read the whole call ({} bytes)",
                end - start,
                whole.len()
            );
            return Some(call);
        }
    }
    parse_object(&raw[start..end]).or_else(|| lenient_call(raw))
}

/// The whole call when the first object at `start..end` closed in the middle
/// of a line and the text further on ends like an envelope (`"}` or `]}`
/// before the last brace).
fn closed_early(raw: &str, start: usize, end: usize) -> Option<&str> {
    let next = raw[end..].chars().next()?;
    if next.is_whitespace() || matches!(next, '{' | '`' | '<') {
        return None;
    }
    let last = raw.trim_end().rfind('}').filter(|last| *last >= end)?;
    let before = raw[..last].trim_end().trim_end_matches('}').trim_end();
    (before.ends_with('"') || before.ends_with(']')).then(|| &raw[start..=last])
}

/// A call whose JSON does not parse: the name, and the arguments as text for
/// the tool's own tolerant reader. Object arguments are kept as written, up
/// to the envelope's closing brace.
fn lenient_call(text: &str) -> Option<Value> {
    let salvaged = salvage_tool_call(text)?;
    let at = text.find("\"arguments\"")?;
    let after = text[at + "\"arguments\"".len()..]
        .trim_start()
        .strip_prefix(':')?
        .trim_start();
    if !after.starts_with('{') {
        return Some(salvaged);
    }
    let open = text.len() - after.len();
    let close = text.trim_end().rfind('}').filter(|close| *close > open)?;
    let arguments = text[open..close].trim_end();
    Some(serde_json::json!({ "name": salvaged.get("name").cloned().unwrap_or_else(|| { log::debug!("using default for salvaged name"); Default::default() }), "arguments": arguments }))
}

#[cfg(test)]
mod tests {
    use super::read_call;
    use crate::core::bot::pipeline::native_args::parse_args;
    use serde_json::Value;

    /// The program `vm_exec` would run for the call in `raw`.
    fn program(raw: &str) -> String {
        let call = read_call(raw).unwrap_or_default();
        let args = match call.get("arguments") {
            Some(Value::String(text)) => text.clone(),
            Some(other) => other.to_string(),
            None => String::new(),
        };
        let parsed = parse_args(&args).unwrap_or_default();
        parsed.get("code").and_then(Value::as_str).unwrap_or_default().to_string()
    }

    const LAST_LINE: &str = "doc.save('output/memo.docx')";

    #[test]
    fn a_well_formed_call_with_trace_lines_keeps_the_whole_program() {
        let raw = "{\"__tool_call__\": true, \"name\": \"vm_exec\", \"arguments\": {\"language\": \"python\", \"code\": \"import os\\nprint('@@GB_TRACE@@ {\\\"task\\\":\\\"x\\\"}')\\ndoc.save('output/memo.docx')\\nprint('@@GB_TRACE@@ {\\\"task\\\":\\\"x\\\",\\\"status\\\":\\\"ok\\\"}')\"}}\nVou gerar o memorando.";
        let code = program(raw);
        assert!(code.contains("print('@@GB_TRACE@@ {\"task\":\"x\"}')"), "got: {code}");
        assert!(code.contains(LAST_LINE), "got: {code}");
        assert!(code.ends_with("\"status\":\"ok\"}')"), "got: {code}");
    }

    #[test]
    fn a_missed_escape_at_the_first_trace_line_does_not_cut_the_program() {
        // `\"x"}}` closes the code string and both objects: the first object
        // is valid JSON that ends in the middle of the program's first line.
        let raw = "{\"__tool_call__\": true, \"name\": \"vm_exec\", \"arguments\": {\"language\": \"python\", \"code\": \"import os\\nprint('@@GB_TRACE@@ {\\\"task\\\":\\\"x\"}}')\\ndoc.save('output/memo.docx')\\nprint('@@GB_TRACE@@ {\\\"task\\\":\\\"x\\\",\\\"status\\\":\\\"ok\\\"}')\"}}";
        let code = program(raw);
        assert!(code.contains(LAST_LINE), "the program was cut: {code}");
        assert!(code.ends_with("\"status\":\"ok\"}')"), "got: {code}");
    }

    #[test]
    fn a_missed_escape_in_string_arguments_does_not_cut_the_program() {
        let raw = "{\"__tool_call__\": true, \"name\": \"vm_exec\", \"arguments\": \"{\\\"language\\\": \\\"python\\\", \\\"code\\\": \\\"import os\\nprint('@@GB_TRACE@@ {\\\"task\\\":\\\"x\\\"}\"}')\\ndoc.save('output/memo.docx')\\nprint(1)\\\"}\"}";
        let code = program(raw);
        assert!(code.contains(LAST_LINE), "the program was cut: {code}");
        assert!(code.ends_with("print(1)"), "got: {code}");
    }

    #[test]
    fn object_arguments_that_break_the_json_keep_their_braces() {
        // Unescaped quotes in the program: no strict parse succeeds.
        let raw = "{\"__tool_call__\": true, \"name\": \"vm_exec\", \"arguments\": {\"language\": \"python\", \"code\": \"import os\nprint('@@GB_TRACE@@ {\"task\":\"x\"}')\ndoc.save('output/memo.docx')\nprint('@@GB_TRACE@@ {\"task\":\"x\",\"status\":\"ok\"}')\"}}";
        let code = program(raw);
        assert!(code.starts_with("import os"), "got: {code}");
        assert!(code.ends_with("\"status\":\"ok\"}')"), "got: {code}");
    }

    #[test]
    fn a_call_followed_by_prose_with_braces_is_read_alone() {
        let raw = "{\"__tool_call__\": true, \"name\": \"vm_exec\", \"arguments\": {\"language\": \"python\", \"code\": \"print(1)\"}}. Depois {isso}";
        assert_eq!(program(raw), "print(1)");
    }
}
