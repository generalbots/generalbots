//! Fraud, inventory, GL, retail, banking, M365, weba, directory, social,
//! learn, meet, billing, SaaS and Office event-stream router merges.

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use botcore::shared::state::AppState;
use botevents::office::runtime::runtime;
use botevents::{DomainEvent, NativeEnvelope, OfficeEventType, ReplayError};
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

#[cfg(all(test, feature = "fraud"))]
#[path = "fraud_route_contract_tests.rs"]
mod fraud_route_contract_tests;

use super::events_scope::caller_scope;

type NativeEventStream = Pin<Box<dyn futures::Stream<Item = Result<SseEvent, Infallible>> + Send>>;

/// The issue text uses a singular alias; the taxonomy publishes the plural
/// wire name. Both resolve to the same canonical event type.
fn canonical_type(name: &str) -> &str {
    if name == "sheet.exported" {
        "sheets.exported"
    } else {
        name
    }
}

fn parse_types(raw: Option<&str>) -> Vec<String> {
    raw.map_or_else(Vec::new, |value| {
        value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(canonical_type)
            .map(str::to_string)
            .collect()
    })
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|pair| {
                let mut parts = pair.trim().splitn(2, '=');
                if parts.next() == Some(name) {
                    parts.next().map(str::to_string)
                } else {
                    None
                }
            })
        })
}

fn token_authenticated(token: Option<&str>) -> bool {
    token
        .and_then(crate::main_module::routes::anonymous_auth::resolve_token_user)
        .is_some()
}

fn error_json(status: StatusCode, code: &str) -> axum::response::Response {
    (status, Json(json!({ "error": code }))).into_response()
}

#[derive(Deserialize)]
struct ReplayRequest {
    idempotency_key: String,
}

#[derive(Deserialize)]
struct PublishRequest {
    event_type: String,
    #[serde(default)]
    doc_id: Option<String>,
    #[serde(default)]
    workspace_id: Option<String>,
    #[serde(default)]
    actor_id: Option<String>,
    #[serde(default)]
    webhook_url: Option<String>,
}

#[derive(Deserialize)]
struct StreamQuery {
    #[serde(default)]
    types: Option<String>,
    #[serde(default)]
    access_token: Option<String>,
}

/// `GET /api/events/dead-letters` — privacy-safe DLQ summary for the admin UI.
/// Only dead letters owned by the caller's verified org/branch are returned.
async fn list_dead_letters(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    let Some(scope) = caller_scope(&headers, &state) else {
        return error_json(StatusCode::NOT_FOUND, "not_found");
    };
    Json(runtime().dead_letters_scoped(&scope)).into_response()
}

/// `POST /api/events/replay` — exactly-once replay by idempotency key, limited
/// to a dead letter in the caller's own tenant.
async fn replay_dead_letter(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<ReplayRequest>,
) -> axum::response::Response {
    let Some(scope) = caller_scope(&headers, &state) else {
        return error_json(StatusCode::NOT_FOUND, "not_found");
    };
    match runtime().replay_scoped(&scope, &request.idempotency_key) {
        Ok(result) => (
            StatusCode::OK,
            Json(json!({
                "idempotency_key": request.idempotency_key,
                "event_id": result.receipt.event_id,
                "subscription_id": result.receipt.subscription_id,
                "status_code": result.receipt.status_code,
                "attempt_count": result.receipt.attempt_count,
                "delivered_at": result.receipt.delivered_at,
                "deduped": result.deduped,
            })),
        )
            .into_response(),
        Err(ReplayError::NotFound) => error_json(StatusCode::NOT_FOUND, "not_found"),
        Err(ReplayError::SubscriptionGone) => error_json(StatusCode::CONFLICT, "subscription_gone"),
        Err(ReplayError::DeliveryFailed(message)) => (
            StatusCode::BAD_GATEWAY,
            Json(crate::security::error_sanitizer::log_and_sanitize_json(
                &message,
                "events.replay",
                None,
            )),
        )
            .into_response(),
    }
}

/// `POST /api/events/publish` — emits one metadata-only event into the
/// dispatcher. Clients never supply content or an event id.
async fn publish_event(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PublishRequest>,
) -> axum::response::Response {
    // The event is written into the caller's own pipeline so its dead letters
    // and ledger entry are owned by the same verified tenant (issue #2825).
    let Some(scope) = caller_scope(&headers, &state) else {
        return error_json(StatusCode::NOT_FOUND, "not_found");
    };
    let Some(name) = OfficeEventType::from_name(canonical_type(&request.event_type)) else {
        return error_json(StatusCode::BAD_REQUEST, "unknown_event_type");
    };
    let event = DomainEvent::new(
        runtime().next_event_id(),
        name,
        request.workspace_id.unwrap_or_else(|| "ws_123".to_string()),
        request.doc_id.unwrap_or_else(|| "doc_456".to_string()),
        request.actor_id.unwrap_or_else(|| "usr_789".to_string()),
    );
    if let Err(issues) = event.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_event", "issues": issues })),
        )
            .into_response();
    }
    let report = runtime().publish_scoped(&scope, &event, request.webhook_url.as_deref());
    (
        StatusCode::OK,
        Json(json!({
            "event_id": event.event_id,
            "event_type": name.as_str(),
            "webhooks_attempted": report.webhooks_attempted,
            "delivered": report.delivered,
            "dead_lettered": report.dead_lettered,
            "native_enqueued": report.native_enqueued,
        })),
    )
        .into_response()
}

/// `GET /ws/events/stream` — server-sent events for the native queue. It lives
/// under the anonymous-allowed `/ws` prefix so `EventSource` (which cannot set
/// an Authorization header) can authenticate with the `access_token` query
/// parameter, exactly like the chat socket.
async fn stream_events(
    Query(params): Query<StreamQuery>,
    headers: HeaderMap,
) -> Result<Sse<NativeEventStream>, axum::response::Response> {
    let token = params
        .access_token
        .clone()
        .or_else(|| cookie_value(&headers, "gb-access-token"));
    if !token_authenticated(token.as_deref()) {
        return Err(error_json(
            StatusCode::UNAUTHORIZED,
            "authentication_required",
        ));
    }
    let types = parse_types(params.types.as_deref());
    let cursor = runtime().stream().len();
    let stream = futures::stream::unfold(
        (cursor, types, Vec::<NativeEnvelope>::new()),
        move |(mut cursor, types, mut pending)| async move {
            loop {
                if pending.is_empty() {
                    let fresh = runtime().stream().since(cursor);
                    if fresh.is_empty() {
                        tokio::time::sleep(Duration::from_millis(400)).await;
                        continue;
                    }
                    cursor += fresh.len();
                    pending = fresh.into_iter().rev().collect();
                }
                let envelope = match pending.pop() {
                    Some(envelope) => envelope,
                    None => continue,
                };
                if !types.is_empty() && !types.iter().any(|name| name == &envelope.event_type) {
                    continue;
                }
                let data = serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_string());
                let event = SseEvent::default().event("native").data(data);
                return Some((Ok(event), (cursor, types, pending)));
            }
        },
    );
    let boxed: NativeEventStream = Box::pin(stream);
    Ok(Sse::new(boxed).keep_alive(KeepAlive::default()))
}

/// `GET /ws/events` — the same native stream over a WebSocket, for browsers
/// that cannot use `EventSource` (and for the suite's push automation).
async fn events_ws(
    ws: WebSocketUpgrade,
    Query(params): Query<StreamQuery>,
    headers: HeaderMap,
) -> axum::response::Response {
    let token = params
        .access_token
        .clone()
        .or_else(|| cookie_value(&headers, "gb-access-token"));
    if !token_authenticated(token.as_deref()) {
        return error_json(StatusCode::UNAUTHORIZED, "authentication_required");
    }
    let types = parse_types(params.types.as_deref());
    let cursor = runtime().stream().len();
    ws.on_upgrade(move |socket| serve_events_ws(socket, types, cursor))
}

async fn serve_events_ws(mut socket: WebSocket, types: Vec<String>, mut cursor: usize) {
    let mut ticker = tokio::time::interval(Duration::from_millis(400));
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let fresh = runtime().stream().since(cursor);
                for envelope in fresh {
                    cursor += 1;
                    if !types.is_empty()
                        && !types.iter().any(|name| name == &envelope.event_type)
                    {
                        continue;
                    }
                    let text =
                        serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_string());
                    if socket.send(Message::Text(text)).await.is_err() {
                        return;
                    }
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(_)) => continue,
                    _ => return,
                }
            }
        }
    }
}

/// Mounts the Office event surface (authenticated JSON endpoints plus the
/// push stream) on the botserver API router.
fn merge_events_router(api_router: &mut Router<Arc<AppState>>) {
    let require_auth = axum::middleware::from_fn(
        crate::security::botsecurity_auth::auth_api::middleware::require_auth_middleware,
    );
    let require_admin =
        axum::middleware::from_fn(crate::security::rbac_middleware::require_admin_middleware);
    // Dead-letter inspection and replay are admin-only: the DLQ carries
    // subscription URLs and replay causes a real delivery (issue #2825).
    // `require_auth` is layered last so it runs first and populates the
    // AuthenticatedUser extension the admin middleware checks.
    *api_router = api_router.clone().merge(
        Router::new()
            .route("/api/events/dead-letters", get(list_dead_letters))
            .route("/api/events/replay", post(replay_dead_letter))
            .layer(require_admin)
            .layer(require_auth.clone()),
    );
    *api_router = api_router.clone().merge(
        Router::new()
            .route("/api/events/publish", post(publish_event))
            .layer(require_auth),
    );
    *api_router = api_router.clone().merge(
        Router::new()
            .route("/ws/events/stream", get(stream_events))
            .route("/ws/events", get(events_ws)),
    );
}

pub(super) async fn merge_commerce(
    app_state: &Arc<AppState>,
    api_router: &mut Router<Arc<AppState>>,
    sub_router: &mut Router<()>,
) {
    merge_events_router(api_router);

    #[cfg(feature = "fraud")]
    {
        let fraud_state = Arc::new(crate::fraud::FraudState::new(app_state.conn.clone()));
        *sub_router = sub_router
            .clone()
            .merge(crate::fraud::configure_fraud_routes().with_state(fraud_state));
    }

    #[cfg(feature = "inventory")]
    {
        let inventory_state = Arc::new(crate::inventory::InventoryState {
            pool: app_state.conn.clone(),
        });
        *sub_router = sub_router
            .clone()
            .merge(crate::inventory::configure_inventory_routes().with_state(inventory_state));
    }

    #[cfg(feature = "gl")]
    {
        let gl_state = Arc::new(crate::gl::GlState {
            pool: app_state.conn.clone(),
        });
        *sub_router = sub_router
            .clone()
            .merge(crate::gl::configure_gl_routes().with_state(gl_state));
    }

    #[cfg(feature = "retail")]
    {
        *sub_router = sub_router.clone().merge(botretail::configure());
    }

    #[cfg(feature = "banking")]
    {
        *api_router = api_router.clone().merge(botbanking::configure());
    }

    #[cfg(feature = "m365")]
    {
        *api_router = api_router.clone().merge(botm365::configure());
    }

    #[cfg(feature = "weba")]
    {
        let weba_state = Arc::new(crate::weba::WebaState::new());
        *sub_router = sub_router
            .clone()
            .merge(crate::weba::configure_routes(weba_state));
    }

    #[cfg(feature = "directory")]
    {
        *sub_router = sub_router.clone().merge(
            crate::directory::scim::server::configure_scim_routes().with_state(app_state.clone()),
        );
    }

    #[cfg(feature = "social")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_social_router(app_state));
    }

    #[cfg(feature = "learn")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::learn::ui::configure_learn_ui_routes());
        *sub_router = sub_router.clone().merge(
            crate::learn::creator::configure_learn_api_routes()
                .with_state(Arc::new(botlearn::GamificationService::new())),
        );
    }

    #[cfg(feature = "meet")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::meet::ui::configure_meet_ui_routes().with_state(app_state.clone()));
    }

    #[cfg(feature = "billing")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_billing_router(app_state));
    }

    #[cfg(feature = "saas")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_saas_router(app_state));
    }
}
