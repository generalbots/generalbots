use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use botcore::shared::state::AppState;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct WsQuery {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub bot_name: Option<String>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<WsQuery>,
) -> impl IntoResponse {
    let session_id = params.session_id.and_then(|s| Uuid::parse_str(&s).ok()).unwrap_or_else(Uuid::new_v4);
    let user_id = params.user_id.and_then(|s| Uuid::parse_str(&s).ok()).unwrap_or_else(Uuid::new_v4);
    let bot_name = params.bot_name.unwrap_or_else(|| "default".to_string());
    let bot_uuid = lookup_bot_id(&state, &bot_name);
    info!("WebSocket: bot={}, session={}, user={}", bot_name, session_id, user_id);
    ws.on_upgrade(move |socket| handle_ws(socket, state, session_id, user_id, bot_uuid, bot_name))
}

pub async fn websocket_handler_with_bot(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(bot_name): axum::extract::Path<String>,
    Query(mut params): Query<WsQuery>,
) -> impl IntoResponse {
    if params.bot_name.is_none() && !bot_name.is_empty() {
        params.bot_name = Some(bot_name);
    }
    websocket_handler(ws, State(state), Query(params)).await
}

fn lookup_bot_id(_state: &Arc<AppState>, _bot_name: &str) -> Uuid {
    let mut conn = match _state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            warn!("DB conn: {}", e);
            return Uuid::nil();
        }
    };

    use botcorebot::schema::bots::dsl::{bots, id, name};
    use diesel::prelude::*;

    if let Ok(uuid) = Uuid::parse_str(_bot_name) {
        bots.filter(id.eq(uuid))
            .select(id)
            .first::<Uuid>(&mut conn)
            .unwrap_or(Uuid::nil())
    } else {
        bots.filter(name.eq(_bot_name))
            .select(id)
            .first::<Uuid>(&mut conn)
            .unwrap_or_else(|_| {
                warn!("Bot not found: {}", _bot_name);
                Uuid::nil()
            })
    }
}

async fn handle_ws(
    socket: WebSocket,
    state: Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    bot_uuid: Uuid,
    bot_name: String,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<botlib::models::BotResponse>(100);
    {
        let mut channels = state.response_channels.lock().await;
        channels.insert(session_id.to_string(), tx);
    }
    info!("WebSocket connected: bot={}, session={}", bot_name, session_id);

    let welcome = serde_json::json!({
        "type": "connected", "session_id": session_id, "user_id": user_id,
        "bot_id": bot_uuid, "message": "Connected to bot server", "tools": []
    });
    let _ = ws_sender.send(Message::Text(welcome.to_string().into())).await;

    // Run start.bas
    {
        let s = state.clone();
        let sid = session_id;
        let uid = user_id;
        let bid = bot_uuid;
        let bn = bot_name.clone();
        tokio::task::spawn_blocking(move || {
            let work_path = botcore::shared::utils::get_work_path();
            let ast_path = format!("{}/{}.gbai/{}.gbdialog/start.ast", work_path, bn, bn);
            match std::fs::read_to_string(&ast_path) {
                Ok(content) => {
                    let session = botlib::models::UserSession {
                        id: sid, user_id: uid, bot_id: bid,
                        title: String::new(),
                        context_data: serde_json::Value::Null,
                        current_tool: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    };
                    let mut svc = crate::basic::ScriptService::new(Arc::clone(&s), session);
                    match svc.run(&content) {
                        Ok(_) => info!("start.bas OK"),
                        Err(e) => error!("start.bas error: {}", e),
                    }
                }
                Err(e) => warn!("start.bas AST not found at {}: {}", ast_path, e),
            }
        });
    }
    // Message loop
    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        info!("WS msg: {}", text);
                        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                        let user_text = parsed.get("text").and_then(|v| v.as_str()).unwrap_or("");

                        // Try to deliver to a waiting HEAR keyword first
                        let runtime: Arc<dyn botbasic_types::BasicRuntime> =
                            Arc::new(crate::basic::AppStateBasicRuntime(state.clone()));
                        let delivered = crate::basic::keywords::hearing::deliver_hear_input(
                            &runtime,
                            session_id,
                            user_text.to_string(),
                        );

                        if !delivered {
                            // Try LLM first, fallback to suggestions
                            let reply = match state.llm_provider {
                                Some(ref llm) => {
                                    let prompt = format!("Você é o assistente virtual Salesianos. O usuário disse: \"{}\". Responda de forma breve e útil, oferecendo as opções: Cartas, Procedimentos, Ramais ou Todos.", user_text);
                                    match llm.generate_simple(&prompt).await {
                                        Ok(resp) => resp,
                                        Err(e) => {
                                            warn!("LLM generate_simple failed: {}", e);
                                            if bot_name.eq_ignore_ascii_case("salesianos") {
                                                "Escolha uma opção: Cartas, Procedimentos, Ramais ou Todos.".to_string()
                                            } else {
                                                "Como posso ajudar?".to_string()
                                            }
                                        }
                                    }
                                }
                                None => {
                                    info!("No LLM provider available for user message");
                                    if bot_name.eq_ignore_ascii_case("salesianos") {
                                        "Escolha uma opção: Cartas, Procedimentos, Ramais ou Todos.".to_string()
                                    } else {
                                        "Como posso ajudar?".to_string()
                                    }
                                }
                            };
                            let resp = serde_json::json!({
                                "bot_id": bot_uuid.to_string(),
                                "user_id": user_id.to_string(),
                                "session_id": session_id.to_string(),
                                "channel": "web",
                                "content": reply,
                                "message_type": 2,
                                "is_complete": true,
                                "suggestions": [],
                                "switchers": [],
                                "context_length": 0,
                                "context_max_length": 0,
                            });
                            let _ = ws_sender.send(Message::Text(resp.to_string().into())).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => { error!("WS err: {}", e); break; }
                    _ => {}
                }
            }
            Some(response) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&response) {
                    let _ = ws_sender.send(Message::Text(json.into())).await;
                }
            }
            else => break,
        }
    }

    {
        let mut channels = state.response_channels.lock().await;
        channels.remove(&session_id.to_string());
    }
    info!("WS disconnected: session={}", session_id);
}
