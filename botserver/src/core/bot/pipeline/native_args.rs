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
}
