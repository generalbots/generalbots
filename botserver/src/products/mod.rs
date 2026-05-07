use axum::Router;
use std::sync::Arc;

use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;

pub fn configure_products_routes(app_state: &Arc<AppState>) -> Router<()> {
    let state = Arc::new(botproducts::ProductsState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: Some(get_default_bot as botproducts::GetDefaultBotFn),
    });
    botproducts::configure_products_routes().with_state(state)
}

pub fn configure_products_api_routes(app_state: &Arc<AppState>) -> Router<()> {
    let state = Arc::new(botproducts::ProductsState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: Some(get_default_bot as botproducts::GetDefaultBotFn),
    });
    botproducts::configure_products_api_routes().with_state(state)
}
