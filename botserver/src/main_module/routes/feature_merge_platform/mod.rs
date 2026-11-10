//! Second half of the feature-block router merges.
//!
//! #1730 — the merge list outgrew the 450-line limit, so it is grouped by
//! responsibility into sibling modules. `merge_platform_routers` keeps the
//! original order (governance, workspace, commerce, channels) and the
//! `sub_router.rs` call site is unchanged.

use axum::Router;
use botcore::shared::state::AppState;
use std::sync::Arc;

mod channels;
mod commerce;
mod governance;
mod workspace;

/// Applies the second half of the feature-block merges (legal, compliance,
/// monitoring, scripting, people, vibe, project, goals, sheet, canvas,
/// fraud, inventory, gl, retail, banking, weba and the channel routers)
/// onto the api/sub routers.
pub(super) async fn merge_platform_routers(
    app_state: &Arc<AppState>,
    api_router: &mut Router<Arc<AppState>>,
    sub_router: &mut Router<()>,
) {
    // Only the banking/m365 blocks merge into the authenticated API router;
    // a build without those features (the default) leaves it untouched.
    let _ = &api_router;
    governance::merge_governance(app_state, sub_router).await;
    workspace::merge_workspace(app_state, sub_router).await;
    commerce::merge_commerce(app_state, api_router, sub_router).await;
    channels::merge_channels(app_state, sub_router).await;
}
