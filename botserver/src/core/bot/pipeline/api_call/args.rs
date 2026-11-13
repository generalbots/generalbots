//! Argument recovery for command calls: deep links and monetary amounts.

/// Collects every `app://...` deep-link string present anywhere in a command
/// result so the final answer can surface the most relevant one even when the
/// compose LLM call fails.
pub(super) fn extract_deep_links(value: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![value.clone()];
    while let Some(v) = stack.pop() {
        match v {
            serde_json::Value::String(s) => {
                if s.starts_with("app://") {
                    out.push(s);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    stack.push(item);
                }
            }
            serde_json::Value::Object(map) => {
                for (_, val) in map {
                    stack.push(val);
                }
            }
            _ => {}
        }
    }
    out
}

/// Injects a `value` param for tax commands when the model omitted it but the
/// user's message contains a monetary amount ("R$ 10.000", "10000 reais").
pub(super) fn inject_tax_value_from_user(
    params: &serde_json::Value,
    user_text: &str,
) -> serde_json::Value {
    let has_value = params
        .get("value")
        .and_then(|v| v.as_str())
        .is_some_and(|v| !v.trim().is_empty());
    if has_value {
        return params.clone();
    }
    let amount = extract_money_amount(user_text);
    match amount {
        Some(value) => {
            let mut params = params.clone();
            if let serde_json::Value::Object(map) = &mut params {
                map.insert("value".to_string(), serde_json::Value::String(value));
            }
            params
        }
        None => params.clone(),
    }
}

/// Fills the gaps in `params` from an earlier, approved call to the same
/// command. Only keys the model left out (absent or empty string) are taken
/// from the approved payload, so a fresh value still wins over the old one.
pub(super) fn merge_missing_params(
    params: &serde_json::Value,
    approved: &serde_json::Value,
) -> serde_json::Value {
    let (serde_json::Value::Object(current), serde_json::Value::Object(approved)) =
        (params, approved)
    else {
        return params.clone();
    };
    let mut merged = current.clone();
    for (key, value) in approved {
        let missing = match merged.get(key) {
            None | Some(serde_json::Value::Null) => true,
            Some(serde_json::Value::String(s)) => s.trim().is_empty(),
            Some(_) => false,
        };
        if missing {
            merged.insert(key.clone(), value.clone());
        }
    }
    serde_json::Value::Object(merged)
}

/// The BRL amount pattern, compiled once per process: this runs on every
/// API-call fallback in the message pipeline (see #1493).
const MONEY_AMOUNT_PATTERN: &str = r"(?i)R\$\s*([\d.,]+)|\b([\d.,]+)\s*reais";

fn money_amount_re() -> Option<&'static regex::Regex> {
    static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(MONEY_AMOUNT_PATTERN)
            .inspect_err(|e| log::error!("money amount pattern failed to compile: {e}"))
            .ok()
    })
    .as_ref()
}

/// Extracts a BRL amount from free text, e.g. "R$ 10.000,50" or "10.000 reais".
/// Returns a normalized decimal string ("10000.50").
fn extract_money_amount(text: &str) -> Option<String> {
    let re = money_amount_re()?;
    let caps = re.captures(text)?;
    let raw = caps.get(1).or_else(|| caps.get(2))?.as_str();
    let cleaned = raw.replace('.', "");
    let normalized = if cleaned.contains(',') {
        cleaned.replace(',', ".")
    } else {
        cleaned
    };
    if normalized.parse::<f64>().is_ok() {
        Some(normalized)
    } else {
        None
    }
}

#[cfg(test)]
mod money_tests {
    use super::*;

    /// The cached accessor must extract exactly what the per-call compile did.
    #[test]
    fn extracts_the_same_amounts_as_before() {
        assert_eq!(
            extract_money_amount("custa R$ 10.000,50 hoje"),
            Some("10000.50".into())
        );
        assert_eq!(
            extract_money_amount("pague 10.000 reais"),
            Some("10000".into())
        );
        assert_eq!(extract_money_amount("R$ 250"), Some("250".into()));
        assert_eq!(extract_money_amount("sem valor aqui"), None);
    }

    /// The pattern is compiled once and reused; a second call must not fail.
    #[test]
    fn the_cached_pattern_is_reusable() {
        assert!(money_amount_re().is_some());
        assert_eq!(extract_money_amount("R$ 1,50"), Some("1.50".into()));
    }
}
