//! Channel, source, attendant, browser, player, terminal, push and
//! automation router merges.

use axum::Router;
use botcore::shared::state::AppState;
use std::sync::Arc;

pub(super) async fn merge_channels(
    app_state: &Arc<AppState>,
    sub_router: &mut Router<()>,
) {
    #[cfg(feature = "whatsapp")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::whatsapp::configure(app_state));
    }

    #[cfg(feature = "marketing")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_marketing_router(app_state));
    }

    #[cfg(feature = "telegram")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_telegram_router(app_state));
    }

    #[cfg(feature = "instagram")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_instagram_router(app_state));
    }

    #[cfg(feature = "msteams")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_msteams_router(app_state));
    }

    #[cfg(feature = "sources")]
    {
        let sources_state = crate::sources::make_sources_state(app_state.conn.clone());
        *sub_router = sub_router
            .clone()
            .merge(crate::sources::configure_sources_api_routes().with_state(sources_state));
    }

    #[cfg(feature = "attendant")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_attendant_router(app_state));
    }

    #[cfg(feature = "browser")]
    {
        use tokio::sync::Mutex;
        *sub_router = sub_router.clone().merge(
            crate::browser::api::configure_routes()
                .with_state(Arc::new(Mutex::new(std::collections::HashMap::new()))),
        );
    }

    #[cfg(feature = "player")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::player::configure_player_routes().with_state(app_state.clone()));
    }

    #[cfg(feature = "terminal")]
    {
        *sub_router =
            sub_router
                .clone()
                .merge(crate::api::terminal::routes::configure_terminal_routes(
                    app_state.conn.clone(),
                ));
    }
    // #1841 — a build without the terminal feature still answers the terminal
    // client with a clear capability error instead of the generic API 404.
    #[cfg(not(feature = "terminal"))]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::api::terminal::routes::configure_unavailable_terminal_routes());
    }

    // #1247 — Web Push subscription backend (Settings push toggle).
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::api::push::configure_push_routes(
                app_state.conn.clone(),
            ));
    }

    #[cfg(feature = "automations")]
    {
        // AutoTask routes. The ops live in `crate::auto_task::ops` so the chat
        // command executor can build the same API for `tasks.autotask.create`.
        let api = crate::autotask_host::ops::build_api(app_state);
        *sub_router = sub_router.clone().merge(botautotask::api::router(
            api.state().clone(),
            api.config_ops().clone(),
            api.llm_ops().clone(),
        ));
    }
}
