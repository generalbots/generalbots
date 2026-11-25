use super::*;
use crate::ws_auth::{extract_user_from_token, reason_code, unauthorized, WsAuthQuery};
use axum::extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    Path, Query, State,
};
use axum::response::{IntoResponse, Response};
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

static CHANNELS: Lazy<RwLock<HashMap<String, broadcast::Sender<CanvasCollabMessage>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// #2733 — why a canvas collaboration upgrade was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CanvasAccessError {
    InvalidCanvasId,
    InvalidIdentity,
    Forbidden,
}

impl CanvasAccessError {
    fn code(self) -> &'static str {
        match self {
            Self::InvalidCanvasId => "invalid_canvas_id",
            Self::InvalidIdentity => "invalid_identity",
            Self::Forbidden => "canvas_access_denied",
        }
    }
}

/// Pure access predicate over the facts the database can establish: the
/// caller is verified, the canvas belongs to their tenant scope, and they are
/// either its creator or hold an explicit collaborator grant. A foreign or
/// unknown canvas can satisfy none of those, so it is never joined.
fn grants_canvas_access(verified: bool, in_scope: bool, owner: bool, role: Option<&str>) -> bool {
    verified
        && in_scope
        && (owner
            || matches!(
                role,
                Some("view" | "viewer" | "commenter" | "edit" | "editor" | "owner")
            ))
}

/// Both tenant columns must match the verified context; an unresolved context
/// can never own a canvas (#2735).
fn is_canvas_in_scope(canvas: &DbCanvas, org_id: Uuid, bot_id: Uuid) -> bool {
    !org_id.is_nil() && !bot_id.is_nil() && canvas.org_id == org_id && canvas.bot_id == bot_id
}

async fn authorize_canvas_ws(
    state: &CanvasState,
    canvas_id: &str,
    user_id: &str,
) -> Result<String, CanvasAccessError> {
    if canvas_id.trim() != canvas_id {
        return Err(CanvasAccessError::InvalidCanvasId);
    }
    let id = Uuid::parse_str(canvas_id).map_err(|_| CanvasAccessError::InvalidCanvasId)?;
    let caller = match Uuid::parse_str(user_id.trim()) {
        Ok(caller) if !caller.is_nil() => caller,
        _ => return Err(CanvasAccessError::InvalidIdentity),
    };

    let mut conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            log::warn!("canvas ws: database unavailable for canvas {id}");
            return Err(CanvasAccessError::Forbidden);
        }
    };

    let canvas: DbCanvas = match canvases::table
        .filter(canvases::id.eq(id))
        .first::<DbCanvas>(&mut conn)
    {
        Ok(canvas) => canvas,
        Err(_) => {
            log::warn!("canvas ws: canvas {id} not found");
            return Err(CanvasAccessError::Forbidden);
        }
    };

    let role = match canvas_collaborators::table
        .filter(canvas_collaborators::canvas_id.eq(id))
        .filter(canvas_collaborators::user_id.eq(caller))
        .select(canvas_collaborators::permission)
        .first::<String>(&mut conn)
    {
        Ok(permission) => Some(permission),
        Err(diesel::result::Error::NotFound) => None,
        Err(_) => {
            log::warn!("canvas ws: collaborator lookup failed for canvas {id}");
            None
        }
    };

    let owner = !canvas.created_by.is_nil() && canvas.created_by == caller;
    // A published canvas is an explicit view grant, in the same tenant scope.
    // Without this, a legitimate viewer of a public canvas could never join.
    let role = match role {
        Some(permission) => Some(permission),
        None if canvas.is_public => Some("viewer".to_string()),
        None => None,
    };
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    if grants_canvas_access(
        true,
        is_canvas_in_scope(&canvas, org_id, bot_id),
        owner,
        role.as_deref(),
    ) {
        Ok(id.to_string())
    } else {
        Err(CanvasAccessError::Forbidden)
    }
}

/// A refusal never upgrades the socket and carries only a stable machine code.
fn forbidden(reason: CanvasAccessError) -> Response {
    let status = match reason {
        CanvasAccessError::InvalidCanvasId => StatusCode::BAD_REQUEST,
        CanvasAccessError::InvalidIdentity => StatusCode::UNAUTHORIZED,
        CanvasAccessError::Forbidden => StatusCode::FORBIDDEN,
    };
    let code = reason.code();
    (
        status,
        [
            ("x-auth-error", "forbidden_resource"),
            ("x-auth-reason", code),
        ],
        Json(serde_json::json!({
            "error": "canvas_access_denied",
            "code": "canvas_ws_forbidden",
            "reason": code,
        })),
    )
        .into_response()
}

pub async fn handle_canvas_websocket(
    ws: WebSocketUpgrade,
    Path(canvas_id): Path<String>,
    Query(q): Query<WsAuthQuery>,
    State(state): State<Arc<CanvasState>>,
) -> Response {
    let identity = match extract_user_from_token(&q.token) {
        Ok(identity) => identity,
        Err(reason) => {
            log::warn!(
                "canvas ws refused for canvas {canvas_id}: {}",
                reason_code(&reason)
            );
            return unauthorized(reason);
        }
    };

    match authorize_canvas_ws(&state, &canvas_id, &identity.0).await {
        Ok(canvas_id) => ws
            .on_upgrade(move |socket| handle_canvas_connection(socket, canvas_id, identity))
            .into_response(),
        Err(reason) => {
            log::warn!(
                "canvas ws refused for canvas {canvas_id}: {}",
                reason.code()
            );
            forbidden(reason)
        }
    }
}

async fn handle_canvas_connection(
    socket: WebSocket,
    canvas_id: String,
    identity: (String, String),
) {
    let (mut sender, mut receiver) = socket.split();

    let broadcast_tx = {
        let mut channels = CHANNELS.write().await;
        channels
            .entry(canvas_id.clone())
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    };
    let mut broadcast_rx = broadcast_tx.subscribe();

    let (user_id, user_name) = identity;
    let user_id_for_send = user_id.clone();
    let user_color = "#4ECDC4".to_string();

    let join_msg = CanvasCollabMessage {
        msg_type: "join".to_string(),
        canvas_id: canvas_id.clone(),
        user_id: user_id.clone(),
        user_name: user_name.clone(),
        user_color: user_color.clone(),
        elements: None,
        timestamp: chrono::Utc::now(),
    };
    if let Err(e) = broadcast_tx.send(join_msg) {
        log::warn!("Canvas WebSocket: failed to broadcast join for {canvas_id}: {e}");
    }

    let broadcast_tx_clone = broadcast_tx.clone();
    let user_id_clone = user_id.clone();
    let user_name_clone = user_name.clone();
    let user_color_clone = user_color.clone();
    let canvas_id_clone = canvas_id.clone();

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(mut collab_msg) = serde_json::from_str::<CanvasCollabMessage>(&text) {
                        collab_msg.user_id = user_id_clone.clone();
                        collab_msg.user_name = user_name_clone.clone();
                        collab_msg.user_color = user_color_clone.clone();
                        collab_msg.canvas_id = canvas_id_clone.clone();
                        collab_msg.timestamp = chrono::Utc::now();
                        if let Err(e) = broadcast_tx_clone.send(collab_msg) {
                            log::warn!(
                                "Canvas WebSocket: failed to broadcast update for {canvas_id_clone}: {e}"
                            );
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    log::error!("Canvas WebSocket error: {e}");
                    break;
                }
                _ => {}
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if msg.user_id == user_id_for_send {
                continue;
            }
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = receive_task => {}
        _ = send_task => {}
    }

    let leave_msg = CanvasCollabMessage {
        msg_type: "leave".to_string(),
        canvas_id: canvas_id.clone(),
        user_id: user_id.clone(),
        user_name,
        user_color,
        elements: None,
        timestamp: chrono::Utc::now(),
    };
    if let Err(e) = broadcast_tx.send(leave_msg) {
        log::warn!("Canvas WebSocket: failed to broadcast leave for {canvas_id}: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::{grants_canvas_access, is_canvas_in_scope};
    use crate::DbCanvas;
    use uuid::Uuid;

    fn canvas() -> DbCanvas {
        let org_id = Uuid::from_u128(1);
        let bot_id = Uuid::from_u128(2);
        DbCanvas {
            id: Uuid::from_u128(3),
            branch_id: bot_id,
            name: "canvas".to_string(),
            description: None,
            width: 100,
            height: 100,
            background_color: None,
            thumbnail_url: None,
            is_public: false,
            is_template: false,
            created_by: Uuid::from_u128(3),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            org_id,
            bot_id,
        }
    }

    #[test]
    fn owner_and_explicit_grants_are_accepted_in_scope() {
        let c = canvas();
        assert!(grants_canvas_access(true, true, true, None));
        for role in ["view", "viewer", "commenter", "edit", "editor", "owner"] {
            assert!(
                grants_canvas_access(true, true, false, Some(role)),
                "role {role}"
            );
        }
        assert!(is_canvas_in_scope(&c, c.org_id, c.bot_id));
    }

    #[test]
    fn foreign_unknown_and_unverified_are_denied() {
        let c = canvas();
        for role in [None, Some(""), Some("guest"), Some("admin")] {
            assert!(
                !grants_canvas_access(true, true, false, role),
                "role {role:?}"
            );
        }
        assert!(!grants_canvas_access(false, true, true, Some("owner")));
        assert!(!grants_canvas_access(true, false, true, Some("owner")));
        assert!(!is_canvas_in_scope(&c, Uuid::nil(), c.bot_id));
        assert!(!is_canvas_in_scope(&c, c.org_id, Uuid::nil()));
        assert!(!is_canvas_in_scope(&c, Uuid::from_u128(99), c.bot_id));
    }
}
