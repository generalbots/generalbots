//! Plan route table (issue #2136).
//!
//! `/ws/plan/:plan_id` is still matched by the middleware's anonymous `/ws`
//! prefix — a browser WebSocket cannot send an `Authorization` header — so the
//! upgrade handler itself proves the identity and the plan membership before
//! subscribing (see [`crate::collaboration::handle_plan_websocket`]). Every
//! REST route below resolves the caller from the verified Bearer token.

use super::*;
use collaboration::{
    handle_get_plan_collaborators, handle_get_plan_presence, handle_get_plan_typing,
    handle_plan_websocket,
};
use axum::routing::{get, post};
use axum::Router;

pub fn configure_plan_routes() -> Router {
    Router::new()
        .route("/ws/plan/:plan_id", get(handle_plan_websocket))
        .route("/api/plan", post(handle_create_plan))
        .route(
            "/api/plan/:plan_id/collaborators",
            get(handle_get_plan_collaborators),
        )
        .route(
            "/api/plan/:plan_id/presence",
            get(handle_get_plan_presence),
        )
        .route("/api/plan/:plan_id/typing", get(handle_get_plan_typing))
        .route(
            "/api/plan/:plan_id",
            get(handle_get_plan).delete(handle_delete_plan),
        )
        .route(
            "/api/plan/:plan_id/members",
            post(handle_add_member).delete(handle_remove_member),
        )
        .route("/api/plan/task", post(handle_create_task))
        .route("/api/plan/task/update", post(handle_update_task))
        .route("/api/plan/task/delete", post(handle_delete_task))
        .route("/api/plan/task/depends", post(handle_set_dependencies))
        .route("/api/plan/ai/assist", post(handle_plan_ai_assist))
}
