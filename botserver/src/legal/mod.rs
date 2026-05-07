use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub use botlegal::LegalService;

pub mod account_deletion {
    pub use botlegal::account_deletion::*;
}

pub mod ui {
    use axum::Router;
    use std::sync::Arc;

    use crate::core::shared::state::AppState;

    pub fn configure_legal_ui_routes(app_state: &Arc<AppState>) -> Router<()> {
        let pool = Arc::new(app_state.conn.clone());
        botlegal::configure_legal_ui_routes()
            .with_state(pool)
    }
}

pub fn configure_legal_routes(app_state: &Arc<AppState>) -> Router<()> {
    let pool = Arc::new(app_state.conn.clone());
    botlegal::configure_legal_routes()
        .with_state(pool)
}
