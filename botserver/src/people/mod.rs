use axum::Router;
use std::sync::Arc;

use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;

pub fn configure_people_routes(app_state: &Arc<AppState>) -> Router<()> {
    let people_state = Arc::new(botpeople::PeopleState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: Arc::new(get_default_bot) as botpeople::GetDefaultBotFn,
    });
    botpeople::configure_people_routes().with_state(people_state)
}

pub fn configure_people_ui_routes(app_state: &Arc<AppState>) -> Router<()> {
    let people_state = Arc::new(botpeople::PeopleState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: Arc::new(get_default_bot) as botpeople::GetDefaultBotFn,
    });
    botpeople::ui::configure_people_ui_routes().with_state(people_state)
}
