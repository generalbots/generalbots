use axum::Router;
use std::sync::Arc;

use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;

pub use botcanvas::CanvasState;

pub mod ui {
    use axum::Router;
    use std::sync::Arc;

    use crate::core::bot::get_default_bot;
    use crate::core::shared::state::AppState;

    pub fn configure_canvas_ui_routes(app_state: &Arc<AppState>) -> Router<()> {
        let pool = Arc::new(app_state.conn.clone());
        let get_bot_context: botcanvas::GetBotContextFn = Arc::new(|pool_ref| {
            let Ok(mut conn) = pool_ref.get() else {
                return (uuid::Uuid::nil(), uuid::Uuid::nil());
            };
            let (bot_id, _) = get_default_bot(&mut conn);
            (uuid::Uuid::nil(), bot_id)
        });
        let state = Arc::new(botcanvas::CanvasState {
            pool,
            get_bot_context,
        });
        botcanvas::configure_canvas_ui_routes()
            .with_state(state)
    }
}

pub fn configure_canvas_routes(app_state: &Arc<AppState>) -> Router<()> {
    let pool = Arc::new(app_state.conn.clone());
    let get_bot_context: botcanvas::GetBotContextFn = Arc::new(|pool_ref| {
        let Ok(mut conn) = pool_ref.get() else {
            return (uuid::Uuid::nil(), uuid::Uuid::nil());
        };
        let (bot_id, _) = get_default_bot(&mut conn);
        (uuid::Uuid::nil(), bot_id)
    });
    let state = Arc::new(botcanvas::CanvasState {
        pool,
        get_bot_context,
    });
    botcanvas::configure_canvas_routes()
        .with_state(state)
}
