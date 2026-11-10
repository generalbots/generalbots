//! HTTP surface for the opt-in peer network (#1864).
//!
//! The registry itself lives in [`crate::core::peer_network`]; this module only
//! authenticates machine callers and translates registry errors to JSON/HTTP.
//! When `GB_PEER_NETWORK` is off every handler still answers, but registration
//! and heartbeat return `403 peer_network_disabled` and status carries no node
//! identity.

use axum::{
    extract::{Json, Query},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use botcore::shared::state::AppState;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::core::home_site::{HomeSite, HomeSiteGenerator};
use crate::core::peer_network::{self, HeartbeatRequest, PeerError, PeerNetwork, RegisterRequest};
use crate::core::peer_payloads::{self, PeerPayload};

/// Routes are merged into the main API router.
pub fn configure_peer_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/peer/status", get(status))
        .route("/api/peer/register", post(register))
        .route("/api/peer/heartbeat", post(heartbeat))
        .route("/api/peer/enable", post(enable))
        .route("/api/peer/disable", post(disable))
        .route("/api/peer/home-site", get(home_site))
        .route("/api/peer/payloads", post(enqueue_payload))
        .route("/api/peer/payloads", get(list_payloads))
}

/// `GET /api/peer/status` — this node's identity and the live peer table.
pub async fn status(headers: HeaderMap) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize(&headers, network) {
        return error_response(&locale, error);
    }
    Json(network.status()).into_response()
}

/// `POST /api/peer/register` — announce a peer for the first time.
pub async fn register(headers: HeaderMap, Json(request): Json<RegisterRequest>) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize(&headers, network) {
        return error_response(&locale, error);
    }
    match network.register(request) {
        Ok(peer) => (StatusCode::OK, Json(json!({ "ok": true, "peer": peer }))).into_response(),
        Err(error) => error_response(&locale, error),
    }
}

/// `POST /api/peer/heartbeat` — refresh an existing registration.
pub async fn heartbeat(headers: HeaderMap, Json(request): Json<HeartbeatRequest>) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize(&headers, network) {
        return error_response(&locale, error);
    }
    match network.heartbeat(request) {
        Ok(peer) => (StatusCode::OK, Json(json!({ "ok": true, "peer": peer }))).into_response(),
        Err(error) => error_response(&locale, error),
    }
}

/// `POST /api/peer/enable` — turn the peer network on at runtime.
pub async fn enable(headers: HeaderMap) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize(&headers, network) {
        return error_response(&locale, error);
    }
    network.enable();
    persist_mode("p2p");
    (
        StatusCode::OK,
        Json(json!({ "ok": true, "enabled": network.enabled() })),
    )
        .into_response()
}

/// #1865 — the toggle is a hard gate, so it must outlive the process. The
/// wizard writes the same key at first run; Settings → Network writes it here.
fn persist_mode(mode: &str) {
    if let Err(error) = peer_network::persist_network_mode(mode) {
        log::warn!("could not persist network mode {mode}: {error}");
    }
}

/// `POST /api/peer/disable` — stop all P2P traffic at runtime.
pub async fn disable(headers: HeaderMap) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize(&headers, network) {
        return error_response(&locale, error);
    }
    network.disable();
    persist_mode("offline");
    (
        StatusCode::OK,
        Json(json!({ "ok": true, "enabled": network.enabled() })),
    )
        .into_response()
}

#[derive(Deserialize)]
struct PeerPayloadQuery {
    recipient: Option<uuid::Uuid>,
}

/// `POST /api/peer/payloads` — enqueue an encrypted payload for a peer.
pub async fn enqueue_payload(headers: HeaderMap, Json(request): Json<PeerPayload>) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize(&headers, network) {
        return error_response(&locale, error);
    }
    if !network.enabled() {
        return error_response(&locale, PeerError::Disabled);
    }
    match peer_payloads::global_store().enqueue(request.clone()) {
        Ok(payload) => (
            StatusCode::OK,
            Json(json!({ "ok": true, "payload": payload })),
        )
            .into_response(),
        Err(message) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "payload_store_full",
                "message": botcore::i18n::t_args(
                    &locale,
                    "error-peer-network-payload-store-full",
                    &[("message", message.as_str())],
                ),
            })),
        )
            .into_response(),
    }
}

/// `GET /api/peer/payloads?recipient=<uuid>` — pending payloads for a recipient.
pub async fn list_payloads(headers: HeaderMap, Query(query): Query<PeerPayloadQuery>) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize(&headers, network) {
        return error_response(&locale, error);
    }
    if !network.enabled() {
        return error_response(&locale, PeerError::Disabled);
    }
    let Some(recipient) = query.recipient else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "missing_recipient",
                "message": botcore::i18n::t(&locale, "error-peer-network-recipient-required"),
            })),
        )
            .into_response();
    };
    let payloads = peer_payloads::global_store().pending_for(recipient);
    (
        StatusCode::OK,
        Json(json!({ "ok": true, "payloads": payloads })),
    )
        .into_response()
}

/// `GET /api/peer/home-site` — the signed home site for this node (if P2P).
pub async fn home_site(headers: HeaderMap) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize(&headers, network) {
        return error_response(&locale, error);
    }
    if !network.enabled() {
        return error_response(&locale, PeerError::Disabled);
    }
    // #1968 — `GB_PEER_SIGNING_KEY` → Vault `secret/gbo/peer` `signing-key`
    // → the node id, resolved once in `PeerConfig::load`.
    let signing_key = network.signing_key();
    match network.identity() {
        Some(identity) => {
            let site: HomeSite = HomeSiteGenerator::generate(
                identity.name.clone(),
                identity.node_id,
                format!("{} on General Bots", identity.name),
                "Welcome to my peer home site.",
                0,
                &signing_key,
            );
            (StatusCode::OK, Json(json!({ "ok": true, "site": site }))).into_response()
        }
        None => error_response(&locale, PeerError::Disabled),
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

fn error_response(locale: &botcore::i18n::Locale, error: PeerError) -> Response {
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
    // #2094 — `error` keeps the stable machine code, `message` is localized.
    let (key, detail) = match &error {
        PeerError::Disabled => ("error-peer-network-disabled", None),
        PeerError::Unauthorized => ("error-peer-network-unauthorized", None),
        PeerError::Invalid(message) => ("error-peer-network-invalid", Some(message.clone())),
        PeerError::UnknownNode(id) => ("error-peer-network-unknown-node", Some(id.to_string())),
        PeerError::RegistryFull => ("error-peer-network-registry-full", None),
    };
    let message = match detail {
        Some(detail) => botcore::i18n::t_args(locale, key, &[("message", detail.as_str())]),
        None => botcore::i18n::t(locale, key),
    };
    (
        status,
        Json(json!({
            "error": code,
            "message": message,
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
