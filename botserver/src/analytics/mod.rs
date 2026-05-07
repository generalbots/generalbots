use axum::Router;
use std::sync::Arc;

use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;

pub use botanalytics::{AnalyticsStats, AnalyticsQuery, AuthenticatedUser, DbPool, GetBotContextFn, GetDefaultBotFn};

#[cfg(feature = "goals")]
pub mod goals {
    use axum::Router;
    use std::sync::Arc;

    use crate::core::bot::get_default_bot;
    use crate::core::shared::state::AppState;

    pub fn configure_goals_routes(app_state: &Arc<AppState>) -> Router<()> {
        let pool = Arc::new(app_state.conn.clone());
        let get_bot_context: botanalytics::GetBotContextFn = Arc::new(|| {
            let Ok(mut conn) = pool.get() else {
                return (uuid::Uuid::nil(), uuid::Uuid::nil());
            };
            let (bot_id, _) = get_default_bot(&mut conn);
            (uuid::Uuid::nil(), bot_id)
        });
        botanalytics::routes::create_goals_api_router((pool, get_bot_context))
    }
}

#[cfg(feature = "goals")]
pub mod goals_ui {
    use axum::Router;
    use std::sync::Arc;

    use crate::core::bot::get_default_bot;
    use crate::core::shared::state::AppState;

    pub fn configure_goals_ui_routes(app_state: &Arc<AppState>) -> Router<()> {
        let pool = Arc::new(app_state.conn.clone());
        let get_default_bot_fn: botanalytics::GetDefaultBotFn = Arc::new(move |conn| get_default_bot(conn));
        botanalytics::routes::create_goals_ui_router((pool, get_default_bot_fn))
    }
}

pub mod insights {
    use axum::Router;
    use std::sync::Arc;

    use crate::core::shared::state::AppState;

    pub fn configure_insights_routes(app_state: &Arc<AppState>) -> Router<()> {
        let pool = Arc::new(app_state.conn.clone());
        botanalytics::routes::create_insights_router(pool)
    }
}

pub fn configure_analytics_routes(app_state: &Arc<AppState>) -> Router<()> {
    let pool = Arc::new(app_state.conn.clone());
    botanalytics::routes::create_analytics_router(pool)
}
