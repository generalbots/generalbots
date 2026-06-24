use axum::Router;
use std::sync::Arc;

use crate::session_pool::DbPool;
use botanalytics::GetBotContextFn;

pub fn configure_goals_routes() -> Router<(Arc<DbPool>, GetBotContextFn)> {
    botanalytics::goals::configure_goals_routes()
}
