use serde_json::Value;

use super::state::ReplayError;

/// Applies the JSON Patch subset the surface protocol uses (`add`, `remove`,
/// `replace`). Unknown operations are rejected so a malformed patch can never
/// silently produce a different client state.
pub fn apply_patch(document: &mut Value, patch: &[Value]) -> Result<(), ReplayError> {
    for operation in patch {
        let op = operation
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| ReplayError::InvalidPayload("patch op is missing".to_string()))?;
        let path = operation
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ReplayError::InvalidPayload("patch path is missing".to_string()))?;
        if path.is_empty() {
            match op {
                "replace" | "add" => {
                    *document = operation.get("value").cloned().unwrap_or(Value::Null);
                }
                _ => {
                    return Err(ReplayError::InvalidPayload(
                        "cannot remove the document root".to_string(),
                    ))
                }
            }
            continue;
        }
        let (parent_pointer, last) = match path.rfind('/') {
            Some(index) => (&path[..index], &path[index + 1..]),
            None => ("", path.trim_start_matches('/')),
        };
        let parent = if parent_pointer.is_empty() {
            &mut *document
        } else {
            document.pointer_mut(parent_pointer).ok_or_else(|| {
                ReplayError::InvalidPayload(format!("patch path {parent_pointer} not found"))
            })?
        };
        let last = decode_token(last);
        match parent {
            Value::Object(map) => match op {
                "add" | "replace" => {
                    map.insert(last, operation.get("value").cloned().unwrap_or(Value::Null));
                }
                "remove" => {
                    map.remove(&last);
                }
                other => {
                    return Err(ReplayError::InvalidPayload(format!(
                        "unsupported patch op {other}"
                    )))
                }
            },
            Value::Array(items) => {
                if last == "-" {
                    if op != "add" {
                        return Err(ReplayError::InvalidPayload(
                            "'-' is only valid for add".to_string(),
                        ));
                    }
                    items.push(operation.get("value").cloned().unwrap_or(Value::Null));
                    continue;
                }
                let index: usize = last
                    .parse()
                    .map_err(|_| ReplayError::InvalidPayload(format!("bad index {last}")))?;
                match op {
                    "add" => {
                        if index > items.len() {
                            return Err(ReplayError::InvalidPayload(format!(
                                "index {index} out of bounds"
                            )));
                        }
                        items.insert(index, operation.get("value").cloned().unwrap_or(Value::Null));
                    }
                    "replace" => {
                        let slot = items.get_mut(index).ok_or_else(|| {
                            ReplayError::InvalidPayload(format!("index {index} out of bounds"))
                        })?;
                        *slot = operation.get("value").cloned().unwrap_or(Value::Null);
                    }
                    "remove" => {
                        if index >= items.len() {
                            return Err(ReplayError::InvalidPayload(format!(
                                "index {index} out of bounds"
                            )));
                        }
                        items.remove(index);
                    }
                    other => {
                        return Err(ReplayError::InvalidPayload(format!(
                            "unsupported patch op {other}"
                        )))
                    }
                }
            }
            _ => {
                return Err(ReplayError::InvalidPayload(
                    "patch parent is not a container".to_string(),
                ))
            }
        }
    }
    Ok(())
}

fn decode_token(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}
