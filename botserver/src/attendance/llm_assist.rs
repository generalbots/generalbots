use crate::attendance::{llm_assist_types, llm_assist_config, llm_assist_handlers, llm_assist_commands};

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::core::shared::state::AppState;

pub use llm_assist_types::*;
pub use llm_assist_handlers::*;
pub use llm_assist_commands::*;

pub fn llm_assist_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/llm-assist/config/:bot_id", get(get_llm_config))
        .route("/llm-assist/tips", post(generate_tips))
        .route("/llm-assist/polish", post(polish_message))
        .route("/llm-assist/replies", post(generate_smart_replies))
        .route("/llm-assist/summary/:session_id", get(generate_summary))
        .route("/llm-assist/sentiment", post(analyze_sentiment))
}
