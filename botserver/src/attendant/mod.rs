use axum::Router;
use std::sync::Arc;

use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;

pub fn configure_attendant_routes(app_state: &Arc<AppState>) -> Router<()> {
    let config = Arc::new(botattendant::AttendantConfig {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: get_default_bot as botattendant::GetDefaultBotFn,
    });
    botattendant::configure_attendant_routes().with_state(config)
}

pub fn configure_attendant_ui_routes(app_state: &Arc<AppState>) -> Router<()> {
    let config = Arc::new(botattendant::AttendantConfig {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: get_default_bot as botattendant::GetDefaultBotFn,
    });
    botattendant::configure_attendant_ui_routes().with_state(config)
}
