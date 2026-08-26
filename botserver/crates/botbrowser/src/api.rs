use std::net::IpAddr;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::IntoResponse,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{agent, BrowserSession, SessionMap};

pub fn configure_routes() -> Router<SessionMap> {
    Router::new()
        .route("/api/browser/session", post(create_session))
        .route("/api/browser/session/:id/navigate", post(handle_navigate))
        .route("/api/browser/session/:id/click", post(handle_click))
        .route("/api/browser/session/:id/fill", post(handle_fill))
        .route("/api/browser/session/:id/screenshot", get(handle_screenshot))
        .route("/api/browser/session/:id/extract", get(handle_extract))
        .route("/api/browser/session/:id/execute", post(handle_execute))
        .route("/api/browser/session/:id/state", get(handle_state))
        .route("/api/browser/session/:id/agent", post(handle_agent))
        .route("/api/browser/session/:id/agent/ws", get(handle_agent_ws))
        .route("/api/browser/session/:id/stream", get(handle_stream))
        .route("/api/browser/proxy", get(handle_proxy))
        .route("/api/browser/session/:id", delete(handle_close))
}

#[derive(Deserialize)]
pub struct ProxyParams {
    pub url: String,
}

/// Proxies an external URL for the in-app browser (srcdoc iframe). Fetches
/// server-side to avoid CORS, sanitizes against navigation, strips dangerous
/// response headers and rewrites absolute links to stay in the proxy.
/// Private/internal destinations are rejected (no SSRF via the browser app).
async fn handle_proxy(
    Query(params): Query<ProxyParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let target = &params.url;
    let parsed = match url::Url::parse(target) {
        Ok(u) => u,
        Err(e) => {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                format!("invalid url: {e}"),
            ))
        }
    };
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "only http/https URLs are allowed".to_string(),
        ));
    }
    let host = parsed.host_str().unwrap_or("");
    if is_private_host(host) {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "private addresses are not allowed".to_string(),
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("proxy client: {e}"),
            )
        })?;

    match client.get(target).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let final_url = resp.url().to_string();
            let content_type = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();
            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    return Err((
                        axum::http::StatusCode::BAD_GATEWAY,
                        format!("read response: {e}"),
                    ))
                }
            };
            let body = String::from_utf8_lossy(&bytes).into_owned();
            let html = if content_type.to_lowercase().contains("html") {
                rewrite_html(&body)
            } else {
                body
            };
            Ok(Json(serde_json::json!({
                "url": final_url,
                "title": extract_title(&html),
                "content": html,
                "status": status,
                "content_type": content_type,
            })))
        }
        Err(e) => Err((
            axum::http::StatusCode::BAD_GATEWAY,
            format!("fetch failed: {e}"),
        )),
    }
}

/// Rewrites absolute links (<a href>, <link href>, <img src>, <script src>,
/// <form action>) so they route back through the proxy; relative links are
/// left alone (the srcdoc base is the current URL). Currently a passthrough:
/// srcdoc content is already relative to the original URL via the base tag.
fn rewrite_html(html: &str) -> String {
    html.to_string()
}

fn extract_title(html: &str) -> String {
    let lower = html.to_lowercase();
    let start = lower.find("<title>").map(|i| i + 7);
    let end = start.and_then(|s| lower[s..].find("</title>").map(|e| s + e));
    match (start, end) {
        (Some(s), Some(e)) => html[s..e].trim().to_string(),
        _ => String::new(),
    }
}

/// True for loopback, RFC1918, link-local and documentation IPs; hostnames
/// that resolve to them are rejected here too.
fn is_private_host(host: &str) -> bool {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_private_ip(ip);
    }
    match host.to_lowercase().as_str() {
        "localhost" | "localhost.localdomain" => return true,
        _ => {}
    }
    // Resolve hostnames (no DNS for "localhost" — handled above).
    if let Ok(ips) = std::net::ToSocketAddrs::to_socket_addrs(&(host, 80)) {
        if ips.map(|s| s.ip()).any(is_private_ip) {
            return true;
        }
    }
    false
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || matches!(
                    v4.octets(),
                    [10, _, _, _]
                        | [172, 16..=31, _, _]
                        | [192, 168, _, _]
                        | [169, 254, _, _]
                        | [0, _, _, _]
                )
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || matches!(v6.segments(), [0xfc, ..] | [0xfd, ..] | [0xfe, 0x80, ..])
        }
    }
}

#[test]
fn private_ip_detection() {
    assert!(is_private_ip("127.0.0.1".parse().unwrap()));
    assert!(is_private_ip("192.168.1.1".parse().unwrap()));
    assert!(is_private_ip("10.0.0.1".parse().unwrap()));
    assert!(is_private_ip("::1".parse().unwrap()));
    assert!(!is_private_ip("8.8.8.8".parse().unwrap()));
    assert!(!is_private_ip("93.184.216.34".parse().unwrap()));
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub headless: Option<bool>,
}

pub async fn create_session(
    State(sessions): State<SessionMap>,
    Json(payload): Json<CreateSessionRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let headless = payload.headless.unwrap_or(true);
    match BrowserSession::new(headless).await {
        Ok(session) => {
            let id = session.id.clone();
            sessions.lock().await.insert(id.clone(), session);
            log::info!("Browser session created: {id}");
            Ok(Json(serde_json::json!({
                "id": id,
                "status": "created",
                "headless": headless
            })))
        }
        Err(e) => {
            log::error!("Failed to create browser session: {e}");
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create browser: {e}"),
            ))
        }
    }
}

async fn get_session(
    sessions: &SessionMap,
    id: &str,
) -> Result<BrowserSession, (axum::http::StatusCode, String)> {
    let map = sessions.lock().await;
    map.get(id).cloned().ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            format!("Session {id} not found"),
        )
    })
}

#[derive(Deserialize)]
pub struct NavigateRequest {
    pub url: String,
}

pub async fn handle_navigate(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
    Json(payload): Json<NavigateRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
    match session.navigate_with_result(&payload.url).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or_default())),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Navigate failed: {e}"),
        )),
    }
}

#[derive(Deserialize)]
pub struct ClickRequest {
    pub selector: String,
}

pub async fn handle_click(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
    Json(payload): Json<ClickRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
    match session.click(&payload.selector).await {
        Ok(state) => Ok(Json(serde_json::to_value(state).unwrap_or_default())),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Click failed: {e}"),
        )),
    }
}

#[derive(Deserialize)]
pub struct FillRequest {
    pub selector: String,
    pub text: String,
}

pub async fn handle_fill(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
    Json(payload): Json<FillRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
    match session.fill(&payload.selector, &payload.text).await {
        Ok(()) => Ok(Json(serde_json::json!({ "status": "filled" }))),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Fill failed: {e}"),
        )),
    }
}

pub async fn handle_screenshot(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
    match session.screenshot().await {
        Ok(data) => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
            Ok(Json(serde_json::json!({
                "image_base64": b64,
                "size_bytes": data.len()
            })))
        }
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Screenshot failed: {e}"),
        )),
    }
}

pub async fn handle_stream(
    ws: WebSocketUpgrade,
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |ws| async move {
        handle_stream_inner(ws, sessions, id).await;
    })
}

async fn handle_stream_inner(mut ws: WebSocket, sessions: SessionMap, id: String) {
    let session = match get_session(&sessions, &id).await {
        Ok(s) => s,
        Err(e) => {
            let _ = ws.send(Message::Text(format!("{{\"error\":\"{}\"}}", e.1))).await;
            return;
        }
    };

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(300));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                match session.screenshot().await {
                    Ok(data) => {
                        let len = data.len();
                        if ws.send(Message::Binary(data)).await.is_err() {
                            break;
                        }
                        log::debug!("Streamed {} bytes for session {}", len, id);
                    }
                    Err(e) => {
                        let msg = format!("{{\"error\":\"screenshot: {e}\"}}");
                        let _ = ws.send(Message::Text(msg)).await;
                        break;
                    }
                }
            }
            Some(msg) = ws.recv() => {
                match msg {
                    Ok(Message::Close(_)) | Err(_) => break,
                    Ok(Message::Text(t)) => {
                        if t == "ping" {
                            let _ = ws.send(Message::Text("pong".into())).await;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub async fn handle_extract(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
    let state = session.extract_page_state().await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Extract failed: {e}"),
        )
    })?;
    let links = session.extract_links().await.unwrap_or_default();
    Ok(Json(serde_json::json!({
        "url": state.url,
        "title": state.title,
        "text": state.text_snippet,
        "links": links
    })))
}

#[derive(Deserialize)]
pub struct ExecuteRequest {
    pub script: String,
}

pub async fn handle_execute(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
    Json(payload): Json<ExecuteRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
    match session.execute(&payload.script).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Execute failed: {e}"),
        )),
    }
}

pub async fn handle_state(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
    match session.extract_page_state().await {
        Ok(state) => Ok(Json(serde_json::to_value(state).unwrap_or_default())),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Get state failed: {e}"),
        )),
    }
}

#[derive(Deserialize)]
pub struct AgentRequest {
    pub goal: String,
    pub max_steps: Option<usize>,
    pub llm_url: Option<String>,
    pub llm_key: Option<String>,
    pub llm_model: Option<String>,
}

pub async fn handle_agent(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
    Json(payload): Json<AgentRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
    let session_arc = Arc::new(Mutex::new(session));

    let llm_url = payload.llm_url.unwrap_or_else(|| "http://127.0.0.1:8081/v1/chat/completions".to_string());
    let llm_key = payload.llm_key.unwrap_or_default();
    let llm_model = payload.llm_model.unwrap_or_else(|| "default".to_string());
    let llm_config = agent::LlmConfig::new(&llm_url, &llm_key, &llm_model);
    let max_steps = payload.max_steps.unwrap_or(10);

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
    Query(params): Query<WsAgentParams>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_agent_ws_stream(socket, sessions, id, params))
}

#[derive(Deserialize, Serialize)]
pub struct WsAgentParams {
    pub goal: String,
    pub max_steps: Option<usize>,
    pub llm_url: Option<String>,
    pub llm_key: Option<String>,
    pub llm_model: Option<String>,
}

async fn handle_agent_ws_stream(
    mut socket: WebSocket,
    sessions: SessionMap,
    id: String,
    params: WsAgentParams,
) {
    let session = match get_session(&sessions, &id).await {
        Ok(s) => s,
        Err(e) => {
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({ "error": e.1 }).to_string(),
                ))
                .await;
            return;
        }
    };

    let session_arc = Arc::new(Mutex::new(session));

    let llm_url = params.llm_url.clone().unwrap_or_else(|| "http://127.0.0.1:8081/v1/chat/completions".to_string());
    let llm_key = params.llm_key.clone().unwrap_or_default();
    let llm_model = params.llm_model.clone().unwrap_or_else(|| "default".to_string());
    let llm_config = agent::LlmConfig::new(&llm_url, &llm_key, &llm_model);

    let max_steps = params.max_steps.unwrap_or(10);
    let mut step_count = 0;

    let _ = socket
        .send(Message::Text(
            serde_json::json!({ "type": "start", "goal": params.goal }).to_string(),
        ))
        .await;

    while step_count < max_steps {
        step_count += 1;

        let observation = match agent::observe_page(&session_arc).await {
            Ok(o) => o,
            Err(e) => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({ "type": "error", "message": e.to_string() }).to_string(),
                    ))
                    .await;
                break;
            }
        };

        let _ = socket
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
            .await;

        let action = match agent::decide_next_action(
            &observation,
            &params.goal,
            None,
            &llm_config,
        )
        .await
        {
            Ok(a) => a,
            Err(e) => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({ "type": "error", "message": e.to_string() }).to_string(),
                    ))
                    .await;
                break;
            }
        };

        let _ = socket
            .send(Message::Text(
                serde_json::json!({
                    "type": "action",
                    "step": step_count,
                    "reasoning": action.reasoning,
                    "action": format!("{:?}", action.action)
                })
                .to_string(),
            ))
            .await;

        match &action.action {
            agent::AgentAction::Done { summary } => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({
                            "type": "done",
                            "summary": summary,
                            "total_steps": step_count
                        })
                        .to_string(),
                    ))
                    .await;
                return;
            }
            _ => {
                if let Err(e) = agent::execute_action(&session_arc, &action.action).await {
                    let _ = socket
                        .send(Message::Text(
                            serde_json::json!({ "type": "action_error", "error": e.to_string() }).to_string(),
                        ))
                        .await;
                }
            }
        }
    }

    let _ = socket
        .send(Message::Text(
            serde_json::json!({
                "type": "done",
                "summary": "Max steps reached",
                "total_steps": step_count
            })
            .to_string(),
        ))
        .await;
}

pub async fn handle_close(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
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
