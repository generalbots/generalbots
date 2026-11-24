use axum::{
    extract::{
        ws::{Message as AxumMessage, WebSocket, WebSocketUpgrade},
        OriginalUri, Query, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde::Deserialize;
use tokio_tungstenite::{
    connect_async_tls_with_config, tungstenite,
    tungstenite::protocol::Message as TungsteniteMessage,
};

use crate::shared::AppState;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// Forwarded verbatim to the backend WebSocket URL; not read here.
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub bot_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OptionalWsQuery {
    pub(crate) task_id: Option<String>,
}

/// Remove credential values from a query string before it is written to a log.
/// Callers pass the live JWT as `token=` / `access_token=`; the raw query is
/// still forwarded to the backend unchanged (#2703).
fn redact_query(query: &str) -> String {
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            if key.eq_ignore_ascii_case("token") || key.eq_ignore_ascii_case("access_token") {
                if value.is_empty() {
                    key.to_string()
                } else {
                    format!("{key}=<redacted>")
                }
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}

/// Build exactly the string the API WS proxy writes to its INFO log.
/// Kept separate from the `info!` call so the "no credential in logs"
/// property can be asserted without a log capture harness (#2703).
fn redacted_api_ws_target(backend_base: &str, path: &str, query: &str) -> String {
    format!(
        "{backend_base}{path}?{}",
        redact_query(query.trim_start_matches('?'))
    )
}

#[cfg(test)]
mod redaction_tests {
    use super::{redact_query, redacted_api_ws_target};

    #[test]
    fn tokens_are_redacted_but_other_parameters_survive() {
        let redacted = redact_query("session_id=abc&token=SENTINEL&bot_name=demo");
        assert!(!redacted.contains("SENTINEL"));
        assert!(redacted.contains("session_id=abc"));
        assert!(redacted.contains("bot_name=demo"));
        assert!(redacted.contains("token=<redacted>"));
    }

    #[test]
    fn access_token_and_mixed_case_variants_are_redacted() {
        let redacted = redact_query("ACCESS_TOKEN=SENTINEL-A&Token=SENTINEL-B&id=ok");
        assert!(!redacted.contains("SENTINEL-A"));
        assert!(!redacted.contains("SENTINEL-B"));
        assert!(redacted.contains("ACCESS_TOKEN=<redacted>"));
        assert!(redacted.contains("Token=<redacted>"));
        assert!(redacted.contains("id=ok"));
    }

    #[test]
    fn empty_credential_is_not_turned_into_a_value() {
        assert_eq!(redact_query("token&id=ok"), "token&id=ok");
    }

    #[test]
    fn logged_target_never_contains_a_sentinel_token() {
        let logged = redacted_api_ws_target(
            "ws://backend",
            "/api/terminal/ws",
            "?id=session-1&capability=cap-1&token=SENTINEL-JWT&access_token=SENTINEL-2",
        );
        assert!(!logged.contains("SENTINEL-JWT"));
        assert!(!logged.contains("SENTINEL-2"));
        assert!(logged.starts_with("ws://backend/api/terminal/ws?"));
        assert!(logged.contains("id=session-1"));
        assert!(logged.contains("capability=cap-1"));
        assert!(logged.contains("token=<redacted>"));
        assert!(logged.contains("access_token=<redacted>"));
    }
}

/// Generic WebSocket proxy for backend API WS endpoints (/api/terminal/ws,
/// /api/browser/.../ws, etc.). reqwest-based proxy_api cannot tunnel WS
/// upgrades, so API WS routes must be extened here with a real proxy that
/// preserves the original path + query (id, token, ...).
pub async fn api_ws_proxy(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
) -> impl IntoResponse {
    let forwarded_path = uri.path().to_string();
    let forwarded_query = uri.query().map_or_else(String::new, |q| format!("?{q}"));
    ws.on_upgrade(move |socket| handle_api_ws_proxy(socket, state, forwarded_path, forwarded_query))
}

async fn handle_api_ws_proxy(
    client_socket: WebSocket,
    state: AppState,
    path: String,
    query: String,
) {
    let backend_base = state
        .client
        .base_url()
        .replace("https://", "wss://")
        .replace("http://", "ws://");
    let backend_url = format!("{backend_base}{path}{query}");

    info!(
        "Proxying API WebSocket to: {}",
        redacted_api_ws_target(&backend_base, &path, &query)
    );

    let backend_socket = if backend_url.starts_with("wss://") {
        let mut tls_builder = native_tls::TlsConnector::builder();
        let ca_path = botlib::security::ca_cert_path();
        if let Ok(pem) = std::fs::read(&ca_path) {
            match native_tls::Certificate::from_pem(&pem) {
                Ok(cert) => {
                    tls_builder.add_root_certificate(cert);
                }
                Err(e) => warn!("Invalid platform CA PEM at {ca_path}: {e}"),
            }
        }
        let Ok(tls_connector) = tls_builder.build() else {
            error!("Failed to build TLS connector for API WebSocket proxy");
            return;
        };
        let connector = tokio_tungstenite::Connector::NativeTls(tls_connector);
        match connect_async_tls_with_config(&backend_url, None, false, Some(connector)).await {
            Ok((socket, _)) => socket,
            Err(e) => {
                error!("Failed to connect to backend API WebSocket: {e}");
                return;
            }
        }
    } else {
        match tokio_tungstenite::connect_async(&backend_url).await {
            Ok((socket, _)) => socket,
            Err(e) => {
                error!("Failed to connect to backend API WebSocket: {e}");
                return;
            }
        }
    };

    info!("Connected to backend API WebSocket");

    let (mut client_tx, mut client_rx) = client_socket.split();
    let (mut backend_tx, mut backend_rx) = backend_socket.split();

    loop {
        tokio::select! {
            msg = client_rx.next() => {
                match msg {
                    Some(Ok(AxumMessage::Text(text))) => {
                        if backend_tx.send(TungsteniteMessage::Text(text)).await.is_err() { break; }
                    }
                    Some(Ok(AxumMessage::Binary(data))) => {
                        if backend_tx.send(TungsteniteMessage::Binary(data)).await.is_err() { break; }
                    }
                    Some(Ok(AxumMessage::Ping(data))) => {
                        let _ = client_tx.send(AxumMessage::Pong(data)).await;
                    }
                    Some(Ok(AxumMessage::Pong(_))) => {}
                    Some(Ok(AxumMessage::Close(_))) | None => break,
                    Some(Err(_)) => break,
                }
            }
            msg = backend_rx.next() => {
                match msg {
                    Some(Ok(TungsteniteMessage::Text(text))) => {
                        if client_tx.send(AxumMessage::Text(text)).await.is_err() { break; }
                    }
                    Some(Ok(TungsteniteMessage::Binary(data))) => {
                        if client_tx.send(AxumMessage::Binary(data)).await.is_err() { break; }
                    }
                    Some(Ok(TungsteniteMessage::Ping(data))) => {
                        let _ = backend_tx.send(TungsteniteMessage::Pong(data)).await;
                    }
                    Some(Ok(TungsteniteMessage::Pong(_))) => {}
                    Some(Ok(TungsteniteMessage::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
    debug!("[API_WS_PROXY:{path}] Proxy connection closed");
}

fn normalize_ws_bot_name(name: &str) -> String {
    const APP_ROUTES: &[&str] = &[
        "chat",
        "app",
        "ws",
        "cloud",
        "suite",
        "ui",
        "api",
        "auth",
        "desktop",
        "sources",
        "integrations",
        "settings",
        "admin",
        "wizard",
        "login",
        "signup",
        "drive",
        "tasks",
        "mail",
        "calendar",
        "meet",
        "docs",
        "sheet",
        "slides",
        "paper",
        "research",
        "learn",
        "analytics",
        "dashboards",
        "monitoring",
        "governance",
        "people",
        "crm",
        "tickets",
        "billing",
        "products",
        "video",
        "player",
        "canvas",
        "social",
        "project",
        "goals",
        "workspace",
        "designer",
        "vibe",
        "erp",
        "fraud",
        "attendant",
        "banking",
        "biometry",
        "brazil",
        "browser",
        "campaigns",
        "events",
        "approvals",
        "compliance",
        "database",
        "email",
        "handoff",
        "hr",
        "itsm",
        "kyc",
        "lists",
        "o365",
        "minutes",
        "plan",
        "plugins",
        "pos",
        "retail",
        "sales",
        "tax",
        "templates",
        "templates-app",
        "terminal",
        "timeclock",
        "tools",
        "vision",
        "about",
        "notepad",
        "notes",
        "assistant",
        "audit",
        "automations",
        "calculator",
        "clock",
        "concierge",
        "editor",
        "jukebox",
        "memory",
        "photos",
        "recycle",
        "snapshot",
        "store",
        "timer",
        "weather",
        "widgets",
    ];
    if name.is_empty() || APP_ROUTES.contains(&name) || name.contains('.') {
        "default".to_string()
    } else {
        name.to_string()
    }
}

pub async fn ws_proxy(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    Query(params): Query<WsQuery>,
) -> impl IntoResponse {
    let path_parts: Vec<&str> = uri.path().split('/').collect();
    let raw_bot_name = params
        .bot_name
        .filter(|name| name != "ws" && !name.is_empty())
        .or_else(|| {
            path_parts
                .iter()
                .find(|part| {
                    !part.is_empty()
                        && **part != "chat"
                        && **part != "app"
                        && **part != "ws"
                        && **part != "cloud"
                })
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "default".to_string());
    let bot_name = normalize_ws_bot_name(&raw_bot_name);

    let params_with_bot = WsQuery {
        bot_name: Some(bot_name),
        ..params
    };

    // #2447 — `/ws/{app}/{doc}` is a realtime COLLABORATION route, not a chat
    // bot route. The old fallback rewrote every one of them to `/ws?bot_name=…`,
    // so sheet/docs/slides edits went to the chat handler and were never
    // broadcast to peers. Preserve the full path (and raw token query) for
    // apps whose WS endpoint actually exists in botserver.
    let segments: Vec<&str> = uri.path().split('/').filter(|s| !s.is_empty()).collect();
    let is_app_route = segments.len() >= 3
        && segments.first() == Some(&"ws")
        && matches!(segments.get(1).copied(), Some("sheet" | "docs" | "slides" | "canvas" | "plan"));
    if is_app_route {
        let mut url = format!(
            "{}{}",
            state.client.base_url().replace("https://", "wss://").replace("http://", "ws://"),
            uri.path()
        );
        if let Some(query) = uri.query() {
            url.push('?');
            url.push_str(query);
        }
        ws.on_upgrade(move |socket| super::ws_proxy_loop::handle_ws_passthrough(socket, url))
    } else {
        ws.on_upgrade(move |socket| super::ws_proxy_loop::handle_ws_proxy(socket, state, params_with_bot))
    }
}

pub async fn ws_task_progress_proxy(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<OptionalWsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| super::ws_proxy_loop::handle_task_progress_ws_proxy(socket, state, params))
}

