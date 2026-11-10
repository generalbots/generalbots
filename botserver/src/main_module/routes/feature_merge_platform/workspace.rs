//! Vibe, project, goals, analytics, sheet and canvas router merges.

use axum::Router;
use botcore::shared::state::AppState;
use std::sync::Arc;

pub(super) async fn merge_workspace(
    app_state: &Arc<AppState>,
    sub_router: &mut Router<()>,
) {
    #[cfg(feature = "vibe")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::vibe::configure_vibe_routes(app_state).await);
    }

    #[cfg(feature = "project")]
    {
        let project_service = Arc::new(crate::project::ProjectService::with_pool(
            app_state.conn.clone(),
        ));
        project_service.load_from_db().await;
        let project_router = crate::project::configure(project_service.clone());
        *sub_router = sub_router
            .clone()
            .merge(project_router.with_state(project_service.clone()));
        *sub_router = sub_router.clone().merge(
            crate::project::project_ui::configure_project_ui_routes().with_state(project_service),
        );
    }

    #[cfg(all(feature = "analytics", feature = "goals"))]
    {
        let goals_pool = Arc::new(app_state.conn.clone());

        // The previous closure returned (nil, nil). `okr_objectives.org_id`
        // has an FK to `organizations`, so every create failed with a
        // foreign-key violation. Resolve the default bot's real tenant once;
        // the goals API has no request-scoped context to read it from.
        let goals_context = (|| {
            #[derive(diesel::QueryableByName)]
            struct DefaultBotContext {
                #[diesel(sql_type = diesel::sql_types::Text)]
                org_id: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                branch_id: String,
            }
            use diesel::RunQueryDsl;
            let mut conn = match goals_pool.get() {
                Ok(conn) => conn,
                Err(error) => {
                    log::warn!("goals: no DB connection for tenant context: {error}");
                    return (uuid::Uuid::nil(), uuid::Uuid::nil());
                }
            };
            let row: Result<DefaultBotContext, _> = diesel::sql_query(
                "SELECT org_id::text AS org_id, branch_id::text AS branch_id \
                 FROM bots WHERE name = 'default' ORDER BY created_at LIMIT 1",
            )
            .get_result(&mut conn);
            match row {
                Ok(row) => (
                    uuid::Uuid::parse_str(&row.org_id).unwrap_or_default(),
                    uuid::Uuid::parse_str(&row.branch_id).unwrap_or_default(),
                ),
                Err(error) => {
                    log::warn!("goals: default bot tenant context unavailable: {error}");
                    (uuid::Uuid::nil(), uuid::Uuid::nil())
                }
            }
        })();
        let goals_bot_context: crate::analytics::GetBotContextFn = Arc::new(move || goals_context);
        let goals_default_bot: crate::analytics::GetDefaultBotFn =
            Arc::new(|_c: &mut diesel::PgConnection| uuid::Uuid::nil());
        // The global auth middleware classifies an invalid bearer token as
        // anonymous and still inserts an extension, so the shared extractor
        // alone let anonymous callers read and mutate goals (a forged JWT was
        // one way in). Require a real authenticated user on both surfaces.
        let require_auth = axum::middleware::from_fn(
            crate::security::botsecurity_auth::auth_api::middleware::require_auth_middleware,
        );
        *sub_router = sub_router.clone().merge(
            crate::analytics::goals::configure_goals_routes()
                .with_state((goals_pool.clone(), goals_bot_context))
                .layer(require_auth.clone()),
        );
        *sub_router = sub_router.clone().merge(
            crate::analytics::goals_ui::configure_goals_ui_routes()
                .with_state((goals_pool, goals_default_bot))
                .layer(require_auth),
        );
    }

    #[cfg(feature = "analytics")]
    {
        let reports_default_bot: crate::analytics::GetDefaultBotFn =
            Arc::new(|_c: &mut diesel::PgConnection| uuid::Uuid::nil());
        let reports_state = Arc::new(crate::analytics::reports::ReportsState {
            pool: Arc::new(app_state.conn.clone()),
            default_bot: Arc::new(reports_default_bot),
        });
        *sub_router = sub_router
            .clone()
            .merge(crate::analytics::reports::configure_reports_routes().with_state(reports_state));
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
        *sub_router = sub_router.clone().merge(
            crate::sheet::routes::configure_sheet_routes()
                .layer(axum::middleware::from_fn(
                    crate::sheet::user_middleware::sheet_user_middleware,
                ))
                .with_state(sheet_state),
        );
    }

    #[cfg(feature = "canvas")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_canvas_router(app_state));
    }
}
