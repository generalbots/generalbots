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

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use botcore::shared::state::AppState;
use serde_json::{json, Value};

const URI_PREFIX: &str = "ui://generated/";

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/mcp/apps", post(rpc).get(info))
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

async fn rpc(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    Json(request): Json<Value>,
) -> Response {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let bucket = params
        .get("bucket")
        .cloned()
        .unwrap_or_else(|| state.bucket_name.clone());

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
        "resources/list" => list_resources(&state, &bucket).await,
        "resources/read" => read_resource(&state, &bucket, &request).await,
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
async fn list_resources(state: &AppState, bucket: &str) -> Result<Value, String> {
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

async fn read_resource(
    state: &AppState,
    bucket: &str,
    request: &Value,
) -> Result<Value, String> {
    let uri = request
        .pointer("/params/uri")
        .and_then(Value::as_str)
        .ok_or_else(|| "params.uri is required".to_string())?;
    let hash = hash_from_uri(uri).ok_or_else(|| format!("unsupported resource uri {uri}"))?;
    let resource = crate::core::bot::sandbox::read_resource(state, bucket, hash)
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
    use super::{hash_from_uri, URI_PREFIX};

    #[test]
    fn accepts_only_generated_app_uris() {
        let hash = "a".repeat(64);
        assert_eq!(
            hash_from_uri(&format!("{URI_PREFIX}{hash}")),
            Some(hash.as_str())
        );
        assert_eq!(hash_from_uri("ui://generated/short"), None);
        assert_eq!(hash_from_uri("https://example.com/app"), None);
        assert_eq!(hash_from_uri(&format!("{URI_PREFIX}{}", "z".repeat(64))), None);
    }
}
