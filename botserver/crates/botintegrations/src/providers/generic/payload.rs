use serde_json::Value;

use super::{cred_str, invalid, require_param_str, ActionOutcome, ActionSpec, MAX_LIST_ITEMS};
pub(crate) fn build_body(action: &ActionSpec, params: &Value) -> Result<Option<Vec<u8>>, String> {
    let Some(body_param) = action.body_param else {
        return Ok(None);
    };
    let payload = match params.get(body_param).and_then(Value::as_object) {
        Some(map) => Value::Object(map.clone()),
        _ => {
            return Err(invalid(format!(
                "parameter {body_param} must be a JSON object"
            )))
        }
    };
    let payload = match action.body_wrapper {
        Some(wrapper) => {
            let mut outer = serde_json::Map::new();
            outer.insert(wrapper.to_string(), payload);
            Value::Object(outer)
        }
        None => payload,
    };
    Ok(Some(payload.to_string().into_bytes()))
}

pub(crate) fn flat_body(
    credentials: &Value,
    field: &str,
    action: &ActionSpec,
    params: &Value,
) -> Result<Vec<u8>, String> {
    let mut body = serde_json::Map::new();
    body.insert(
        field.to_string(),
        Value::String(cred_str(credentials, field)?.to_string()),
    );
    for param in action.params {
        if let Some(value) = params.get(param.name) {
            match value {
                Value::Null => {}
                Value::String(text) if text.trim().is_empty() => {}
                other => {
                    body.insert((*param.name).to_string(), other.clone());
                }
            }
        }
    }
    Ok(Value::Object(body).to_string().into_bytes())
}

pub(crate) fn shape_outcome(action: &ActionSpec, status: u16, body: &[u8]) -> ActionOutcome {
    let parsed: Option<Value> = serde_json::from_slice(body).ok();
    let (data, truncated) = match parsed {
        Some(Value::Array(items)) if items.len() > MAX_LIST_ITEMS => (
            Value::Array(items[..MAX_LIST_ITEMS].to_vec()),
            true,
        ),
        Some(Value::Object(mut map)) => {
            let mut truncated = false;
            for (_key, value) in map.iter_mut() {
                if let Value::Array(items) = value {
                    if items.len() > MAX_LIST_ITEMS {
                        *value = Value::Array(items[..MAX_LIST_ITEMS].to_vec());
                        truncated = true;
                    }
                }
            }
            (Value::Object(map), truncated)
        }
        Some(other) => (other, false),
        None => (Value::Null, false),
    };
    let count_suffix = match (&data, action.is_read()) {
        (Value::Array(items), true) => format!(" {} items", items.len()),
        _ => String::new(),
    };
    ActionOutcome {
        summary: format!("{}{} (status {status})", action.summary, count_suffix),
        data,
        truncated,
    }
}

pub(crate) fn build_url_from_parts(
    origin: &str,
    action: &ActionSpec,
    params: &Value,
    query: Vec<(String, String)>,
) -> Result<String, String> {
    let mut path = action.path.to_string();
    for placeholder in action.path_params {
        let value = require_param_str(params, placeholder)?;
        let encoded = urlencoding::encode(&value).into_owned();
        path = path.replace(&format!("{{{placeholder}}}"), &encoded);
    }
    let mut url = format!("{origin}{path}");
    if !query.is_empty() {
        let rendered: Vec<String> = query
            .into_iter()
            .map(|(name, value)| {
                format!(
                    "{}={}",
                    urlencoding::encode(&name),
                    urlencoding::encode(&value)
                )
            })
            .collect();
        let glue = if url.contains('?') { "&" } else { "?" };
        url = format!("{url}{glue}{}", rendered.join("&"));
    }
    Ok(url)
}
