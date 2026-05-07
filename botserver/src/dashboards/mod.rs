use axum::Router;
use std::sync::Arc;

use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;

pub use botdashboards::{
    DashboardsError,
    handlers::*,
    storage::*,
    types::*,
};

pub fn configure_dashboards_routes(app_state: &Arc<AppState>) -> Router<()> {
    let state = Arc::new(botdashboards::DashboardsState {
        pool: app_state.conn.clone(),
        get_default_bot: get_default_bot as botdashboards::GetDefaultBotFn,
    });
    botdashboards::configure_dashboards_routes()
        .with_state(state)
}

pub fn configure_dashboards_ui_routes(app_state: &Arc<AppState>) -> Router<()> {
    let state = Arc::new(botdashboards::DashboardsState {
        pool: app_state.conn.clone(),
        get_default_bot: get_default_bot as botdashboards::GetDefaultBotFn,
    });
    botdashboards::ui::configure_dashboards_ui_routes()
        .with_state(state)
}
