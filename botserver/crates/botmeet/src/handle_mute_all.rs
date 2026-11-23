use super::*;
use axum::extract::{Extension, Query};
use axum::http::HeaderMap;
use botsecurity::auth::AuthenticatedUser;

pub async fn handle_mute_all(Form(payload): Form<MuteAllRequest>) -> impl IntoResponse {
    let room = payload.room_id.unwrap_or_else(|| "default".to_string());
    info!("Muting all participants in room {room}");
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "muted", "room_id": room })),
    )
}
pub async fn handle_voice_toggle() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "toggled" }))
}
pub async fn start_transcription(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> impl IntoResponse {
    let transcription_service = Arc::new(DefaultTranscriptionService);
    let meeting_service = MeetingService::new(state.clone(), transcription_service);

    match meeting_service.start_transcription(&room_id).await {
        Ok(()) => {
            info!("Started transcription for room {room_id}");
            (
                StatusCode::OK,
                Json(serde_json::json!({"status": "transcription_started"})),
            )
        }
        Err(e) => {
            error!("Failed to start transcription for room {room_id}: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({"error": botsecurity::error_sanitizer::log_and_sanitize_json(&e.to_string(), "botmeet::handle_mute_all", None)}),
                ),
            )
        }
    }
}
fn request_error(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({ "error": message.to_string() })),
    )
}

/// `#2736`: issue an expiring, signed token from the verified identity. The
/// room is accepted only after a membership check; the user always comes from
/// `AuthenticatedUser`, never from the request body.
pub async fn get_meeting_token(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<GetTokenRequest>,
) -> impl IntoResponse {
    if user.user_id.is_nil() {
        return request_error(StatusCode::UNAUTHORIZED, "authentication required");
    }
    let room = payload.room_id.trim();
    if room.is_empty() {
        return request_error(StatusCode::BAD_REQUEST, "room_id is required");
    }

    let allowed = match room_persistence::user_can_access_room(&state, room, user.user_id).await {
        Ok(allowed) => allowed,
        Err(e) => {
            error!("Room membership check failed: {e}");
            let sanitized = botsecurity::error_sanitizer::log_and_sanitize_json(
                &e,
                "botmeet::handle_mute_all",
                None,
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(sanitized));
        }
    };
    if !allowed {
        return request_error(StatusCode::FORBIDDEN, "not a member of this room");
    }

    let user_id = user.user_id.to_string();
    match token::issue_meeting_token(room, &user_id) {
        Ok(issued) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "token": issued.token,
                "room_id": issued.claims.room,
                "user_id": issued.claims.user,
                "expires_at": issued.claims.exp,
            })),
        ),
        Err(e) => {
            error!("Failed to sign meeting token: {e}");
            let sanitized = botsecurity::error_sanitizer::log_and_sanitize_json(
                &e.to_string(),
                "botmeet::handle_mute_all",
                None,
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(sanitized))
        }
    }
}
pub async fn send_meeting_invites(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<SendInvitesRequest>,
) -> impl IntoResponse {
    info!("Sending meeting invites for room {}", payload.room_id);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "invites_sent",
            "recipients": payload.emails
        })),
    )
}
pub async fn meeting_websocket(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_meeting_socket(socket, state))
}
pub(super) async fn handle_meeting_socket(
    socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
) {
    info!("Meeting WebSocket connection established");
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                info!("Meeting message received: {text}");
                if sender
                    .send(axum::extract::ws::Message::Text(format!("Echo: {text}")))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => break,
            Err(e) => {
                error!("WebSocket error: {e}");
                break;
            }
            _ => {}
        }
    }

    drop(state);
}
pub async fn signaling_websocket(
    ws: axum::extract::ws::WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let rooms = state
        .extensions
        .get::<SignalingRooms>()
        .await
        .map(|arc| (*arc).clone())
        .unwrap_or_else(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())));
    ws.on_upgrade(move |socket| signaling::handle_signaling_socket(socket, room_id, state, rooms))
}
/// `#2736`: TURN credentials are minted only for an authenticated user who
/// belongs to the requested room, and are bound to `(room, user, short exp)`.
/// With no verified room we return STUN-only defaults — never a shared-secret
/// credential that could be replayed in another room.
pub async fn turn_credentials(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<TurnCredentialsQuery>,
) -> impl IntoResponse {
    let config = turn::TurnConfig::from_env();
    let fallback = || turn::get_turn_credentials_for(&config, "", "", 0, 0);
    if user.user_id.is_nil() {
        return Json(fallback());
    }
    let room = match query.room_id.as_deref().map(str::trim) {
        Some(room) if !room.is_empty() => room,
        _ => return Json(fallback()),
    };
    match room_persistence::user_can_access_room(&state, room, user.user_id).await {
        Ok(true) => Json(turn::get_turn_credentials_for(
            &config,
            room,
            &user.user_id.to_string(),
            turn::TURN_CREDENTIAL_TTL_SECS,
            chrono::Utc::now().timestamp(),
        )),
        Ok(false) => Json(fallback()),
        Err(e) => {
            error!("TURN room membership check failed: {e}");
            Json(fallback())
        }
    }
}
pub async fn schedule_meeting_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<ScheduleMeetingRequest>,
) -> impl IntoResponse {
    match room_persistence::schedule_meeting(
        &state,
        request,
        crate::dashboard::caller_scope(&headers),
    )
    .await
    {
        Ok(room) => {
            info!("Scheduled meeting: {} ({})", room.title, room.id);
            (StatusCode::OK, Json(serde_json::json!(room)))
        }
        Err(e) => {
            error!("Failed to schedule meeting: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({"error": botsecurity::error_sanitizer::log_and_sanitize_json(&e, "botmeet::handle_mute_all", None)}),
                ),
            )
        }
    }
}
pub async fn dashboard_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match room_persistence::get_dashboard_stats(&state, crate::dashboard::caller_scope(&headers))
        .await
    {
        Ok(stats) => (StatusCode::OK, Json(serde_json::json!(stats))),
        Err(e) => {
            error!("Failed to get dashboard stats: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({"error": botsecurity::error_sanitizer::log_and_sanitize_json(&e, "botmeet::handle_mute_all", None)}),
                ),
            )
        }
    }
}
pub async fn list_meetings_htmx(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Html<String> {
    let query = room_persistence::DashboardQuery {
        status: None,
        limit: Some(20),
        offset: Some(0),
    };

    match room_persistence::list_meetings(&state, query, crate::dashboard::caller_scope(&headers))
        .await
    {
        Ok(meetings) => {
            if meetings.is_empty() {
                return Html(
                    r#"<div class="empty-state"><p>No meetings found</p></div>"#.to_string(),
                );
            }
            let mut html = String::new();
            for m in &meetings {
                let status_class = match m.status.as_str() {
                    "live" => "status-live",
                    "scheduled" => "status-scheduled",
                    _ => "status-ended",
                };
                let title = crate::dashboard::escape_html(&m.title);
                let status = crate::dashboard::escape_html(&m.status);
                html.push_str(&format!(
                    r#"<div class="meeting-card {status_class}" data-id="{id}">
                        <h4>{title}</h4>
                        <p>Status: {status} | Participants: {participants}</p>
                    </div>"#,
                    id = m.id,
                    title = title,
                    status = status,
                    participants = m.participant_count,
                ));
            }
            Html(html)
        }
        Err(e) => Html(format!("<div class='error'>Error: {e}</div>")),
    }
}
pub async fn all_participants(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html(
        r##"<div class="empty-state">
        <p>No participants</p>
    </div>"##
            .to_string(),
    )
}
pub async fn recent_meetings(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html(
        r##"<div class="empty-state">
        <div class="empty-icon">📋</div>
        <p>No recent meetings</p>
    </div>"##
            .to_string(),
    )
}
pub async fn scheduled_meetings(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html(
        r##"<div class="empty-state">
        <div class="empty-icon">📅</div>
        <p>No scheduled meetings</p>
    </div>"##
            .to_string(),
    )
}
