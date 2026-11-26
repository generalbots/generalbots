//! A one-line, human description of a command result.

/// A one-line, human description of a command result. Used when the model
/// asked for a lookup but did not ask for a composed answer, so the chat never
/// shows the raw JSON envelope.
pub(super) fn humanize_result(name: &str, result: &serde_json::Value) -> String {
    if let Some(message) = result
        .get("message")
        .and_then(|m| m.as_str())
        .map(str::trim)
        .filter(|m| !m.is_empty())
    {
        return with_deep_link(message.to_string(), result);
    }
    // #2583 — a list command must show the values it actually returned. This
    // is also the deterministic fallback when a composed answer is rejected
    // for quoting a figure that is not in the result.
    if let Some(rows) = result
        .get("products")
        .and_then(|p| p.as_array())
        .or_else(|| result.get("items").and_then(|i| i.as_array()))
    {
        let lines: Vec<String> = rows
            .iter()
            .take(10)
            .map(|row| {
                let label = row
                    .get("name")
                    .and_then(|n| n.as_str())
                    .filter(|n| !n.is_empty())
                    .unwrap_or("—");
                match row.get("price").and_then(|p| p.as_str()) {
                    Some(price) if !price.is_empty() => format!("- {label} — {price}"),
                    _ => format!("- {label}"),
                }
            })
            .collect();
        if !lines.is_empty() {
            return lines.join("\n");
        }
    }
    if let Some(items) = result.get("matches").and_then(|m| m.as_array()) {
        let names: Vec<&str> = items
            .iter()
            .filter_map(|item| item.get("name").and_then(|n| n.as_str()))
            .take(5)
            .collect();
        let listed = if names.is_empty() {
            String::new()
        } else {
            format!(" The closest are: {}.", names.join(", "))
        };
        return format!(
            "I found {} action{} that can do this.{listed} Tell me which one to run and I will \
             execute it now.",
            items.len(),
            if items.len() == 1 { "" } else { "s" }
        );
    }
    let deep_link = result
        .get("deep_link")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if let Some(id) = result.get("id").and_then(|v| v.as_str()) {
        let base = match name {
            n if n.starts_with("sheet.") => format!("Spreadsheet ready ({id})."),
            n if n.starts_with("docs.") => format!("Document ready ({id})."),
            n if n.starts_with("slides.") => format!("Presentation ready ({id})."),
            _ => format!("Done ({id})."),
        };
        return with_deep_link(base, result);
    }
    if !deep_link.is_empty() {
        let label = result
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("Open");
        return format!("Done. [{label}]({deep_link})");
    }
    if let Some(count) = result.get("count").and_then(|c| c.as_u64()) {
        return format!("Done — {count} item{}.", if count == 1 { "" } else { "s" });
    }
    format!("`{name}` finished. Ask me for the details you need and I will show them.")
}

/// #2583 — every figure in a composed answer must come from the command result
/// or from the user's own message. The model may otherwise turn a lookup into
/// "Widget Pro · 24.99" when the row says 19.99, or narrate a write that never
/// ran. Only decimal figures are checked: integers such as list indices and
/// dates are too common in prose to reject safely.
pub(super) fn grounded_numbers(text: &str, result: &serde_json::Value, user_text: &str) -> bool {
    let mut allowed = Vec::new();
    collect_numbers(result, &mut allowed);
    for token in decimal_tokens(user_text) {
        extend_with_candidates(&token, &mut allowed);
    }
    decimal_tokens(text)
        .iter()
        .all(|token| number_candidates(token).iter().any(|value| {
            allowed
                .iter()
                .any(|grounded| approximately_equal(*value, *grounded))
        }))
}

fn collect_numbers(value: &serde_json::Value, out: &mut Vec<f64>) {
    match value {
        serde_json::Value::Number(number) => {
            if let Some(value) = number.as_f64() {
                out.push(value);
            }
        }
        serde_json::Value::String(text) => {
            if let Ok(value) = text.trim().parse::<f64>() {
                out.push(value);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_numbers(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            for value in map.values() {
                collect_numbers(value, out);
            }
        }
        serde_json::Value::Bool(_) | serde_json::Value::Null => {}
    }
}

/// Decimal-looking runs of digits, with `.`/`,` kept so prices such as
/// `19.99` and `1.200,00` are compared as whole figures.
fn decimal_tokens(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            let start = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_digit() || bytes[index] == b'.' || bytes[index] == b',')
            {
                index += 1;
            }
            let token = &text[start..index];
            if token.contains('.') || token.contains(',') {
                tokens.push(token.trim_matches(|c: char| c == '.' || c == ',').to_string());
            }
        } else {
            index += 1;
        }
    }
    tokens
}

fn extend_with_candidates(token: &str, out: &mut Vec<f64>) {
    out.extend(number_candidates(token));
}

/// Accept both `19.99`/`19,99` and thousands separators (`1,200.99`).
fn number_candidates(token: &str) -> Vec<f64> {
    let variants = [
        token.to_string(),
        token.replace(',', ""),
        token.replace(',', "."),
    ];
    let mut values = Vec::new();
    for variant in variants {
        if let Ok(value) = variant.parse::<f64>() {
            if !values.iter().any(|known| approximately_equal(value, *known)) {
                values.push(value);
            }
        }
    }
    values
}

fn approximately_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-6 * left.abs().max(right.abs()).max(1.0)
}

fn with_deep_link(mut text: String, result: &serde_json::Value) -> String {
    let Some(link) = result.get("deep_link").and_then(|v| v.as_str()) else {
        return text;
    };
    if text.contains("app://") || link.is_empty() {
        return text;
    }
    let label = result
        .get("label")
        .and_then(|v| v.as_str())
        .unwrap_or("Open in app");
    if !text.ends_with('.') && !text.ends_with('!') && !text.ends_with('?') {
        text.push('.');
    }
    text.push_str(&format!(" [{label}]({link})"));
    text
}

/// What the person sees when a command fails: a sentence in their language,
/// never the raw (English, internal) error, which goes to the server log only.
pub(super) fn failure_text(pool: &botcore::shared::utils::DbPool, user_id: uuid::Uuid) -> String {
    let locale = botcore::i18n::locale_for_user_pool(pool, user_id);
    botcore::i18n::t(&locale, "error-command-failed")
}

#[cfg(test)]
mod humanize_tests {
    use super::{grounded_numbers, humanize_result};

    #[test]
    fn catalog_results_are_described_not_dumped() {
        let result = serde_json::json!({
            "matches": [
                {"name": "web.search"}, {"name": "research.discover"}, {"name": "drive.file"}
            ]
        });
        let text = humanize_result("api.find", &result);
        assert!(!text.contains('{'), "no JSON in the answer: {text}");
        assert!(text.contains("3 actions"));
        assert!(text.contains("web.search"));
    }

    #[test]
    fn a_command_message_wins() {
        let result = serde_json::json!({"message": "Saved 2 files to Drive", "count": 2});
        assert_eq!(
            humanize_result("drive.save", &result),
            "Saved 2 files to Drive"
        );
    }

    #[test]
    fn list_results_show_the_returned_prices() {
        let result = serde_json::json!({
            "count": 1,
            "products": [{"name": "Widget Pro", "price": "19.99"}]
        });
        let text = humanize_result("products.items.list", &result);
        assert!(text.contains("Widget Pro"));
        assert!(text.contains("19.99"));
    }

    #[test]
    fn a_fabricated_price_is_not_grounded() {
        let result = serde_json::json!({
            "count": 1,
            "products": [{"name": "Widget Pro", "price": "19.99"}]
        });
        assert!(grounded_numbers(
            "Widget Pro · 19.99 · USD",
            &result,
            "List the products in the catalog"
        ));
        assert!(!grounded_numbers(
            "Widget Pro · 24.99 · USD",
            &result,
            "List the products in the catalog"
        ));
        // A figure the user supplied is allowed in the answer.
        assert!(grounded_numbers(
            "Widget Pro · 24.99 · USD",
            &result,
            "Change Widget Pro price to 24.99"
        ));
    }
}

