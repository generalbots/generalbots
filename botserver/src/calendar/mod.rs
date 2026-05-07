use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub fn configure_calendar_routes(app_state: &Arc<AppState>) -> Router<()> {
    let pool = Arc::new(app_state.conn.clone());
    botcalendar::configure_calendar_routes().with_state(pool)
}

pub fn configure_calendar_ui_routes(app_state: &Arc<AppState>) -> Router<()> {
    let pool = Arc::new(app_state.conn.clone());
    botcalendar::configure_calendar_ui_routes().with_state(pool)
}

pub fn create_caldav_router(app_state: &Arc<AppState>) -> Router<()> {
    let pool = Arc::new(app_state.conn.clone());
    botcalendar::create_caldav_router().with_state(pool)
}
