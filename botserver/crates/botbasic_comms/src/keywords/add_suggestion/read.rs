use super::*;

/// Retrieve suggestions from Valkey/Redis for a given user session
/// Returns a vector of Suggestion structs that can be included in BotResponse
/// Note: This function clears suggestions from Redis after fetching them to prevent duplicates
pub fn get_suggestions(
    cache: Option<&Arc<redis::Client>>,
    org_id: Option<uuid::Uuid>,
    bot_id: &str,
    session_id: &str,
) -> Vec<botlib::models::Suggestion> {
    let mut suggestions = Vec::new();

    if let Some(cache_client) = cache {
        // #2547 — read the org-scoped set first, then the legacy set that
        // pre-field sessions (and dual-writes) still populate. Session ids are
        // UUID v4, so the legacy key cannot leak across organisations.
        let parts = ["suggestions", bot_id, session_id];
        let keys = [
            botlib::key_utils::scoped_key(org_id, &parts),
            botlib::key_utils::legacy_key(&parts),
        ];

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, returning empty suggestions");
                return suggestions;
            }
        };

        let mut seen = std::collections::HashSet::new();
        for redis_key in &keys {
            let result: Result<Vec<String>, redis::RedisError> =
                redis::cmd("SMEMBERS").arg(redis_key).query(&mut conn);

            match result {
                Ok(items) => {
                    for item in items {
                        if !seen.insert(item.clone()) {
                            continue;
                        }
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&item) {
                            let item_type = json["type"].as_str().unwrap_or("");
                            if item_type == "switcher" || item_type == "switch_context" {
                                continue;
                            }
                            suggestions.push(botlib::models::Suggestion {
                                text: json["text"].as_str().unwrap_or("").to_string(),
                                context: json["context"].as_str().map(|s| s.to_string()),
                                action: json
                                    .get("action")
                                    .and_then(|v| serde_json::to_string(v).ok()),
                                icon: json
                                    .get("icon")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                            });
                        }
                    }
                }
                Err(e) => log::error!("Failed to get suggestions from Redis: {}", e),
            }
        }
        botlog::safe_info!(
            "Retrieved {} suggestions for session {}",
            suggestions.len(),
            session_id
        );
    } else {
        info!("No cache configured, cannot retrieve suggestions");
    }

    suggestions
}
