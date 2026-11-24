//! Pure computation of the two Vault maps (central `gbo/llm` and the nil
//! per-bot path) for every model-switch direction.
//!
//! All Vault persistence in the model-config area **must** go through
//! [`compute`] so the central and per-bot maps always agree and stale keys
//! or URLs never survive a mode switch.

use std::collections::HashMap;

use super::{ModelMode, FREE_ENDPOINT, FREE_MODEL, FREE_PROVIDER};

/// Local chat-completions endpoint bound by the on-device engine.
const LOCAL_ENGINE_URL: &str = "http://localhost:8081/v1/chat/completions";

/// What the caller wants to switch to.
#[derive(Debug, Clone)]
pub enum Target {
    Free,
    Local {
        /// Bare GGUF filename (e.g. `Qwen3.5-9B-Q4_K_M.gguf`).
        file: String,
    },
    OwnKey {
        endpoint: String,
        model: String,
        key: Option<String>,
        provider: Option<String>,
    },
}

impl Target {
    pub fn mode(&self) -> ModelMode {
        match self {
            Self::Free => ModelMode::Free,
            Self::Local { .. } => ModelMode::Local,
            Self::OwnKey { .. } => ModelMode::OwnKey,
        }
    }
}

/// Returned pair: `(central_map, per_bot_map)`.
///
/// Both maps preserve every key that this module does not manage (e.g.
/// profile settings, llm-server-ctx-size) while guaranteeing that the
/// managed keys are either set to the new values or removed.
pub fn compute(
    central_in: &HashMap<String, String>,
    per_bot_in: &HashMap<String, String>,
    target: &Target,
) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut central = central_in.clone();
    let mut per_bot = per_bot_in.clone();

    // ── central map ─────────────────────────────────────────────────
    match target {
        Target::Free => {
            central.insert("url".into(), FREE_ENDPOINT.into());
            central.insert("model".into(), FREE_MODEL.into());
            central.insert("provider".into(), FREE_PROVIDER.into());
            central.remove("openai_key");
        }
        Target::Local { file } => {
            central.insert("url".into(), LOCAL_ENGINE_URL.into());
            central.insert("model".into(), file.clone());
            central.remove("openai_key");
            central.remove("provider");
        }
        Target::OwnKey {
            endpoint,
            model,
            key,
            provider,
        } => {
            central.insert("url".into(), endpoint.clone());
            central.insert("model".into(), model.clone());
            match key {
                Some(k) if !k.trim().is_empty() => {
                    central.insert("openai_key".into(), k.trim().into());
                }
                _ => {
                    central.remove("openai_key");
                }
            }
            match provider {
                Some(p) if !p.trim().is_empty() => {
                    central.insert("provider".into(), p.trim().into());
                }
                _ => {
                    central.remove("provider");
                }
            }
        }
    }

    // ── per-bot map (mirror or clear) ───────────────────────────────
    // The per-bot path must never hold stale values from a previous mode.
    // Mirror the central values using per-bot key names, or remove them.
    let url = central.get("url").cloned().unwrap_or_else(String::new);
    let model = central.get("model").cloned().unwrap_or_else(String::new);
    let key = central.get("openai_key").cloned();
    let provider = central.get("provider").cloned();

    if url.is_empty() {
        per_bot.remove("llm-url");
    } else {
        per_bot.insert("llm-url".into(), url);
    }

    if model.is_empty() {
        per_bot.remove("llm-model");
    } else {
        per_bot.insert("llm-model".into(), model);
    }

    match key {
        Some(k) if !k.is_empty() => {
            per_bot.insert("llm-key".into(), k);
        }
        _ => {
            per_bot.remove("llm-key");
        }
    }

    match provider {
        Some(p) if !p.is_empty() => {
            per_bot.insert("llm-provider".into(), p);
        }
        _ => {
            per_bot.remove("llm-provider");
        }
    }

    (central, per_bot)
}

/// True when the stored pair is contradictory: a hosted/free URL with a
/// local model filename, or a loopback URL with a hosted model name that
/// is not a `.gguf` file.
pub fn is_inconsistent(url: &str, model: &str) -> bool {
    if url.trim().is_empty() || model.trim().is_empty() {
        return false;
    }
    let url_is_local = super::is_loopback(url);
    let model_is_local = model.ends_with(".gguf");

    // Hosted URL + local model file → inconsistent.
    if !url_is_local && model_is_local {
        return true;
    }
    // Loopback URL + clearly hosted model → inconsistent.
    if url_is_local && !model_is_local && model.contains('/') {
        return true;
    }
    false
}

#[cfg(test)]
#[path = "write_sets_tests.rs"]
mod write_sets_tests;
