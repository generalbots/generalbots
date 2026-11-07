//! HTTP surface for the opt-in peer network (#1864).
//!
//! The registry itself lives in [`crate::core::peer_network`]; this module only
//! authenticates machine callers and translates registry errors to JSON/HTTP.
//! When `GB_PEER_NETWORK` is off every handler still answers, but registration
//! and heartbeat return `403 peer_network_disabled` and status carries no node
//! identity.

use axum::{
    extract::Json,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use botcore::shared::state::AppState;
use serde_json::json;
use std::sync::Arc;

use crate::core::peer_network::{self, HeartbeatRequest, PeerError, PeerNetwork, RegisterRequest};

/// Routes are merged into the main API router.
pub fn configure_peer_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/peer/status", get(status))
        .route("/api/peer/register", post(register))
        .route("/api/peer/heartbeat", post(heartbeat))
}

/// `GET /api/peer/status` — this node's identity and the live peer table.
pub async fn status(headers: HeaderMap) -> Response {
    let network = peer_network::global();
    if let Err(error) = authorize(&headers, network) {
        return error_response(error);
    }
    Json(network.status()).into_response()
}

/// `POST /api/peer/register` — announce a peer for the first time.
pub async fn register(headers: HeaderMap, Json(request): Json<RegisterRequest>) -> Response {
    let network = peer_network::global();
    if let Err(error) = authorize(&headers, network) {
        return error_response(error);
    }
    match network.register(request) {
        Ok(peer) => (StatusCode::OK, Json(json!({ "ok": true, "peer": peer }))).into_response(),
        Err(error) => error_response(error),
    }
}

/// `POST /api/peer/heartbeat` — refresh an existing registration.
pub async fn heartbeat(headers: HeaderMap, Json(request): Json<HeartbeatRequest>) -> Response {
    let network = peer_network::global();
    if let Err(error) = authorize(&headers, network) {
        return error_response(error);
    }
    match network.heartbeat(request) {
        Ok(peer) => (StatusCode::OK, Json(json!({ "ok": true, "peer": peer }))).into_response(),
        Err(error) => error_response(error),
    }
}

fn authorize(headers: &HeaderMap, network: &PeerNetwork) -> Result<(), PeerError> {
    let Some(expected) = network.token() else {
        // Local/self-hosted opt-in without a shared token keeps the endpoint
        // usable; the feature itself is still off unless explicitly enabled.
        return Ok(());
    };

    let supplied = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .or_else(|| {
            headers
                .get("x-peer-token")
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
        })
        .filter(|value| !value.is_empty());

    match supplied {
        Some(value) if constant_time_eq(value.as_bytes(), expected.as_bytes()) => Ok(()),
        _ => Err(PeerError::Unauthorized),
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}

fn error_response(error: PeerError) -> Response {
    let status = match &error {
        PeerError::Disabled => StatusCode::FORBIDDEN,
        PeerError::Unauthorized => StatusCode::UNAUTHORIZED,
        PeerError::Invalid(_) => StatusCode::BAD_REQUEST,
        PeerError::UnknownNode(_) => StatusCode::NOT_FOUND,
        PeerError::RegistryFull => StatusCode::SERVICE_UNAVAILABLE,
    };
    let code = match &error {
        PeerError::Disabled => "peer_network_disabled",
        PeerError::Unauthorized => "unauthorized",
        PeerError::Invalid(_) => "invalid_peer",
        PeerError::UnknownNode(_) => "unknown_peer",
        PeerError::RegistryFull => "peer_registry_full",
    };
    (
        status,
        Json(json!({
            "error": code,
            "message": error.to_string(),
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disabled_network_with_token(token: Option<&str>) -> PeerNetwork {
        let mut config = peer_network::PeerConfig::disabled();
        config.token = token.map(str::to_string);
        PeerNetwork::new(config)
    }

    fn headers_with(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, value.parse().expect("header"));
        headers
    }

    #[test]
    fn no_configured_token_allows_local_calls() {
        let network = disabled_network_with_token(None);
        assert!(authorize(&HeaderMap::new(), &network).is_ok());
    }

    #[test]
    fn configured_token_is_required_and_checked() {
        let network = disabled_network_with_token(Some("secret"));

        assert_eq!(
            authorize(&HeaderMap::new(), &network),
            Err(PeerError::Unauthorized)
        );
        assert_eq!(
            authorize(&headers_with("Bearer wrong"), &network),
            Err(PeerError::Unauthorized)
        );
        assert!(authorize(&headers_with("Bearer secret"), &network).is_ok());
    }

    #[test]
    fn comparison_is_length_safe() {
        assert!(constant_time_eq(b"same", b"same"));
        assert!(!constant_time_eq(b"same", b"samf"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }
}
