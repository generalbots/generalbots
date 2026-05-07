use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub use botresearch::{
    CollectionRow, KbDocumentRow, NewCollectionRequest, ResearchState, SearchQuery, SearchRequest,
};

impl ResearchState for AppState {
    fn db_pool(&self) -> &botlib::db_pool::DbPool {
        &self.conn
    }
}

pub fn configure_research_routes(state: &Arc<AppState>) -> Router {
    botresearch::configure_research_routes::<AppState>()
        .with_state(state.clone())
}

pub fn configure_research_ui_routes(state: &Arc<AppState>) -> Router {
    botresearch::ui::configure_research_ui_routes::<AppState>()
        .with_state(state.clone())
}
