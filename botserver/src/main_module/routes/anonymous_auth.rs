use axum::{Json, extract::State, response::IntoResponse};
use std::sync::Arc;
use std::collections::HashMap;
use log::info;
use botcore::shared::state::AppState;

pub async fn anonymous_auth_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let bot_name = params.get("bot_name").cloned().unwrap_or_default();
    let existing_session_id = params.get("session_id").cloned();
    let existing_user_id = params.get("user_id").cloned();

    // Resolve the real user from the suite session token (gb-access-token)
    // when present as `Authorization: Bearer <token>`.
    let (auth_user_id, auth_roles, is_authenticated) = resolve_session_user(&request);

    let user_id = if is_authenticated {
        auth_user_id.clone()
    } else {
        existing_user_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
    };
    let session_id = existing_session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let session_uuid = match uuid::Uuid::parse_str(&session_id) {
        Ok(uuid) => uuid,
        Err(_) => uuid::Uuid::new_v4(),
    };
    let user_uuid = if let Ok(uuid) = uuid::Uuid::parse_str(&user_id) {
        uuid
    } else {
        // Deterministic UUID derived from non-UUID user identifiers
        // (e.g. Zitadel numeric user_id / sub claim).
        uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_DNS,
            format!("zitadel:{}", user_id).as_bytes(),
        )
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

    let role = if is_authenticated && auth_roles.iter().any(|r| r.to_lowercase().contains("admin")) {
        crate::security::user_role::ROLE_ADMIN.to_string()
    } else {
        crate::security::user_role::resolve_user_role(&state.conn, user_uuid)
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

    info!("Anonymous auth for bot: {}, session: {}, authenticated: {}", bot_name, final_session_id, is_authenticated);

    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({
            "user_id": user_id,
            "session_id": final_session_id,
            "bot_id": found_bot_id,
            "bot_name": bot_name,
            "status": if is_authenticated { "authenticated" } else { "anonymous" },
            "role": role,
            "is_authenticated": is_authenticated
        })),
    )
}

fn resolve_session_user(
    request: &axum::extract::Request,
) -> (String, Vec<String>, bool) {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    let token = match token {
        Some(t) => t,
        None => return (String::new(), Vec::new(), false),
    };

    let cache = match botcoredirectory::auth_routes::SESSION_CACHE.try_read() {
        Ok(c) => c,
        Err(_) => return (String::new(), Vec::new(), false),
    };
    match cache.get(&token) {
        Some(user_data) => (
            user_data.user_id.clone(),
            user_data.roles.clone(),
            true,
        ),
        None => (String::new(), Vec::new(), false),
    }
}
