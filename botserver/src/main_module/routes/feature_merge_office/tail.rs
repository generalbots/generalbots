//! Remaining mounts: the website re-crawl route plus the designer and
//! dashboards routers.

use axum::{routing::post, Router};
use botcore::shared::state::AppState;
use std::sync::Arc;

pub(super) async fn merge_office_tail(
    app_state: &Arc<AppState>,
    api_router: &mut Router<Arc<AppState>>,
    sub_router: &mut Router<()>,
) {
    let _ = &app_state;
    let _ = &api_router;
    let _ = &sub_router;
    #[cfg(any(feature = "research", feature = "llm"))]
    {
        *api_router = api_router.clone().route(
            "/api/website/force-recrawl",
            post(crate::core::kb::website_crawler_service::handle_force_recrawl),
        );
    }

    #[cfg(feature = "designer")]
    {
        *sub_router = sub_router
            .clone()
            .merge(super::feature_routers::make_designer_router(app_state));
    }

    #[cfg(feature = "dashboards")]
    {
        *sub_router = sub_router
            .clone()
            .merge(super::feature_routers::make_dashboards_router(app_state));
    }
}
