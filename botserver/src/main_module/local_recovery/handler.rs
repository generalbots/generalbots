//! `POST /local-recovery/reset` on the loopback recovery listener.

use super::secret::{SecretStore, SECRET_HEADER};
use axum::extract::{ConnectInfo, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use botcore::shared::state::AppState;
use botcoredirectory::auth_routes::{reset_local_admin_password, LocalResetError};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct RecoveryState {
    pub app: Arc<AppState>,
    pub store: Arc<SecretStore>,
}

#[derive(serde::Deserialize)]
pub struct ResetRequest {
    pub new_password: String,
}

fn answer(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({ "success": false, "code": code, "error": message })),
    )
        .into_response()
}

/// A request is local only when it reached the loopback interface and carries
/// no browser `Origin`: the desktop shell calls from Rust, never from a page.
pub(super) fn is_local_shell_call(peer: &SocketAddr, headers: &HeaderMap) -> bool {
    peer.ip().is_loopback() && !headers.contains_key(header::ORIGIN)
}

pub async fn reset(
    State(state): State<RecoveryState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<ResetRequest>,
) -> Response {
    let refused = || {
        answer(
            StatusCode::FORBIDDEN,
            "recovery_refused",
            "Password recovery works only from the desktop app on this computer.",
        )
    };
    if !is_local_shell_call(&peer, &headers) {
        log::warn!("Local recovery: refused a request that did not come from the desktop shell");
        return refused();
    }
    let presented = headers
        .get(SECRET_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_else(|| { log::debug!("using default for recovery header"); Default::default() });
    if !state.store.consume(presented) {
        log::warn!("Local recovery: refused a request without the current recovery secret");
        return refused();
    }
    match reset_local_admin_password(&state.app, &req.new_password).await {
        Ok(outcome) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "access_token": outcome.access_token,
                "email": outcome.email,
                "expires_in": outcome.expires_in,
                "sessions_revoked": outcome.sessions_revoked,
                "directory_updated": outcome.directory_updated,
            })),
        )
            .into_response(),
        Err(LocalResetError::Weak(problem)) => {
            answer(StatusCode::BAD_REQUEST, problem.code(), problem.message())
        }
        Err(LocalResetError::NoAdministrator) => answer(
            StatusCode::NOT_FOUND,
            "no_local_administrator",
            "This computer has no administrator account to recover.",
        ),
        Err(LocalResetError::Io(e)) => {
            let safe = crate::security::error_sanitizer::log_and_sanitize(&e, "local_recovery_reset", None);
            answer(StatusCode::INTERNAL_SERVER_ERROR, "password_not_saved", &safe.message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_loopback_calls_without_an_origin_are_local() {
        let local: SocketAddr = ([127, 0, 0, 1], 50000).into();
        let lan: SocketAddr = ([192, 168, 0, 10], 50000).into();
        let empty = HeaderMap::new();
        let mut from_page = HeaderMap::new();
        from_page.insert(header::ORIGIN, "http://127.0.0.1:8080".parse().expect("origin"));
        assert!(is_local_shell_call(&local, &empty));
        assert!(!is_local_shell_call(&lan, &empty));
        assert!(!is_local_shell_call(&local, &from_page));
    }
}
