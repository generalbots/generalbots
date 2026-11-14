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

#[cfg(test)]
mod humanize_tests {
    use super::humanize_result;

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
}
