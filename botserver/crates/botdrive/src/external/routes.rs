//! HTTP surface for external drives.
//!
//! Every route is tenant-scoped by the caller's branch, resolved from the
//! authenticated user (or, for the OAuth callback, from the signed state). A
//! branch can only ever see the accounts it connected itself.
//!
//! The callback is the one anonymous route: the provider redirects a browser
//! that carries no API token. Its authenticity comes from the HMAC-signed
//! `state`, verified inside the handler.

use crate::external::{config, crypto, db, engine, routes_files, types::Provider};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Extension, Json,
};
use botcore::shared::state::AppState;
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use diesel::{OptionalExtension, RunQueryDsl};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

pub(super) type ApiError = (StatusCode, Json<serde_json::Value>);
pub(super) type ApiResult<T> = Result<T, ApiError>;

pub fn configure() -> axum::Router<Arc<AppState>> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/api/external-drives/connections", get(routes_files::list_connections))
        .route("/api/external-drives/files", get(routes_files::list_files))
        .route(
            "/api/external-drives/download/:provider/:remote_id",
            get(routes_files::download_file),
        )
        .route("/api/external-drives/connect", post(connect))
        .route("/api/external-drives/callback", get(oauth_callback))
        .route("/api/external-drives/sync", post(sync_now))
        .route("/api/external-drives/settings", post(update_settings))
        .route("/api/external-drives/disconnect", post(disconnect))
}

pub(super) fn err(status: StatusCode, message: &str) -> ApiError {
    log::warn!("external drive api error ({}): {}", status.as_u16(), message);
    (status, Json(serde_json::json!({ "error": message })))
}

/// Resolve the caller's tenant branch.
///
/// The connection is the signup-derived `crm_contacts` binding — the same source
/// that decides which buckets `drive_handlers` lets someone address, so a user
/// cannot reach an external account through a different tenant's identity. An
/// account with no binding falls back to the global (nil) branch, which is what
/// the suite does everywhere else.
pub(super) fn caller_branch(state: &AppState, user: &AuthenticatedUser) -> Result<Uuid, ApiError> {
    let mut conn = db::conn(&state.conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database unavailable: {e}")))?;
    let Some(email) = user.email.as_deref().filter(|e| !e.is_empty() && *e != "session-user") else {
        return Ok(Uuid::nil());
    };
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }
    let row = diesel::sql_query(
        "SELECT branch_id FROM crm_contacts WHERE lower(email) = lower($1) LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(email)
    .get_result::<Row>(&mut conn)
    .optional()
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("tenant lookup failed: {e}")))?;
    Ok(row.map(|r| r.branch_id).unwrap_or_else(Uuid::nil))
}

pub(super) fn parse_provider(value: &str) -> Result<Provider, ApiError> {
    Provider::parse(value)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "provider must be onedrive or gdrive"))
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// `POST /api/external-drives/connect` — begin the consent flow.
///
/// Returns the provider URL and a signed `state` bound to this branch; the
/// client redirects the browser there.
pub async fn connect(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
    Json(body): Json<ConnectRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let provider = parse_provider(&body.provider)?;
    let client = config::client_for(provider).ok_or_else(|| {
        err(
            StatusCode::NOT_IMPLEMENTED,
            &format!(
                "no OAuth client is configured for {} on this deployment",
                provider.display_name()
            ),
        )
    })?;
    let branch_id = caller_branch(&state, &user)?;
    let base = config::base_from_headers(
        &headers,
        header_value(&headers, "x-forwarded-host").as_deref(),
        header_value(&headers, "x-forwarded-proto").as_deref(),
    )
    .or_else(|| config::base_from_headers(&headers, None, None));
    let redirect_uri = config::redirect_uri(base.as_deref());
    let token = crypto::state_token(branch_id, provider);
    let authorize = match provider {
        Provider::OneDrive => crate::external::onedrive::authorize_url(
            &client.tenant_id,
            &client.client_id,
            &redirect_uri,
            &token,
        ),
        Provider::GoogleDrive => {
            crate::external::gdrive::authorize_url(&client.client_id, &redirect_uri, &token)
        }
    };
    Ok(Json(serde_json::json!({
        "provider": provider.as_str(),
        "authorize_url": authorize,
        "redirect_uri": redirect_uri,
    })))
}

/// `GET /api/external-drives/callback` — provider redirect target.
///
/// Anonymous by necessity; the signed `state` is what proves the flow belongs
/// to this branch. On success the browser is sent back to the Drive app with the
/// connection already stored, so the user never lands on a raw JSON page.
pub async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<CallbackQuery>,
) -> Response {
    let base = config::base_from_headers(&headers, None, None).unwrap_or_default();
    let back = |query: &str| {
        let prefix = if base.is_empty() { "/drive".to_string() } else { format!("{base}/drive") };
        Redirect::to(&format!("{prefix}?{query}"))
    };

    if let Some(provider_error) = params.error.as_deref().filter(|e| !e.is_empty()) {
        log::warn!("external drive oauth denied by provider: {provider_error}");
        return back(&format!(
            "external=error&reason={}",
            urlencode(provider_error)
        ))
        .into_response();
    }
    let Some(token) = params.state.as_deref().filter(|s| !s.is_empty()) else {
        return back("external=error&reason=missing_state").into_response();
    };
    let Some((branch_id, provider)) = crypto::peek_state(token) else {
        return back("external=error&reason=invalid_state").into_response();
    };
    if !crypto::verify_state(token, branch_id, provider, chrono::Utc::now()) {
        log::warn!("external drive oauth state rejected for provider {}", provider.as_str());
        return back("external=error&reason=invalid_state").into_response();
    }
    let Some(code) = params.code.as_deref().filter(|c| !c.is_empty()) else {
        return back("external=error&reason=missing_code").into_response();
    };
    let Some(client) = config::client_for(provider) else {
        return back("external=error&reason=not_configured").into_response();
    };
    let redirect_uri = config::redirect_uri(config::base_from_headers(&headers, None, None).as_deref());
    let http = match engine::http_client() {
        Ok(c) => c,
        Err(_) => return back("external=error&reason=client_unavailable").into_response(),
    };

    // Each provider has its own token response type, so the two arms are
// flattened into the three fields that actually get stored.
    let (access_token, refresh_token, expires_at) = match provider {
        Provider::OneDrive => {
            match crate::external::onedrive::exchange_code(
                &http,
                &client.tenant_id,
                &client.client_id,
                &client.client_secret,
                &redirect_uri,
                code,
            )
            .await
            {
                Ok(t) => (t.access_token, t.refresh_token, t.expires_at),
                Err(e) => {
                    log::warn!("external drive oauth exchange failed: {e}");
                    return back("external=error&reason=exchange_failed").into_response();
                }
            }
        }
        Provider::GoogleDrive => {
            match crate::external::gdrive::exchange_code(
                &http,
                &client.client_id,
                &client.client_secret,
                &redirect_uri,
                code,
            )
            .await
            {
                Ok(t) => (t.access_token, t.refresh_token, t.expires_at),
                Err(e) => {
                    log::warn!("external drive oauth exchange failed: {e}");
                    return back("external=error&reason=exchange_failed").into_response();
                }
            }
        }
    };

    let stored = (|| -> Result<(), String> {
        let access_enc = crypto::encrypt_token(&access_token)?;
        let refresh_enc = refresh_token
            .as_deref()
            .map(crypto::encrypt_token)
            .transpose()?;
        let mut conn = db::conn(&state.conn)?;
        db::upsert_connection(
            &mut conn,
            branch_id,
            provider,
            &access_enc,
            refresh_enc.as_deref(),
            Some(expires_at),
            None,
        )
    })();
    if let Err(e) = stored {
        log::error!("external drive connection could not be stored: {e}");
        return back("external=error&reason=storage_failed").into_response();
    }

    // The first pass mirrors the account, so the tab has content immediately
    // instead of an empty list the user has to trigger.
    let report = engine::sync_connection(&state.conn, branch_id, provider).await;
    if !report.ok() {
        log::warn!(
            "external drive {} connected but the first sync reported {:?}",
            provider.as_str(),
            report.error
        );
    }
    back(&format!(
        "external=connected&provider={}&files={}",
        provider.as_str(),
        report.upserted
    ))
    .into_response()
}

/// `POST /api/external-drives/sync` — run one pass now, bypassing the cadence.
pub async fn sync_now(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<ProviderRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let provider = parse_provider(&body.provider)?;
    let branch_id = caller_branch(&state, &user)?;
    let report = engine::sync_connection(&state.conn, branch_id, provider).await;
    // A failed pass is still a successful HTTP call: the report carries why.
    Ok(Json(serde_json::json!({
        "provider": provider.as_str(),
        "upserted": report.upserted,
        "removed": report.removed,
        "pages": report.pages,
        "error": report.error,
        "error_kind": report.error_kind,
    })))
}

/// `POST /api/external-drives/settings` — change the sync cadence.
pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<SettingsRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let provider = parse_provider(&body.provider)?;
    let branch_id = caller_branch(&state, &user)?;
    let minutes = body.sync_minutes.clamp(5, 1440);
    let mut conn = db::conn(&state.conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database unavailable: {e}")))?;
    let updated = diesel::sql_query(
        "UPDATE external_drive_connections SET sync_minutes = $3, updated_at = NOW()
         WHERE branch_id = $1 AND provider = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Text, _>(provider.as_str())
    .bind::<diesel::sql_types::Integer, _>(minutes)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    if updated == 0 {
        return Err(err(StatusCode::NOT_FOUND, "account is not connected"));
    }
    Ok(Json(serde_json::json!({
        "provider": provider.as_str(),
        "sync_minutes": minutes,
    })))
}

/// `POST /api/external-drives/disconnect` — forget the tokens and the mirror.
pub async fn disconnect(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<ProviderRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let provider = parse_provider(&body.provider)?;
    let branch_id = caller_branch(&state, &user)?;
    let mut conn = db::conn(&state.conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database unavailable: {e}")))?;
    db::disconnect(&mut conn, branch_id, provider)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e))?;
    log::info!(
        "external drive {} disconnected for branch {}",
        provider.as_str(),
        branch_id
    );
    Ok(Json(serde_json::json!({ "provider": provider.as_str(), "connected": false })))
}

#[derive(Debug, Deserialize)]
pub struct ConnectRequest {
    pub provider: String,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderRequest {
    pub provider: String,
}

#[derive(Debug, Deserialize)]
pub struct SettingsRequest {
    pub provider: String,
    pub sync_minutes: i32,
}

/// Filenames come from a provider, so a name containing quotes or newlines must
/// not be able to break out of the Content-Disposition header.
pub(super) fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '"' | '\\'))
        .take(120)
        .collect();
    if cleaned.trim().is_empty() {
        "download".to_string()
    } else {
        cleaned
    }
}

fn urlencode(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c.to_string(),
            other => other
                .to_string()
                .bytes()
                .map(|b| format!("%{:02X}", b))
                .collect::<String>(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_parsing_rejects_unknown_values() {
        assert_eq!(parse_provider("onedrive").unwrap(), Provider::OneDrive);
        assert_eq!(parse_provider("gdrive").unwrap(), Provider::GoogleDrive);
        assert_eq!(parse_provider("o365").unwrap(), Provider::OneDrive);
        assert!(parse_provider("dropbox").is_err());
        assert!(parse_provider("").is_err());
    }

    #[test]
    fn filenames_cannot_escape_the_header() {
        assert_eq!(sanitize_filename("report\".pdf"), "report.pdf");
        assert_eq!(sanitize_filename("a\\b.txt"), "ab.txt");
        assert_eq!(sanitize_filename("line\nbreak"), "linebreak");
        assert_eq!(sanitize_filename("   "), "download");
        assert_eq!(sanitize_filename("ok.txt"), "ok.txt");
    }

    #[test]
    fn reason_codes_are_url_encoded() {
        assert_eq!(urlencode("access_denied"), "access_denied");
        assert_eq!(urlencode("a b&c"), "a%20b%26c");
    }
}