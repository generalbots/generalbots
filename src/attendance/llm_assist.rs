pub mod llm_assist_types;
pub mod llm_assist_config;
pub mod llm_assist_handlers;
pub mod llm_assist_commands;
pub mod llm_assist_helpers;

// Re-export commonly used types
pub use llm_assist_types::*;

// Re-export handlers for routing
pub use llm_assist_handlers::*;
pub use llm_assist_commands::*;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::core::shared::state::AppState;

pub fn llm_assist_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/llm-assist/config/:bot_id", get(get_llm_config))
        .route("/llm-assist/tips", post(generate_tips))
        .route("/llm-assist/polish", post(polish_message))
        .route("/llm-assist/replies", post(generate_smart_replies))
        .route("/llm-assist/summary/:session_id", get(generate_summary))
        .route("/llm-assist/sentiment", post(analyze_sentiment))
}
