use axum::{Router, routing::post};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use botcore::shared::state::AppState;

#[cfg(feature = "deployment")]
pub async fn build_sub_router(
    app_state: &Arc<AppState>,
    port: u16,
    api_router: &mut Router<Arc<AppState>>,
) -> Router<()> {
    let mut sub_router = inner_build_sub_router(app_state, port, api_router).await;
    let dep_pool = app_state.conn.clone();
    let dep_router = crate::deployment::configure_deployment_routes(dep_pool);
    sub_router = sub_router.merge(dep_router);
    sub_router
}

#[cfg(not(feature = "deployment"))]
pub async fn build_sub_router(
    app_state: &Arc<AppState>,
    port: u16,
    api_router: &mut Router<Arc<AppState>>,
) -> Router<()> {
    inner_build_sub_router(app_state, port, api_router).await
}

async fn inner_build_sub_router(
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

    *api_router = api_router
        .clone()
        .merge(super::workspace_tabs::configure_workspace_tabs_routes());

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

    // Canonical tenant-scoped integration connection control plane (#939).
    // Mounted only when the secrets manager initializes; a Vault outage skips
    // the mount (fail closed) while the rest of the API keeps serving.
    #[cfg(feature = "integrations")]
    match botcoresecrets::SecretsManager::get_clone() {
        Ok(secrets_manager) => {
            let connections_state = std::sync::Arc::new(botintegrations::IntegrationState::new(
                app_state.conn.clone(),
                botintegrations::secrets::ConnectionVault::new(secrets_manager),
            ));
            *api_router = api_router
                .clone()
                .merge(botintegrations::configure_connection_routes().with_state(connections_state.clone()));
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

    #[cfg(feature = "contacts")]
    {
        let contacts_state = Arc::new(crate::contacts::CrateState {
            db_pool: app_state.conn.clone(),
            get_default_bot: Arc::new(|_c: &mut diesel::PgConnection| uuid::Uuid::nil()),
            trigger_contact_change: Arc::new(|_c: &mut diesel::PgConnection, _id: uuid::Uuid, _kind: &str, _by: uuid::Uuid| {}),
            trigger_deal_stage_change: Arc::new(
                |_c: &mut diesel::PgConnection, _id: uuid::Uuid, _old: &str, _new: &str, _by: uuid::Uuid| {},
            ),
        });
        *api_router = api_router.clone().merge(crate::contacts::routes::configure_all_routes().with_state(contacts_state));
    }

    // ===== AI OS routers (issues #1167-#1179) =====
    #[cfg(feature = "agent-vm")]
    {
        let agent_state = Arc::new(botagent::AgentService::new(app_state.conn.clone()));
        crate::core::bot::agent_vm_hook::init_with(agent_state.clone());
        *api_router = api_router
            .clone()
            .merge(botagent::configure_routes().with_state(agent_state));
    }

    #[cfg(feature = "automations")]
    {
        let automation_state = Arc::new(
            botautomation::AutomationService::new(app_state.conn.clone())
                .with_llm(Arc::new(|_system: &str, _user: &str, _params: &str| {
                    Err("LLM not wired for automations".to_string())
                }))
                .with_delivery(Arc::new(|_channel: &str, _to: &str, _subject: &str, _body: &str| {
                    Ok(())
                })),
        );
        botautomation::scheduler::spawn_scheduler(automation_state.clone());
        *api_router = api_router
            .clone()
            .merge(botautomation::configure_routes().with_state(automation_state));
    }

    #[cfg(feature = "marketplace")]
    {
        let marketplace_state = Arc::new(botmarketplace::MarketplaceService::new(app_state.conn.clone()));
        let seed_state = marketplace_state.clone();
        tokio::spawn(async move {
            match botmarketplace::seed_if_empty(&seed_state).await {
                Ok(count) if count > 0 => log::info!("marketplace seeded {count} starter skills"),
                Ok(_) => {}
                Err(e) => log::error!("marketplace seed failed: {e}"),
            }
        });
        *api_router = api_router
            .clone()
            .merge(botmarketplace::configure_routes().with_state(marketplace_state));
    }

    #[cfg(feature = "consent")]
    {
        crate::core::bot::consent_gate::init(app_state.conn.clone());
        let consent_state = Arc::new(botconsent::ConsentService::new(app_state.conn.clone()));
        consent_state.ensure_sweeper();
        *api_router = api_router
            .clone()
            .merge(botconsent::configure_routes().with_state(consent_state));
    }

    #[cfg(feature = "memory-os")]
    {
        let memory_state = Arc::new(botmemory::MemoryService::new(
            app_state.conn.clone(),
            Arc::new(|_system: &str, _user: &str, _params: &str| {
                Err("LLM not wired for memory extraction".to_string())
            }),
        ));
        crate::core::bot::memory_hook::init_with(memory_state.clone());
        *api_router = api_router
            .clone()
            .merge(botmemory::configure_routes().with_state(memory_state));
    }

    #[cfg(feature = "connectors")]
    { *api_router = api_router.clone().merge(botconnectors::configure()); }

    #[cfg(feature = "browser-policy")]
    {
        let browser_policy_state =
            Arc::new(botbrowserpolicy::BrowserPolicyService::new(app_state.conn.clone()));
        *api_router = api_router
            .clone()
            .merge(botbrowserpolicy::configure_routes().with_state(browser_policy_state));
    }

    #[cfg(feature = "channel-bindings")]
    {
        let bindings_state =
            Arc::new(botchannelbindings::ChannelBindingsService::new(app_state.conn.clone()));
        *api_router = api_router
            .clone()
            .merge(botchannelbindings::configure_routes().with_state(bindings_state));
    }

    sub_router = sub_router.merge(crate::core::i18n::configure_i18n_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(crate::security::configure_protection_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(crate::settings::configure_settings_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(crate::jukebox::configure_jukebox_routes());
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
    #[cfg(feature = "desktop")]
    {
        sub_router = sub_router.merge(botdesktop::routes::configure_routes(Some(app_state.conn.clone())));
    }
    sub_router = sub_router.merge(crate::api::system::configure_system_routes().with_state(app_state.clone()));
    #[cfg(feature = "meet")]
    {
        sub_router = sub_router.merge(crate::meet::configure().with_state(app_state.clone()));
    }

    #[cfg(feature = "chat")]
    {
        sub_router = sub_router.merge(super::chat_handlers::configure_chat_routes().with_state(app_state.clone()));
        sub_router = sub_router.merge(super::chat_history::configure_chat_history_routes().with_state(app_state.clone()));
    }

    sub_router = sub_router.merge(super::misc_handlers::configure_misc_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(super::bot_tree::configure_bot_tree_routes().with_state(app_state.clone()));
    sub_router = sub_router.merge(super::unified_search::configure_unified_search_routes().with_state(app_state.clone()));
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
            history: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        })));
    }

    #[cfg(feature = "paper")]
    {
        sub_router = sub_router.merge(crate::paper::configure_paper_routes().with_state(super::feature_routers::make_paper_state(app_state)));
    }

    #[cfg(feature = "research")]
    {
        #[derive(Debug, Clone)]
        struct ResearchAppState {
            pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
            llm: Option<Arc<dyn botlib::traits::LLMProvider>>,
        }
        impl crate::research::ResearchState for ResearchAppState {
            fn db_pool(&self) -> &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>> {
                &self.pool
            }
            fn llm_provider(&self) -> Option<Arc<dyn botlib::traits::LLMProvider>> {
                self.llm.clone()
            }
            fn bot_id(&self) -> Option<uuid::Uuid> {
                self.resolve_bot_id()
            }
        }

        impl ResearchAppState {
            fn resolve_bot_id(&self) -> Option<uuid::Uuid> {
                use diesel::prelude::*;
                let mut conn = match self.pool.get() {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Research bot_id DB connection error: {e}");
                        return None;
                    }
                };
                #[derive(diesel::QueryableByName)]
                #[diesel(check_for_backend(diesel::pg::Pg))]
                struct BotIdRow {
                    #[diesel(sql_type = diesel::sql_types::Uuid)]
                    id: uuid::Uuid,
                }
                diesel::sql_query(
                    "SELECT id FROM bots WHERE is_default_for_branch = TRUE ORDER BY created_at ASC LIMIT 1",
                )
                .get_result::<BotIdRow>(&mut conn)
                .ok()
                .map(|r| r.id)
            }
        }

        let research_state = Arc::new(ResearchAppState {
            pool: app_state.conn.clone(),
            llm: app_state.llm_provider.clone(),
        });
        sub_router = sub_router.merge(crate::research::configure_research_routes().with_state(research_state.clone()));
        sub_router = sub_router.merge(crate::research::ui::configure_research_ui_routes().with_state(research_state));
    }

    #[cfg(feature = "search")]
    {
        let search_state = Arc::new(crate::search::SearchService::new(Arc::new(app_state.conn.clone()), None));
        sub_router = sub_router.merge(crate::search::configure_search_routes().with_state(search_state));
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
        sub_router = sub_router.merge(crate::compliance::ui::configure_compliance_ui_routes().with_state(Arc::new(compliance_pool.clone())));
        sub_router = sub_router.merge(crate::compliance::dashboard::configure_dashboard_routes().with_state(Arc::new(compliance_pool)));
    }

    #[cfg(feature = "monitoring")]
    {
        struct MonitoringAppState {
            app_state: Arc<AppState>,
            collector: Arc<crate::monitoring::MetricsCollector>,
        }

        impl crate::monitoring::MonitoringState for MonitoringAppState {
            fn active_session_count(&self) -> usize {
                use diesel::prelude::*;
                let mut conn = match self.app_state.conn.get() {
                    Ok(c) => c,
                    Err(_) => return 0,
                };

                #[derive(diesel::QueryableByName)]
                struct CountResult {
                    #[diesel(sql_type = diesel::sql_types::BigInt)]
                    count: i64,
                }

                diesel::sql_query(
                    "SELECT COUNT(*) as count FROM user_sessions WHERE expires_at IS NULL OR expires_at > NOW()",
                )
                .get_result::<CountResult>(&mut conn)
                .map(|r| r.count.max(0) as usize)
                .unwrap_or(0)
            }

            fn is_db_healthy(&self) -> bool {
                use diesel::prelude::*;
                let mut conn = match self.app_state.conn.get() {
                    Ok(c) => c,
                    Err(_) => return false,
                };
                diesel::sql_query("SELECT 1").execute(&mut conn).is_ok()
            }

            fn metrics_collector(&self) -> Arc<crate::monitoring::MetricsCollector> {
                self.collector.clone()
            }

            fn dependencies(&self) -> Vec<crate::monitoring::DependencyStatus> {
                use std::time::Instant;
                let mut deps = Vec::new();

                // PostgreSQL
                {
                    let start = Instant::now();
                    let healthy = self.is_db_healthy();
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    deps.push(crate::monitoring::DependencyStatus {
                        name: "PostgreSQL",
                        host: "postgres".to_string(),
                        healthy,
                        latency_ms: latency,
                    });
                }

                // Cache / Valkey
                if let Some(cache) = self.app_state.cache.as_ref() {
                    let start = Instant::now();
                    let healthy = cache
                        .get_connection_with_timeout(std::time::Duration::from_millis(750))
                        .and_then(|mut conn| {
                            redis::cmd("PING").query::<String>(&mut conn)
                        })
                        .map(|reply| reply.to_uppercase() == "PONG")
                        .unwrap_or(false);
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    deps.push(crate::monitoring::DependencyStatus {
                        name: "Valkey",
                        host: "cache".to_string(),
                        healthy,
                        latency_ms: latency,
                    });
                }

                // MinIO / Drive
                if let Some(drive) = self.app_state.drive.as_ref() {
                    let start = Instant::now();
                    let healthy = tokio::task::block_in_place(|| {
                        futures::executor::block_on(drive.list_all_buckets())
                            .map(|buckets| !buckets.is_empty())
                            .unwrap_or(false)
                    });
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    deps.push(crate::monitoring::DependencyStatus {
                        name: "MinIO",
                        host: "drive".to_string(),
                        healthy,
                        latency_ms: latency,
                    });
                }

                // LLM
                {
                    let url = std::env::var("LLM_URL")
                        .or_else(|_| std::env::var("OLLAMA_HOST"))
                        .ok();
                    let start = Instant::now();
                    let healthy = url
                        .and_then(|u| crate::monitoring::host_port_from_url(&u, 8081))
                        .map(|(host, port)| {
                            use std::net::{SocketAddr, TcpStream};
                            let addr: SocketAddr = format!("{host}:{port}").parse().ok()?;
                            Some(TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(750)).is_ok())
                        })
                        .flatten()
                        .unwrap_or(false);
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    deps.push(crate::monitoring::DependencyStatus {
                        name: "LLM",
                        host: "llm".to_string(),
                        healthy,
                        latency_ms: latency,
                    });
                }

                deps
            }
        }

        let collector = Arc::new(crate::monitoring::MetricsCollector::new());
        {
            let c = collector.clone();
            tokio::spawn(async move {
                c.setup_default_alert_rules().await;
                c.start_background_collection().await;
            });
        }

        let monitoring_state = Arc::new(MonitoringAppState {
            app_state: app_state.clone(),
            collector: collector.clone(),
        });
        sub_router = sub_router.merge(
            crate::monitoring::configure::<MonitoringAppState, crate::monitoring::DefaultMonitoringUrls>()
                .with_state(monitoring_state)
        );
        sub_router = sub_router.merge(
            crate::monitoring::governance::configure_routes(collector)
        );
    }

    #[cfg(feature = "scripting")]
    { sub_router = sub_router.merge(crate::basic::keywords::configure_app_server_routes().with_state(app_state.clone())); }

    #[cfg(feature = "people")]
    { sub_router = sub_router.merge(crate::basic::keywords::configure_db_routes().with_state(app_state.clone())); }

    #[cfg(feature = "vibe")]
    { sub_router = sub_router.merge(crate::vibe::configure_vibe_routes(app_state).await); }

    #[cfg(feature = "project")]
    {
        let project_service = Arc::new(crate::project::ProjectService::with_pool(app_state.conn.clone()));
        project_service.load_from_db().await;
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

    #[cfg(feature = "analytics")]
    {
        let reports_default_bot: crate::analytics::GetDefaultBotFn =
            Arc::new(|_c: &mut diesel::PgConnection| uuid::Uuid::nil());
        let reports_state = Arc::new(crate::analytics::reports::ReportsState {
            pool: Arc::new(app_state.conn.clone()),
            default_bot: Arc::new(reports_default_bot),
        });
        sub_router = sub_router.merge(
            crate::analytics::reports::configure_reports_routes().with_state(reports_state),
        );
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
        // Give the session store a state handle so idle-evicted dirty
        // sessions can persist before dropping.
        sheet_state.sessions.set_state_handle(sheet_state.clone());
        sub_router = sub_router.merge(
            crate::sheet::routes::configure_sheet_routes()
                .layer(axum::middleware::from_fn(
                    crate::sheet::user_middleware::sheet_user_middleware,
                ))
                .with_state(sheet_state),
        );
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
    { sub_router = sub_router.merge(botretail::configure()); }

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

    #[cfg(feature = "player")]
    {
        sub_router = sub_router.merge(crate::player::configure_player_routes().with_state(app_state.clone()));
    }

    #[cfg(feature = "terminal")]
    { sub_router = sub_router.merge(crate::api::terminal::routes::configure_terminal_routes(app_state.conn.clone())); }

    // AutoTask routes
    {
        use botautotask::types::{AutoTaskState, ConfigOps};

        struct AutoTaskStateImpl {
            pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
            bucket_name: String,
            manifests: Arc<RwLock<HashMap<String, botautotask::TaskManifest>>>,
            drive_ops: Option<Arc<dyn botautotask::types::DriveOps>>,
        }

        impl AutoTaskState for AutoTaskStateImpl {
            fn db_pool(&self) -> &botautotask::types::DbPool {
                &self.pool
            }
            fn bucket_name(&self) -> &str {
                &self.bucket_name
            }
            fn file_ops(&self) -> Option<&dyn botautotask::types::DriveOps> {
                self.drive_ops.as_deref()
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
            drive_ops: app_state.drive.clone().map(|d| {
                Arc::new(botautotask::drive_ops::DriveRepositoryOps(d)) as Arc<dyn botautotask::types::DriveOps>
            }),
        });
        let config_ops = Arc::new(ConfigOpsImpl);
        let llm_ops = Arc::new(botautotask::llm_adapter::BotlibLlmAdapter(app_state.llm_provider.clone()));
        sub_router = sub_router.merge(botautotask::api::router(autotask_state, config_ops, llm_ops));
    }

    sub_router
}
