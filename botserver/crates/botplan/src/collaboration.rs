use crate::auth::{
    authorize, caller_from_ws, Access, Caller, PlanError, PLAN_WS_CLOSE_FORBIDDEN,
};
use crate::handlers::visible_plan;
use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
        Path, Query,
    },
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

pub type PlanChannels = Arc<tokio::sync::RwLock<HashMap<String, broadcast::Sender<PlanMessage>>>>;
pub type PlanPresence = Arc<tokio::sync::RwLock<HashMap<String, Vec<PlanUserPresence>>>>;
pub type PlanTyping = Arc<tokio::sync::RwLock<HashMap<String, Vec<PlanTypingIndicator>>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMessage {
    pub msg_type: String,
    pub plan_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    pub task_id: Option<String>,
    pub content: Option<String>,
    pub position: Option<usize>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanUserPresence {
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    pub current_view: String,
    pub selected_task: Option<String>,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTypingIndicator {
    pub user_id: String,
    pub user_name: String,
    pub task_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Credential a browser WebSocket can carry: the same JWT (or suite session
/// token) the REST surface accepts, sent as `?token=`.
#[derive(Debug, Default, Deserialize)]
pub struct WsAuthQuery {
    #[serde(default)]
    pub token: String,
}

static PLAN_CHANNELS: std::sync::OnceLock<PlanChannels> = std::sync::OnceLock::new();
static PLAN_PRESENCE: std::sync::OnceLock<PlanPresence> = std::sync::OnceLock::new();
static PLAN_TYPING: std::sync::OnceLock<PlanTyping> = std::sync::OnceLock::new();

pub fn get_plan_channels() -> &'static PlanChannels {
    PLAN_CHANNELS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}
pub fn get_plan_presence() -> &'static PlanPresence {
    PLAN_PRESENCE.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}
pub fn get_plan_typing() -> &'static PlanTyping {
    PLAN_TYPING.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}

fn get_random_color() -> String {
    use rand::Rng;
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        "#BB8FCE", "#85C1E9", "#F1948A", "#82E0AA",
    ];
    let idx = rand::rng().random_range(0..colors.len());
    colors[idx].to_string()
}

/// `GET /ws/plan/:plan_id` — refuses the upgrade unless the caller presents a
/// **verified** credential *and* may read the plan.
///
/// Before #2136 a missing token produced a guest identity that still joined the
/// plan channel. There is no guest path now: a missing/invalid token answers
/// 401 with an `x-auth-reason` code, and a plan the caller cannot see answers
/// 404 (never 403, so a guessed id stays indistinguishable from an unused one).
pub async fn handle_plan_websocket(
    ws: WebSocketUpgrade,
    Path(plan_id): Path<String>,
    Query(q): Query<WsAuthQuery>,
    headers: HeaderMap,
) -> Response {
    // Identity first: an anonymous caller must not be able to tell a plan that
    // exists from one that does not by comparing 401 against 404.
    let caller = match caller_from_ws(Some(&q.token), &headers) {
        Ok(caller) => caller,
        Err(refusal) => {
            log::warn!("plan ws refused for {plan_id}: {refusal:?}");
            return refusal.into_response();
        }
    };
    let plan = match crate::load_plan(&plan_id).await {
        Some(plan) => plan,
        None => {
            log::warn!("plan ws refused for {plan_id}: plan_not_found");
            return PlanError::NotFound.into_response();
        }
    };
    match authorize(&plan, &caller, Access::Read) {
        Ok(()) => ws
            .on_upgrade(move |socket| handle_plan_connection(socket, plan_id, caller))
            .into_response(),
        Err(refusal) => {
            log::warn!("plan ws refused for {plan_id}: {refusal:?}");
            refusal.into_response()
        }
    }
}

/// Re-checks access right after the upgrade: a membership revocation (or a
/// deleted plan) between the handshake and the subscription closes the socket
/// with an explicit code instead of silently subscribing it.
async fn post_upgrade_refusal(plan_id: &str, caller: &Caller) -> Option<u16> {
    match crate::load_plan(plan_id).await {
        None => Some(PLAN_WS_CLOSE_UNAUTHORIZED),
        Some(plan) => match authorize(&plan, caller, Access::Read) {
            Ok(()) => None,
            Err(_) => Some(PLAN_WS_CLOSE_FORBIDDEN),
        },
    }
}

async fn handle_plan_connection(socket: WebSocket, plan_id: String, caller: Caller) {
    let (mut sender, mut receiver) = socket.split();

    if let Some(code) = post_upgrade_refusal(&plan_id, &caller).await {
        let reason = if code == PLAN_WS_CLOSE_UNAUTHORIZED {
            "plan_unavailable"
        } else {
            "plan_access_denied"
        };
        let _ = sender
            .send(Message::Close(Some(CloseFrame {
                code,
                reason: reason.into(),
            })))
            .await;
        return;
    }

    let channels = get_plan_channels();
    let broadcast_tx = {
        let mut w = channels.write().await;
        w.entry(plan_id.clone())
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    };
    let mut broadcast_rx = broadcast_tx.subscribe();

    // Only a verified identity reaches this point; there is no guest fallback.
    let user_id = caller.user_id.clone();
    let user_name = caller.display_name();
    let user_id_for_send = user_id.clone();
    let user_color = get_random_color();

    {
        let mut presence = get_plan_presence().write().await;
        let users = presence.entry(plan_id.clone()).or_default();
        users.push(PlanUserPresence {
            user_id: user_id.clone(),
            user_name: user_name.clone(),
            user_color: user_color.clone(),
            current_view: "kanban".to_string(),
            selected_task: None,
            last_active: chrono::Utc::now(),
            status: "active".to_string(),
        });
    }

    let join_msg = PlanMessage {
        msg_type: "join".to_string(),
        plan_id: plan_id.clone(),
        user_id: user_id.clone(),
        user_name: user_name.clone(),
        user_color: user_color.clone(),
        task_id: None,
        content: None,
        position: None,
        timestamp: chrono::Utc::now(),
    };
    let _ = broadcast_tx.send(join_msg);

    let bc_tx_clone = broadcast_tx.clone();
    let uid_clone = user_id.clone();
    let pid_clone = plan_id.clone();
    let uname_clone = user_name.clone();
    let ucolor_clone = user_color.clone();

    let receive = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(mut m) = serde_json::from_str::<PlanMessage>(&text) {
                        m.user_id = uid_clone.clone();
                        m.user_name = uname_clone.clone();
                        m.user_color = ucolor_clone.clone();
                        m.plan_id = pid_clone.clone();
                        m.timestamp = chrono::Utc::now();

                        match m.msg_type.as_str() {
                            "typing_start" => {
                                if let Some(tid) = m.task_id.clone() {
                                    let mut typing = get_plan_typing().write().await;
                                    let inds = typing.entry(pid_clone.clone()).or_default();
                                    inds.retain(|t| t.user_id != uid_clone);
                                    inds.push(PlanTypingIndicator {
                                        user_id: uid_clone.clone(),
                                        user_name: uname_clone.clone(),
                                        task_id: tid,
                                        started_at: chrono::Utc::now(),
                                    });
                                }
                            }
                            "typing_stop" => {
                                let mut typing = get_plan_typing().write().await;
                                if let Some(inds) = typing.get_mut(&pid_clone) {
                                    inds.retain(|t| t.user_id != uid_clone);
                                }
                            }
                            "view_change" => {
                                let mut presence = get_plan_presence().write().await;
                                if let Some(users) = presence.get_mut(&pid_clone) {
                                    for u in users.iter_mut() {
                                        if u.user_id == uid_clone {
                                            u.current_view = m
                                                .content
                                                .clone()
                                                .unwrap_or_else(|| "kanban".to_string());
                                            u.last_active = chrono::Utc::now();
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }

                        let _ = bc_tx_clone.send(m);
                    }
                }
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }
    });

    let send = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if msg.user_id == user_id_for_send {
                continue;
            }
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender
                    .send(Message::Text(json))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    });

    let _ = tokio::join!(receive, send);

    {
        let mut presence = get_plan_presence().write().await;
        if let Some(users) = presence.get_mut(&plan_id) {
            users.retain(|u| u.user_id != user_id);
        }
    }
    {
        let mut typing = get_plan_typing().write().await;
        if let Some(inds) = typing.get_mut(&plan_id) {
            inds.retain(|t| t.user_id != user_id);
        }
    }

    let leave_msg = PlanMessage {
        msg_type: "leave".to_string(),
        plan_id: plan_id.clone(),
        user_id: user_id.clone(),
        user_name,
        user_color,
        task_id: None,
        content: None,
        position: None,
        timestamp: chrono::Utc::now(),
    };
    let _ = broadcast_tx.send(leave_msg);
}

/// `GET /api/plan/:plan_id/collaborators`
pub async fn handle_get_plan_collaborators(
    headers: HeaderMap,
    Path(plan_id): Path<String>,
) -> Result<Json<serde_json::Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    visible_plan(&plan_id, &caller, Access::Read).await?;
    let presence = get_plan_presence().read().await;
    let users: Vec<PlanUserPresence> = presence.get(&plan_id).cloned().unwrap_or_default();
    Ok(Json(
        serde_json::json!({ "count": users.len(), "users": users }),
    ))
}

/// `GET /api/plan/:plan_id/presence`
pub async fn handle_get_plan_presence(
    headers: HeaderMap,
    Path(plan_id): Path<String>,
) -> Result<Json<serde_json::Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    visible_plan(&plan_id, &caller, Access::Read).await?;
    let presence = get_plan_presence().read().await;
    let users = presence.get(&plan_id).cloned().unwrap_or_default();
    Ok(Json(serde_json::json!({ "users": users })))
}

/// `GET /api/plan/:plan_id/typing`
pub async fn handle_get_plan_typing(
    headers: HeaderMap,
    Path(plan_id): Path<String>,
) -> Result<Json<serde_json::Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    visible_plan(&plan_id, &caller, Access::Read).await?;
    let typing = get_plan_typing().read().await;
    let indicators = typing.get(&plan_id).cloned().unwrap_or_default();
    let now = chrono::Utc::now();
    let active: Vec<PlanTypingIndicator> = indicators
        .into_iter()
        .filter(|t| (now - t.started_at).num_seconds() < 5)
        .collect();
    Ok(Json(serde_json::json!({ "typing": active })))
}

#[cfg(test)]
#[path = "collaboration_tests.rs"]
mod collaboration_tests;
