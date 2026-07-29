pub mod api;
pub mod message_processing;
pub mod models;
pub mod schema;
pub mod session_management;
pub mod state;
pub mod utils;
pub mod webhooks;

pub const FB_WEBHOOK: &str = "/api/facebook/webhook";
pub const FB_SEND: &str = "/api/facebook/send";
pub const FB_STATUS: &str = "/api/facebook/status";
pub const FB_SESSIONS: &str = "/api/facebook/sessions";

use axum::{Router, routing::{get, post}};
use std::sync::Arc;

pub fn configure_facebook_routes() -> Router<Arc<state::FacebookState>> {
    Router::new()
        .route(FB_WEBHOOK, post(webhooks::handle_webhook))
        .route(FB_WEBHOOK, get(webhooks::handle_webhook_verify))
        .route(FB_SEND, post(api::handle_send_message))
        .route(FB_STATUS, get(api::handle_status))
        .route(FB_SESSIONS, get(api::handle_sessions))
}
