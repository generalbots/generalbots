//! Locating and stripping the model's `{"__api_call__": …}` JSON block.

/// Extracts the first `{"__api_call__": {...}}` object from a response,
/// returning its payload (the object's `__api_call__` value, or the whole
/// object when it is not nested).
pub(super) fn extract_api_call_payload(full_response: &str) -> Option<serde_json::Value> {
    use crate::core::bot::api_catalog::find_api_call;
    let pos = find_api_call(full_response)?;
    // #2583 — a model often drops the outer braces and emits only
    // `"__api_call__": {…}`. Treat the inner object as the payload in that
    // case instead of rejecting the call and letting the prose stand in for
    // its result.
    let inner_start = pos + full_response[pos..].find('{')?;
    let parsed = match full_response[..pos].rfind('{') {
        Some(obj_start) => parse_object(full_response, obj_start).or_else(|| {
            parse_object(full_response, inner_start)
        })?,
        None => parse_object(full_response, inner_start)?,
    };
    let inner = parsed
        .get("__api_call__")
        .or_else(|| parsed.get("api_call"))
        .or_else(|| parsed.get("_api_call_"))
        .cloned()
        .unwrap_or(parsed);
    Some(inner)
}

/// Removes the `{"__api_call__": ...}` JSON block (the model's tool-call
/// preamble) from a response, leaving just the user-facing prose before and
/// after it. The block is matched with brace balancing so it is stripped in
/// full no matter which key variant the model emitted.
pub(crate) fn strip_api_call_block(text: &str) -> String {
    // A turn can carry more than one envelope (the model re-emits after a
    // consent round-trip); strip every one of them, keeping surrounding prose.
    let mut current = text.to_string();
    for _ in 0..8 {
        let next = strip_first_api_call_block(&current);
        if next == current {
            break;
        }
        current = next;
    }
    current
}

fn strip_first_api_call_block(text: &str) -> String {
    use crate::core::bot::api_catalog::find_api_call;
    let Some(pos) = find_api_call(text) else {
        return text.to_string();
    };
    let Some(inner_start) = text[pos..].find('{').map(|offset| pos + offset) else {
        return text.to_string();
    };
    let outer_start = text[..pos].rfind('{');
    let scan_start = outer_start.unwrap_or(inner_start);
    // #2583 — the block was cut off mid-object (the model hit the token
    // cap inside a large payload), so it never closes. Everything from the
    // opening brace is machine payload: drop it instead of showing JSON.
    let Some(end) = object_end(text, scan_start) else {
        let from = outer_start.unwrap_or(pos);
        return text[..from].trim_end().to_string();
    };
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..outer_start.unwrap_or(pos)]);
    let mut tail = end;
    if outer_start.is_none() {
        // With no outer brace the model may still close one after the inner
        // object; never leave that stray `}` in the answer.
        if text[tail..].trim_start().starts_with('}') {
            tail += text[tail..].find('}').unwrap_or(0) + 1;
        }
    }
    out.push_str(&text[tail..]);
    out
}

/// Parses one balanced JSON object starting at `start`.
fn parse_object(text: &str, start: usize) -> Option<serde_json::Value> {
    let end = object_end(text, start)?;
    serde_json::from_str(&text[start..end]).ok()
}

/// Byte index just after the `}` closing the object opened at `start`.
fn object_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (i, &b) in bytes[start..].iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod parse_tests {
    use super::{extract_api_call_payload, strip_api_call_block};

    #[test]
    fn extracts_a_normal_envelope() {
        let text = "{\"__api_call__\": {\"name\": \"products.items.list\", \"params\": {}}}\nnote";
        let payload = extract_api_call_payload(text).unwrap_or_default();
        assert_eq!(payload["name"], "products.items.list");
        assert_eq!(strip_api_call_block(text).trim(), "note");
    }

    #[test]
    fn extracts_an_envelope_without_outer_braces() {
        // #2583 — the shape the live model emitted before the price was
        // narrated as if the write had succeeded.
        let text = "\"__api_call__\": {\"name\": \"products.items.update\"}\nI changed it.";
        let payload = extract_api_call_payload(text).unwrap_or_default();
        assert_eq!(payload["name"], "products.items.update");
        assert_eq!(strip_api_call_block(text).trim(), "I changed it.");
    }

    #[test]
    fn drops_an_unterminated_envelope() {
        let text = "{\"__api_call__\": {\"name\": \"sheet.cells.set\", \"params\": {\"cells\": ";
        assert!(strip_api_call_block(text).trim().is_empty());
        assert!(extract_api_call_payload(text).is_none());
    }
}
