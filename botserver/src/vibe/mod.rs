use botcore::shared::state::AppState;
use botvibe::types::{VibeProgressEvent, VibeRun, VibeState};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

struct BotServerVibeState {
    app_state: Arc<AppState>,
    active_runs: Arc<RwLock<HashMap<Uuid, VibeRun>>>,
    vibe_sender: broadcast::Sender<VibeProgressEvent>,
}

impl VibeState for BotServerVibeState {
    fn db_pool(&self) -> &botvibe::types::DbPool {
        &self.app_state.conn
    }

    fn broadcast_progress(&self, event: VibeProgressEvent) {
        let _ = self.vibe_sender.send(event.clone());
        if let Some(ref sender) = self.app_state.task_progress_broadcast {
            let core_event = botcore::shared::state::TaskProgressEvent::new(
                &event.run_id,
                &event.step,
                &event.message,
            )
            .with_progress(event.current_step, event.total_steps);
            let _ = sender.send(core_event);
        }
    }

    fn progress_sender(&self) -> Option<&broadcast::Sender<VibeProgressEvent>> {
        Some(&self.vibe_sender)
    }

    fn active_runs(&self) -> &Arc<RwLock<HashMap<Uuid, VibeRun>>> {
        &self.active_runs
    }
}

pub fn configure_vibe_routes(_app_state: &Arc<AppState>) -> axum::Router {
    let (vibe_sender, _) = broadcast::channel(256);
    let vibe_state: Arc<dyn VibeState> = Arc::new(BotServerVibeState {
        app_state: app_state.clone(),
        active_runs: Arc::new(RwLock::new(HashMap::new())),
        vibe_sender,
    });

    let registry = Arc::new(botvibe::ToolRegistry::new());
    let executor = Arc::new(botvibe::VibeToolExecutor::new(registry));
    let prompt_manager = Arc::new(botvibe::VibePromptManager::new());
    let telemetry = Arc::new(botvibe::VibeTelemetry::new());

    botvibe::router(vibe_state, prompt_manager, executor, telemetry)
}
