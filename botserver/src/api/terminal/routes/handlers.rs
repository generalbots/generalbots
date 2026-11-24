use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use futures_util::{SinkExt, StreamExt};

use crate::security::auth_api::types::AuthenticatedUser;

use super::super::sanitize_container_cwd;
use super::authz;
use super::{CreateTerminalRequest, KillTerminalRequest, TerminalRouterState};

pub(super) async fn create_terminal(
    State(state): State<Arc<TerminalRouterState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateTerminalRequest>,
) -> impl IntoResponse {
    // The chat workspace IS the user's VM: when the caller does not name a
    // Vibe project VM, attach to their agent container so the terminal opens
    // where the conversation files live.
    #[cfg(feature = "agent-vm")]
    let agent_vm = crate::core::bot::agent_vm_hook::user_vm_name(user.user_id).await;
    #[cfg(not(feature = "agent-vm"))]
    let agent_vm: Option<String> = None;

    let explicit_container = req.container.clone().filter(|c| !c.trim().is_empty());
    let container = explicit_container.or_else(|| agent_vm.clone());
    // SECURITY: a container-less session spawns a PTY on the botserver host
    // itself — any caller could read the whole server filesystem ("terminal
    // exposes server contents" report). Web terminals must run inside a VM
    // container; refuse anything else outright.
    //
    // #1841 — name the actual missing capability instead of letting this end
    // as a generic, sanitized failure: a build without agent-vm has no
    // workspace to attach to, and a build with it still needs the user to
    // start the workspace before the shell exists.
    let container = match container {
        Some(c) => c,
        None => {
            let (error, message) = if cfg!(feature = "agent-vm") {
                (
                    "no_workspace_vm",
                    "No workspace VM is running. Send a message in chat to start your workspace, then reopen the terminal.",
                )
            } else {
                (
                    "agent_vm_unavailable",
                    "Terminal requires the agent VM, which is not available in this build. The desktop edition includes it; otherwise open a project VM terminal instead.",
                )
            };
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": error, "message": message })),
            );
        }
    };
    // SECURITY: the caller may attach only to their OWN agent VM or to a
    // `vm_instances` container of a project they can access (viewer+).
    // Arbitrary names (e.g. the server's own `tables`, `vault`, `drive`,
    // `system` containers) would otherwise grant an `incus exec` shell on
    // production infrastructure.
    let is_own_agent_vm = agent_vm.as_deref() == Some(container.as_str());
    if !is_own_agent_vm {
        if let Err(e) = authz::authorize_project_container(&state.pool, user.user_id, &container) {
            botlog::safe_warn!("terminal create forbidden (user {}): {e}", user.user_id);
            return (
                axum::http::StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": e })),
            );
        }
    }
    let shell = req
        .shell
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "/bin/bash".to_string());
    // A download/preview of conversation files is only useful if the shell
    // starts where they are. When the caller does not name a directory, open
    // the newest conversation folder of the user's own workspace VM.
    let cwd = match req.cwd.as_deref().filter(|c| !c.trim().is_empty()) {
        Some(raw) => sanitize_container_cwd(raw).unwrap_or_else(|| "/opt/vibe/app".to_string()),
        None if is_own_agent_vm => agent_conversation_cwd(&container).await,
        None => "/opt/vibe/app".to_string(),
    };
    // #2729 — bind the PTY to the authenticated user + tenant from the moment
    // it is created. Every later operation is checked against this scope.
    let scope = authz::SessionScope::of(&user);
    let result = state
        .manager
        .create_scoped_session(shell, cwd, Some(container), scope);
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
                // #2729 — the capability is an unguessable per-session token
                // the client must echo on the WS handshake. Returning it only
                // in this authenticated response keeps it out of other users'
                // hands (the scoped list endpoint returns it to the owner too).
                "capability": session.capability,
                "ws_url": format!(
                    "/api/terminal/ws?id={}&capability={}",
                    session.id, session.capability
                ),
            })),
        ),
        Err(e) => {
            // Keep the raw cause in the operator log (the sanitizer used to
            // collapse it into "internal_error" before the user saw it), and
            // return a fixed, safe category the terminal can explain (#1841).
            log::error!("terminal create failed: {e}");
            let lower = e.to_ascii_lowercase();
            let (status, error, message) = if lower.contains("incus")
                && (lower.contains("not found")
                    || lower.contains("no such file")
                    || lower.contains("cannot find"))
            {
                (
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    "incus_unavailable",
                    "Terminal needs Incus, but the incus command was not found on this machine.",
                )
            } else if lower.contains("instance")
                && (lower.contains("not found")
                    || lower.contains("does not exist")
                    || lower.contains("no such"))
            {
                (
                    axum::http::StatusCode::CONFLICT,
                    "vm_not_found",
                    "The workspace VM was not found. Send a message in chat to start it, then reopen the terminal.",
                )
            } else if lower.contains("stopped") || lower.contains("not running") {
                (
                    axum::http::StatusCode::CONFLICT,
                    "vm_stopped",
                    "The workspace VM is stopped. Send a message in chat to start it, then reopen the terminal.",
                )
            } else if lower.contains("permission denied") {
                (
                    axum::http::StatusCode::FORBIDDEN,
                    "terminal_forbidden",
                    "The terminal cannot access the workspace VM. Check the Incus permissions and try again.",
                )
            } else {
                (
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    "terminal_unavailable",
                    "The terminal could not start. Check that the workspace VM is running and try again.",
                )
            };
            (
                status,
                Json(serde_json::json!({ "error": error, "message": message })),
            )
        }
    };
    (code, body)
}

pub(super) async fn list_terminals(
    State(state): State<Arc<TerminalRouterState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    state.manager.reap();
    // #2729 — never enumerate the process-wide registry. Only the caller's
    // own user + tenant scope is visible, including its reconnect capability.
    let scope = authz::SessionScope::of(&user);
    Json(serde_json::json!({ "terminals": state.manager.list_sessions_for(&scope) }))
}

pub(super) async fn kill_terminal(
    State(state): State<Arc<TerminalRouterState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<KillTerminalRequest>,
) -> impl IntoResponse {
    match state.manager.kill_session_scoped(&req.id, &user).await {
        Ok(()) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({ "ok": true })),
        ),
        Err(_) => {
            botlog::safe_warn!(
                "terminal kill forbidden (user {}): {}",
                user.user_id,
                req.id
            );
            (
                axum::http::StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "forbidden" })),
            )
        }
    }
}

pub(super) async fn terminal_ws(
    State(state): State<Arc<TerminalRouterState>>,
    user: Option<Extension<AuthenticatedUser>>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let id = query.get("id").cloned().map_or_else(Default::default, |v| v);
    let capability = query
        .get("capability")
        .cloned()
        .unwrap_or_else(String::new);
    // `/api/terminal/ws` is a registered capability route: the browser proxy
    // cannot forward an Authorization header, so the unguessable per-session
    // capability is the credential. When the middleware does supply a real
    // authenticated principal (direct call), also enforce the exact user +
    // tenant scope — a matching capability from another tenant is a 403.
    let principal = user
        .as_ref()
        .map(|Extension(u)| u)
        .filter(|u| u.is_authenticated());
    match state.manager.get_session_scoped(&id, principal, &capability) {
        Ok(session) => ws.on_upgrade(move |socket| terminal_ws_loop(socket, session)),
        Err(_) => {
            botlog::safe_warn!(
                "terminal ws handshake rejected (user {}): bad capability or scope",
                user.as_ref().map_or_else(
                    || "none".to_string(),
                    |Extension(u)| u.user_id.to_string()
                )
            );
            axum::response::Response::builder()
                .status(axum::http::StatusCode::FORBIDDEN)
                .body("terminal session forbidden".into())
                .unwrap_or_else(|_| axum::response::Response::default())
        }
    }
}

async fn terminal_ws_loop(mut socket: WebSocket, session: Arc<super::super::TerminalSession>) {
    authz::send_history(&session, &mut socket).await;
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
                        if let Some((cols, rows)) = authz::parse_resize(&text) {
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

/// Newest conversation folder inside the user's agent container, or the
/// conversations root when the container has none yet.
async fn agent_conversation_cwd(container: &str) -> String {
    const ROOT: &str = "/opt/vibe/conversations";
    let container = container.to_string();
    let found = tokio::task::spawn_blocking(move || -> Option<String> {
        let mut cmd = botlib::security::command_guard::SafeCommand::new("incus").ok()?;
        cmd = cmd.arg("exec").ok()?;
        cmd = cmd.arg(&container).ok()?;
        cmd = cmd.arg("--").ok()?;
        cmd = cmd.arg("sh").ok()?;
        cmd = cmd.arg("-c").ok()?;
        cmd = cmd
            .trusted_shell_script_arg(&format!("ls -dt {ROOT}/*/ 2>/dev/null | head -1"))
            .ok()?;
        let output = cmd.execute().ok()?;
        let path = String::from_utf8_lossy(&output.stdout)
            .trim()
            .trim_end_matches('/')
            .to_string();
        (path.starts_with(ROOT) && path.len() > ROOT.len()).then_some(path)
    })
    .await
    .ok()
    .flatten();
    found.unwrap_or_else(|| ROOT.to_string())
}
