use axum::http::StatusCode;
use botcore::shared::state::AppState;
use botschema::conversation_autotasks as table;
use diesel::prelude::*;
use uuid::Uuid;

use super::types::AutotaskRow;
use super::{api_error, ApiError};

pub(super) fn load_rows(
    state: &AppState,
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<AutotaskRow>, ApiError> {
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error(StatusCode::SERVICE_UNAVAILABLE, &error.to_string()))?;
    table::table
        .filter(table::conversation_id.eq(conversation_id))
        .filter(table::user_id.eq(user_id))
        .order(table::created_at.desc())
        .load::<AutotaskRow>(&mut conn)
        .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()))
}

/// Resolves the bot that owns a conversation and refuses another user's
/// session. `user_sessions.user_id` is text in the canonical schema, so the
/// lookup binds text and parses the owner back into a UUID.
pub(super) fn conversation_bot(
    state: &AppState,
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<Uuid, ApiError> {
    use diesel::sql_types::Text;

    #[derive(QueryableByName)]
    struct SessionRow {
        #[diesel(sql_type = Text)]
        bot_id: String,
        #[diesel(sql_type = Text)]
        user_id: String,
    }

    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error(StatusCode::SERVICE_UNAVAILABLE, &error.to_string()))?;
    let row = diesel::sql_query(
        "SELECT bot_id::text AS bot_id, user_id::text AS user_id \
         FROM user_sessions WHERE id::text = $1 LIMIT 1",
    )
    .bind::<Text, _>(conversation_id.to_string())
    .get_result::<SessionRow>(&mut conn)
    .optional()
    .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()))?
    .ok_or_else(|| api_error(StatusCode::NOT_FOUND, "conversation not found"))?;
    let owner = Uuid::parse_str(row.user_id.trim())
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "bad session owner"))?;
    if owner != user_id {
        return Err(api_error(StatusCode::NOT_FOUND, "conversation not found"));
    }
    Uuid::parse_str(row.bot_id.trim())
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "bad session bot"))
}
