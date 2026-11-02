//! Tolerant parsing for native tool arguments (#1582/#1584 follow-up).
//!
//! Models routinely put a multi-line program inside the `code` field and emit
//! raw newlines/tabs instead of `\n`/`\t`. `serde_json` rejects those control
//! characters inside a string, so a perfectly good program used to fail with
//! "Could not read the execution request". This module repairs the argument
//! text before parsing and reports truncation distinctly.

use serde_json::Value;

/// Escapes control characters that appear *inside* JSON string literals while
/// leaving the surrounding structure untouched. A raw newline becomes `\n`, a
/// tab becomes `\t`, and any other C0 control byte is dropped.
fn escape_control_chars_in_strings(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 16);
    let mut in_string = false;
    let mut escaped = false;
    for ch in raw.chars() {
        if in_string {
            if escaped {
                out.push(ch);
                escaped = false;
                continue;
            }
            match ch {
                '\\' => {
                    out.push(ch);
                    escaped = true;
                }
                '"' => {
                    out.push(ch);
                    in_string = false;
                }
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => {}
                c => out.push(c),
            }
        } else {
            if ch == '"' {
                in_string = true;
            }
            out.push(ch);
        }
    }
    out
}

/// Reads a quoted field starting at `from` (the index just past the opening
/// quote) and returns (unescaped value, index just past the closing quote).
/// The scan stops at the first unescaped quote, which is exact for short
/// scalar fields such as `language`.
fn read_quoted(raw: &str, from: usize) -> Option<(String, usize)> {
    let mut out = String::new();
    let mut chars = raw[from..].char_indices();
    while let Some((idx, ch)) = chars.next() {
        match ch {
            '\\' => {
                let (_, next) = chars.next()?;
                match next {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    other => out.push(other),
                }
            }
            '"' => return Some((out, from + idx + 1)),
            c => out.push(c),
        }
    }
    None
}

/// The `code` field is a whole program: models often emit it with real
/// newlines AND unescaped double quotes (`print("hi")`), which breaks the JSON
/// structure even after control-character repair. The field is written last in
/// the `vm_exec` schema, so the closing quote is the last `"` before the final
/// `}`. Everything between is the program, with JSON escapes resolved.
fn salvage_code_field(raw: &str) -> Option<String> {
    let key = raw.find("\"code\"")?;
    let colon = raw[key..].find(':')? + key;
    let open = raw[colon..].find('"')? + colon;
    let body = &raw[open + 1..];

    let end = body.rfind('"')?;
    // Prefer the last quote that is followed only by whitespace and `}`;
    // fall back to the last quote when trailing fields follow the program.
    let mut close = None;
    for (idx, _) in body.rmatch_indices('"') {
        let tail = body[idx + 1..].trim_end();
        if tail.is_empty() || tail == "}" {
            close = Some(idx);
            break;
        }
        if tail.starts_with(',') {
            close = Some(idx);
            break;
        }
    }
    let close = close.unwrap_or(end);
    let mut out = String::new();
    let slice = &body[..close];
    let mut chars = slice.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    Some(out)
}

fn salvage_tool_args(raw: &str) -> Option<Value> {
    let code = salvage_code_field(raw)?;
    let language = raw
        .find("\"language\"")
        .and_then(|key| raw[key..].find(':').map(|c| key + c + 1))
        .and_then(|after| raw[after..].find('"').map(|q| after + q + 1))
        .and_then(|start| read_quoted(raw, start).map(|(value, _)| value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "python".to_string());
    let mut object = serde_json::Map::new();
    object.insert("language".to_string(), Value::String(language));
    object.insert("code".to_string(), Value::String(code));
    Some(Value::Object(object))
}

/// Parses native tool arguments, repairing raw control characters. The error
/// string distinguishes a truncated request (the provider cut the stream) from
/// a genuinely malformed one, so the model knows to send a smaller program.
pub fn parse_args(raw: &str) -> Result<Value, String> {
    match serde_json::from_str::<Value>(raw) {
        Ok(value) => return Ok(value),
        Err(first) => {
            let repaired = escape_control_chars_in_strings(raw);
            if repaired != raw {
                if let Ok(value) = serde_json::from_str::<Value>(&repaired) {
                    return Ok(value);
                }
            }
            // Programs with unescaped quotes inside the code string break the
            // JSON structure itself; recover the program text directly.
            if let Some(value) = salvage_tool_args(&repaired) {
                return Ok(value);
            }
            let looks_truncated = first.to_string().contains("EOF while parsing")
                || first.to_string().contains("end of input");
            if looks_truncated {
                Err(format!(
                    "the tool arguments were truncated after {} bytes (the model output was cut \
                     before the JSON closed). Send the program again in one shorter `vm_exec` \
                     call, or write it to a file first and run the file.",
                    raw.len()
                ))
            } else {
                Err(first.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    #[test]
    fn accepts_raw_newlines_inside_the_program() {
        let raw = "{\"language\":\"python\",\"code\":\"import os\nprint('hi')\n\"}";
        let value = parse_args(raw).expect("repaired");
        assert_eq!(value["code"], "import os\nprint('hi')\n");
    }

    #[test]
    fn accepts_raw_tabs_and_keeps_escaped_quotes() {
        let raw = "{\"code\":\"if True:\n\tprint(\\\"ok\\\")\"}";
        let value = parse_args(raw).expect("repaired");
        assert_eq!(value["code"], "if True:\n\tprint(\"ok\")");
    }

    #[test]
    fn parses_well_formed_arguments_unchanged() {
        let value = parse_args("{\"code\":\"print(1)\"}").expect("valid");
        assert_eq!(value["code"], "print(1)");
    }

    #[test]
    fn reports_truncation() {
        let err = parse_args("{\"language\":\"python\",\"code\":\"print(1)").expect_err("truncated");
        assert!(err.contains("truncated"), "got: {err}");
    }

    #[test]
    fn recovers_program_with_unescaped_quotes() {
        // Real model output: the program carries `print("...")` without
        // escaping the inner quotes, so the JSON is structurally broken.
        let raw = "{\"language\":\"python\",\"code\":\"from reportlab.pdfgen import canvas\nc = canvas.Canvas(\\\"out.pdf\\\")\nc.drawString(72, 720, \\\"Hello\\\")\nprint(\\\"done\\\")\"}";
        let value = parse_args(raw).expect("salvaged");
        assert_eq!(value["language"], "python");
        let code = value["code"].as_str().unwrap_or_default();
        assert!(code.contains("canvas.Canvas(\"out.pdf\")"), "got: {code}");
        assert!(code.contains("print(\"done\")"), "got: {code}");
        assert!(!code.contains("\\\""), "escapes must be resolved: {code}");
    }

    #[test]
    fn recovers_program_with_raw_quotes_and_newlines() {
        let raw = "{\"language\": \"python\", \"code\": \"line1 = \\\"a\\\"\nline2 = \\\"b\\\"\nprint(line1)\"}";
        let value = parse_args(raw).expect("salvaged");
        let code = value["code"].as_str().unwrap_or_default();
        assert_eq!(code, "line1 = \"a\"\nline2 = \"b\"\nprint(line1)");
    }
}
