use axum::routing::{get, put, post, patch};
use axum::Router;
use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/minutes/meetings", get(handlers::list_meetings))
        .route("/api/minutes/transcripts", get(handlers::list_transcripts))
        .route("/api/minutes/documents", get(handlers::list_documents))
        .route("/api/minutes/documents/{id}", put(handlers::update_document))
        .route("/api/minutes/forms/meeting/start/{id}", post(handlers::start_meeting))
        .route("/api/minutes/forms/meeting/{id}", patch(handlers::update_meeting))
}
