//! Built-in MCP server for generated apps (#1676).
//!
//! Apps published by `show_app` are content-addressed sandbox resources. This
//! module exposes the same resources over the Model Context Protocol so ANY
//! compliant host can list and render them — not only our chat surface:
//!
//! - `initialize` → protocol version + `resources` capability;
//! - `resources/list` → `ui://generated/{hash}` with
//!   `text/html;profile=mcp-app` and `_meta.ui` carrying the declared CSP;
//! - `resources/read` → the HTML for one resource.
//!
//! Transport is streamable-HTTP style JSON-RPC over `POST /api/mcp/apps`
//! (a GET returns a small discovery document). Consent/pinning stays where it
//! already is: resources are immutable by hash, and the chat consent ledger
//! governs mounting them.
//!
//! #2661 — resource retrieval is an authenticated, org-scoped operation. The
//! resource tenant is `state.bucket_name`: the very bucket `publish_resource`
//! writes to and the only bucket a read may resolve against. A caller-supplied
//! `?bucket=` is never used to locate a resource — it is rejected when it
//! disagrees with the server's tenant (403). Reads additionally require the
//! short-lived sandbox signature (`?t=`) that `/sandbox/ui/{hash}` requires.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use botcore::shared::state::AppState;
use serde_json::{json, Value};

use crate::core::bot::sandbox::verify_resource;
use crate::security::auth::AuthenticatedUser;

const URI_PREFIX: &str = "ui://generated/";

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/mcp/apps", post(rpc).get(info))
}

async fn info() -> impl IntoResponse {
    Json(json!({
        "name": "gb-generated-apps",
        "transport": "streamable-http",
        "endpoint": "/api/mcp/apps",
        "resources": "ui://generated/{hash}",
    }))
}

/// Extracts the content hash from an MCP resource URI. Only the exact
/// `ui://generated/{64-hex}` form is accepted.
#[must_use]
pub fn hash_from_uri(uri: &str) -> Option<&str> {
    let hash = uri.strip_prefix(URI_PREFIX)?;
    if hash.len() == 64 && hash.chars().all(|character| character.is_ascii_hexdigit()) {
        Some(hash)
    } else {
        None
    }
}

/// #2661 — why an MCP resource request was refused. Every variant maps to a
/// fail-closed HTTP status: unknown/missing credentials are 401, an explicit
/// cross-tenant bucket override is 403.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceAccessError {
    Unauthenticated,
    TenantMismatch,
    MissingToken,
    InvalidToken,
}

impl ResourceAccessError {
    fn status_code(self) -> StatusCode {
        match self {
            Self::TenantMismatch => StatusCode::FORBIDDEN,
            Self::Unauthenticated | Self::MissingToken | Self::InvalidToken => {
                StatusCode::UNAUTHORIZED
            }
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::Unauthenticated => "authentication required",
            Self::TenantMismatch => "bucket does not belong to the caller's organization",
            Self::MissingToken => "missing resource token",
            Self::InvalidToken => "invalid or expired resource token",
        }
    }
}

/// #2661 — pure tenant gate shared by `resources/list` and `resources/read`.
/// The resource's tenant is the bucket this server is configured for. A caller
/// may cross it only with an authenticated principal; a `?bucket=` override is
/// tolerated only when it names that exact tenant, otherwise the request is
/// refused (403) instead of silently reading a different tenant's bucket.
fn authorize_tenant(
    authenticated: bool,
    requested_bucket: Option<&str>,
    resource_bucket: &str,
) -> Result<(), ResourceAccessError> {
    if !authenticated {
        return Err(ResourceAccessError::Unauthenticated);
    }
    if requested_bucket.is_some_and(|requested| requested != resource_bucket) {
        return Err(ResourceAccessError::TenantMismatch);
    }
    Ok(())
}

/// #2661 — pure read authorization predicate. On top of the tenant gate, a
/// read must carry a valid `sandbox::verify_resource` signature for the exact
/// hash being read. The signature is conversation-scoped, so a token minted
/// for one resource cannot be replayed to read another.
fn authorize_resource_read(
    authenticated: bool,
    requested_bucket: Option<&str>,
    resource_bucket: &str,
    hash: &str,
    token: Option<&str>,
) -> Result<(), ResourceAccessError> {
    authorize_tenant(authenticated, requested_bucket, resource_bucket)?;
    let token = token.ok_or(ResourceAccessError::MissingToken)?;
    if verify_resource(hash, token).is_err() {
        return Err(ResourceAccessError::InvalidToken);
    }
    Ok(())
}

fn access_error_response(id: &Value, error: ResourceAccessError) -> Response {
    (
        error.status_code(),
        Json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32001, "message": error.message() },
        })),
    )
        .into_response()
}

fn invalid_params_response(id: &Value) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32602, "message": "params.uri must be a ui://generated resource" },
        })),
    )
        .into_response()
}

async fn rpc(
    State(state): State<Arc<AppState>>,
    user: Option<Extension<AuthenticatedUser>>,
    Query(params): Query<HashMap<String, String>>,
    Json(request): Json<Value>,
) -> Response {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .map_or_else(Default::default, |v| v)
        .to_string();
    // #2661 — fail closed: a missing/incomplete principal is never treated as
    // authenticated. The auth middleware installs `AuthenticatedUser` for the
    // authenticated `/api/mcp/**` catalogue; `Option` guards a misconfiguration.
    let authenticated = user
        .as_ref()
        .is_some_and(|Extension(principal)| principal.is_authenticated());
    // #2661 — the query values are untrusted selectors only. They are passed
    // to the predicates above for rejection; they never choose a bucket.
    let requested_bucket = params.get("bucket").map(String::as_str);
    let token = params.get("t").map(String::as_str);

    let result = match method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": "2025-06-18",
            "serverInfo": {
                "name": "gb-generated-apps",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "capabilities": { "resources": {} },
        })),
        "notifications/initialized" | "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": [] })),
        "resources/list" => {
            if let Err(error) =
                authorize_tenant(authenticated, requested_bucket, &state.bucket_name)
            {
                return access_error_response(&id, error);
            }
            list_resources(&state).await
        }
        "resources/read" => {
            let Some(hash) = request
                .pointer("/params/uri")
                .and_then(Value::as_str)
                .and_then(hash_from_uri)
            else {
                return invalid_params_response(&id);
            };
            if let Err(error) = authorize_resource_read(
                authenticated,
                requested_bucket,
                &state.bucket_name,
                hash,
                token,
            ) {
                return access_error_response(&id, error);
            }
            read_resource(&state, &request).await
        }
        other => Err(format!("unsupported method {other}")),
    };

    match result {
        Ok(value) => Json(json!({ "jsonrpc": "2.0", "id": id, "result": value })).into_response(),
        Err(message) => (
            StatusCode::OK,
            Json(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": message },
            })),
        )
            .into_response(),
    }
}

/// Lists every published generated app. Metadata sidecars written by
/// `publish_resource` carry the title, thread and creation time.
/// #2661 — the bucket is always the server's configured tenant bucket.
async fn list_resources(state: &AppState) -> Result<Value, String> {
    let bucket = state.bucket_name.as_str();
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "drive is not configured".to_string())?;
    let keys = drive
        .list_objects(bucket, Some("_sandbox/ui/"))
        .await
        .map_err(|error| format!("resource listing failed: {error}"))?;

    let mut resources = Vec::new();
    for key in keys {
        let Some(hash) = key
            .strip_prefix("_sandbox/ui/")
            .and_then(|name| name.strip_suffix(".meta.json"))
        else {
            continue;
        };
        let meta: Value = match drive.get_object(bucket, &key).await {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or(Value::Null),
            Err(_) => Value::Null,
        };
        let csp: Value = match drive
            .get_object(bucket, &format!("_sandbox/ui/{hash}.json"))
            .await
        {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or(Value::Null),
            Err(_) => Value::Null,
        };
        let title = meta
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Generated app");
        resources.push(json!({
            "uri": format!("{URI_PREFIX}{hash}"),
            "name": title,
            "description": meta.get("thread_id").and_then(Value::as_str)
                .map(|thread| format!("Generated app for thread {thread}"))
                .unwrap_or_else(|| "Generated app".to_string()),
            "mimeType": "text/html;profile=mcp-app",
            "_meta": {
                "ui": {
                    "csp": csp,
                    "domain": state.bucket_name,
                }
            }
        }));
    }
    Ok(json!({ "resources": resources }))
}

/// #2661 — the bucket is resolved from server configuration, never from the
/// request, so a caller cannot reach another tenant's object store.
async fn read_resource(state: &AppState, request: &Value) -> Result<Value, String> {
    let uri = request
        .pointer("/params/uri")
        .and_then(Value::as_str)
        .ok_or_else(|| "params.uri is required".to_string())?;
    let hash = hash_from_uri(uri).ok_or_else(|| format!("unsupported resource uri {uri}"))?;
    let resource = crate::core::bot::sandbox::read_resource(state, &state.bucket_name, hash)
        .await
        .ok_or_else(|| format!("resource {uri} not found"))?;
    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "text/html;profile=mcp-app",
            "text": resource.html,
        }]
    }))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::{
        authorize_resource_read, authorize_tenant, hash_from_uri, ResourceAccessError, URI_PREFIX,
    };
    use crate::core::bot::sandbox::sign::sign_resource;

    const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const BUCKET: &str = "acme_default";

    #[test]
    fn accepts_only_generated_app_uris() {
        let hash = "a".repeat(64);
        assert_eq!(
            hash_from_uri(&format!("{URI_PREFIX}{hash}")),
            Some(hash.as_str())
        );
        assert_eq!(hash_from_uri("ui://generated/short"), None);
        assert_eq!(hash_from_uri("https://example.com/app"), None);
        assert_eq!(
            hash_from_uri(&format!("{URI_PREFIX}{}", "z".repeat(64))),
            None
        );
    }

    /// #2661 — an unsigned read (no `?t=`) is refused with 401, even for an
    /// otherwise authenticated caller on the correct tenant.
    #[test]
    fn unsigned_read_is_unauthorized() {
        let error = authorize_resource_read(true, None, BUCKET, HASH, None).unwrap_err();
        assert_eq!(error, ResourceAccessError::MissingToken);
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
    }

    /// #2661 — a caller that tries to redirect the read to another tenant's
    /// bucket is refused with 403, even with a valid signature.
    #[test]
    fn cross_tenant_bucket_is_forbidden() {
        let token = sign_resource("thread-1", HASH, 60);
        let error =
            authorize_resource_read(true, Some("other-org_default"), BUCKET, HASH, Some(&token))
                .unwrap_err();
        assert_eq!(error, ResourceAccessError::TenantMismatch);
        assert_eq!(error.status_code(), StatusCode::FORBIDDEN);
    }

    /// #2661 — a signed, same-tenant read succeeds, whether or not the caller
    /// repeats its own bucket in the query string.
    #[test]
    fn signed_same_tenant_read_is_allowed() {
        let token = sign_resource("thread-1", HASH, 60);
        assert_eq!(
            authorize_resource_read(true, None, BUCKET, HASH, Some(&token)),
            Ok(())
        );
        assert_eq!(
            authorize_resource_read(true, Some(BUCKET), BUCKET, HASH, Some(&token)),
            Ok(())
        );
    }

    /// #2661 — fail closed: anonymous callers, malformed tokens and tokens
    /// minted for a different resource are all refused.
    #[test]
    fn authorization_fails_closed() {
        let token = sign_resource("thread-1", HASH, 60);
        assert_eq!(
            authorize_resource_read(false, None, BUCKET, HASH, Some(&token)).unwrap_err(),
            ResourceAccessError::Unauthenticated
        );
        assert_eq!(
            authorize_resource_read(true, None, BUCKET, HASH, Some("garbage")).unwrap_err(),
            ResourceAccessError::InvalidToken
        );
        let other = sign_resource("thread-1", &"b".repeat(64), 60);
        assert_eq!(
            authorize_resource_read(true, None, BUCKET, HASH, Some(&other)).unwrap_err(),
            ResourceAccessError::InvalidToken
        );
        // `resources/list` has no token but still enforces the tenant gate.
        assert_eq!(
            authorize_tenant(true, Some("other-org_default"), BUCKET).unwrap_err(),
            ResourceAccessError::TenantMismatch
        );
        assert_eq!(authorize_tenant(true, None, BUCKET), Ok(()));
    }

    #[test]
    fn access_error_status_codes_are_fail_closed() {
        assert_eq!(
            ResourceAccessError::Unauthenticated.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ResourceAccessError::MissingToken.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ResourceAccessError::InvalidToken.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ResourceAccessError::TenantMismatch.status_code(),
            StatusCode::FORBIDDEN
        );
    }
}
