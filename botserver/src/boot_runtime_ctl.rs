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

use botcore::config::ConfigManager;
use botcore::shared::state::AppState;
use botsettings::on_device_ai::runtime_ctl;

/// Read the stored model choice and ensure the runtime matches.
/// Only acts when the active mode is local (the stored model is a `.gguf`).
pub async fn enforce_stored_model(state: Arc<AppState>) {
    let cfg = ConfigManager::new(state.conn.clone());
    let nil = uuid::Uuid::nil();

    let model = cfg
        .get_config(&nil, "llm-model", Some(""))
        .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);

    // Gate: only enforce in local mode.  In free mode the nil-id llm-model
    // holds a hosted name like "dots-studio/dots-3-note-preview:free".
    // In own-key mode it holds whatever model name the user configured.
    // Neither is a .gguf file, so is_local_model_name returns false.
    if !runtime_ctl::is_local_model_name(&model) {
        return;
    }

    let gpu_layers = cfg
        .get_config(&nil, "llm-server-gpu-layers", Some(""))
        .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);
    let ctx_size = cfg
        .get_config(&nil, "llm-server-ctx-size", Some(""))
        .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);

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

    let outcome = runtime_ctl::ensure(&desired).await;
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
            log::error!("[boot_runtime_ctl] failed to enforce stored model: {reason}");
        }
    }
}
