//! Background recovery: when a chat turn fails because the local engine is
//! not running, kick off a single background `runtime_ctl::ensure` so the
//! next turn has a chance of working without a manual restart.
//!
//! The guard (`IN_FLIGHT`) ensures at most one recovery task runs at a time.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use botcore::config::ConfigManager;
use botcore::shared::state::AppState;
use botsettings::on_device_ai::runtime_ctl;

/// True while a recovery task is running; prevents duplicate spawns.
static IN_FLIGHT: AtomicBool = AtomicBool::new(false);

/// If the stored mode is local and the chat port is not listening, spawn a
/// background task to start the runtime. No-op when a recovery is already
/// in flight or the mode is not local.
pub fn maybe_recover(state: &Arc<AppState>) {
    if IN_FLIGHT.load(Ordering::Relaxed) {
        return;
    }
    let state = Arc::clone(state);
    tokio::spawn(async move {
        recover_inner(state).await;
    });
}

async fn recover_inner(state: Arc<AppState>) {
    // Double-check and claim the slot.
    if IN_FLIGHT.swap(true, Ordering::SeqCst) {
        return;
    }
    let _guard = DropGuard;

    // Port might have come back between the error and this task starting.
    if runtime_ctl::port_open().await {
        return;
    }

    let cfg = ConfigManager::new(state.conn.clone());
    let nil = uuid::Uuid::nil();
    let model = cfg
        .get_config(&nil, "llm-model", Some(""))
        .unwrap_or_default();
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
        None => return,
    };

    log::info!("[local_recovery] starting background recovery for {}", desired.model_file);
    let outcome = runtime_ctl::ensure(&desired).await;
    match &outcome {
        runtime_ctl::EnsureOutcome::Started => {
            log::info!("[local_recovery] runtime recovered: {}", desired.model_file);
        }
        runtime_ctl::EnsureOutcome::AlreadyCorrect => {
            log::info!("[local_recovery] runtime was already running");
        }
        runtime_ctl::EnsureOutcome::Failed(reason) => {
            log::warn!("[local_recovery] recovery failed: {reason}");
        }
    }
}

/// Clears `IN_FLIGHT` on drop so the guard is released even on early return.
struct DropGuard;
impl Drop for DropGuard {
    fn drop(&mut self) {
        IN_FLIGHT.store(false, Ordering::SeqCst);
    }
}
