//! WebSocket bridge to BotModels' `/v1/audio/duplex` endpoint (#1792).
//!
//! The browser only ever sees `/api/speech/duplex`: the BotModels host, API
//! key and transport stay server-side. The client speaks the same frame
//! protocol as BotModels (`meta`, `partial`/`final`, `speech_start`,
//! binary 16-bit mono 16 kHz PCM, `speech_end`, `barge_in`), so this module
//! forwards frames verbatim in both directions. When BotModels is disabled or
//! unreachable the upgrade is refused with `503`, which lets the chat client
//! fall back to the turn-based recorder instead of showing a false live state.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{
    CloseFrame as AxumCloseFrame, Message as AxumMessage, WebSocket, WebSocketUpgrade,
};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame as WsCloseFrame;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::connect_async_tls_with_config;
use uuid::Uuid;

use botcore::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DuplexQuery {
    /// Bot whose `botmodels-*` configuration should be used.
    pub bot_id: Option<String>,
}

/// Resolve the BotModels base URL and API key exactly like the other speech
/// paths (bot config first, process environment as the fallback).
fn botmodels_endpoint(state: &Arc<AppState>, bot_uuid: Uuid) -> Option<(String, String)> {
    use botcore::config::ConfigManager;

    let cfg = ConfigManager::new(state.conn.clone());
    // `get_config` answers `Ok("")` for a missing key, so an explicit false is
    // the only value that disables the service; no setting means "follow the
    // environment", matching the attachment transcription path.
    let enabled = match cfg.get_config(&bot_uuid, "botmodels-enabled", None) {
        Ok(value) if !value.trim().is_empty() => value.eq_ignore_ascii_case("true"),
        _ => std::env::var("BOTMODELS_HOST").is_ok(),
    };
    if !enabled {
        return None;
    }

    let port = cfg
        .get_config(&bot_uuid, "botmodels-port", Some("8085"))
        .ok()
        .filter(|port| !port.trim().is_empty())
        .or_else(|| {
            std::env::var("BOTMODELS_PORT")
                .ok()
                .filter(|port| !port.trim().is_empty())
        })
        .unwrap_or_else(|| "8085".to_string());

    let base = cfg
        .get_config(&bot_uuid, "botmodels-host", None)
        .ok()
        .filter(|host| !host.trim().is_empty())
        .or_else(|| {
            std::env::var("BOTMODELS_HOST")
                .ok()
                .filter(|host| !host.trim().is_empty())
        })
        .map(|host| {
            let host = host.trim().to_string();
            let host = if host == "0.0.0.0" {
                "127.0.0.1".to_string()
            } else {
                host
            };
            if host.starts_with("http://")
                || host.starts_with("https://")
                || host.starts_with("ws://")
                || host.starts_with("wss://")
            {
                host
            } else {
                format!("http://{host}:{port}")
            }
        })
        .unwrap_or_else(|| format!("http://127.0.0.1:{port}"));

    let key = cfg
        .get_config(&bot_uuid, "botmodels-api-key", None)
        .ok()
        .filter(|key| !key.trim().is_empty())
        .or_else(|| std::env::var("BOTMODELS_API_KEY").ok())
        .unwrap_or_default();

    Some((base.trim_end_matches('/').to_string(), key))
}

fn unavailable(message: &'static str) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "duplex_unavailable",
            "message": message,
        })),
    )
        .into_response()
}

fn upstream_url(base: &str, key: &str) -> String {
    let ws_base = if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else if base.starts_with("ws://") || base.starts_with("wss://") {
        base.to_string()
    } else {
        format!("ws://{base}")
    };

    let mut url = format!("{}/v1/audio/duplex", ws_base.trim_end_matches('/'));
    if !key.is_empty() {
        url.push_str("?api_key=");
        url.push_str(&urlencoding::encode(key));
    }
    url
}

/// Authenticated upgrade; the auth middleware has already validated the
/// `?token=` query parameter (browsers cannot set an Authorization header on
/// a WebSocket) or the session cookie.
pub async fn duplex(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<DuplexQuery>,
) -> Response {
    let bot_uuid = query
        .bot_id
        .as_deref()
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or(Uuid::nil());

    let Some((base, key)) = botmodels_endpoint(&state, bot_uuid) else {
        return unavailable("BotModels is not enabled, so live duplex voice is unavailable.");
    };

    let url = upstream_url(&base, &key);
    // Connect before upgrading the client: a refused upgrade is visible to the
    // browser as a failed WebSocket, which is what triggers the fallback UI.
    let upstream = match tokio::time::timeout(
        Duration::from_secs(6),
        connect_async_tls_with_config(url.as_str(), None, false, None),
    )
    .await
    {
        Ok(Ok((socket, _response))) => socket,
        Ok(Err(_error)) => {
            log::warn!("botmodels duplex connection failed; client will fall back");
            return unavailable("BotModels speech service is unavailable.");
        }
        Err(_elapsed) => {
            log::warn!("botmodels duplex connection timed out; client will fall back");
            return unavailable("BotModels speech service did not answer in time.");
        }
    };

    ws.on_upgrade(move |client| relay(client, upstream))
        .into_response()
}

fn close_frame(code: u16, reason: &'static str) -> AxumMessage {
    AxumMessage::Close(Some(AxumCloseFrame {
        code,
        reason: reason.into(),
    }))
}

async fn relay<S>(client: WebSocket, upstream: tokio_tungstenite::WebSocketStream<S>)
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (mut client_tx, mut client_rx) = client.split();
    let (mut upstream_tx, mut upstream_rx) = upstream.split();

    loop {
        tokio::select! {
            message = client_rx.next() => {
                match message {
                    Some(Ok(AxumMessage::Text(text))) => {
                        if upstream_tx.send(WsMessage::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(AxumMessage::Binary(data))) => {
                        if upstream_tx.send(WsMessage::Binary(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(AxumMessage::Ping(data))) => {
                        if upstream_tx.send(WsMessage::Ping(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(AxumMessage::Pong(data))) => {
                        let _ = upstream_tx.send(WsMessage::Pong(data)).await;
                    }
                    Some(Ok(AxumMessage::Close(frame))) => {
                        let mapped = frame.map(|frame| WsCloseFrame {
                            code: CloseCode::from(frame.code),
                            reason: frame.reason,
                        });
                        let _ = upstream_tx.send(WsMessage::Close(mapped)).await;
                        break;
                    }
                    Some(Err(_error)) | None => break,
                }
            }
            message = upstream_rx.next() => {
                match message {
                    Some(Ok(WsMessage::Text(text))) => {
                        if client_tx.send(AxumMessage::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(WsMessage::Binary(data))) => {
                        if client_tx.send(AxumMessage::Binary(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(WsMessage::Ping(data))) => {
                        if client_tx.send(AxumMessage::Ping(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(WsMessage::Pong(data))) => {
                        let _ = client_tx.send(AxumMessage::Pong(data)).await;
                    }
                    Some(Ok(WsMessage::Close(frame))) => {
                        let normal = frame
                            .as_ref()
                            .map(|frame| frame.code == CloseCode::Normal)
                            .unwrap_or(false);
                        if normal {
                            let _ = client_tx
                                .send(close_frame(1000, "duplex closed"))
                                .await;
                        } else {
                            let _ = client_tx
                                .send(AxumMessage::Text(
                                    serde_json::json!({
                                        "type": "error",
                                        "message": "duplex upstream closed unexpectedly",
                                    })
                                    .to_string(),
                                ))
                                .await;
                            let _ = client_tx
                                .send(close_frame(1011, "duplex upstream closed unexpectedly"))
                                .await;
                        }
                        break;
                    }
                    Some(Err(_error)) | None => {
                        let _ = client_tx
                            .send(AxumMessage::Text(
                                serde_json::json!({
                                    "type": "error",
                                    "message": "duplex upstream connection lost",
                                })
                                .to_string(),
                            ))
                            .await;
                        let _ = client_tx
                            .send(close_frame(1011, "duplex upstream connection lost"))
                            .await;
                        break;
                    }
                }
            }
        }
    }
}
