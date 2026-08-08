//! Vibe subsystem wiring (#743): mounts the botvibe router with a concrete
//! `VibeState` implementation backed by the main AppState pool, and seeds the
//! `vibe_projects` schema at boot.

use botcore::shared::state::AppState;
use botvibe::backups::Backups;
use botvibe::domains::ProjectDomains;
use botvibe::domains_api::domains_router;
use botvibe::members_api::members_router;
use botvibe::metering::VMetering;
use botvibe::metering_api::metering_router;
use botvibe::ops::VmOps;
use botvibe::ops_api::{ops_router, OpsRoutes};
use botvibe::projects::ProjectRegistry;
use botvibe::projects_api::projects_router;
use botvibe::rbac::ProjectRbac;
use botvibe::types::{VibeProgressEvent, VibeRun, VibeState};
use botvibe::vm_lifecycle::VmLifecycle;
use botvibe::vms_api::vms_router;
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

    let vm_lifecycle = Arc::new(VmLifecycle::new(app_state.conn.clone()));
    match vm_lifecycle.ensure_schema() {
        Ok(()) => info!("Vibe: vm_instances schema ensured"),
        Err(e) => log::error!("Vibe: ensure vm_instances schema failed: {e}"),
    }

    let domain_binds = Arc::new(ProjectDomains::new(app_state.conn.clone()));
    match domain_binds.ensure_schema() {
        Ok(()) => info!("Vibe: project_domains schema ensured"),
        Err(e) => log::error!("Vibe: ensure project_domains schema failed: {e}"),
    }

    let prompt_manager = Arc::new(VibePromptManager::new());
    let tool_executor = Arc::new(VibeToolExecutor::new(Arc::new(ToolRegistry::new())));
    let telemetry = Arc::new(VibeTelemetry::new());

    let vm_ops = Arc::new(VmOps::new(app_state.conn.clone()));
    let backups = Arc::new(Backups::new(app_state.conn.clone()));
    match backups.ensure_schema() {
        Ok(()) => info!("Vibe: vm_backups schema ensured"),
        Err(e) => log::error!("Vibe: ensure vm_backups schema failed: {e}"),
    }

    let project_rbac = ProjectRbac::new(app_state.conn.clone());
    match project_rbac.ensure_schema() {
        Ok(()) => info!("Vibe: project_members schema ensured"),
        Err(e) => log::error!("Vibe: ensure project_members schema failed: {e}"),
    }

    let metering = Arc::new(VMetering::new(app_state.conn.clone()));
    match metering.ensure_schema() {
        Ok(()) => info!("Vibe: vm_metering schema ensured"),
        Err(e) => log::error!("Vibe: ensure vm_metering schema failed: {e}"),
    }

    botvibe::api::router(state, prompt_manager, tool_executor, telemetry)
        .merge(projects_router(
            project_registry.clone(),
            project_rbac.clone(),
            metering.clone(),
        ))
        .merge(vms_router(
            vm_lifecycle,
            project_registry.clone(),
            project_rbac.clone(),
            metering.clone(),
        ))
        .merge(domains_router(domain_binds, project_rbac.clone(), metering.clone()))
        .merge(members_router(project_rbac.clone()))
        .merge(metering_router(metering.clone(), project_rbac.clone()))
        .merge(ops_router(OpsRoutes {
            vm_ops,
            backups,
            registry: project_registry,
            pool: app_state.conn.clone(),
            rbac: project_rbac,
            metering: metering.clone(),
        }))
}