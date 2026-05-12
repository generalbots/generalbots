use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::handlers::*;
use crate::AttendantConfig;

pub fn configure_attendant_routes() -> Router<Arc<AttendantConfig>> {
    Router::new()
        .route("/api/attendant/queues", get(list_queues).post(create_queue))
        .route("/api/attendant/queues/:id", get(get_queue).delete(delete_queue))
        .route("/api/attendant/queues/:id/agents", post(add_queue_agent))
        .route("/api/attendant/queues/:queue_id/agents/:agent_id", delete(remove_queue_agent))
        .route("/api/attendant/sessions", get(list_sessions).post(create_session))
        .route("/api/attendant/sessions/:id", get(get_session))
        .route("/api/attendant/sessions/:id/assign", put(assign_session))
        .route("/api/attendant/sessions/:id/transfer", put(transfer_session))
        .route("/api/attendant/sessions/:id/end", put(end_session))
        .route("/api/attendant/sessions/:id/rate", put(rate_session))
        .route("/api/attendant/sessions/:id/messages", post(send_message))
        .route("/api/attendant/agents", get(list_agent_statuses))
        .route("/api/attendant/agents/:id/status", put(update_agent_status))
        .route("/api/attendant/canned", get(list_canned_responses).post(create_canned_response))
        .route("/api/attendant/tags", get(list_tags))
        .route("/api/attendant/wrap-up-codes", get(list_wrap_up_codes))
        .route("/api/attendant/stats", get(get_attendant_stats))
}
