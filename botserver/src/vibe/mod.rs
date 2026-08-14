//! Vibe subsystem wiring (#743): mounts the botvibe router with a concrete
//! `VibeState` implementation backed by the main AppState pool, and seeds the
//! `vibe_projects` schema at boot.

use botcore::config::ConfigManager;
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
use botvibe::types::{
    LlmConfig, VibeProgressEvent, VibeRun, VibeRunSignal, VibeState,
};
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
    signals: Option<broadcast::Sender<VibeRunSignal>>,
    config: ConfigManager,
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

    fn run_signal_sender(&self) -> Option<&broadcast::Sender<VibeRunSignal>> {
        self.signals.as_ref()
    }

    /// Resolves the bot's LLM settings via ConfigManager (Issue #795).
    /// Sensitive keys (e.g. `llm-key`) come from Vault per-bot paths;
    /// non-sensitive keys (url/model/provider) from the bot's Drive
    /// config.csv via the `bot_configuration` table. `get_config` returns
    /// `Ok("")` on a miss, so empty values are filtered out and fall through
    /// to the environment in the agent loop.
    fn llm_config(&self, bot_id: &Uuid) -> Option<LlmConfig> {
        let value = |key: &str| {
            self.config
                .get_config(bot_id, key, None)
                .ok()
                .filter(|v| !v.is_empty())
        };
        match value("llm-model") {
            Some(model) => Some(LlmConfig {
                model,
                key: value("llm-key").unwrap_or_default(),
                url: value("llm-url").unwrap_or_default(),
            }),
            None => {
                log::warn!("Vibe: no llm-model configured for bot {bot_id}; falling back to env");
                None
            }
        }
    }
}

pub async fn configure_vibe_routes(app_state: &Arc<AppState>) -> axum::Router {
    let pool = app_state.conn.clone();

    let state: Arc<dyn VibeState> = Arc::new(VibeStateImpl {
        pool,
        progress: Some(broadcast::channel(128).0),
        runs: Arc::new(RwLock::new(HashMap::new())),
        signals: Some(broadcast::channel(128).0),
        config: ConfigManager::new(app_state.conn.clone()),
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
    let permissions = Arc::new(botvibe::PermissionEngine::new());
    let skills = Arc::new(botvibe::SkillStore::new());
    if let Err(e) = skills.seed_bootstrap().await {
        log::error!("Vibe: bootstrap skills seeding failed: {e}");
    }
    // #816 — write-through persistence so canvases/issues/sessions/teams
    // survive server restarts (previously in-memory RwLock<Vec> only).
    let canvases = Arc::new(botvibe::CanvasStore::with_persistence(app_state.conn.clone()));
    let issues = Arc::new(botvibe::IssueStore::with_persistence(app_state.conn.clone()));
    let sessions = Arc::new(botvibe::SessionStore::with_persistence(app_state.conn.clone()));
    let teams = Arc::new(botvibe::TeamStore::with_persistence(app_state.conn.clone()));

    let tool_registry = Arc::new(ToolRegistry::new());
    if let Err(e) = tool_registry
        .register_m5_tools(skills.clone(), canvases.clone(), issues.clone())
        .await
    {
        log::error!("Vibe: M5 tool registration failed: {e}");
    }
    // VIBE_SCHEMA persistence (Issue #793): ensure tables at boot.
    match botvibe::run_store::ensure_vibe_schema(&app_state.conn) {
        Ok(()) => info!("Vibe: vibe_runs schema ensured"),
        Err(e) => log::error!("Vibe: ensure vibe_runs schema failed: {e}"),
    }
    let tool_executor = Arc::new(VibeToolExecutor::new(tool_registry));
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
    VMetering::spawn_vm_hours_sampler(metering.clone(), 3600);
    info!("Vibe: vm-hours metering sampler started (hourly)");

    botvibe::api::router(
        state.clone(),
        prompt_manager.clone(),
        tool_executor.clone(),
        telemetry.clone(),
        permissions.clone(),
        skills.clone(),
        app_state.conn.clone(),
    )
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
        .merge(botvibe::permissions_router(permissions.clone()))
        .merge(botvibe::skills_router(skills.clone()))
        .merge(botvibe::doctor_router(skills.clone(), sessions.clone()))
        .merge(botvibe::canvases_router(canvases.clone()))
        .merge(botvibe::issues_router(issues.clone()))
        .merge(botvibe::sessions_router(botvibe::SessionRoutes {
            sessions: sessions.clone(),
            state: state.clone(),
            prompt_manager: prompt_manager.clone(),
            tool_executor: tool_executor.clone(),
            telemetry: telemetry.clone(),
            permissions: permissions.clone(),
            skills: skills.clone(),
        }))
        .merge(botvibe::teams_router(botvibe::TeamRoutes {
            teams: teams.clone(),
            state: state.clone(),
            prompt_manager: prompt_manager.clone(),
            tool_executor: tool_executor.clone(),
            telemetry: telemetry.clone(),
            permissions: permissions.clone(),
            skills: skills.clone(),
        }))
}