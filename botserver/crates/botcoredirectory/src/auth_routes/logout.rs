use super::*;

pub async fn logout(
    State(_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<LogoutResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(String::from);

    if let Some(ref token_str) = token {
        let mut cache = SESSION_CACHE.write().await;
        if cache.remove(token_str).is_some() {
            info!("User logged out, session removed from cache");
        } else {
            info!("User logged out (session was not in cache)");
        }
        remove_persisted_session(token_str);
        drop(cache);

        // #2631 — removing the opaque session is not enough: the cloud JWT
        // itself is accepted by every API route until `exp`. Revoke its jti
        // in the shared blacklist with the token's remaining TTL so a stolen
        // copy is rejected immediately, on this and every other instance.
        if token_str.split('.').count() == 3 {
            use botsecurity::botsecurity_auth::jwt::revocation as revocation;

            let verified = botsecurity::botsecurity_core::verified_jwt::verify_saas_jwt(token_str)
                .map_err(|reason| {
                    warn!("logout: presented JWT could not be verified: {reason}");
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(ErrorResponse {
                            error: "Invalid or expired token".to_string(),
                            details: None,
                        }),
                    )
                })?;
            match verified.jti.as_deref() {
                Some(jti) => {
                    let entry = revocation::revocation_entry(jti, verified.exp, chrono::Utc::now());
                    revocation::global_blacklist().insert(entry).await.map_err(|e| {
                        // Honest failure: the caller must not believe logout
                        // succeeded while the token is still usable.
                        log::error!("logout: failed to blacklist JWT jti: {e}");
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Could not revoke token".to_string(),
                                details: None,
                            }),
                        )
                    })?;
                    // Also mirror it for synchronous resolvers that cannot
                    // await Redis (e.g. /api/auth/me's cloud-JWT fallback).
                    revocation::record_local_revocation(jti);
                    info!("logout: revoked JWT (jti on shared blacklist)");
                }
                None => {
                    // Legacy pre-#2631 token: no jti, so it cannot be revoked
                    // individually. Its session is gone and it expires at exp.
                    warn!("logout: JWT has no jti claim; it stays valid until exp");
                }
            }
        }
    }

    Ok(Json(LogoutResponse {
        success: true,
        message: "Logged out successfully".to_string(),
    }))
}
pub async fn get_current_user(
    State(_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Json<CurrentUserResponse> {
    let session_token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "));

    match session_token {
        None => {
            info!("get_current_user: no authorization header - returning anonymous user");
            Json(CurrentUserResponse {
                id: None,
                username: None,
                email: None,
                first_name: None,
                last_name: None,
                display_name: None,
                roles: None,
                organization_id: None,
                bucket: None,
                avatar_url: None,
                is_anonymous: true,
            })
        }
        Some("") => {
            info!("get_current_user: empty authorization token - returning anonymous user");
            Json(CurrentUserResponse {
                id: None,
                username: None,
                email: None,
                first_name: None,
                last_name: None,
                display_name: None,
                roles: None,
                organization_id: None,
                bucket: None,
                avatar_url: None,
                is_anonymous: true,
            })
        }
        Some(session_token) => {
            // #2766 — never emit token material (not even a 20-char prefix).
            // Only the length and a one-way fingerprint are logged.
            info!("{}", session_lookup_log_message(session_token));

            let cache = SESSION_CACHE.read().await;
            // Clone the entry so the read guard's borrow ends before any write
            // lock is taken for TTL eviction (E0505: cannot drop while borrowed).
            if let Some(user_data) = cache.get(session_token).cloned() {
                if session_expired(&user_data) {
                    drop(cache);
                    info!("get_current_user: session expired (TTL exceeded) - removing {}", user_data.email);
                    let mut cache = SESSION_CACHE.write().await;
                    cache.remove(session_token);
                    remove_persisted_session(session_token);
                    return Json(CurrentUserResponse {
                        id: None,
                        username: None,
                        email: None,
                        first_name: None,
                        last_name: None,
                        display_name: None,
                        roles: None,
                        organization_id: None,
                        bucket: None,
                        avatar_url: None,
                        is_anonymous: true,
                    });
                }
                info!("get_current_user: found cached session for user: {}", user_data.email);
                return Json(CurrentUserResponse {
                    id: Some(user_data.user_id),
                    username: Some(user_data.username),
                    email: Some(user_data.email),
                    first_name: user_data.first_name,
                    last_name: user_data.last_name,
                    display_name: user_data.display_name,
                    roles: Some(user_data.roles),
                    organization_id: user_data.organization_id,
                    bucket: user_data.bucket,
                    avatar_url: None,
                    is_anonymous: false,
                });
            }
            drop(cache);

            // Cache miss: rehydrate from the persisted `login_sessions` table so
            // sessions survive server restarts instead of silently logging users out.
            if let Some(user_data) = session_from_persisted(session_token) {
                if session_expired(&user_data) {
                    info!("get_current_user: persisted session expired - removing {}", user_data.email);
                    remove_persisted_session(session_token);
                    return Json(CurrentUserResponse {
                        id: None,
                        username: None,
                        email: None,
                        first_name: None,
                        last_name: None,
                        display_name: None,
                        roles: None,
                        organization_id: None,
                        bucket: None,
                        avatar_url: None,
                        is_anonymous: true,
                    });
                }
                info!("get_current_user: rehydrated persisted session for user: {}", user_data.email);
                {
                    let mut cache = SESSION_CACHE.write().await;
                    cache.insert(session_token.to_string(), user_data.clone());
                }
                Json(CurrentUserResponse {
                    id: Some(user_data.user_id.clone()),
                    username: Some(user_data.username.clone()),
                    email: Some(user_data.email.clone()),
                    first_name: user_data.first_name.clone(),
                    last_name: user_data.last_name.clone(),
                    display_name: user_data.display_name.clone(),
                    roles: Some(user_data.roles.clone()),
                    organization_id: user_data.organization_id.clone(),
                    bucket: user_data.bucket.clone(),
                    avatar_url: None,
                    is_anonymous: false,
                })
            } else if let Some(user_data) = resolve_cloud_bearer(session_token) {
                // #1364 — not a suite session token: it may still be a cloud
                // management JWT (SaaS surface). Resolve it through the wired
                // external verifier so signed-in cloud users see their identity
                // in the suite sidebar instead of a stale "Sign in" affordance.
                info!("get_current_user: resolved external (cloud JWT) bearer");
                Json(CurrentUserResponse {
                    id: Some(user_data.user_id),
                    username: Some(user_data.username),
                    email: Some(user_data.email),
                    first_name: user_data.first_name,
                    last_name: user_data.last_name,
                    display_name: user_data.display_name,
                    roles: Some(user_data.roles),
                    organization_id: user_data.organization_id,
                    bucket: user_data.bucket,
                    avatar_url: None,
                    is_anonymous: false,
                })
            } else {
                info!("get_current_user: session not found in cache or persisted store - returning anonymous user");
                Json(CurrentUserResponse {
                    id: None,
                    username: None,
                    email: None,
                    first_name: None,
                    last_name: None,
                    display_name: None,
                    roles: None,
                    organization_id: None,
                    bucket: None,
                    avatar_url: None,
                    is_anonymous: true,
                })
            }
        }
    }
}
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    let token_url = format!("{}/oauth/v2/token", auth_service.api_url());

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            log::error!("Failed to create HTTP client: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                    details: None,
                }),
            )
        })?;

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", &req.refresh_token),
        ("scope", "openid profile email offline_access"),
    ];

    let response = http_client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to refresh token: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Token refresh failed".to_string(),
                    details: None,
                }),
            )
        })?;

    if !response.status().is_success() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid or expired refresh token".to_string(),
                details: None,
            }),
        ));
    }

    let token_data: serde_json::Value = response.json().await.map_err(|e| {
        log::error!("Failed to parse token response: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Invalid response from authentication server".to_string(),
                details: None,
            }),
        )
    })?;

    let access_token = token_data
        .get("access_token")
        .and_then(|t| t.as_str())
        .map(String::from);

    let refresh_token = token_data
        .get("refresh_token")
        .and_then(|t| t.as_str())
        .map(String::from);

    let expires_in = token_data.get("expires_in").and_then(|t| t.as_i64());

    Ok(Json(LoginResponse {
        success: true,
        user_id: None,
        session_id: None,
        access_token,
        refresh_token,
        expires_in,
        requires_2fa: false,
        session_token: None,
        redirect: None,
        message: Some("Token refreshed successfully".to_string()),
    }))
}
/// Short, non-reversible fingerprint of a secret for log correlation (#2634).
/// The raw token is never emitted, so a clear-text credential cannot reach
/// stdout/journald. A stable hash keeps "same token retried" traceable without
/// exposing the token itself.
fn redact_session_token(token: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("redacted:{:016x}", hasher.finish())
}

/// Builds the exact INFO payload emitted for a session-token lookup (#2766).
/// Only the token length and the one-way fingerprint from
/// [`redact_session_token`] are included — no prefix, suffix or any other
/// token material — so a clear-text credential can never reach
/// stdout/journald. Kept as a pure function so tests can assert on the exact
/// string that `info!` logs without installing a global logger.
fn session_lookup_log_message(token: &str) -> String {
    format!(
        "get_current_user: looking up session token (len={}, fingerprint={})",
        token.len(),
        redact_session_token(token)
    )
}

pub async fn verify_2fa(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TwoFactorRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "2FA verification attempt for session: {}",
        redact_session_token(&req.session_token)
    );

    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "2FA verification not yet implemented".to_string(),
            details: Some("This feature will be available in a future update".to_string()),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::{redact_session_token, session_lookup_log_message};
    use log::{LevelFilter, Log, Metadata, Record};
    use std::sync::{Mutex, OnceLock};

    // Distinctive token: its fragments contain non-hex letters so they cannot
    // coincide with the fingerprint's hex digits (no flaky matches).
    const RAW: &str = "sess_supersecret_9f8e7d6c5b4a3210_verification_token_ZZZ";

    #[test]
    fn redacted_session_token_never_contains_raw_secret() {
        let redacted = redact_session_token(RAW);
        assert!(!redacted.contains(RAW), "raw token leaked: {redacted}");
        assert!(!redacted.contains("supersecret"), "fragment leaked: {redacted}");
        assert!(redacted.starts_with("redacted:"), "marker missing: {redacted}");
    }

    // Capturing logger proves the real `info!` output carries no token material.
    static CAPTURED: Mutex<Vec<String>> = Mutex::new(Vec::new());
    struct CaptureLogger;
    impl Log for CaptureLogger {
        fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
            true
        }
        fn log(&self, record: &Record<'_>) {
            if let Ok(mut captured) = CAPTURED.lock() {
                captured.push(record.args().to_string());
            }
        }
        fn flush(&self) {}
    }
    static LOGGER: CaptureLogger = CaptureLogger;
    static INSTALLED: OnceLock<bool> = OnceLock::new();

    #[test]
    fn captured_info_log_has_no_token_material() {
        let using_capture = *INSTALLED.get_or_init(|| log::set_logger(&LOGGER).is_ok());
        log::set_max_level(LevelFilter::Trace);
        if let Ok(mut captured) = CAPTURED.lock() {
            captured.clear();
        }
        log::info!("{}", session_lookup_log_message(RAW)); // get_current_user's call
        let snapshot = match CAPTURED.lock() {
            Ok(captured) if using_capture => captured.join("\n"),
            // Another test owns the global logger: assert the exact payload
            // that was handed to info! instead of its unobservable output.
            _ => session_lookup_log_message(RAW),
        };
        assert!(snapshot.contains("looking up session token"), "missing entry: {snapshot}");
        for fragment in [RAW, &RAW[..20], "supersecret", "verification_token_ZZZ"] {
            assert!(!snapshot.contains(fragment), "token material leaked: {fragment:?}");
        }
        assert!(snapshot.contains("fingerprint=redacted:"), "no fingerprint: {snapshot}");
        assert!(snapshot.contains(&format!("len={}", RAW.len())), "no length: {snapshot}");
    }
}
