use axum::{Router, routing::post};
use std::sync::Arc;
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

    sub_router = sub_router.merge(crate::core::i18n::configure_i18n_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(crate::security::configure_protection_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(crate::settings::configure_settings_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(botcore::shared::admin::configure().with_state(app_state.clone()));
    sub_router = sub_router.merge(botcore::shared::analytics::configure().with_state(app_state.clone()));
    sub_router = sub_router.merge(botcore::organization_invitations::configure().with_state(app_state.clone()));

    // BotCoder IDE APIs
    sub_router = sub_router.merge(crate::api::editor::configure_editor_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(crate::api::database::configure_database_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(crate::api::git::configure_git_routes().with_state(app_state.clone()));
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
        let goals_default_bot: crate::analytics::GetDefaultBotFn = Arc::new(|_c: &mut diesel::PgConnection| (uuid::Uuid::nil(), "default".to_string()));
        sub_router = sub_router.merge(crate::analytics::goals::configure_goals_routes().with_state((goals_pool.clone(), goals_bot_context)));
        sub_router = sub_router.merge(crate::analytics::goals_ui::configure_goals_ui_routes().with_state((goals_pool, goals_default_bot)));
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
    { sub_router = sub_router.merge(crate::whatsapp::configure(app_state.clone()).with_state(app_state.clone())); }

    #[cfg(feature = "marketing")]
    { sub_router = sub_router.merge(super::feature_routers::make_marketing_router(app_state)); }

    #[cfg(feature = "telegram")]
    { sub_router = sub_router.merge(super::feature_routers::make_telegram_router(app_state)); }

    #[cfg(feature = "instagram")]
    { sub_router = sub_router.merge(super::feature_routers::make_instagram_router()); }

    #[cfg(feature = "msteams")]
    { sub_router = sub_router.merge(super::feature_routers::make_msteams_router(app_state)); }

    #[cfg(feature = "sources")]
    {
        let sources_state = crate::sources::make_sources_state(app_state.conn.clone());
        sub_router = sub_router.merge(crate::sources::configure_sources_api_routes().with_state(sources_state.clone()));
        sub_router = sub_router.merge(crate::sources::configure_sources_ui_routes().with_state(sources_state));
    }

    #[cfg(feature = "attendant")]
    { sub_router = sub_router.merge(super::feature_routers::make_attendant_router(app_state)); }

    #[cfg(feature = "browser")]
    {
        sub_router = sub_router.merge(
            crate::browser::api::configure_browser_routes::<crate::browser::AppStateBrowserState>()
                .with_state(Arc::new(crate::browser::AppStateBrowserState(Arc::clone(app_state)))),
        );
    }

    #[cfg(feature = "terminal")]
    { sub_router = sub_router.merge(crate::api::terminal::configure_terminal_routes()); }

    sub_router
}
