//! Boot-time enforcement of the stored local model choice (#2561).
//!
//! Called once from `main_body` after the package manager has spawned the
//! llm component (which starts with a hardcoded default model).  When the
//! stored choice is a different local model, the chat runtime is replaced.
//!
//! The package manager spawns the llm component during `bootstrap.start_all()`
//! (`lifecycle.rs:105`), which runs at `main_body.rs:152` via `run_bootstrap`.
//! `ensure_llama_servers_running` runs later (`main_body.rs:361`), finds the
//! port already answering, and returns early.  This module runs after both
//! (`main_body.rs:367+`), so the package manager's process is guaranteed to
//! have been spawned (though it may still be loading).  `runtime_ctl::ensure`
//! handles the loading case with `wait_for_initial_probe`.

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
