//! Canvas router wiring.
//!
//! Lives outside the `saas`-gated helper module: the release build ships
//! `canvas` without `saas`, and the previous home
//! (`resolve_default_bot_id.rs`, cfg(saas)) left `make_canvas_router`
//! unresolved — the whole botserver failed to compile (E0425).

use super::*;

#[cfg(feature = "canvas")]
fn canvas_branch_pool(
    pool: &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
) -> (uuid::Uuid, uuid::Uuid) {
    let mut conn = match pool.get() {
        Ok(connection) => connection,
        Err(_) => return (uuid::Uuid::nil(), uuid::Uuid::nil()),
    };
    let branch = resolve_default_branch(&mut conn);
    (branch, branch)
}

#[cfg(feature = "canvas")]
pub(crate) fn make_canvas_router(app_state: &Arc<AppState>) -> Router<()> {
    let state = Arc::new(botcanvas::CanvasState {
        pool: Arc::new(app_state.conn.clone()),
        get_bot_context: Arc::new(canvas_branch_pool
            as fn(
                &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
            ) -> (uuid::Uuid, uuid::Uuid)),
    });
    Router::new()
        .merge(crate::canvas::configure_canvas_routes().with_state(state.clone()))
        .merge(crate::canvas::configure_canvas_ui_routes().with_state(state))
}
