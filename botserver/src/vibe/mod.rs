//! Vibe subsystem wiring (#743): mounts the botvibe router with a concrete
//! `VibeState` implementation backed by the main AppState pool, and seeds the
//! `vibe_projects` schema at boot.

use botcore::shared::state::AppState;
use botvibe::projects::ProjectRegistry;
use botvibe::projects_api::projects_router;
use botvibe::types::{VibeProgressEvent, VibeRun, VibeState};
use botvibe::{ToolRegistry, VibePromptManager, VibeTelemetry, VibeToolExecutor};
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

struct VibeStateImpl {
    pool: botvibe::types::DbPool,
    progress: Option<broadcast::Sender<VibeProgressEvent>>,
    runs: Arc<RwLock<HashMap<Uuid, VibeRun>>>,
}

impl VibeState for VibeStateImpl {
    fn db_pool(&self) -> &botvibe::types::DbPool {
        &self.pool
    }

    fn broadcast_progress(&self, event: VibeProgressEvent) {
        if let Some(tx) = &self.progress {
            let _ = tx.send(event);
        }
    }

    fn progress_sender(&self) -> Option<&broadcast::Sender<VibeProgressEvent>> {
        self.progress.as_ref()
    }

    fn active_runs(&self) -> &Arc<RwLock<HashMap<Uuid, VibeRun>>> {
        &self.runs
    }
}

pub fn configure_vibe_routes(app_state: &Arc<AppState>) -> axum::Router {
    let pool = app_state.conn.clone();

    let state: Arc<dyn VibeState> = Arc::new(VibeStateImpl {
        pool,
        progress: Some(broadcast::channel(128).0),
        runs: Arc::new(RwLock::new(HashMap::new())),
    });

    let registry = ProjectRegistry::new(app_state.conn.clone());
    match registry.ensure_schema() {
        Ok(()) => info!("Vibe: vibe_projects schema ensured"),
        Err(e) => log::error!("Vibe: ensure vibe_projects schema failed: {e}"),
    }
    let project_registry = Arc::new(registry);

    let prompt_manager = Arc::new(VibePromptManager::new());
    let tool_executor = Arc::new(VibeToolExecutor::new(Arc::new(ToolRegistry::new())));
    let telemetry = Arc::new(VibeTelemetry::new());

    botvibe::api::router(state, prompt_manager, tool_executor, telemetry)
        .merge(projects_router(project_registry))
}