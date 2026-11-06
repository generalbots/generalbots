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
        return message.to_string();
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
    if let Some(count) = result.get("count").and_then(|c| c.as_u64()) {
        return format!("Done — {count} item{}.", if count == 1 { "" } else { "s" });
    }
    format!("`{name}` finished. Ask me for the details you need and I will show them.")
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
        assert_eq!(humanize_result("drive.save", &result), "Saved 2 files to Drive");
    }
}
