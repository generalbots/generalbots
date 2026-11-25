use super::*;

pub(super) async fn handle_agent_ws_stream(
    mut socket: WebSocket,
    sessions: SessionMap,
    id: String,
    caller: SessionCaller,
    params: WsAgentParams,
) {
    let session = match get_session(&sessions, &id, &caller).await {
        Ok(s) => s,
        Err(e) => {
            if let Err(send_err) = socket
                .send(Message::Text(
                    serde_json::json!({ "error": e.1 }).to_string(),
                ))
                .await
            {
                log::warn!("Failed to send session error to client: {send_err}");
            }
            return;
        }
    };

    let session_arc = Arc::new(Mutex::new(session));

    // #2883: ignore caller-supplied LLM endpoint/key/model and cap the step
    // budget. Only the server-configured trusted endpoint is ever used.
    let llm_config = agent::LlmConfig::for_request(
        params.llm_url.as_deref(),
        params.llm_key.as_deref(),
        params.llm_model.as_deref(),
    );
    let max_steps = agent::bounded_max_steps(params.max_steps);
    let mut step_count = 0;

    if let Err(e) = socket
        .send(Message::Text(
            serde_json::json!({ "type": "start", "goal": params.goal }).to_string(),
        ))
        .await
    {
        log::warn!("Failed to send agent start event: {e}");
    }

    while step_count < max_steps {
        step_count += 1;

        let observation = match agent::observe_page(&session_arc).await {
            Ok(o) => o,
            Err(e) => {
                if let Err(send_err) = socket
                    .send(Message::Text(
                        serde_json::json!({ "type": "error", "message": e.to_string() })
                            .to_string(),
                    ))
                    .await
                {
                    log::warn!("Failed to send observe error to client: {send_err}");
                }
                break;
            }
        };

        if let Err(e) = socket
            .send(Message::Text(
                serde_json::json!({
                    "type": "observation",
                    "step": step_count,
                    "url": observation.url,
                    "title": observation.title,
                    "visible_text": observation.visible_text.chars().take(2000).collect::<String>(),
                    "links_count": observation.links.len()
                })
                .to_string(),
            ))
            .await
        {
            log::warn!("Failed to send agent observation event: {e}");
        }

        let action =
            match agent::decide_next_action(&observation, &params.goal, None, &llm_config).await {
                Ok(a) => a,
                Err(e) => {
                    if let Err(send_err) = socket
                        .send(Message::Text(
                            serde_json::json!({ "type": "error", "message": e.to_string() })
                                .to_string(),
                        ))
                        .await
                    {
                        log::warn!("Failed to send decide error to client: {send_err}");
                    }
                    break;
                }
            };

        if let Err(e) = socket
            .send(Message::Text(
                serde_json::json!({
                    "type": "action",
                    "step": step_count,
                    "reasoning": action.reasoning,
                    "action": format!("{:?}", action.action)
                })
                .to_string(),
            ))
            .await
        {
            log::warn!("Failed to send agent action event: {e}");
        }

        match &action.action {
            agent::AgentAction::Done { summary } => {
                if let Err(e) = socket
                    .send(Message::Text(
                        serde_json::json!({
                            "type": "done",
                            "summary": summary,
                            "total_steps": step_count
                        })
                        .to_string(),
                    ))
                    .await
                {
                    log::warn!("Failed to send agent done event: {e}");
                }
                return;
            }
            _ => {
                if let Err(e) = agent::execute_action(&session_arc, &action.action).await {
                    if let Err(send_err) = socket
                        .send(Message::Text(
                            serde_json::json!({ "type": "action_error", "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e.to_string(), "botbrowser::api::handle_agent_ws_stream", None) }).to_string(),
                        ))
                        .await
                    {
                        log::warn!("Failed to send action error to client: {send_err}");
                    }
                }
            }
        }
    }

    if let Err(e) = socket
        .send(Message::Text(
            serde_json::json!({
                "type": "done",
                "summary": "Max steps reached",
                "total_steps": step_count
            })
            .to_string(),
        ))
        .await
    {
        log::warn!("Failed to send agent max-steps done event: {e}");
    }
}
pub async fn handle_close(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
    caller: SessionCaller,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id, &caller).await?;
    session.close().await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Close failed: {e}"),
        )
    })?;
    let mut map = sessions.lock().await;
    map.remove(&id);
    log::info!("Browser session closed: {id}");
    Ok(Json(serde_json::json!({ "status": "closed", "id": id })))
}
