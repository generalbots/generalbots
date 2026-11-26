//! `POST /api/llm/local/restart`: reload the local model with a fresh launch
//! plan (free video memory re-measured after the previous engine stops).
//!
//! Used after the person closes other programs that were using the graphics
//! card, so the model fits fully without reopening the application.

use axum::http::StatusCode;
use axum::{Extension, Json};
use serde_json::{json, Value};
use std::sync::Arc;

use botcore::config::ConfigManager;
use botcore::shared::state::AppState;
use botsecurity::auth::AuthenticatedUser;

use super::launch_plan;
use super::runtime_ctl;

type ApiError = (StatusCode, Json<Value>);

fn error(status: StatusCode, code: &str) -> ApiError {
    (status, Json(json!({ "error": code })))
}

/// Whether the restart is available: the stored main choice is a local model.
pub fn can_restart(stored_model: &str) -> bool {
    runtime_ctl::is_local_model_name(stored_model)
}

/// Handler: stop the chat engine and start it again with a fresh plan.
///
/// # Route contract
///
/// `POST /api/llm/local/restart`
///
/// **Auth:** admin token (same as `/api/llm/local/activate`).
///
/// **200** (success):
/// ```json
/// {
///   "ok": true,
///   "last_launch": { "model_file", "model_mib", "ctx_size", "gpu_layers",
///                     "vram_free_before_mib", "vram_free_after_mib",
///                     "placement" }
/// }
/// ```
///
/// **409** when no local model is the stored choice:
/// ```json
/// { "error": "not_local_mode" }
/// ```
///
/// **500** when the engine could not be started (the previous engine is
/// already stopped; `last_failure()` reports why):
/// ```json
/// { "error": "<failure_code>: <detail>" }
/// ```
pub async fn handle(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Value>, ApiError> {
    super::require_admin(&user)?;
    let cfg = ConfigManager::new(state.conn.clone());
    let nil = uuid::Uuid::nil();

    let model = cfg.get_config(&nil, "llm-model", Some("")).unwrap_or_default();
    if !can_restart(&model) {
        return Err(error(StatusCode::CONFLICT, "not_local_mode"));
    }

    let gpu_layers = cfg.get_config(&nil, "llm-server-gpu-layers", Some("")).unwrap_or_default();
    // An empty ctx forces the launch plan to size it from the free video
    // memory measured after the old engine has stopped.
    let desired = runtime_ctl::desired_from_config(&model, &gpu_layers, "")
        .ok_or_else(|| error(StatusCode::INTERNAL_SERVER_ERROR, "model_file_not_found"))?;

    // The ensure lock serialises against concurrent switches and releases.
    let _serial = runtime_ctl::ENSURE_LOCK.lock().await;
    if runtime_ctl::port_open().await {
        match tokio::task::spawn_blocking(|| super::runtime_stop::stop_chat_runtime(runtime_ctl::CHAT_PORT)).await {
            Ok(Ok(pid)) => {
                log::info!("[runtime_restart] stopped previous engine (PID {pid})");
                runtime_ctl::wait_port_free_public().await;
            }
            Ok(Err(detail)) => log::info!("[runtime_restart] no engine to stop: {detail}"),
            Err(error) => log::warn!("[runtime_restart] stop task failed: {error}"),
        }
    }

    let log_mark = runtime_ctl::llm_log_len();
    match runtime_ctl::start_chat_runtime(&desired) {
        Ok(child_pid) => match runtime_ctl::wait_healthy(&desired.model_file, child_pid).await {
            Ok(()) => {
                runtime_ctl::clear_last_failure();
                Ok(Json(json!({
                    "ok": true,
                    "last_launch": launch_plan::last_launch(),
                })))
            }
            Err(reason) => {
                let _ = tokio::task::spawn_blocking(move || super::runtime_stop::kill_pid(child_pid)).await;
                let failure = runtime_ctl::handle_start_failure(&desired.model_file, log_mark, &reason);
                Err(error(StatusCode::INTERNAL_SERVER_ERROR, &failure))
            }
        },
        Err(reason) => {
            let failure = runtime_ctl::handle_start_failure(&desired.model_file, log_mark, &reason);
            Err(error(StatusCode::INTERNAL_SERVER_ERROR, &failure))
        }
    }
}

#[cfg(test)]
mod runtime_restart_tests {
    use super::can_restart;

    #[test]
    fn restart_is_available_only_for_local_models() {
        assert!(can_restart("Qwen3.5-9B-Q4_K_M.gguf"));
        assert!(can_restart("DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf"));
        assert!(!can_restart("nvidia/nemotron-3-ultra-550b-a55b:free"));
        assert!(!can_restart("gpt-4o"));
        assert!(!can_restart(""));
        assert!(!can_restart("   "));
    }
}
