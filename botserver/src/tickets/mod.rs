use axum::Router;
use std::sync::Arc;

use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;

pub fn configure_tickets_routes(app_state: &Arc<AppState>) -> Router<()> {
    let tickets_state = Arc::new(bottickets::TicketsState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: get_default_bot as bottickets::GetDefaultBotFn,
    });
    bottickets::configure_tickets_routes().with_state(tickets_state)
}

pub fn configure_tickets_ui_routes(app_state: &Arc<AppState>) -> Router<()> {
    let tickets_state = Arc::new(bottickets::TicketsState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: get_default_bot as bottickets::GetDefaultBotFn,
    });
    bottickets::ui::configure_tickets_ui_routes().with_state(tickets_state)
}
