//! HTTP surface for the opt-in peer network (#1864).
//!
//! The registry itself lives in [`crate::core::peer_network`]; this module only
//! authenticates machine callers and translates registry errors to JSON/HTTP.
//! When `GB_PEER_NETWORK` is off every handler still answers, but registration
//! and heartbeat return `403 peer_network_disabled` and status carries no node
//! identity.

use axum::{
    extract::{Extension, Json, Query},
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
use crate::security::auth_api::types::AuthenticatedUser;

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
///
/// #2141 — control surface: a peer token or an admin session. It is never
/// anonymous, so node identity is not disclosed to unauthenticated callers.
pub async fn status(
    headers: HeaderMap,
    user: Option<Extension<AuthenticatedUser>>,
) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize_control(&headers, network, user.as_deref()) {
        return error_response(&locale, error);
    }
    Json(network.status()).into_response()
}

/// `POST /api/peer/register` — announce a peer for the first time.
pub async fn register(headers: HeaderMap, Json(request): Json<RegisterRequest>) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize_machine(&headers, network) {
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
    if let Err(error) = authorize_machine(&headers, network) {
        return error_response(&locale, error);
    }
    match network.heartbeat(request) {
        Ok(peer) => (StatusCode::OK, Json(json!({ "ok": true, "peer": peer }))).into_response(),
        Err(error) => error_response(&locale, error),
    }
}

/// `POST /api/peer/enable` — turn the peer network on at runtime.
pub async fn enable(
    headers: HeaderMap,
    user: Option<Extension<AuthenticatedUser>>,
) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize_control(&headers, network, user.as_deref()) {
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
pub async fn disable(
    headers: HeaderMap,
    user: Option<Extension<AuthenticatedUser>>,
) -> Response {
    let network = peer_network::global();
    let locale = botcore::i18n::locale_from_headers(&headers);
    if let Err(error) = authorize_control(&headers, network, user.as_deref()) {
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
    if let Err(error) = authorize_machine(&headers, network) {
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
    if let Err(error) = authorize_machine(&headers, network) {
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
    if let Err(error) = authorize_machine(&headers, network) {
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

/// Machine callers (`register`, `heartbeat`, `payloads`, `home-site`).
///
/// #2141 — the shared node token is mandatory. When it is not configured the
/// endpoint fails closed with 503; there is no "local calls are fine" path.
fn authorize_machine(headers: &HeaderMap, network: &PeerNetwork) -> Result<(), PeerError> {
    let expected = network.token().ok_or(PeerError::Unconfigured)?;
    match supplied_peer_token(headers) {
        Some(value) if peer_token_matches(value, expected) => Ok(()),
        _ => Err(PeerError::Unauthorized),
    }
}

/// Control callers (`status`, `enable`, `disable`).
///
/// A peer token authenticates a machine caller; otherwise the request must
/// carry an administrator session. With no token configured and no admin
/// identity the call is refused as unconfigured — never accepted.
fn authorize_control(
    headers: &HeaderMap,
    network: &PeerNetwork,
    user: Option<&AuthenticatedUser>,
) -> Result<(), PeerError> {
    let token_configured = network.token().is_some();
    let token_accepted = network
        .token()
        .is_some_and(|expected| supplied_peer_token(headers).is_some_and(|value| peer_token_matches(value, expected)));

    if token_accepted {
        return Ok(());
    }

    match user {
        Some(user) if user.is_admin() => Ok(()),
        Some(_) => Err(PeerError::Forbidden),
        None if token_configured => Err(PeerError::Unauthorized),
        None => Err(PeerError::Unconfigured),
    }
}

/// The peer credential, from `X-Peer-Token` or `Authorization: Bearer`.
fn supplied_peer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-peer-token")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
}

fn peer_token_matches(supplied: &str, expected: &str) -> bool {
    botsecurity_core::constant_time::eq(supplied.as_bytes(), expected.as_bytes())
}

fn error_response(locale: &botcore::i18n::Locale, error: PeerError) -> Response {
    let status = match &error {
        PeerError::Disabled => StatusCode::FORBIDDEN,
        PeerError::Unconfigured => StatusCode::SERVICE_UNAVAILABLE,
        PeerError::Unauthorized => StatusCode::UNAUTHORIZED,
        PeerError::Forbidden => StatusCode::FORBIDDEN,
        PeerError::Invalid(_) => StatusCode::BAD_REQUEST,
        PeerError::UnknownNode(_) => StatusCode::NOT_FOUND,
        PeerError::RegistryFull => StatusCode::SERVICE_UNAVAILABLE,
    };
    let code = match &error {
        PeerError::Disabled => "peer_network_disabled",
        PeerError::Unconfigured => "peer_network_unconfigured",
        PeerError::Unauthorized => "unauthorized",
        PeerError::Forbidden => "forbidden",
        PeerError::Invalid(_) => "invalid_peer",
        PeerError::UnknownNode(_) => "unknown_peer",
        PeerError::RegistryFull => "peer_registry_full",
    };
    // #2094 — `error` keeps the stable machine code, `message` is localized.
    let (key, detail) = match &error {
        PeerError::Disabled => ("error-peer-network-disabled", None),
        PeerError::Unconfigured => ("error-peer-network-unconfigured", None),
        PeerError::Unauthorized => ("error-peer-network-unauthorized", None),
        PeerError::Forbidden => ("error-peer-network-forbidden", None),
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
    use crate::security::auth_api::types::Role;

    const PEER_TOKEN: &str = "peer-node-token-with-enough-entropy";

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

    fn peer_headers(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-peer-token", value.parse().expect("header"));
        headers
    }

    fn admin() -> AuthenticatedUser {
        AuthenticatedUser::new(uuid::Uuid::new_v4(), "root".to_string()).with_role(Role::Admin)
    }

    fn regular_user() -> AuthenticatedUser {
        AuthenticatedUser::new(uuid::Uuid::new_v4(), "alice".to_string())
    }

    /// #2141 — the old behaviour (`token = None` ⇒ allow) is gone: an
    /// unconfigured peer network fails closed instead of accepting payloads.
    #[test]
    fn unconfigured_peer_network_is_not_open() {
        let network = disabled_network_with_token(None);

        assert_eq!(
            authorize_machine(&HeaderMap::new(), &network),
            Err(PeerError::Unconfigured)
        );
        assert_eq!(
            authorize_machine(&peer_headers(PEER_TOKEN), &network),
            Err(PeerError::Unconfigured)
        );
        assert_eq!(
            authorize_control(&HeaderMap::new(), &network, None),
            Err(PeerError::Unconfigured)
        );
    }

    /// A machine call without the shared token is 401, never a silent pass.
    #[test]
    fn an_unauthenticated_peer_call_is_rejected() {
        let network = disabled_network_with_token(Some(PEER_TOKEN));

        assert_eq!(
            authorize_machine(&HeaderMap::new(), &network),
            Err(PeerError::Unauthorized)
        );
        assert_eq!(
            authorize_machine(&headers_with("Bearer wrong"), &network),
            Err(PeerError::Unauthorized)
        );
        assert_eq!(
            authorize_machine(&headers_with(&format!("Bearer {PEER_TOKEN}")), &network),
            Ok(())
        );
        assert_eq!(
            authorize_machine(&peer_headers(PEER_TOKEN), &network),
            Ok(())
        );
    }

    /// `status`/`enable`/`disable` accept a peer token or an admin session;
    /// a plain authenticated user is forbidden.
    #[test]
    fn control_routes_require_an_admin_without_a_peer_token() {
        let network = disabled_network_with_token(None);
        assert_eq!(
            authorize_control(&HeaderMap::new(), &network, Some(&regular_user())),
            Err(PeerError::Forbidden)
        );
        assert_eq!(
            authorize_control(&HeaderMap::new(), &network, Some(&admin())),
            Ok(())
        );

        let network = disabled_network_with_token(Some(PEER_TOKEN));
        assert_eq!(
            authorize_control(&peer_headers(PEER_TOKEN), &network, None),
            Ok(())
        );
        assert_eq!(
            authorize_control(&HeaderMap::new(), &network, Some(&admin())),
            Ok(())
        );
        assert_eq!(
            authorize_control(&HeaderMap::new(), &network, None),
            Err(PeerError::Unauthorized)
        );
    }

    /// A JWT in `Authorization` is not a peer credential: only the configured
    /// token opens a machine route.
    #[test]
    fn a_user_token_is_not_a_peer_token() {
        let network = disabled_network_with_token(Some(PEER_TOKEN));
        assert_eq!(
            authorize_machine(&headers_with("Bearer eyJhbGciOiJIUzI1NiJ9.e30.sig"), &network),
            Err(PeerError::Unauthorized)
        );
    }

    #[test]
    fn peer_token_comparison_is_length_safe() {
        assert!(peer_token_matches("same", "same"));
        assert!(!peer_token_matches("same", "samf"));
        assert!(!peer_token_matches("short", "longer"));
    }
}
