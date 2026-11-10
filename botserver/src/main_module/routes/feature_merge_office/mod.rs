//! First half of the feature-block router merges.
//!
//! #1730 — the merge list outgrew the 450-line limit, so it is grouped by
//! responsibility into sibling modules. `merge_office_routers` keeps the
//! original order (api surfaces, AI OS, sub routes, tail) and the
//! `sub_router.rs` call site is unchanged.

use axum::Router;
use botcore::shared::state::AppState;
use std::sync::Arc;

mod ai_os;
mod api_surfaces;
mod sub_routes;
mod tail;

/// Applies the first half of the feature-block merges (directory, apps,
/// workspace tabs, integrations, AI OS routers, memory, editor, database,
/// tasks, analytics, docs, research) onto the api/sub routers. Every block
/// is an independent `#[cfg]` merge preserved verbatim from the original
/// monolithic `inner_build_sub_router`.
pub(super) async fn merge_office_routers(
    app_state: &Arc<AppState>,
    port: u16,
    api_router: &mut Router<Arc<AppState>>,
    sub_router: &mut Router<()>,
) {
    api_surfaces::merge_office_api_surfaces(app_state, port, api_router).await;
    ai_os::merge_office_ai_os(app_state, api_router).await;
    sub_routes::merge_office_sub_routes(app_state, sub_router).await;
    tail::merge_office_tail(app_state, api_router, sub_router).await;
}
