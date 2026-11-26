//! Boot-time enforcement of the stored local model choice (#2561).
//!
//! Called once from `main_body` after the legacy launcher.  On the Lite
//! edition (Windows) the legacy launcher is told that the chat engine is
//! managed elsewhere (`set_chat_engine_managed_elsewhere`), so it leaves
//! port 8081 free.  This module then starts the chat runtime from scratch
//! with the launch plan (context size and GPU layers sized from free video
//! memory).  When the stored model is not a local `.gguf`, this module
//! releases any orphan engine instead.
//!
//! `runtime_ctl::ensure` handles both a free port (cold start) and an
//! already-loading engine (orphan from a previous session) through
//! `wait_for_initial_probe`.

use std::sync::Arc;
use std::time::Duration;

use botcore::config::ConfigManager;
use botcore::shared::state::AppState;
use botsettings::on_device_ai::runtime_ctl;

/// Maximum number of attempts to start the runtime at boot.
const BOOT_MAX_ATTEMPTS: u32 = 3;
/// Pause between retry attempts.
const BOOT_RETRY_DELAY: Duration = Duration::from_secs(5);

/// Read the stored model choice and ensure the runtime matches.
/// Only acts when the active mode is local (the stored model is a `.gguf`).
pub async fn enforce_stored_model(state: Arc<AppState>) {
    let cfg = ConfigManager::new(state.conn.clone());
    let nil = uuid::Uuid::nil();

    let model = cfg
        .get_config(&nil, "llm-model", Some(""))
        .unwrap_or_default();

    // Gate: only enforce in local mode.  In free mode the nil-id llm-model
    // holds a hosted name like "dots-studio/dots-3-note-preview:free".
    // In own-key mode it holds whatever model name the user configured.
    // Neither is a .gguf file, so is_local_model_name returns false.
    if !runtime_ctl::is_local_model_name(&model) {
        // The package manager started the bundled chat model anyway; it
        // would hold video memory that nothing uses.
        let choices = botsettings::on_device_ai::runtime_release::stored_choices(&cfg);
        botsettings::on_device_ai::runtime_release::release_if_not_local(&choices, "no setting routes to a local model").await;
        return;
    }

    let gpu_layers = cfg
        .get_config(&nil, "llm-server-gpu-layers", Some(""))
        .unwrap_or_default();
    let ctx_size = cfg
        .get_config(&nil, "llm-server-ctx-size", Some(""))
        .unwrap_or_default();

    let desired = match runtime_ctl::desired_from_config(&model, &gpu_layers, &ctx_size) {
        Some(d) => d,
        None => {
            log::warn!(
                "[boot_runtime_ctl] model file for '{}' not found on disk; skipping",
                model,
            );
            return;
        }
    };

    let outcome = boot_ensure_with_retries(&desired).await;
    match &outcome {
        runtime_ctl::EnsureOutcome::AlreadyCorrect => {
            log::info!(
                "[boot_runtime_ctl] chat runtime already running correct model: {}",
                desired.model_file,
            );
        }
        runtime_ctl::EnsureOutcome::Started => {
            log::info!(
                "[boot_runtime_ctl] chat runtime (re)started with model: {}",
                desired.model_file,
            );
        }
        runtime_ctl::EnsureOutcome::Failed(reason) => {
            log::error!("[boot_runtime_ctl] failed to enforce stored model after all attempts: {reason}");
        }
    }
}

/// Try `runtime_ctl::ensure` up to [`BOOT_MAX_ATTEMPTS`] times. The second
/// attempt usually succeeds quickly because the model file is then in the OS
/// disk cache. Each failed attempt logs at `warn`; `error` is left to the
/// caller when all attempts fail.
async fn boot_ensure_with_retries(
    desired: &runtime_ctl::DesiredRuntime,
) -> runtime_ctl::EnsureOutcome {
    for attempt in 1..=BOOT_MAX_ATTEMPTS {
        let outcome = runtime_ctl::ensure(desired).await;
        if outcome.is_ok() {
            return outcome;
        }
        if let Some(reason) = outcome.error_message() {
            if attempt < BOOT_MAX_ATTEMPTS {
                log::warn!(
                    "[boot_runtime_ctl] attempt {attempt}/{BOOT_MAX_ATTEMPTS} failed: {reason}; retrying in {} s",
                    BOOT_RETRY_DELAY.as_secs(),
                );
                tokio::time::sleep(BOOT_RETRY_DELAY).await;
            } else {
                return outcome;
            }
        }
    }
    // Unreachable, but satisfies the type system.
    runtime_ctl::EnsureOutcome::Failed("boot retries exhausted".into())
}

/// Pure-logic helper: returns `true` when the loop should keep retrying.
/// Extracted for unit testing without async or I/O.
pub fn should_retry(attempt: u32, max: u32, outcome_ok: bool) -> bool {
    !outcome_ok && attempt < max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_policy_gives_up_after_max() {
        assert!(should_retry(1, 3, false));
        assert!(should_retry(2, 3, false));
        assert!(!should_retry(3, 3, false), "must not retry past max");
    }

    #[test]
    fn retry_policy_stops_on_success() {
        assert!(!should_retry(1, 3, true));
    }

    #[test]
    fn boot_constants_are_sensible() {
        assert_eq!(BOOT_MAX_ATTEMPTS, 3);
        assert!(BOOT_RETRY_DELAY.as_secs() >= 2);
    }
}
