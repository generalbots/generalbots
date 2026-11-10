//! i18n, security, settings, admin, editor, database, git, desktop, system,
//! chat, task, analytics, docs, paper, research and search routers merged
//! into the sub router.

use axum::Router;
use botcore::shared::state::AppState;
use std::sync::Arc;

pub(super) async fn merge_office_sub_routes(
    app_state: &Arc<AppState>,
    sub_router: &mut Router<()>,
) {
    *sub_router = sub_router
        .clone()
        .merge(crate::core::i18n::configure_i18n_routes().with_state(app_state.clone()));
    *sub_router = sub_router.clone().merge(
        crate::security::protection::configure_protection_routes().with_state(app_state.clone()),
    );
    *sub_router = sub_router
        .clone()
        .merge(crate::settings::configure_settings_routes().with_state(app_state.clone()));
    *sub_router = sub_router
        .clone()
        .merge(crate::jukebox::configure_jukebox_routes());
    *sub_router = sub_router
        .clone()
        .merge(crate::botmodels_gateway::configure_botmodels_gateway_routes());
    *sub_router = sub_router
        .clone()
        .merge(botcore::shared::admin::configure().with_state(app_state.clone()));
    *sub_router = sub_router
        .clone()
        .merge(botcore::shared::analytics::configure().with_state(app_state.clone()));
    *sub_router = sub_router
        .clone()
        .merge(botcore::organization_invitations::configure().with_state(app_state.clone()));

    // BotCoder IDE APIs
    #[cfg(feature = "editor")]
    {
        *sub_router = sub_router
            .clone()
            .merge(boteditor::configure().with_state(app_state.clone()));
    }
    #[cfg(feature = "database")]
    {
        *sub_router = sub_router
            .clone()
            .merge(botdatabase::configure().with_state(app_state.clone()));
    }
    #[cfg(feature = "git")]
    {
        *sub_router = sub_router
            .clone()
            .merge(botgit::configure().with_state(app_state.clone()));
    }
    #[cfg(feature = "desktop")]
    {
        *sub_router = sub_router
            .clone()
            .merge(botdesktop::routes::configure_routes(Some(
                app_state.conn.clone(),
            )));
    }
    *sub_router = sub_router
        .clone()
        .merge(crate::api::system::configure_system_routes().with_state(app_state.clone()));
    #[cfg(feature = "meet")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::meet::configure().with_state(app_state.clone()));
    }

    #[cfg(feature = "chat")]
    {
        *sub_router = sub_router
            .clone()
            .merge(super::chat_handlers::configure_chat_routes().with_state(app_state.clone()));
        *sub_router = sub_router.clone().merge(
            super::chat_history::configure_chat_history_routes().with_state(app_state.clone()),
        );
    }

    *sub_router = sub_router
        .clone()
        .merge(super::misc_handlers::configure_misc_routes().with_state(app_state.clone()));
    *sub_router = sub_router
        .clone()
        .merge(super::bot_tree::configure_bot_tree_routes().with_state(app_state.clone()));
    *sub_router = sub_router.clone().merge(
        super::unified_search::configure_unified_search_routes().with_state(app_state.clone()),
    );
    #[cfg(feature = "tasks")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::tasks::configure_tasks_routes().with_state(Arc::new(
                bottasks::state::TasksState {
                    pool: app_state.conn.clone(),
                    run_command: Arc::new(|_cmd: &str, _args: &[&str]| -> Result<String, String> {
                        Ok(String::new())
                    }),
                    call_llm: Arc::new(|_prompt: &str, _ctx: &str| {
                        Box::pin(async { Ok(String::new()) })
                    }),
                    get_config: Arc::new(|_key: &str| -> Result<String, String> {
                        Ok(String::new())
                    }),
                    cache_get: Arc::new(|_key: String| Box::pin(async { Ok(None) })),
                    cache_set: Arc::new(|_key: String, _val: String, _ttl: Option<u64>| {
                        Box::pin(async { Ok(()) })
                    }),
                },
            )));
    }

    #[cfg(feature = "analytics")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::analytics::routes::create_analytics_router(Arc::new(
                app_state.conn.clone(),
            )));
        *sub_router = sub_router.clone().merge(
            crate::analytics::insights::configure_insights_routes()
                .with_state(Arc::new(app_state.conn.clone())),
        );
    }

    #[cfg(feature = "docs")]
    {
        // #2135 — every Docs handler derives its namespace from the verified
        // owner, so the router gets the middleware that turns the platform
        // principal into a `DocUser` (owner resolution is shared with Slides and
        // Paper in `security::office_owner`).
        let docs_state = Arc::new(botdocs::state::DocState {
            pool: Arc::new(app_state.conn.clone()),
            drive: app_state
                .drive
                .clone()
                .unwrap_or_else(|| Arc::new(crate::drive::NoopDrive)),
            bucket_name: app_state.bucket_name.clone(),
            history: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        });

        // Pre-#2135 documents live under `users/default-user/docs`, a namespace
        // every account shared. On a single-user install the install owner is
        // provably the only account that wrote them, so they are copied into its
        // namespace (sources kept). A multi-user deployment has no derivable
        // owner, so the objects stay orphaned and are reported once.
        #[cfg(feature = "desktop-local")]
        let legacy_adopter: Option<String> = {
            let no_security = botcore::product::ProductConfig::load()
                .map(|product| product.no_security)
                .unwrap_or_else(|_| {
                    botcore::product::PRODUCT_CONFIG
                        .read()
                        .map(|product| product.no_security)
                        .unwrap_or(false)
                });
            if no_security {
                Some(crate::security::office_owner::office_install_owner())
            } else {
                None
            }
        };
        #[cfg(not(feature = "desktop-local"))]
        let legacy_adopter: Option<String> = None;

        let legacy_state = docs_state.clone();
        tokio::spawn(async move {
            match legacy_adopter {
                Some(owner) => {
                    botdocs::legacy::adopt_legacy_namespace(&legacy_state, &owner).await;
                }
                None => {
                    botdocs::legacy::audit_legacy_namespace(&legacy_state).await;
                }
            }
        });

        *sub_router = sub_router.clone().merge(
            crate::docs::configure_docs_routes()
                .layer(axum::middleware::from_fn(
                    crate::docs::user_middleware::docs_user_middleware,
                ))
                .with_state(docs_state),
        );
    }

    #[cfg(feature = "paper")]
    {
        *sub_router = sub_router.clone().merge(
            crate::paper::configure_paper_routes()
                .layer(axum::middleware::from_fn(
                    crate::paper::user_middleware::paper_user_middleware,
                ))
                .with_state(super::feature_routers::make_paper_state(app_state)),
        );
    }

    #[cfg(feature = "research")]
    {
        #[derive(Debug, Clone)]
        struct ResearchAppState {
            pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
            llm: Option<Arc<dyn botlib::traits::LLMProvider>>,
        }
        impl crate::research::ResearchState for ResearchAppState {
            fn db_pool(
                &self,
            ) -> &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>
            {
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
        *sub_router = sub_router
            .clone()
            .merge(crate::research::configure_research_routes().with_state(research_state.clone()));
        *sub_router = sub_router
            .clone()
            .merge(crate::research::ui::configure_research_ui_routes().with_state(research_state));
    }

    #[cfg(feature = "search")]
    {
        let search_state = Arc::new(crate::search::SearchService::new(
            Arc::new(app_state.conn.clone()),
            None,
        ));
        *sub_router = sub_router
            .clone()
            .merge(crate::search::configure_search_routes().with_state(search_state));
    }
}
