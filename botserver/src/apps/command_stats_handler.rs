use super::*;

/// Returns command/API usage ranking (#2372).
pub async fn command_stats_handler() -> Json<serde_json::Value> {
    let stats = crate::apps::installs::command_stats();
    let entries = match stats.entries.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };

    let mut commands: Vec<serde_json::Value> = entries
        .into_iter()
        .map(|(command, entry)| {
            json!({
                "command": command,
                "count": entry.count,
                "app": entry.app,
                "label": entry.label,
            })
        })
        .collect();
    commands.sort_by(|a, b| {
        b.get("count")
            .and_then(serde_json::Value::as_u64)
            .cmp(&a.get("count").and_then(serde_json::Value::as_u64))
            .then_with(|| {
                a.get("command")
                    .and_then(serde_json::Value::as_str)
                    .cmp(&b.get("command").and_then(serde_json::Value::as_str))
            })
    });
    let total: u64 = commands
        .iter()
        .filter_map(|v| v.get("count").and_then(serde_json::Value::as_u64))
        .sum();

    Json(json!({
        "commands": commands,
        "total": total,
    }))
}
