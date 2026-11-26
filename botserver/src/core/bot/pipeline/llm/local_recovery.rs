//! Background and inline recovery for the local engine, with workspace
//! progress frames so the person sees the engine starting.
//!
//! The guard (`IN_FLIGHT`) ensures at most one background recovery runs.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use botcore::config::ConfigManager;
use botcore::shared::state::AppState;
use botsettings::on_device_ai::runtime_ctl;

use super::super::types::SessionIdentity;
use super::super::workspace_progress::{WorkspaceProgress, PHASE_INFRA};

/// True while a background recovery task is running; prevents duplicate spawns.
static IN_FLIGHT: AtomicBool = AtomicBool::new(false);

/// Step id the shell labels (e.g. "Starting the language model").
const STEP: &str = "local_engine_start";

/// Maximum time the turn waits for the engine to come back.
const RECOVERY_BUDGET: std::time::Duration = std::time::Duration::from_secs(120);
/// Polling interval while waiting for recovery.
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(2000);

// ── background recovery ─────────────────────────────────────────────────

/// If the stored mode is local and the chat port is not listening, spawn a
/// background task to start the runtime. No-op when already in flight or
/// the mode is not local.
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
    if IN_FLIGHT.swap(true, Ordering::SeqCst) {
        return;
    }
    let _guard = DropGuard;

    if runtime_ctl::port_open().await {
        return;
    }
    let Some(desired) = desired_from_state(&state) else { return };

    log::info!("[local_recovery] background recovery for {}", desired.model_file);
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

// ── preflight ───────────────────────────────────────────────────────────

/// Before the first attempt of a turn that targets the local engine, make
/// sure the port is reachable. When it is not, runs inline recovery with
/// progress frames. Returns immediately when the port is already open.
pub async fn preflight(
    identity: &SessionIdentity<'_>,
    state: &Arc<AppState>,
    session_id_s: &str,
) {
    if runtime_ctl::port_open().await {
        return;
    }
    log::info!("[local_recovery] preflight: port closed, starting inline recovery");
    let _ = recover_inline_with_progress(identity, state, session_id_s).await;
}

// ── inline recovery with progress frames ────────────────────────────────

/// Like `try_recover_inline` but emits workspace progress frames through
/// the existing `WorkspaceProgress` mechanism so the person sees
/// "Starting the language model" in the chat.
///
/// Returns `true` when the engine is reachable and the caller should retry.
pub async fn try_recover_with_progress(
    identity: &SessionIdentity<'_>,
    state: &Arc<AppState>,
    session_id_s: &str,
) -> bool {
    recover_inline_with_progress(identity, state, session_id_s).await
}

async fn recover_inline_with_progress(
    identity: &SessionIdentity<'_>,
    state: &Arc<AppState>,
    session_id_s: &str,
) -> bool {
    let Some(desired) = desired_from_state(state) else { return false };
    if runtime_ctl::port_open().await {
        return true;
    }

    let progress = WorkspaceProgress::new(identity);
    let locale = botcore::i18n::locale_for_user_pool(&state.conn, identity.user_id);
    let label = i18n_label(&locale, "workspace-step-starting-engine", "Starting the language model");
    let detail = i18n_label(&locale, "workspace-step-starting-engine-detail", "");
    if let Some(ref progress) = progress {
        progress.start(PHASE_INFRA, STEP, &label, &detail).await;
    }

    log::info!("[local_recovery] inline recovery: starting {}", desired.model_file);
    let ensure_handle = tokio::spawn(async move {
        runtime_ctl::ensure(&desired).await
    });
    let deadline = tokio::time::Instant::now() + RECOVERY_BUDGET;
    let mut interval = tokio::time::interval(POLL_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        if crate::core::bot::runs::bus::bus().is_cancelled(session_id_s) {
            log::info!("[local_recovery] inline recovery cancelled by user");
            if let Some(ref progress) = progress {
                progress.finish(PHASE_INFRA, STEP, "error", "cancelled").await;
            }
            return false;
        }
        if runtime_ctl::port_open().await {
            log::info!("[local_recovery] inline recovery: port is open");
            if let Some(ref progress) = progress {
                progress.finish(PHASE_INFRA, STEP, "ok", "").await;
            }
            return true;
        }
        if ensure_handle.is_finished() || tokio::time::Instant::now() >= deadline {
            break;
        }
    }
    let ok = runtime_ctl::port_open().await;
    if let Some(ref progress) = progress {
        if ok {
            progress.finish(PHASE_INFRA, STEP, "ok", "").await;
        } else {
            progress.finish(PHASE_INFRA, STEP, "error", "timeout").await;
        }
    }
    if ok {
        log::info!("[local_recovery] inline recovery succeeded");
    } else {
        log::warn!("[local_recovery] inline recovery timed out or failed");
        maybe_recover(state);
    }
    ok
}

// ── helpers ─────────────────────────────────────────────────────────────

fn desired_from_state(state: &Arc<AppState>) -> Option<runtime_ctl::DesiredRuntime> {
    let cfg = ConfigManager::new(state.conn.clone());
    let nil = uuid::Uuid::nil();
    let model = cfg.get_config(&nil, "llm-model", Some("")).unwrap_or_default();
    if !runtime_ctl::is_local_model_name(&model) {
        return None;
    }
    let gpu_layers = cfg
        .get_config(&nil, "llm-server-gpu-layers", Some(""))
        .unwrap_or_default();
    let ctx_size = cfg
        .get_config(&nil, "llm-server-ctx-size", Some(""))
        .unwrap_or_default();
    runtime_ctl::desired_from_config(&model, &gpu_layers, &ctx_size)
}

fn i18n_label(locale: &botcore::i18n::Locale, key: &str, fallback: &str) -> String {
    let translated = botcore::i18n::t(locale, key);
    if translated.starts_with('[') && translated.ends_with(']') {
        fallback.to_string()
    } else {
        translated
    }
}

/// Clears `IN_FLIGHT` on drop so the guard is released even on early return.
struct DropGuard;
impl Drop for DropGuard {
    fn drop(&mut self) {
        IN_FLIGHT.store(false, Ordering::SeqCst);
    }
}
