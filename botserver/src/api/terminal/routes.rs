use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::StreamExt;
use serde::Deserialize;

use super::{default_shell, sanitize_cwd, TerminalManager};

#[derive(Deserialize)]
pub struct CreateTerminalRequest {
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub shell: Option<String>,
    /// Optional project VM container to exec into (e.g. `calculator-prod`).
    /// When set the shell runs inside that container via `incus exec`.
    #[serde(default)]
    pub container: Option<String>,
}

#[derive(Deserialize)]
pub struct KillTerminalRequest {
    pub id: String,
}

pub fn configure_terminal_routes() -> Router {
    let manager = Arc::new(TerminalManager::new());
    Router::new()
        .route("/api/terminal/create", post(create_terminal))
        .route("/api/terminal/list", get(list_terminals))
        .route("/api/terminal/kill", post(kill_terminal))
        .route("/api/terminal/ws", get(terminal_ws))
        .with_state(manager)
}

async fn create_terminal(
    State(manager): State<Arc<TerminalManager>>,
    Json(req): Json<CreateTerminalRequest>,
) -> impl IntoResponse {
    let container = req.container.clone().filter(|c| !c.trim().is_empty());
    let shell = req.shell.unwrap_or_else(|| {
        if container.is_some() {
            "/bin/bash".to_string()
        } else {
            default_shell()
        }
    });
    let cwd = sanitize_cwd(req.cwd.as_deref().unwrap_or(""));
    let result = match container {
        Some(name) => manager.create_session_with_container(shell, cwd, Some(name)),
        None => manager.create_session(shell, cwd),
    };
    let (code, body) = match result {
        Ok(session) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({
                "id": session.id,
                "shell": session.shell,
                "cwd": session.cwd,
                "container": session.container,
                "transport": if session.container.is_some() && cfg!(target_os = "windows") { "wsl-incus" } else if session.container.is_some() { "incus" } else { "local" },
                "created_at": session.created_at,
                "ws_url": format!("/api/terminal/ws?id={}", session.id),
            })),
        ),
        Err(e) => {
            log::error!("terminal create failed: {e}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
        }
    };
    (code, body)
}

async fn list_terminals(State(manager): State<Arc<TerminalManager>>) -> impl IntoResponse {
    manager.reap();
    Json(serde_json::json!({ "terminals": manager.list_sessions() }))
}

async fn kill_terminal(
    State(manager): State<Arc<TerminalManager>>,
    Json(req): Json<KillTerminalRequest>,
) -> impl IntoResponse {
    match manager.kill_session(&req.id).await {
        Ok(()) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({ "ok": true })),
        ),
        Err(e) => {
            log::warn!("terminal kill failed: {e}");
            (
                axum::http::StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        }
    }
}

async fn terminal_ws(
    State(manager): State<Arc<TerminalManager>>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let id = query.get("id").cloned().unwrap_or_default();
    let session = match manager.get_session(&id) {
        Some(s) => s,
        None => {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .body("terminal session not found".into())
                .unwrap_or_else(|_| axum::response::Response::default());
        }
    };
    ws.on_upgrade(move |socket| terminal_ws_loop(socket, session))
}

async fn terminal_ws_loop(mut socket: WebSocket, session: Arc<super::TerminalSession>) {
    send_history(&session, &mut socket).await;
    let mut events_rx = session.events.subscribe();
    loop {
        tokio::select! {
            incoming = socket.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if text == "ping" {
                            let _ = socket.send(Message::Text("pong".into())).await;
                            continue;
                        }
                        // xterm.js sends `resize <cols> <rows>`; forward it to
                        // the PTY instead of feeding it to the shell.
                        if let Some((cols, rows)) = parse_resize(&text) {
                            session.resize(cols, rows);
                            continue;
                        }
                        session.write(&text).ok();
                    }
                    Some(Ok(Message::Binary(bytes))) => {
                        session.write(&String::from_utf8_lossy(&bytes)).ok();
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => {
                        log::warn!("terminal ws error: {e}");
                        break;
                    }
                    _ => {}
                }
            }
            event = events_rx.recv() => {
                match event {
                    Ok(data) => {
                        let payload = serde_json::json!({ "type": "output", "data": data });
                        if socket.send(Message::Text(payload.to_string())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        log::warn!("terminal ws lagged {n} events");
                    }
                    Err(_) => break,
                }
            }
        }
    }
}

/// Parses an xterm.js resize control message: `resize <cols> <rows>`.
fn parse_resize(text: &str) -> Option<(u16, u16)> {
    let rest = text.strip_prefix("resize ")?;
    let mut parts = rest.split_whitespace();
    let cols: u16 = parts.next()?.parse().ok()?;
    let rows: u16 = parts.next()?.parse().ok()?;
    Some((cols, rows))
}

async fn send_history(session: &Arc<super::TerminalSession>, socket: &mut WebSocket) {
    for line in session.history() {
        let payload = serde_json::json!({ "type": "output", "data": line.data });
        if socket
            .send(Message::Text(payload.to_string()))
            .await
            .is_err()
        {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_resize_accepts_valid_and_rejects_garbage() {
        assert_eq!(parse_resize("resize 120 40"), Some((120, 40)));
        assert_eq!(parse_resize("resize 0 0"), Some((0, 0)));
        assert_eq!(parse_resize("resize nope 40"), None);
        assert_eq!(parse_resize("echo hi"), None);
    }
}
