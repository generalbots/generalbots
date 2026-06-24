use std::sync::Arc;

use axum::Router;

use crate::handlers::*;
use crate::handlers_activity::*;
use crate::handlers_charts::*;
use crate::{DbPool, GetBotContextFn, GetDefaultBotFn};

pub fn create_analytics_router(pool: Arc<DbPool>) -> Router {
    Router::new()
        .route("/api/analytics/messages/count", axum::routing::get(handle_message_count))
        .route("/api/analytics/sessions/active", axum::routing::get(handle_active_sessions))
        .route("/api/analytics/response/avg", axum::routing::get(handle_avg_response_time))
        .route("/api/analytics/llm/tokens", axum::routing::get(handle_llm_tokens))
        .route("/api/analytics/storage/usage", axum::routing::get(handle_storage_usage))
        .route("/api/analytics/errors/count", axum::routing::get(handle_errors_count))
        .route("/api/analytics/timeseries/messages", axum::routing::get(handle_timeseries_messages))
        .route("/api/analytics/timeseries/response", axum::routing::get(handle_timeseries_response))
        .route("/api/analytics/channels/distribution", axum::routing::get(handle_channels_distribution))
        .route("/api/analytics/bots/performance", axum::routing::get(handle_bots_performance))
        .route("/api/analytics/activity/recent", axum::routing::get(handle_recent_activity))
        .route("/api/analytics/queries/top", axum::routing::get(handle_top_queries))
        .route("/api/analytics/chat", axum::routing::post(handle_analytics_chat))
        .with_state(pool)
}

pub fn create_goals_api_router(state: (Arc<DbPool>, GetBotContextFn)) -> Router {
    #[cfg(feature = "goals")]
    {
        use crate::goals::configure_goals_routes;
        configure_goals_routes().with_state(state)
    }
    #[cfg(not(feature = "goals"))]
    {
        let _ = state;
        Router::new()
    }
}

pub fn create_goals_ui_router(state: (Arc<DbPool>, GetDefaultBotFn)) -> Router {
    #[cfg(feature = "goals")]
    {
        use crate::goals_ui::configure_goals_ui_routes;
        configure_goals_ui_routes().with_state(state)
    }
    #[cfg(not(feature = "goals"))]
    {
        let _ = state;
        Router::new()
    }
}

pub fn create_insights_router(pool: Arc<DbPool>) -> Router {
    use crate::insights::configure_insights_routes;
    configure_insights_routes().with_state(pool)
}
