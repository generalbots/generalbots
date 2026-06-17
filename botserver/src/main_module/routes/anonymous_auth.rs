use axum::{Json, extract::State, response::IntoResponse};
use std::sync::Arc;
use std::collections::HashMap;
use log::info;
use botcore::shared::state::AppState;

pub async fn anonymous_auth_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let bot_name = params.get("bot_name").cloned().unwrap_or_default();
    let existing_session_id = params.get("session_id").cloned();
    let existing_user_id = params.get("user_id").cloned();

    let user_id = existing_user_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let session_id = existing_session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let session_uuid = match uuid::Uuid::parse_str(&session_id) {
        Ok(uuid) => uuid,
        Err(_) => uuid::Uuid::new_v4(),
    };
    let user_uuid = match uuid::Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => uuid::Uuid::new_v4(),
    };

    let found_bot_id = {
        let conn = state.conn.get().ok();
        if let Some(mut db_conn) = conn {
            use botcore::shared::models::schema::bots::dsl::*;
            use diesel::prelude::*;
            bots.filter(name.eq(&bot_name))
                .select(id)
                .first::<uuid::Uuid>(&mut db_conn)
                .ok()
                .unwrap_or_else(uuid::Uuid::nil)
        } else {
            uuid::Uuid::nil()
        }
    };

    let mut final_session_id = session_id.clone();
    {
        let mut sm = state.session_manager.lock().await;
        sm.get_or_create_anonymous_user(Some(user_uuid)).ok();
        let session = sm.get_or_create_session_by_id(
            session_uuid, user_uuid, found_bot_id, "Anonymous Chat"
        );
        if let Ok(sess) = session {
            final_session_id = sess.id.to_string();
        }
    }

    info!("Anonymous auth for bot: {}, session: {}", bot_name, final_session_id);

    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({
            "user_id": user_id,
            "session_id": final_session_id,
            "bot_id": found_bot_id,
            "bot_name": bot_name,
            "status": "anonymous"
        })),
    )
}
