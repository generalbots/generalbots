//! Suite session store helpers for the SSO hops.

use botcore::shared::state::AppState;
use botcoredirectory::auth_routes::{persist_session, SessionUserData, SESSION_CACHE};
use std::sync::Arc;

/// Cria uma sessão no SESSION_CACHE do Suite e retorna o token `gb_{uuid}_{timestamp}`.
/// O papel da sessão é resolvido por grupo RBAC (fix #843) — nunca hardcoded.
pub(crate) async fn create_suite_session(
    email: &str,
    user_id: &str,
    bucket: Option<String>,
    roles: Vec<String>,
    organization_id: Option<String>,
) -> String {
    let api_token = format!(
        "gb_{}_{}",
        uuid::Uuid::new_v4(),
        chrono::Utc::now().timestamp()
    );
    let session_user = SessionUserData {
        user_id: user_id.to_string(),
        email: email.to_string(),
        username: email.split('@').next().unwrap_or("user").to_string(),
        first_name: None,
        last_name: None,
        display_name: Some(email.split('@').next().unwrap_or("User").to_string()),
        organization_id,
        roles,
        bucket,
        created_at: chrono::Utc::now().timestamp(),
    };

    {
        let mut cache = SESSION_CACHE.write().await;
        cache.insert(api_token.clone(), session_user.clone());
        persist_session(&api_token, &session_user);
    }

    log::info!(
        "Created Suite session for {} (token: {}...)",
        email,
        &api_token[..6.min(api_token.len())]
    );
    api_token
}

/// Resolves the effective role vector for a user id from RBAC group membership.
/// Falls back to the plain "user" role when the user has no admin group.
pub(super) fn resolve_session_roles(state: &Arc<AppState>, user_id: &str) -> Vec<String> {
    let stable_uuid = crate::security::user_role::derive_stable_user_uuid(user_id);
    let role = crate::security::user_role::resolve_user_role(&state.conn, stable_uuid);
    vec![role]
}
