//! Vault persistence for the on-device model choice (#2109).
//!
//! The choice lives where the local engine already reads it: the central
//! `secret/gbo/llm` secret (`model` field, which `ConfigManager` maps from the
//! `llm-model` config key), mirrored on the deployment-wide bot path so a
//! per-bot lookup resolves too. No new store, no API keys — this page only
//! manages weights that run on the device.

use std::collections::HashMap;
use uuid::Uuid;

use botcore::config::ConfigManager;
use botcoresecrets::SecretsManager;

use super::catalog::{ModelSpec, Profile};

/// Vault path holding the deployment-wide LLM provider settings.
const CENTRAL_LLM_PATH: &str = "gbo/llm";

/// Fallback local chat-completions endpoint used when Vault has no `url`.
const DEFAULT_LLM_URL: &str = "http://localhost:8081/v1/chat/completions";

/// Bare filename of a configured model. Config values may carry a legacy
/// relative path, so only the last segment is compared.
pub(crate) fn bare_name(model: &str) -> &str {
    let trimmed = model.trim();
    trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed)
}

pub(crate) fn is_active(configured_model: &str, spec: &ModelSpec) -> bool {
    !configured_model.trim().is_empty() && bare_name(configured_model) == spec.file
}

/// Currently configured model, central Vault first (that is what the engine
/// boots from), then the per-bot/DB fallback.
pub(crate) fn active_model(state: &std::sync::Arc<botcore::shared::state::AppState>) -> String {
    let config = ConfigManager::new(state.conn.clone());
    config
        .get_config(&Uuid::nil(), "llm-model", Some(""))
        .unwrap_or_default()
}

/// True for the endpoints the on-device engine binds to. A hosted provider
/// URL must not be replaced by activating a local model.
pub(crate) fn is_loopback(url: &str) -> bool {
    let lowered = url.to_ascii_lowercase();
    lowered.contains("localhost") || lowered.contains("127.0.0.1") || lowered.contains("[::1]")
}

/// Persist the model (and optional profile) to Vault using the unified
/// write-sets computation so both maps are always consistent.
pub(crate) async fn save_choice(
    state: &std::sync::Arc<botcore::shared::state::AppState>,
    spec: &ModelSpec,
    profile: Option<&Profile>,
) -> Result<bool, String> {
    use crate::model_config::write_sets;

    let secrets = SecretsManager::get_clone().map_err(|error| format!("vault_unavailable: {error}"))?;
    if !secrets.is_enabled() {
        return Err("vault_not_enabled".to_string());
    }

    let central: HashMap<String, String> = secrets
        .get_secret(CENTRAL_LLM_PATH)
        .await
        .unwrap_or_default();
    let nil = Uuid::nil();
    let bot_path = format!("gbo/{nil}/{nil}/{nil}");
    let per_bot: HashMap<String, String> = secrets
        .get_secret(&bot_path)
        .await
        .unwrap_or_default();

    let target = write_sets::Target::Local { file: spec.file.to_string() };
    let (new_central, new_per_bot) = write_sets::compute(&central, &per_bot, &target);

    let wrote_url = new_central.get("url").map(|u| u == DEFAULT_LLM_URL).unwrap_or(false);

    secrets
        .put_secret(CENTRAL_LLM_PATH, new_central)
        .await
        .map_err(|error| format!("vault_write_failed: {error}"))?;
    secrets
        .put_secret(&bot_path, new_per_bot)
        .await
        .map_err(|error| format!("vault_write_failed: {error}"))?;

    if let Some(profile) = profile {
        save_profile(state, profile)?;
    }

    Ok(wrote_url)
}

/// Write performance-profile settings (ctx-size, gpu-layers) to ConfigManager.
pub(crate) fn save_profile(
    state: &std::sync::Arc<botcore::shared::state::AppState>,
    profile: &Profile,
) -> Result<(), String> {
    let config = ConfigManager::new(state.conn.clone());
    let nil = Uuid::nil();
    let ctx = profile.ctx_size.to_string();
    for (key, value) in [
        ("llm-server-ctx-size", ctx.as_str()),
        ("llm-server-gpu-layers", profile.gpu_layers),
    ] {
        config
            .set_config(&nil, key, value)
            .map_err(|error| format!("profile_write_failed: {error}"))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "vault_store_tests.rs"]
mod vault_store_tests;
