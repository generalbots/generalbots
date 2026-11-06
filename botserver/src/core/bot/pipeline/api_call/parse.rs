//! Locating and stripping the model's `{"__api_call__": …}` JSON block.

/// Extracts the first `{"__api_call__": {...}}` object from a response,
/// returning its payload (the object's `__api_call__` value, or the whole
/// object when it is not nested).
pub(super) fn extract_api_call_payload(full_response: &str) -> Option<serde_json::Value> {
    use crate::core::bot::api_catalog::find_api_call;
    let pos = find_api_call(full_response)?;
    let obj_start = full_response[..pos].rfind('{')?;
    let bytes = full_response.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    for (i, &b) in bytes[obj_start..].iter().enumerate() {
        match b {
            b'"' => in_string = !in_string,
            b'{' if !in_string => depth += 1,
            b'}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    let end = obj_start + i + 1;
                    let parsed: serde_json::Value = match serde_json::from_str(&full_response[obj_start..end]) {
                        Ok(v) => v,
                        Err(_) => return None,
                    };
                    let inner = parsed.get("__api_call__").or_else(|| parsed.get("api_call")).or_else(|| parsed.get("_api_call_")).cloned().unwrap_or(parsed);
                    return Some(inner);
                }
            }
            _ => {}
        }
    }
    None
}

/// Removes the `{"__api_call__": ...}` JSON block (the model's tool-call
/// preamble) from a response, leaving just the user-facing prose before and
/// after it. The block is matched with brace balancing so it is stripped in
/// full no matter which key variant the model emitted.
pub(super) fn strip_api_call_block(text: &str) -> String {
    use crate::core::bot::api_catalog::find_api_call;
    let Some(pos) = find_api_call(text) else {
        return text.to_string();
    };
    let Some(obj_start) = text[..pos].rfind('{') else {
        return text.to_string();
    };
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut end = None;
    for (i, &b) in bytes[obj_start..].iter().enumerate() {
        match b {
            b'"' => in_string = !in_string,
            b'{' if !in_string => depth += 1,
            b'}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    end = Some(obj_start + i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    match end {
        Some(e) => {
            let mut out = String::with_capacity(text.len());
            out.push_str(&text[..obj_start]);
            out.push_str(&text[e..]);
            out
        }
        None => text.to_string(),
    }
}
