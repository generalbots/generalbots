//! `POST /api/auth/password`: the signed-in person changes their own password,
//! and the reset the desktop shell asks for from this computer.

use super::*;

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

type Refusal = (StatusCode, Json<serde_json::Value>);

fn refusal(status: StatusCode, code: &str, message: &str) -> Refusal {
    (
        status,
        Json(serde_json::json!({ "success": false, "code": code, "error": message })),
    )
}

fn bearer(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, Refusal> {
    let unauthenticated = || {
        refusal(
            StatusCode::UNAUTHORIZED,
            "session_required",
            "Sign in again to change your password.",
        )
    };
    let token = bearer(&headers).ok_or_else(unauthenticated)?;
    let user = resolve_token_user(token).ok_or_else(unauthenticated)?;
    if let Err(problem) = check_new_password(&req.new_password, Some(&req.current_password)) {
        return Err(refusal(StatusCode::BAD_REQUEST, problem.code(), problem.message()));
    }
    let wrong_current = || {
        refusal(
            StatusCode::BAD_REQUEST,
            "current_password_wrong",
            "The current password is not correct.",
        )
    };

    if let Some((path, record)) = find_admin_record(Some(&user.email)) {
        local_admin_user_id(&user.email, &req.current_password).ok_or_else(wrong_current)?;
        write_admin_record(&path, &record_with_password(&record, &req.new_password)).map_err(|e| {
            log::error!("Local admin: could not write the new password: {e}");
            refusal(
                StatusCode::INTERNAL_SERVER_ERROR,
                "password_not_saved",
                "The new password could not be saved.",
            )
        })?;
        let directory_updated =
            sync_directory_admin_password(Arc::clone(&state), req.new_password.clone(), path).await;
        let revoked = revoke_user_sessions(&user.user_id, &user.email, Some(token)).await;
        remember_desktop_session(token);
        info!("Local administrator changed the password; {revoked} other session(s) signed out");
        return Ok(Json(serde_json::json!({
            "success": true,
            "directory_updated": directory_updated,
            "sessions_revoked": revoked,
        })));
    }

    change_directory_user_password(
        &state,
        &user.email,
        &user.user_id,
        &req.current_password,
        &req.new_password,
    )
    .await
    .map_err(|e| match e {
        DirectoryChangeError::CurrentPasswordWrong => wrong_current(),
        DirectoryChangeError::Refused => refusal(
            StatusCode::BAD_REQUEST,
            "password_refused",
            "The directory refused this password. Choose a different one.",
        ),
        DirectoryChangeError::Unavailable => refusal(
            StatusCode::SERVICE_UNAVAILABLE,
            "directory_unavailable",
            "The password service is not available right now. Try again in a moment.",
        ),
    })?;
    let revoked = revoke_user_sessions(&user.user_id, &user.email, Some(token)).await;
    botlog::safe_info!(
        "Directory account {} changed the password; {} other session(s) signed out",
        user.email,
        revoked
    );
    Ok(Json(serde_json::json!({
        "success": true,
        "directory_updated": true,
        "sessions_revoked": revoked,
    })))
}

/// The outcome of a reset from this computer: a fresh session for the
/// administrator, who signs in with it straight away.
#[derive(Debug, Serialize)]
pub struct LocalResetOutcome {
    pub access_token: String,
    pub email: String,
    pub expires_in: i64,
    pub sessions_revoked: usize,
    pub directory_updated: bool,
}

/// Why a reset from this computer could not run.
#[derive(Debug)]
pub enum LocalResetError {
    Weak(PasswordProblem),
    NoAdministrator,
    Io(std::io::Error),
}

/// Gives the local administrator a new password without the old one. Only the
/// loopback recovery listener calls this, after it checked the one-use secret
/// the desktop shell read from this computer's disk.
pub async fn reset_local_admin_password(
    state: &Arc<AppState>,
    new_password: &str,
) -> Result<LocalResetOutcome, LocalResetError> {
    check_new_password(new_password, None).map_err(LocalResetError::Weak)?;
    let (path, record) = find_admin_record(None).ok_or(LocalResetError::NoAdministrator)?;
    let email = record
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let user_id = record
        .get("user_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(String::from)
        .unwrap_or_else(|| format!("local-{email}"));
    write_admin_record(&path, &record_with_password(&record, new_password)).map_err(LocalResetError::Io)?;
    let sessions_revoked = revoke_user_sessions(&user_id, &email, None).await;
    persist_local_admin(state, &user_id, &email);
    let access_token = mint_local_session(&user_id, &email).await;
    remember_desktop_session(&access_token);
    let directory_updated =
        sync_directory_admin_password(Arc::clone(state), new_password.to_string(), path).await;
    info!(
        "Local administrator password reset from this computer; {sessions_revoked} session(s) signed out"
    );
    Ok(LocalResetOutcome {
        access_token,
        email,
        expires_in: session_ttl_secs(),
        sessions_revoked,
        directory_updated,
    })
}
