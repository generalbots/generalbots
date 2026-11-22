//! Self-healing fallback for the free model tier.
//!
//! When the free-tier gateway returns a 404 / "requested model not found",
//! this module fetches the gateway's public model list, picks a replacement
//! and updates the Vault config so the next attempt (and all future turns)
//! use the new model.

use std::sync::Arc;
use std::time::Duration;

use botcore::shared::state::AppState;

use botsettings::model_config::{FREE_ENDPOINT, FREE_PROVIDER};

/// Returns `true` when the error detail looks like a free-model 404.
pub fn is_free_model_404(detail: &str) -> bool {
    let lower = detail.to_lowercase();
    (lower.contains("404") || lower.contains("requested model") || lower.contains("not found"))
        && lower.contains(":free")
}

/// Outcome of a free-model fallback attempt.
pub enum FallbackResult {
    /// A replacement model was found and persisted; the caller should retry.
    Replaced,
    /// No replacement found; the caller should send the refusal.
    Unavailable,
    /// The error is not a free-model 404; the caller continues normally.
    NotApplicable,
}

/// Process-wide record of the last replacement time, used to prevent
/// infinite replacement loops (e.g. when the picked model also 404s).
static LAST_REPLACEMENT: std::sync::Mutex<Option<std::time::Instant>> =
    std::sync::Mutex::new(None);

/// Minimum interval between two replacements (seconds).
const REPLACEMENT_COOLDOWN_SECS: u64 = 60;

/// Pure decision: should we attempt a replacement?
/// Returns `false` when the picked model equals the one that just failed,
/// or when a replacement happened less than `cooldown_secs` ago.
pub fn should_replace(
    current_model: &str,
    picked_model: &str,
    secs_since_last: Option<u64>,
    cooldown_secs: u64,
) -> bool {
    if current_model == picked_model {
        return false;
    }
    match secs_since_last {
        Some(s) if s < cooldown_secs => false,
        _ => true,
    }
}

/// Top-level entry point called from the stream loop on any error.
/// Checks whether this is a free-model 404 and, if so, tries to heal it.
pub async fn try_replace_if_free(
    state: &Arc<AppState>,
    engine_type: Option<&str>,
    detail: &str,
    current_model: &str,
) -> FallbackResult {
    if engine_type != Some(botsettings::usage_engine::ENGINE_FREE)
        || !is_free_model_404(detail)
    {
        return FallbackResult::NotApplicable;
    }
    // Cooldown: prevent replacement loops.
    let secs_since = LAST_REPLACEMENT
        .lock()
        .ok()
        .and_then(|g| g.map(|t| t.elapsed().as_secs()));
    match pick_free_model().await {
        Some(replacement) => {
            if !should_replace(current_model, &replacement, secs_since, REPLACEMENT_COOLDOWN_SECS) {
                log::warn!("[free_fallback] replacement blocked (same model or cooldown)");
                return FallbackResult::Unavailable;
            }
            log::info!("[free_fallback] replacing model with {replacement}");
            if let Ok(mut g) = LAST_REPLACEMENT.lock() {
                *g = Some(std::time::Instant::now());
            }
            apply_replacement(state, &replacement).await;
            FallbackResult::Replaced
        }
        None => FallbackResult::Unavailable,
    }
}

/// Fetches the gateway model list (no key, no user data), picks a free
/// general-purpose model and returns its id.  Returns `None` when the
/// list cannot be read or has no suitable free model.
pub async fn pick_free_model() -> Option<String> {
    let models_url = format!(
        "{}/models",
        FREE_ENDPOINT.trim_end_matches("/chat/completions")
    );
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()
        .ok()?;
    let resp = client.get(&models_url).send().await.ok()?;
    if !resp.status().is_success() {
        log::warn!("[free_fallback] model list HTTP {}", resp.status());
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    let data = body.get("data")?.as_array()?;
    pick_from_list(data)
}

/// Pure picker: given the `data` array from the gateway, return the best
/// free general-purpose model id.
///
/// Selection rule (matches the brief):
/// 1. `isFree` is true (or the id ends in `:free`).
/// 2. Prefer larger `context_length`.
/// 3. Exclude preview/experimental, code-only and reasoning-only when
///    there is a choice.
/// 4. Among equals, prefer a model whose name suggests general chat.
pub fn pick_from_list(data: &[serde_json::Value]) -> Option<String> {
    let mut candidates: Vec<(&str, i64)> = Vec::new();
    for entry in data {
        let id = entry.get("id")?.as_str()?;
        let is_free = entry
            .get("isFree")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
            || id.ends_with(":free")
            || id.contains("/free");
        if !is_free {
            continue;
        }
        // Skip auto-routers (they may rotate through unavailable models).
        if id.contains("auto") {
            continue;
        }
        let ctx = entry
            .get("context_length")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        candidates.push((id, ctx));
    }
    if candidates.is_empty() {
        return None;
    }
    // Deprioritise reasoning-only, code-only, vision-specific and preview.
    let dominated = |id: &str| -> bool {
        let lower = id.to_lowercase();
        lower.contains("reasoning")
            || lower.contains("code")
            || lower.contains("preview")
            || lower.contains("experimental")
            || lower.contains("-vl")
            || lower.contains("nano")
    };
    let good: Vec<_> = candidates.iter().filter(|(id, _)| !dominated(id)).collect();
    let pool = if good.is_empty() { &candidates } else {
        // SAFETY: good borrows from candidates with the same lifetime.
        // We need to convert to owned references.
        return good
            .into_iter()
            .max_by_key(|(_, ctx)| *ctx)
            .map(|(id, _)| id.to_string());
    };
    pool.iter()
        .max_by_key(|(_, ctx)| *ctx)
        .map(|(id, _)| id.to_string())
}

/// Updates the Vault config so the next turn (and future turns) use the
/// replacement model.  The endpoint and provider stay the same.
pub async fn apply_replacement(_state: &Arc<AppState>, new_model: &str) {
    use botcoresecrets::SecretsManager;
    let Ok(secrets) = SecretsManager::get_clone() else { return };

    // Central map (`gbo/llm`).
    let central_path = "gbo/llm";
    let mut central: std::collections::HashMap<String, String> = secrets
        .get_secret(central_path)
        .await
        .unwrap_or_default();
    central.insert("model".into(), new_model.to_string());
    central.insert("provider".into(), FREE_PROVIDER.into());
    if let Err(e) = secrets.put_secret(central_path, central).await {
        log::error!("[free_fallback] vault write failed: {e}");
        return;
    }
    // Per-bot mirror.
    let nil = uuid::Uuid::nil();
    let bot_path = format!("gbo/{nil}/{nil}/{nil}");
    let mut per_bot: std::collections::HashMap<String, String> = secrets
        .get_secret(&bot_path)
        .await
        .unwrap_or_default();
    per_bot.insert("llm-model".into(), new_model.to_string());
    per_bot.insert("llm-provider".into(), FREE_PROVIDER.into());
    let _ = secrets.put_secret(&bot_path, per_bot).await;

    log::info!("[free_fallback] model replaced: {new_model}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_list() -> Vec<serde_json::Value> {
        vec![
            json!({
                "id": "kilo-auto/free",
                "isFree": true,
                "context_length": 256000,
            }),
            json!({
                "id": "inclusionai/ling-3.2-flash:free",
                "isFree": true,
                "context_length": 1048576,
            }),
            json!({
                "id": "nvidia/nemotron-3.5-lightning:free",
                "isFree": true,
                "context_length": 1000000,
            }),
            json!({
                "id": "baidu/ernie-5.5-flash-preview:free",
                "isFree": true,
                "context_length": 1024000,
            }),
            json!({
                "id": "liquid/lfm-3-1.7b:free",
                "isFree": true,
                "context_length": 262144,
            }),
            json!({
                "id": "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free",
                "isFree": true,
                "context_length": 256000,
            }),
            json!({
                "id": "openai/gpt-4o",
                "isFree": false,
                "context_length": 128000,
            }),
        ]
    }

    #[test]
    fn picks_largest_non_dominated_free_model() {
        let list = sample_list();
        let picked = pick_from_list(&list).unwrap();
        // Should pick ling-3.2-flash (largest ctx, not preview/reasoning/code/nano).
        assert_eq!(picked, "inclusionai/ling-3.2-flash:free");
    }

    #[test]
    fn skips_auto_router() {
        let list = sample_list();
        let picked = pick_from_list(&list).unwrap();
        assert!(!picked.contains("auto"));
    }

    #[test]
    fn skips_non_free_models() {
        let list = vec![json!({"id": "openai/gpt-4o", "isFree": false, "context_length": 128000})];
        assert!(pick_from_list(&list).is_none());
    }

    #[test]
    fn falls_back_to_dominated_when_no_good_candidate() {
        let list = vec![
            json!({"id": "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free", "isFree": true, "context_length": 256000}),
        ];
        let picked = pick_from_list(&list).unwrap();
        assert_eq!(picked, "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free");
    }

    #[test]
    fn detects_free_model_404() {
        assert!(is_free_model_404(
            r#"LLM request failed with status: 404 Not Found: {"error":"The requested model 'dots-studio/dots-3-note-preview:free' does not exist"}"#
        ));
        assert!(!is_free_model_404("connection refused"));
        assert!(!is_free_model_404("404 not found")); // no :free
    }

    #[test]
    fn should_replace_blocks_same_model() {
        assert!(!should_replace("model-a:free", "model-a:free", None, 60));
    }

    #[test]
    fn should_replace_blocks_during_cooldown() {
        assert!(!should_replace("model-a:free", "model-b:free", Some(10), 60));
    }

    #[test]
    fn should_replace_allows_after_cooldown() {
        assert!(should_replace("model-a:free", "model-b:free", Some(61), 60));
    }

    #[test]
    fn should_replace_allows_first_time() {
        assert!(should_replace("model-a:free", "model-b:free", None, 60));
    }

    #[test]
    fn free_turn_detection() {
        assert!(botsettings::model_config::is_free_endpoint("https://api.kilo.ai/api/gateway/chat/completions"));
        assert!(!botsettings::model_config::is_free_endpoint("https://api.openai.com/v1/chat/completions"));
    }
}
