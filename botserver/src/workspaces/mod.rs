use axum::Router;
use std::sync::Arc;

use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;

pub fn configure_workspaces_routes(app_state: &Arc<AppState>) -> Router<()> {
    let ws_state = Arc::new(botworkspaces::WorkspacesState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: get_default_bot as botworkspaces::GetDefaultBotFn,
    });
    botworkspaces::configure_workspaces_routes().with_state(ws_state)
}

pub fn configure_workspaces_ui_routes(app_state: &Arc<AppState>) -> Router<()> {
    let ws_state = Arc::new(botworkspaces::WorkspacesState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: get_default_bot as botworkspaces::GetDefaultBotFn,
    });
    botworkspaces::configure_workspaces_ui_routes().with_state(ws_state)
}
