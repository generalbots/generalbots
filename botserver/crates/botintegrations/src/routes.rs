use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::handlers;
use crate::handlers_actions;
use crate::handlers_connections;
use crate::handlers_context;
use crate::handlers_lifecycle;
use crate::state::IntegrationState;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route(
            "/api/integrations/connectors",
            get(handlers::list_connectors),
        )
        .route(
            "/api/integrations/connectors/:id/connect",
            post(handlers::connect_connector),
        )
        .route(
            "/api/integrations/connectors/:id/disconnect",
            post(handlers::disconnect_connector),
        )
        .route("/api/integrations/etl", get(handlers::list_etl))
}

/// Canonical tenant-scoped integration connection control plane (#939).
///
/// All routes resolve the caller's tenant scope server-side from the
/// authenticated user extension; credentials never appear in responses -
/// they live exclusively in Vault.
pub fn configure_connection_routes() -> Router<Arc<IntegrationState>> {
    Router::new()
        .route(
            "/api/bots/:bot_id/integration-connections",
            get(handlers_connections::list).post(handlers_connections::create),
        )
        .route(
            "/api/bots/:bot_id/integration-connections/:connection_id",
            get(handlers_connections::get_one).delete(handlers_connections::remove),
        )
        .route(
            "/api/bots/:bot_id/integration-connections/:connection_id/test",
            post(handlers_lifecycle::test_connection),
        )
        .route(
            "/api/bots/:bot_id/integration-connections/:connection_id/rotate",
            post(handlers_lifecycle::rotate),
        )
        .route(
            "/api/bots/:bot_id/integration-connections/:connection_id/events",
            get(handlers_lifecycle::list_events),
        )
        .route(
            "/api/bots/:bot_id/integration-actions/invoke",
            post(handlers_actions::invoke_action),
        )
        .route(
            "/api/apps/integrations/context",
            get(handlers_context::context),
        )
}
