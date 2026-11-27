//! Frees the graphics card when no turn can reach the local model.
//!
//! At app start the package manager launches the bundled chat model on the
//! chat port whatever the chosen mode is, and switching from a local model to
//! a hosted one left the local model loaded. Either way the engine held
//! gigabytes of video memory that nothing used, and a later switch back to a
//! local model found the card full. Only this install's chat engine is
//! stopped (`runtime_stop::is_install_chat_engine`): the embedding engine
//! runs on the processor and serves the knowledge search.

use std::collections::HashMap;

use super::runtime_ctl;

/// Settings that decide which model and endpoint a turn uses: the main
/// choice and the three picker tiers (`llm-model-<tier>`).
pub const ROUTING_KEYS: [&str; 5] =
    ["llm-model", "llm-url", "llm-model-fast", "llm-model-balanced", "llm-model-quality"];
/// Environment overrides that win over every stored setting.
const ROUTING_ENV: [&str; 2] = ["LLM_URL", "LLM_MODEL"];

fn is_loopback_url(url: &str) -> bool {
    let low = url.to_ascii_lowercase();
    ["://127.0.0.1", "://localhost", "://[::1]"].iter().any(|host| low.contains(host))
}

/// Whether any setting sends turns to the engine on this machine: a model
/// file name (`.gguf`) in the main choice or a tier, or a loopback endpoint.
pub fn routes_to_local(choices: &HashMap<String, String>) -> bool {
    choices.iter().any(|(key, value)| {
        let value = value.trim();
        !value.is_empty()
            && (runtime_ctl::is_local_model_name(value) || (key.ends_with("url") || key.ends_with("URL")) && is_loopback_url(value))
    })
}

/// Whether the chat engine should be stopped for these settings. Nothing is
/// stopped before a main model is chosen (first run, before the wizard).
pub fn should_release(choices: &HashMap<String, String>) -> bool {
    let chosen = choices.get("llm-model").is_some_and(|model| !model.trim().is_empty());
    chosen && !routes_to_local(choices)
}

/// The routing settings as stored now, plus the environment overrides.
pub fn stored_choices(config: &botcore::config::ConfigManager) -> HashMap<String, String> {
    let nil = uuid::Uuid::nil();
    let mut choices: HashMap<String, String> = ROUTING_KEYS
        .iter()
        .map(|key| {
            (
                key.to_string(),
                config.get_config(&nil, key, Some("")).unwrap_or_else(|e| {
                    log::debug!("[runtime_release] could not read {key} ({e}); using an empty value");
                    String::new()
                }),
            )
        })
        .collect();
    for name in ROUTING_ENV {
        if let Ok(value) = std::env::var(name) {
            choices.insert(name.to_string(), value);
        }
    }
    choices
}

/// Stop this install's chat engine when no setting routes to it. Waits for
/// a start in progress (same lock as `runtime_ctl::ensure`), so a switch
/// back to a local model is never undone.
pub async fn release_if_not_local(choices: &HashMap<String, String>, reason: &str) {
    if !should_release(choices) {
        return;
    }
    let _serial = runtime_ctl::ENSURE_LOCK.lock().await;
    // Found by executable and command line, not by a connection: at app
    // start the engine may still be loading and not listening yet.
    let port = runtime_ctl::CHAT_PORT;
    match tokio::task::spawn_blocking(move || super::runtime_stop::stop_chat_runtime(port)).await {
        Ok(Ok(pid)) => {
            runtime_ctl::clear_last_failure();
            log::info!("[runtime_release] stopped the local chat engine (PID {pid}): {reason}");
        }
        Ok(Err(detail)) => log::info!("[runtime_release] no chat engine of this install to stop ({reason}): {detail}"),
        Err(error) => log::warn!("[runtime_release] stop task failed ({reason}): {error}"),
    }
}

/// After a saved switch: the new main model and endpoint (`model`, `url`
/// from the central settings just written) with the stored tiers.
pub fn release_in_background(config: botcore::config::ConfigManager, saved: &HashMap<String, String>) {
    let saved = saved.clone();
    tokio::spawn(async move {
        let mut choices = stored_choices(&config);
        for (from, to) in [("model", "llm-model"), ("url", "llm-url")] {
            if let Some(value) = saved.get(from) {
                choices.insert(to.to_string(), value.clone());
            }
        }
        release_if_not_local(&choices, "switched to a hosted model").await;
    });
}

#[cfg(test)]
#[path = "runtime_release_tests.rs"]
mod runtime_release_tests;
