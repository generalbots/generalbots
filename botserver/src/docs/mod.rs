use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;
use botlib::traits::DriveRepository;

pub use botdocs::{
    collaboration::{
        handle_docs_websocket, handle_get_collaborators, handle_get_mentions,
        handle_get_presence, handle_get_selections, handle_get_typing,
    },
    ooxml,
    types::{
        AiRequest, AiResponse, CollabMessage, CommentReply, ComparisonSummary, Document,
        DocumentComment, DocumentComparison, DocumentDiff, DocumentMetadata, DocumentStyle,
        Endnote, Footnote, OutlineItem, SaveRequest, SaveResponse, SearchQuery,
        TableOfContents, TocEntry, TrackChange,
    },
    types_requests::*,
};

fn make_doc_state(app_state: &Arc<AppState>) -> Result<Arc<botdocs::DocState>, String> {
    let drive: Arc<dyn DriveRepository> = app_state
        .drive
        .clone()
        .ok_or_else(|| "Drive not available".to_string())?;
    let pool = Arc::new(app_state.conn.clone());
    let bucket_name = app_state.bucket_name.clone();
    Ok(Arc::new(botdocs::DocState {
        pool,
        drive,
        bucket_name,
    }))
}

pub fn configure_docs_routes(state: &Arc<AppState>) -> Router {
    match make_doc_state(state) {
        Ok(doc_state) => botdocs::configure_docs_routes().with_state(doc_state),
        Err(e) => {
            log::warn!("Docs routes disabled: {e}");
            Router::new()
        }
    }
}
