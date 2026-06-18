use axum::Router;
use std::sync::Arc;

use crate::session_pool::DbPool;
use botanalytics::GetDefaultBotFn;

pub fn configure_goals_ui_routes() -> Router<(Arc<DbPool>, GetDefaultBotFn)> {
    botanalytics::goals_ui::configure_goals_ui_routes()
}
