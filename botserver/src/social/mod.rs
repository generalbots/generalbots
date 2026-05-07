use axum::Router;
use std::sync::Arc;

use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;

pub use botsocial::SocialState;

pub mod ui {
    use axum::Router;
    use std::sync::Arc;

    use crate::core::bot::get_default_bot;
    use crate::core::shared::state::AppState;

    pub fn configure_social_ui_routes(app_state: &Arc<AppState>) -> Router<()> {
        let state = Arc::new(botsocial::SocialState {
            pool: Arc::new(app_state.conn.clone()),
            get_default_bot: Arc::new(move |conn| get_default_bot(conn)),
        });
        botsocial::configure_social_ui_routes()
            .with_state(state)
    }
}

pub fn configure_social_routes(app_state: &Arc<AppState>) -> Router<()> {
    let state = Arc::new(botsocial::SocialState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: Arc::new(move |conn| get_default_bot(conn)),
    });
    botsocial::configure_social_routes()
        .with_state(state)
}
