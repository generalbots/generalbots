use axum::{Router, routing::post};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use botcore::shared::state::AppState;

#[cfg(feature = "deployment")]
pub fn build_sub_router(
    app_state: &Arc<AppState>,
    port: u16,
    api_router: &mut Router<Arc<AppState>>,
) -> Router<()> {
    let mut sub_router = inner_build_sub_router(app_state, port, api_router);
    let dep_pool = app_state.conn.clone();
    let dep_router = crate::deployment::configure_deployment_routes(dep_pool);
    sub_router = sub_router.merge(dep_router);
    sub_router
}

#[cfg(not(feature = "deployment"))]
pub fn build_sub_router(
    app_state: &Arc<AppState>,
    port: u16,
    api_router: &mut Router<Arc<AppState>>,
) -> Router<()> {
    inner_build_sub_router(app_state, port, api_router)
}

fn inner_build_sub_router(
    app_state: &Arc<AppState>,
    port: u16,
    api_router: &mut Router<Arc<AppState>>,
) -> Router<()> {
    let mut sub_router: Router<()> = Router::new();

    {
        let directory_api_state = Arc::new(crate::directory::api::DirectoryApiState {
            conn: app_state.conn.clone(),
            base_url: format!("http://localhost:{}", port),
        });
        *api_router = api_router.clone().merge(crate::directory::api::configure_user_routes().with_state(directory_api_state));
    }

    *api_router = api_router.clone().merge(crate::apps::register(Router::new()));

    #[cfg(feature = "tax")]
    { *api_router = api_router.clone().merge(bottax::configure()); }

    #[cfg(feature = "vision")]
    { *api_router = api_router.clone().merge(botvision::configure()); }

    #[cfg(feature = "erp")]
    { *api_router = api_router.clone().merge(boterp::configure()); }

    // botintegrations and botsources both expose /api/integrations/* — sources owns the
    // namespace when both are compiled (its handlers are the superset: sync, run, create).
    #[cfg(all(feature = "integrations", not(feature = "sources")))]
    { *api_router = api_router.clone().merge(botintegrations::configure()); }

    #[cfg(feature = "hr")]
    { *api_router = api_router.clone().merge(bothr::configure()); }

    #[cfg(feature = "sales")]
    { *api_router = api_router.clone().merge(botsales::configure()); }

    #[cfg(feature = "minutes")]
    { *api_router = api_router.clone().merge(botminutes::configure()); }

    #[cfg(feature = "templates")]
    { *api_router = api_router.clone().merge(bottemplates::configure()); }

    #[cfg(feature = "itsm")]
    { *api_router = api_router.clone().merge(botitsm::configure()); }

    #[cfg(feature = "pos")]
    { *api_router = api_router.clone().merge(botpos::configure()); }

    #[cfg(feature = "handoff")]
    { *api_router = api_router.clone().merge(bothandoff::configure()); }

    #[cfg(feature = "kyc")]
    { *api_router = api_router.clone().merge(botkyc::configure()); }

    #[cfg(feature = "timeclock")]
    { *api_router = api_router.clone().merge(bottimeclock::configure()); }

    sub_router = sub_router.merge(crate::core::i18n::configure_i18n_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(crate::security::configure_protection_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(crate::settings::configure_settings_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(botcore::shared::admin::configure().with_state(app_state.clone()));
    sub_router = sub_router.merge(botcore::shared::analytics::configure().with_state(app_state.clone()));
    sub_router = sub_router.merge(botcore::organization_invitations::configure().with_state(app_state.clone()));

    // BotCoder IDE APIs
    #[cfg(feature = "editor")]
    { sub_router = sub_router.merge(boteditor::configure().with_state(app_state.clone())); }
    #[cfg(feature = "database")]
    { sub_router = sub_router.merge(botdatabase::configure().with_state(app_state.clone())); }
    #[cfg(feature = "git")]
    { sub_router = sub_router.merge(botgit::configure().with_state(app_state.clone())); }
    sub_router = sub_router.merge(crate::api::system::configure_system_routes().with_state(app_state.clone()));

    #[cfg(feature = "meet")]
    { sub_router = sub_router.merge(crate::meet::configure().with_state(app_state.clone())); }

    #[cfg(feature = "tasks")]
    {
        sub_router = sub_router.merge(crate::tasks::configure_tasks_routes().with_state(Arc::new(bottasks::state::TasksState {
            pool: app_state.conn.clone(),
            run_command: Arc::new(|_cmd: &str, _args: &[&str]| -> Result<String, String> { Ok(String::new()) }),
            call_llm: Arc::new(|_prompt: &str, _ctx: &str| Box::pin(async { Ok(String::new()) })),
            get_config: Arc::new(|_key: &str| -> Result<String, String> { Ok(String::new()) }),
            cache_get: Arc::new(|_key: String| Box::pin(async { Ok(None) })),
            cache_set: Arc::new(|_key: String, _val: String, _ttl: Option<u64>| Box::pin(async { Ok(()) })),
        })));
    }

    #[cfg(feature = "analytics")]
    {
        sub_router = sub_router.merge(crate::analytics::routes::create_analytics_router(Arc::new(app_state.conn.clone())));
        sub_router = sub_router.merge(crate::analytics::insights::configure_insights_routes().with_state(Arc::new(app_state.conn.clone())));
    }

    #[cfg(feature = "docs")]
    {
        sub_router = sub_router.merge(crate::docs::configure_docs_routes().with_state(Arc::new(botdocs::state::DocState {
            pool: Arc::new(app_state.conn.clone()),
            drive: app_state.drive.clone().unwrap_or_else(|| Arc::new(crate::drive::NoopDrive)),
            bucket_name: app_state.bucket_name.clone(),
        })));
    }

    #[cfg(feature = "paper")]
    {
        sub_router = sub_router.merge(crate::paper::configure_paper_routes().with_state(super::feature_routers::make_paper_state(app_state)));
    }

    #[cfg(feature = "research")]
    {
        #[derive(Debug, Clone)]
        struct ResearchAppState(diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>);

        impl crate::research::ResearchState for ResearchAppState {
            fn db_pool(&self) -> &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>> {
                &self.0
            }
        }

        let research_state = Arc::new(ResearchAppState(app_state.conn.clone()));
        sub_router = sub_router.merge(crate::research::configure_research_routes().with_state(research_state.clone()));
        sub_router = sub_router.merge(crate::research::ui::configure_research_ui_routes().with_state(research_state));
    }

    #[cfg(any(feature = "research", feature = "llm"))]
    {
        *api_router = api_router.clone().route(
            "/api/website/force-recrawl",
            post(crate::core::kb::website_crawler_service::handle_force_recrawl)
        );
    }

    #[cfg(feature = "designer")]
    {
        sub_router = sub_router.merge(super::feature_routers::make_designer_router(app_state));
    }

    #[cfg(feature = "dashboards")]
    {
        sub_router = sub_router.merge(super::feature_routers::make_dashboards_router(app_state));
    }

    #[cfg(feature = "legal")]
    {
        let legal_pool = app_state.conn.clone();
        sub_router = sub_router.merge(crate::legal::configure_legal_routes().with_state(Arc::new(legal_pool.clone())));
        sub_router = sub_router.merge(crate::legal::configure_legal_ui_routes().with_state(Arc::new(legal_pool)));
    }

    #[cfg(feature = "compliance")]
    {
        let compliance_pool = app_state.conn.clone();
        sub_router = sub_router.merge(crate::compliance::configure_compliance_routes().with_state(Arc::new(compliance_pool.clone())));
        sub_router = sub_router.merge(crate::compliance::ui::configure_compliance_ui_routes().with_state(Arc::new(compliance_pool)));
    }

    #[cfg(feature = "monitoring")]
    {
        struct MonitoringAppState;

        impl crate::monitoring::MonitoringState for MonitoringAppState {
            fn active_session_count(&self) -> usize { 0 }
            fn is_db_healthy(&self) -> bool { true }
        }

        let monitoring_state = Arc::new(MonitoringAppState);
        sub_router = sub_router.merge(
            crate::monitoring::configure::<MonitoringAppState, crate::monitoring::DefaultMonitoringUrls>()
                .with_state(monitoring_state)
        );
        sub_router = sub_router.merge(
            crate::monitoring::governance::configure_routes(
                Arc::new(crate::monitoring::MetricsCollector::new())
            )
        );
    }

    #[cfg(feature = "scripting")]
    { sub_router = sub_router.merge(crate::basic::keywords::configure_app_server_routes().with_state(app_state.clone())); }

    #[cfg(feature = "people")]
    { sub_router = sub_router.merge(crate::basic::keywords::configure_db_routes().with_state(app_state.clone())); }

    #[cfg(feature = "vibe")]
    { sub_router = sub_router.merge(crate::vibe::configure_vibe_routes(app_state)); }

    #[cfg(feature = "project")]
    {
        let project_service = Arc::new(crate::project::ProjectService::new());
        let project_router = crate::project::configure(project_service.clone());
        sub_router = sub_router.merge(project_router.with_state(project_service.clone()));
        sub_router = sub_router.merge(crate::project::project_ui::configure_project_ui_routes().with_state(project_service));
    }

    #[cfg(all(feature = "analytics", feature = "goals"))]
    {
        let goals_pool = Arc::new(app_state.conn.clone());
        let goals_bot_context: crate::analytics::GetBotContextFn = Arc::new(|| (uuid::Uuid::nil(), uuid::Uuid::nil()));
        let goals_default_bot: crate::analytics::GetDefaultBotFn = Arc::new(|_c: &mut diesel::PgConnection| uuid::Uuid::nil());
        sub_router = sub_router.merge(crate::analytics::goals::configure_goals_routes().with_state((goals_pool.clone(), goals_bot_context)));
        sub_router = sub_router.merge(crate::analytics::goals_ui::configure_goals_ui_routes().with_state((goals_pool, goals_default_bot)));
    }

    #[cfg(feature = "sheet")]
    {
        let sheet_drive = app_state.drive.as_ref().map(|d| {
            Arc::new(crate::sheet::drive_adapter::DriveOpsAdapter(d.clone()))
                as Arc<dyn crate::sheet::state::DriveOps>
        });
        let mut sheet_state = crate::sheet::state::SheetState::new(sheet_drive.clone());
        // Wire xlsx save-back hook: every sheet save also writes back
        // to the original .xlsx in Drive (if loaded from one).
        if let Some(ref drive) = sheet_drive {
            sheet_state.on_save = Some(crate::sheet::storage::create_save_back_hook(drive.clone()));
        }
        let sheet_state = Arc::new(sheet_state);
        sub_router = sub_router.merge(crate::sheet::routes::configure_sheet_routes().with_state(sheet_state));
    }

    #[cfg(feature = "canvas")]
    { sub_router = sub_router.merge(super::feature_routers::make_canvas_router(app_state)); }

    #[cfg(feature = "fraud")]
    {
        let fraud_state = Arc::new(crate::fraud::FraudState::new(app_state.conn.clone()));
        sub_router = sub_router.merge(crate::fraud::configure_fraud_routes().with_state(fraud_state));
    }

    #[cfg(feature = "inventory")]
    {
        let inventory_state = Arc::new(crate::inventory::InventoryState { pool: app_state.conn.clone() });
        sub_router = sub_router.merge(crate::inventory::configure_inventory_routes().with_state(inventory_state));
    }

    #[cfg(feature = "gl")]
    {
        let gl_state = Arc::new(crate::gl::GlState { pool: app_state.conn.clone() });
        sub_router = sub_router.merge(crate::gl::configure_gl_routes().with_state(gl_state));
    }

    #[cfg(feature = "retail")]
    { sub_router = sub_router.merge(crate::retail::configure_retail_routes().with_state(Arc::new(crate::retail::RetailState))); }

    #[cfg(feature = "banking")]
    { *api_router = api_router.clone().merge(botbanking::configure()); }

    #[cfg(feature = "m365")]
    { *api_router = api_router.clone().merge(botm365::configure()); }

    #[cfg(feature = "weba")]
    {
        let weba_state = Arc::new(crate::weba::WebaState::new());
        sub_router = sub_router.merge(crate::weba::configure_routes(weba_state));
    }

    sub_router = sub_router.merge(crate::directory::scim::server::configure_scim_routes().with_state(app_state.clone()));

    #[cfg(feature = "social")]
    { sub_router = sub_router.merge(super::feature_routers::make_social_router(app_state)); }

    #[cfg(feature = "learn")]
    {
        sub_router = sub_router.merge(crate::learn::ui::configure_learn_ui_routes());
        sub_router = sub_router.merge(crate::learn::creator::configure_learn_api_routes().with_state(Arc::new(botlearn::GamificationService::new())));
    }

    #[cfg(feature = "meet")]
    { sub_router = sub_router.merge(crate::meet::ui::configure_meet_ui_routes().with_state(app_state.clone())); }

    #[cfg(feature = "billing")]
    { sub_router = sub_router.merge(super::feature_routers::make_billing_router(app_state)); }

    #[cfg(feature = "saas")]
    { sub_router = sub_router.merge(super::feature_routers::make_saas_router(app_state)); }

    #[cfg(feature = "whatsapp")]
    { sub_router = sub_router.merge(crate::whatsapp::configure(app_state)); }

    #[cfg(feature = "marketing")]
    { sub_router = sub_router.merge(super::feature_routers::make_marketing_router(app_state)); }

    #[cfg(feature = "telegram")]
    { sub_router = sub_router.merge(super::feature_routers::make_telegram_router(app_state)); }

    #[cfg(feature = "instagram")]
    { sub_router = sub_router.merge(super::feature_routers::make_instagram_router(app_state)); }

    #[cfg(feature = "msteams")]
    { sub_router = sub_router.merge(super::feature_routers::make_msteams_router(app_state)); }

    #[cfg(feature = "sources")]
    {
        let sources_state = crate::sources::make_sources_state(app_state.conn.clone());
        sub_router = sub_router.merge(crate::sources::configure_sources_api_routes().with_state(sources_state));
    }

    #[cfg(feature = "attendant")]
    { sub_router = sub_router.merge(super::feature_routers::make_attendant_router(app_state)); }

    #[cfg(feature = "browser")]
    {
        use std::sync::Arc;
        use tokio::sync::Mutex;
        sub_router = sub_router.merge(
            crate::browser::api::configure_routes()
                .with_state(Arc::new(Mutex::new(std::collections::HashMap::new()))),
        );
    }

    #[cfg(feature = "terminal")]
    { sub_router = sub_router.merge(crate::api::terminal::configure_terminal_routes()); }

    // AutoTask routes
    {
        use botautotask::types::{AutoTaskState, ConfigOps};

        struct AutoTaskStateImpl {
            pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
            bucket_name: String,
            manifests: Arc<RwLock<HashMap<String, botautotask::TaskManifest>>>,
        }

        impl AutoTaskState for AutoTaskStateImpl {
            fn db_pool(&self) -> &botautotask::types::DbPool {
                &self.pool
            }
            fn bucket_name(&self) -> &str {
                &self.bucket_name
            }
            fn broadcast_task_progress(&self, _event: botautotask::types::TaskProgressEvent) {}
            fn emit_activity(&self, _task_id: &str, _step: &str, _message: &str, _current: u8, _total: u8, _activity: botautotask::types::AgentActivity) {}
            fn emit_task_started(&self, _task_id: &str, _message: &str, _total_steps: u8) {}
            fn emit_task_error(&self, _task_id: &str, _step: &str, _error: &str) {}
            fn task_manifests(&self) -> &Arc<RwLock<HashMap<String, botautotask::TaskManifest>>> {
                &self.manifests
            }
            fn task_progress_broadcast(&self) -> Option<&tokio::sync::broadcast::Sender<botautotask::types::TaskProgressEvent>> {
                None
            }
        }

        struct ConfigOpsImpl;

        impl ConfigOps for ConfigOpsImpl {
            fn get_config(&self, _bot_id: &uuid::Uuid, _key: &str, _default: Option<&str>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
                Ok(_default.unwrap_or_default().to_string())
            }
            fn set_config(&self, _bot_id: &uuid::Uuid, _key: &str, _value: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                Ok(())
            }
        }

        let autotask_state = Arc::new(AutoTaskStateImpl {
            pool: Arc::new(app_state.conn.clone()),
            bucket_name: app_state.bucket_name.clone(),
            manifests: Arc::new(RwLock::new(HashMap::new())),
        });
        let config_ops = Arc::new(ConfigOpsImpl);
        sub_router = sub_router.merge(botautotask::api::router(autotask_state, config_ops));
    }

    sub_router
}
