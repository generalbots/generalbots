//! Directory, apps, sandbox, MCP, run-quality, budget, sub-agent and
//! workspace-tab routers merged into the authenticated API router.

use axum::Router;
use botcore::shared::state::AppState;
use std::sync::Arc;

pub(super) async fn merge_office_api_surfaces(
    app_state: &Arc<AppState>,
    port: u16,
    api_router: &mut Router<Arc<AppState>>,
) {
    let _ = port;
    let _ = &app_state;
    #[cfg(feature = "directory")]
    {
        let directory_api_state = Arc::new(crate::directory::api::DirectoryApiState {
            conn: app_state.conn.clone(),
            base_url: format!("http://localhost:{}", port),
        });
        *api_router = api_router
            .clone()
            .merge(crate::directory::api::configure_user_routes().with_state(directory_api_state));
    }

    *api_router = api_router
        .clone()
        .merge(crate::apps::register(Router::new()));

    // #1674 — signed, content-addressed sandbox resources. The handler is
    // origin-independent; the dedicated listener below provides isolation.
    *api_router = api_router
        .clone()
        .merge(crate::core::bot::sandbox::routes::router());
    // #1676 — generated apps over the Model Context Protocol.
    *api_router = api_router
        .clone()
        .merge(crate::core::bot::mcp_apps::router());
    *api_router = api_router
        .clone()
        .merge(crate::core::bot::runs::quality::router());
    // #1700 — first-token latency summary and hourly trend.
    *api_router = api_router
        .clone()
        .merge(crate::core::bot::runs::ttft::router());
    *api_router = api_router
        .clone()
        .merge(crate::core::bot::runs::tools::router());
    // #1697 — pooled group budgets and usage reports.
    *api_router = api_router.clone().merge(crate::main_module::routes::group_budgets::router());
    // #1694 — parallel sub-agent fan-out.
    *api_router = api_router.clone().merge(crate::main_module::routes::subagents::router());
    // #1705 — long-horizon goal runs with automatic checkpoints.
    *api_router = api_router.clone().merge(crate::main_module::routes::goal_runs::router());

    *api_router = api_router
        .clone()
        .merge(crate::main_module::routes::workspace_tabs::configure_workspace_tabs_routes());

    #[cfg(feature = "tax")]
    {
        *api_router = api_router.clone().merge(bottax::configure());
    }

    #[cfg(feature = "vision")]
    {
        *api_router = api_router.clone().merge(botvision::configure());
    }

    #[cfg(feature = "erp")]
    {
        *api_router = api_router.clone().merge(boterp::configure());
    }

    // botsources owns the /api/integrations/* namespace (connectors + ETL);
    // the legacy botintegrations router that used to shadow-mount here is
    // deleted (#1350). The modern connection control plane below occupies a
    // different namespace (/api/bots/:bot_id/integration-*) and mounts
    // unconditionally.

    // Canonical tenant-scoped integration connection control plane (#939).
    // Mounted only when the secrets manager initializes; a Vault outage skips
    // the mount (fail closed) while the rest of the API keeps serving.
    #[cfg(feature = "integrations")]
    {
        // #2100 — the workspace context and the @mention lookup read only the
        // database, so they must survive a missing credential store: the
        // desktop edition ships without Vault and every suite app built its
        // connection URLs from `/api/apps/integrations/context`.
        *api_router = api_router
            .clone()
            .merge(botintegrations::configure_context_routes());
    }
    #[cfg(feature = "integrations")]
    match botcoresecrets::SecretsManager::get_clone() {
        Ok(secrets_manager) => {
            let connections_state = std::sync::Arc::new(botintegrations::IntegrationState::new(
                app_state.conn.clone(),
                botintegrations::secrets::ConnectionVault::new(secrets_manager),
            ));
            *api_router = api_router.clone().merge(
                botintegrations::configure_connection_routes()
                    .with_state(connections_state.clone()),
            );
            botintegrations::automations::spawn(connections_state.clone());
            botintegrations::token_refresh::spawn(connections_state.clone());
        }
        Err(error) => {
            log::error!(
                "integration connection control plane not mounted (secrets manager init failed): {error}"
            );
        }
    }

    #[cfg(feature = "hr")]
    {
        *api_router = api_router.clone().merge(bothr::configure());
    }

    #[cfg(feature = "sales")]
    {
        *api_router = api_router.clone().merge(botsales::configure());
    }

    #[cfg(feature = "minutes")]
    {
        *api_router = api_router.clone().merge(botminutes::configure());
    }

    #[cfg(feature = "templates")]
    {
        *api_router = api_router.clone().merge(bottemplates::configure());
    }

    #[cfg(feature = "itsm")]
    {
        *api_router = api_router.clone().merge(botitsm::configure());
    }

    #[cfg(feature = "pos")]
    {
        *api_router = api_router.clone().merge(botpos::configure());
    }

    #[cfg(feature = "handoff")]
    {
        *api_router = api_router.clone().merge(bothandoff::configure());
    }

    #[cfg(feature = "kyc")]
    {
        *api_router = api_router.clone().merge(botkyc::configure());
    }

    #[cfg(feature = "timeclock")]
    {
        *api_router = api_router.clone().merge(bottimeclock::configure());
    }

    #[cfg(feature = "contacts")]
    {
        let contacts_state = Arc::new(crate::contacts::CrateState {
            db_pool: app_state.conn.clone(),
            get_default_bot: Arc::new(|_c: &mut diesel::PgConnection| uuid::Uuid::nil()),
            trigger_contact_change: Arc::new(
                |_c: &mut diesel::PgConnection, _id: uuid::Uuid, _kind: &str, _by: uuid::Uuid| {},
            ),
            trigger_deal_stage_change: Arc::new(
                |_c: &mut diesel::PgConnection,
                 _id: uuid::Uuid,
                 _old: &str,
                 _new: &str,
                 _by: uuid::Uuid| {},
            ),
        });
        // #1452 — seed default pipeline stages for branches with CRM activity
        // but no stage rows (idempotent; keeps the kanban data-driven).
        crate::contacts::stages::seed_default_stages(&contacts_state);
        *api_router = api_router
            .clone()
            .merge(crate::contacts::routes::configure_all_routes().with_state(contacts_state));
    }
}
