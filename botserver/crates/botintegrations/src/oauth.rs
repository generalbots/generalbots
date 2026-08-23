use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use base64::Engine as _;
use diesel::prelude::*;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::Sha256;
use std::sync::Arc;
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::error::IntegrationError;
use crate::repository;
use crate::scope::{resolve_scope, ConnectionScope};
use crate::state::IntegrationState;

const STATE_MAX_AGE_SECS: i64 = 600;

pub(crate) struct OAuthProviderConfig {
    pub(crate) authorize_url: &'static str,
    pub(crate) token_url: &'static str,
    pub(crate) scopes: &'static str,
    pub(crate) basic_client_auth: bool,
}

pub(crate) fn provider_config(slug: &str) -> Option<OAuthProviderConfig> {
    let (authorize_url, token_url, scopes, basic_client_auth) = match slug {
        "hubspot" => (
            "https://app.hubspot.com/oauth/authorize",
            "https://api.hubapi.com/oauth/v1/token",
            "crm.objects.contacts.write crm.objects.deals.read",
            false,
        ),
        "intercom" => (
            "https://app.intercom.io/oauth",
            "https://api.intercom.io/auth/eagle/token",
            "",
            false,
        ),
        "todoist" => (
            "https://todoist.com/oauth/authorize",
            "https://todoist.com/oauth/access_token",
            "data:read_write",
            false,
        ),
        "zoom" => (
            "https://zoom.us/oauth/authorize",
            "https://zoom.us/oauth/token",
            "",
            true,
        ),
        "notion" => (
            "https://api.notion.com/v1/oauth/authorize",
            "https://api.notion.com/v1/oauth/token",
            "",
            true,
        ),
        "google_drive" => (
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            "https://www.googleapis.com/auth/drive.readonly",
            false,
        ),
        "google_calendar" => (
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            "https://www.googleapis.com/auth/calendar.events",
            false,
        ),
        "google_photos" => (
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            "https://www.googleapis.com/auth/photoslibrary.readonly",
            false,
        ),
        "google_forms" => (
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            "https://www.googleapis.com/auth/forms.body",
            false,
        ),
        "outlook" | "outlook_calendar" | "onedrive" | "sharepoint" => (
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            "offline_access https://graph.microsoft.com/.default",
            true,
        ),
        "xero_accounting" => (
            "https://identity.xero.com/connect/authorize",
            "https://identity.xero.com/connect/token",
            "openid profile email accounting.transactions accounting.contacts",
            true,
        ),
        "highlevel" => (
            "https://marketplace.gohighlevel.com/oauth/chooselocation",
            "https://services.leadconnectorhq.com/oauth/token",
            "",
            true,
        ),
        "docusign" => (
            "https://account.docusign.com/oauth/auth",
            "https://account-d.docusign.com/oauth/token",
            "signature impersonation",
            true,
        ),
        "salesforce" => (
            "https://login.salesforce.com/services/oauth2/authorize",
            "https://login.salesforce.com/services/oauth2/token",
            "full refresh_token",
            false,
        ),
        "quickbooks" => (
            "https://appcenter.intuit.com/connect/oauth2",
            "https://oauth.platform.intuit.com/oauth2/v1/tokens/bearer",
            "com.intuit.quickbooks.accounting",
            false,
        ),
        "box" => (
            "https://account.box.com/api/oauth2/authorize",
            "https://api.box.com/oauth2/token",
            "",
            false,
        ),
        "airtable" => (
            "https://airtable.com/oauth2/v1/authorize",
            "https://airtable.com/oauth2/v1/token",
            "data.records:read data.records:write schema.bases:read",
            true,
        ),
        "webflow" => (
            "https://webflow.com/oauth/authorize",
            "https://api.webflow.com/oauth/access_token",
            "",
            false,
        ),
        "blogger" => (
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            "https://www.googleapis.com/auth/blogger",
            false,
        ),
        "confluence" => (
            "https://auth.atlassian.com/authorize",
            "https://auth.atlassian.com/oauth/token",
            "read:confluence-content.all write:confluence-content offline_access",
            false,
        ),
        "basecamp" => (
            "https://launchpad.37signals.com/authorization/new",
            "https://launchpad.37signals.com/authorization/token",
            "",
            false,
        ),
        "fitbit" => (
            "https://www.fitbit.com/oauth2/authorize",
            "https://api.fitbit.com/oauth2/token",
            "activity heartrate location nutrition profile settings sleep social weight",
            true,
        ),
        "twitch" => (
            "https://id.twitch.tv/oauth2/authorize",
            "https://id.twitch.tv/oauth2/token",
            "",
            true,
        ),
        "canva" => (
            "https://www.canva.com/api/oauth/authorize",
            "https://api.canva.com/rest/v1/oauth/token",
            "asset:read asset:write",
            true,
        ),
        "clickfunnels" => (
            "https://accounts.myclickfunnels.com/api/authorize",
            "https://api.myclickfunnels.com/api/oauth/token",
            "",
            false,
        ),
        _ => return None,
    };
    Some(OAuthProviderConfig {
        authorize_url,
        token_url,
        scopes,
        basic_client_auth,
    })
}

fn signing_key() -> Result<String, Response> {
    std::env::var("INTERNAL_API_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(String::from)
        .ok_or_else(|| error_response(StatusCode::SERVICE_UNAVAILABLE, IntegrationError::Validation("oauth signing key is not configured".to_string())))
}

fn sign(payload: &str, key: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())
        .unwrap_or_else(|_| Hmac::<Sha256>::new_from_slice(b"generalbots-state").expect("constant key"));
    mac.update(payload.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
}

fn error_response(status: StatusCode, error: IntegrationError) -> Response {
    (status, Json(json!({ "detail": error.to_string() }))).into_response()
}

fn parse_uuid(value: &str) -> Result<Uuid, Response> {
    Uuid::parse_str(value)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation("bot_id must be a UUID".to_string())))
}

fn urlencode(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

fn request_base(headers: &axum::http::HeaderMap) -> Option<String> {
    let host = headers
        .get(header::HOST)?
        .to_str()
        .ok()?
        .to_string();
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("https");
    Some(format!("{scheme}://{host}"))
}

#[derive(Deserialize)]
pub struct StartQuery {
    return_to: Option<String>,
}

/// GET /api/bots/:bot_id/integrations/oauth/:provider/start
///
/// Redirects the caller to the provider consent screen. Client credentials
/// load strictly from the branch-scoped Vault path; the signed state carries
/// the tenant scope and expires after a short window.
pub async fn start(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Path((bot_id, provider)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Query(query): Query<StartQuery>,
) -> Result<Response, Response> {
    let config = provider_config(&provider).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, IntegrationError::Validation("unknown oauth provider".to_string()))
    })?;
    let uuid = parse_uuid(&bot_id)?;
    let scope = resolve_scope(&state.pool, &user, uuid)
        .map_err(|error| error_response(StatusCode::FORBIDDEN, error))?;

    let vault_path = format!(
        "gbo/{}/{}/{}/integrations/oauth/{provider}",
        scope.org_id, scope.branch_id, scope.bot_id
    );
    let client_config = state.vault.load_strict(&vault_path).await.map_err(
        |_| {
            log::warn!("oauth start without branch configuration at {vault_path}");
            error_response(StatusCode::PRECONDITION_FAILED, IntegrationError::Validation(
                "this branch has no oauth client credentials configured for the provider".to_string(),
            ))
        },
    )?;
    let client_id = client_config
        .get("client_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            error_response(StatusCode::PRECONDITION_FAILED, IntegrationError::Validation(
                "client_id missing from the provider oauth envelope".to_string(),
            ))
        })?;

    let base = request_base(&headers).ok_or_else(|| {
        error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation("request host missing".to_string()))
    })?;
    let redirect_uri = format!("{base}/api/bots/{bot_id}/integrations/oauth/{provider}/callback");

    let state_payload = json!({
        "u": scope.user_id,
        "o": scope.org_id,
        "br": scope.branch_id,
        "b": scope.bot_id,
        "p": provider,
        "base": base,
        "r": query.return_to,
        "t": chrono::Utc::now().timestamp(),
    });
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(state_payload.to_string());
    let signature = sign(&encoded, &signing_key()?);

    let mut url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&state={}.{}",
        config.authorize_url,
        urlencode(client_id),
        urlencode(&redirect_uri),
        urlencode(&encoded),
        urlencode(&signature),
    );
    if !config.scopes.is_empty() {
        url.push_str("&scope=");
        url.push_str(&urlencode(config.scopes));
    }
    Ok(Redirect::to(&url).into_response())
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

/// GET /api/bots/:bot_id/integrations/oauth/:provider/callback
///
/// Provider browser redirect carrying no JWT. Authenticity comes from the
/// HMAC-signed state issued by [`start`]; the exchanged access token is
/// stored strictly in Vault and an active connection row is created.
pub async fn callback(
    State(state): State<Arc<IntegrationState>>,
    Path((bot_id, provider)): Path<(String, String)>,
    Query(query): Query<CallbackQuery>,
) -> Result<Response, Response> {
    if let Some(error) = query.error {
        return Err(error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation(error)));
    }
    let (code, signed) = match (query.code, query.state) {
        (Some(code), Some(state)) => (code, state),
        _ => {
            return Err(error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation(
                "missing code or state".to_string(),
            )))
        }
    };
    let (encoded, signature) = signed.rsplit_once('.').ok_or_else(|| {
        error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation("malformed state".to_string()))
    })?;
    let expected = sign(encoded, &signing_key()?);
    if expected != signature {
        return Err(error_response(StatusCode::UNAUTHORIZED, IntegrationError::Validation(
            "state signature mismatch".to_string(),
        )));
    }
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encoded).map_err(|_| {
        error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation("malformed state payload".to_string()))
    })?;
    let payload: Value = serde_json::from_slice(&raw).map_err(|_| {
        error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation("malformed state payload".to_string()))
    })?;
    let age = chrono::Utc::now().timestamp() - payload.get("t").and_then(Value::as_i64).unwrap_or(i64::MAX);
    if !(0..=STATE_MAX_AGE_SECS).contains(&age) {
        return Err(error_response(StatusCode::UNAUTHORIZED, IntegrationError::Validation("state expired".to_string())));
    }
    if payload.get("p").and_then(Value::as_str) != Some(provider.as_str()) {
        return Err(error_response(StatusCode::UNAUTHORIZED, IntegrationError::Validation(
            "state provider mismatch".to_string(),
        )));
    }

    let config = provider_config(&provider).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, IntegrationError::Validation("unknown oauth provider".to_string()))
    })?;

    let user_id = Uuid::parse_str(payload.get("u").and_then(Value::as_str).unwrap_or_default())
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation("invalid state user".to_string())))?;
    let org_id = Uuid::parse_str(payload.get("o").and_then(Value::as_str).unwrap_or_default())
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation("invalid state org".to_string())))?;
    let branch_id = Uuid::parse_str(payload.get("br").and_then(Value::as_str).unwrap_or_default())
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation("invalid state branch".to_string())))?;
    let bot_uuid = Uuid::parse_str(payload.get("b").and_then(Value::as_str).unwrap_or_default())
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation("invalid state bot".to_string())))?;
    let base = payload
        .get("base")
        .and_then(Value::as_str)
        .filter(|value| value.starts_with("https://") || value.starts_with("http://localhost"))
        .ok_or_else(|| error_response(StatusCode::UNAUTHORIZED, IntegrationError::Validation("invalid state base".to_string())))?
        .to_string();

    let mut conn = state.pool.get().map_err(|_| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, IntegrationError::VaultUnavailable)
    })?;
    #[derive(diesel::QueryableByName)]
    struct BotOrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }
    let row: BotOrgRow = diesel::sql_query(
        "SELECT b.org_id AS org_id, b.branch_id AS branch_id \
         FROM bots b INNER JOIN branches br ON br.id = b.branch_id AND br.is_active = TRUE AND br.org_id = b.org_id \
         WHERE b.id = $1 AND b.is_active = TRUE LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
    .get_result(&mut conn)
    .optional()
    .map_err(|_| error_response(StatusCode::UNAUTHORIZED, IntegrationError::UnauthorizedScope))?
    .ok_or_else(|| error_response(StatusCode::UNAUTHORIZED, IntegrationError::UnauthorizedScope))?;
    if row.org_id != org_id || row.branch_id != branch_id {
        return Err(error_response(StatusCode::UNAUTHORIZED, IntegrationError::UnauthorizedScope));
    }
    log::debug!("oauth callback scope validated for bot {bot_uuid}");
    drop(conn);

    let scope = ConnectionScope {
        user_id,
        org_id,
        branch_id,
        bot_id: bot_uuid,
    };

    let vault_path = format!(
        "gbo/{}/{}/{}/integrations/oauth/{provider}",
        scope.org_id, scope.branch_id, scope.bot_id
    );
    let client = state.vault.load_strict(&vault_path).await.map_err(|_| {
        error_response(StatusCode::PRECONDITION_FAILED, IntegrationError::Validation(
            "provider oauth client credentials are not configured for this branch".to_string(),
        ))
    })?;
    let client_id = client
        .get("client_id")
        .and_then(Value::as_str)
        .ok_or_else(|| error_response(StatusCode::PRECONDITION_FAILED, IntegrationError::Validation("client_id missing".to_string())))?
        .to_string();
    let client_secret = client
        .get("client_secret")
        .and_then(Value::as_str)
        .ok_or_else(|| error_response(StatusCode::PRECONDITION_FAILED, IntegrationError::Validation("client_secret missing".to_string())))?
        .to_string();
    let redirect_uri = format!("{base}/api/bots/{bot_id}/integrations/oauth/{provider}/callback");

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, IntegrationError::VaultUnavailable))?;
    let mut outgoing = http
        .post(config.token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
        ]);
    if config.basic_client_auth {
        let basic = base64::engine::general_purpose::STANDARD.encode(format!("{client_id}:{client_secret}"));
        outgoing = outgoing.header(header::AUTHORIZATION, format!("Basic {basic}"));
    }
    let response = outgoing.send().await.map_err(|error| {
        log::warn!("oauth token exchange failed for {provider}: {error}");
        error_response(StatusCode::BAD_GATEWAY, IntegrationError::VaultUnavailable)
    })?;
    let status = response.status();
    let body: Value = response.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        log::warn!("oauth token exchange returned {status} for {provider}");
        return Err(error_response(StatusCode::BAD_GATEWAY, IntegrationError::VaultUnavailable));
    }
    let access_token = body
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or_else(|| error_response(StatusCode::BAD_GATEWAY, IntegrationError::VaultUnavailable))?
        .to_string();
    let refresh_token = body.get("refresh_token").and_then(Value::as_str).map(str::to_string);
    let expires_in = body.get("expires_in").and_then(Value::as_i64);
    let granted_scopes = body.get("scope").cloned().unwrap_or(Value::Null);

    let connection_id = Uuid::now_v7();
    let mut envelope = json!({ "token": access_token });
    if let Some(refresh) = &refresh_token {
        envelope["refresh_token"] = Value::String(refresh.clone());
    }
    let stored_path = state
        .vault
        .store(&scope, connection_id, &envelope)
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, IntegrationError::VaultUnavailable))?;

    let mut conn = state.pool.get().map_err(|_| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, IntegrationError::VaultUnavailable)
    })?;
    repository::insert(
        &mut conn,
        &scope,
        &repository::NewConnectionInsert {
            connection_id,
            provider_slug: &provider,
            display_name: &format!("{provider} via oauth"),
            auth_kind: "oauth2",
            vault_path: &stored_path,
            granted_scopes: &granted_scopes,
            visibility: "private",
            configuration: &json!({"via": "authorization-code"}),
            expires_at: expires_in.and_then(|seconds| {
                chrono::Duration::try_seconds(seconds).map(|duration| chrono::Utc::now() + duration)
            }),
        },
    )
    .map_err(|error| {
        log::error!("oauth connection insert failed for {provider}: {error:?}");
        error_response(StatusCode::INTERNAL_SERVER_ERROR, IntegrationError::VaultUnavailable)
    })?;
    let event = repository::NewConnectionEvent {
        connection_id: Some(connection_id),
        actor_user_id: user_id,
        event_type: "connect",
        outcome: "success",
        risk_level: "medium",
        metadata: &json!({"method": "oauth2"}),
    };
    repository::record_event(&mut conn, &scope, &event).ok();
    drop(conn);

    let return_to = payload
        .get("r")
        .and_then(Value::as_str)
        .filter(|value| value.starts_with('/'))
        .unwrap_or("/integrations")
        .to_string();
    Ok(Redirect::to(&return_to).into_response())
}
