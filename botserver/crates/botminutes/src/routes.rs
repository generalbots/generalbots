use axum::routing::{get, put, post, patch};
use axum::Router;
use crate::handlers;
use crate::forms;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/minutes/meetings", get(handlers::list_meetings))
        .route("/api/minutes/transcripts", get(handlers::list_transcripts))
        .route("/api/minutes/documents", get(handlers::list_documents))
        .route("/api/minutes/documents/:id", put(handlers::update_document))
        .route("/api/minutes/forms/meeting/start/:id", post(handlers::start_meeting))
        .route("/api/minutes/forms/meeting/:id", patch(handlers::update_meeting))
        .route("/api/minutes/forms/meeting", post(forms::create_meeting))
        .route("/api/minutes/forms/action", post(forms::create_action))
        .route("/api/minutes/forms/document", post(forms::create_document))
        .route("/api/minutes/forms/sign/:id", post(forms::sign_document))
        .route("/api/minutes/forms/attendance/:id", post(forms::record_attendance))
}
