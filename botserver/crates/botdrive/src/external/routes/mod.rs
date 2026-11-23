//! HTTP surface for external drives.
//!
//! Every route is tenant-scoped by the caller's branch, resolved from the
//! authenticated user (or, for the OAuth callback, from the signed state). A
//! branch can only ever see the accounts it connected itself.
//!
//! The callback is the one anonymous route: the provider redirects a browser
//! that carries no API token. Its authenticity comes from the HMAC-signed
//! `state`, verified inside the handler.

use crate::external::{config, crypto, db, engine, routes_files, routes_upload, types::Provider};
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
        .route(
            "/api/external-drives/connections",
            get(routes_files::list_connections),
        )
        .route("/api/external-drives/files", get(routes_files::list_files))
        .route(
            "/api/external-drives/download/:provider/:remote_id",
            get(routes_files::download_file),
        )
        // #2047 — write half: one upload entry point for both providers.
        .route(
            "/api/external-drives/upload",
            post(routes_upload::upload_file),
        )
        .route("/api/external-drives/connect", post(connect))
        .route("/api/external-drives/callback", get(oauth_callback))
        .route("/api/external-drives/sync", post(sync_now))
        .route("/api/external-drives/settings", post(update_settings))
        .route("/api/external-drives/disconnect", post(disconnect))
}

pub(super) fn err(status: StatusCode, message: &str) -> ApiError {
    log::warn!(
        "external drive api error ({}): {}",
        status.as_u16(),
        message
    );
    (status, Json(serde_json::json!({ "error": message })))
}

/// Decide the tenant branch from a caller's identity and its optional
/// `crm_contacts` binding.
///
/// #2865 — the global (nil) branch is reserved for the single-user desktop
/// principal. Every other authenticated principal must resolve to a real
/// branch; without a binding it is rejected. Connections are unique on
/// `(branch_id, provider)` ([`crate::external::db::ensure_schema`]), so letting
/// two unrelated principals fall back to nil would share one connection — and
/// therefore one set of mirrored files and OAuth grant — between them.
pub(super) fn branch_from_binding(
    user: &AuthenticatedUser,
    binding: Option<Uuid>,
) -> Result<Uuid, ApiError> {
    if user.metadata.contains_key("desktop_local") {
        return Ok(Uuid::nil());
    }
    match binding {
        Some(branch) if !branch.is_nil() => Ok(branch),
        _ => Err(err(
            StatusCode::FORBIDDEN,
            "no workspace binding for this account; connect an external drive from a bound workspace",
        )),
    }
}

/// Resolve the caller's tenant branch.
///
/// The connection is the signup-derived `crm_contacts` binding — the same source
/// that decides which buckets `drive_handlers` lets someone address, so a user
/// cannot reach an external account through a different tenant's identity.
/// Only the desktop-local principal may fall back to the global (nil) branch;
/// any other account without a binding is rejected with 403 (#2865).
pub(super) fn caller_branch(state: &AppState, user: &AuthenticatedUser) -> Result<Uuid, ApiError> {
    // #2865 — keep nil exclusive to the local principal and avoid a DB round
    // trip for it (a desktop install may have no directory to query anyway).
    if user.metadata.contains_key("desktop_local") {
        return Ok(Uuid::nil());
    }
    let mut conn = db::conn(&state.conn).map_err(|e| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("database unavailable: {e}"),
        )
    })?;
    let Some(email) = user
        .email
        .as_deref()
        .filter(|e| !e.is_empty() && *e != "session-user")
    else {
        return branch_from_binding(user, None);
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
    .map_err(|e| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("tenant lookup failed: {e}"),
        )
    })?;
    branch_from_binding(user, row.map(|r| r.branch_id))
}

pub(super) fn parse_provider(value: &str) -> Result<Provider, ApiError> {
    Provider::parse(value).ok_or_else(|| {
        err(
            StatusCode::BAD_REQUEST,
            "provider must be onedrive or gdrive",
        )
    })
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
/// client redirects the browser there. The URL always carries an S256 PKCE
/// challenge and the state hides the matching one-time verifier.
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
    // PKCE is used for every client, public or confidential. The verifier rides
    // inside the signed, short-lived state so the anonymous callback can prove
    // it started this flow; the S256 challenge goes to the provider here.
    let verifier = crypto::code_verifier();
    let challenge = crypto::code_challenge(&verifier);
    let token = crypto::state_token_with_verifier(branch_id, provider, Some(verifier.as_str()));
    let authorize = match provider {
        Provider::OneDrive => crate::external::onedrive::authorize_url_with_pkce(
            &client.tenant_id,
            &client.client_id,
            &redirect_uri,
            &token,
            Some(challenge.as_str()),
        ),
        Provider::GoogleDrive => crate::external::gdrive::authorize_url_with_pkce(
            &client.client_id,
            &redirect_uri,
            &token,
            Some(challenge.as_str()),
        ),
    };
    Ok(Json(serde_json::json!({
        "provider": provider.as_str(),
        "authorize_url": authorize,
        "redirect_uri": redirect_uri,
    })))
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
    let mut conn = db::conn(&state.conn).map_err(|e| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("database unavailable: {e}"),
        )
    })?;
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
    let mut conn = db::conn(&state.conn).map_err(|e| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("database unavailable: {e}"),
        )
    })?;
    db::disconnect(&mut conn, branch_id, provider)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e))?;
    log::info!(
        "external drive {} disconnected for branch {}",
        provider.as_str(),
        branch_id
    );
    Ok(Json(
        serde_json::json!({ "provider": provider.as_str(), "connected": false }),
    ))
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

mod callback;

pub(super) use callback::oauth_callback;

#[cfg(test)]
mod tests;
