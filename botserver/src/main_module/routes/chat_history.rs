//! Chat history HTTP surface: session listing, message decryption, rename and
//! delete, plus inline attachment previews.
//!
//! `sessions`, `messages`, `manage` and `attachment` hold the handlers; this
//! module keeps the shared error/auth helpers and the router.

mod attachment;
mod manage;
mod messages;
mod sessions;
mod truncate;

pub use attachment::handle_attachment_preview;
pub use manage::{
    handle_clear_chat_history_session, handle_delete_chat_history_session,
    handle_rename_chat_history_session,
};
pub use messages::handle_chat_history_messages;
pub use sessions::handle_chat_history_sessions;
pub use truncate::handle_truncate;

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use botcore::shared::state::AppState;
use uuid::Uuid;

use super::anonymous_auth::{resolve_chat_user_uuid, resolve_session_user};

pub(super) type HandlerError = (StatusCode, String);

pub(super) fn bearer_user(request: &axum::extract::Request) -> Result<Uuid, HandlerError> {
    let (user_raw, _roles, is_authenticated) = resolve_session_user(request);
    if !is_authenticated || user_raw.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        ));
    }
    Ok(resolve_chat_user_uuid(&user_raw))
}

pub(super) fn db_error(context: &str, e: String) -> HandlerError {
    log::error!("Chat history error ({context}): {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to load chat history".to_string(),
    )
}

pub fn configure_chat_history_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/chat/history/sessions",
            get(handle_chat_history_sessions),
        )
        .route(
            "/api/chat/history/sessions/:session_id/messages",
            get(handle_chat_history_messages).delete(handle_clear_chat_history_session),
        )
        .merge(super::chat_history_lock::router())
        .route(
            "/api/chat/attachments/preview",
            get(handle_attachment_preview),
        )
        .route(
            "/api/chat/history/sessions/:session_id",
            delete(handle_delete_chat_history_session)
                .put(handle_rename_chat_history_session)
                .patch(handle_rename_chat_history_session),
        )
        .route(
            "/api/chat/history/sessions/:session_id/truncate",
            post(handle_truncate),
        )
}
