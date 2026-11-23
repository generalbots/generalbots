use super::*;

pub async fn handle_agent(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
    caller: SessionCaller,
    Json(payload): Json<AgentRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id, &caller).await?;
    let session_arc = Arc::new(Mutex::new(session));

    // #2883: the endpoint/key/model come from server configuration only and
    // the step budget is hard-capped; caller-supplied values are discarded.
    let llm_config = agent::LlmConfig::for_request(
        payload.llm_url.as_deref(),
        payload.llm_key.as_deref(),
        payload.llm_model.as_deref(),
    );
    let max_steps = agent::bounded_max_steps(payload.max_steps);

    match agent::run_agent_loop(&session_arc, &payload.goal, &llm_config, max_steps).await {
        Ok(steps) => Ok(Json(serde_json::json!({
            "steps": steps,
            "total_steps": steps.len()
        }))),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Agent failed: {e}"),
        )),
    }
}
pub async fn handle_agent_ws(
    ws: WebSocketUpgrade,
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
    caller: SessionCaller,
    Query(params): Query<WsAgentParams>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_agent_ws_stream(socket, sessions, id, caller, params))
}
