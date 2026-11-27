//! HTTP surface for meeting recording.
//!
//! Exposes the `RecordingService` (recording.rs) through REST endpoints:
//! start/stop/pause/resume for a room, plus list/get/delete and file
//! streaming for completed recordings.
//!
//! #2732: every handler derives the acting identity from the verified
//! `AuthenticatedUser` extension. The legacy `user_id` JSON field is never
//! read, so a client can no longer claim a host's UUID. Controls require a
//! host role in `meeting_participants`; list/read/stream require membership;
//! delete requires the host role in the recording's own room. Org/branch scope
//! is resolved from signed claims only, never from the request.

use axum::{
    extract::{Extension, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;
use botsecurity::auth::AuthenticatedUser;

use crate::recording::{
    RecordingConfig, RecordingError, RecordingService, StartRecordingRequest, StopRecordingRequest,
};
use crate::recording_access::{is_host_role, room_in_scope, CallerScope, RoomScope};
use crate::webinar_types::{RecordingStatus, WebinarRecording};

type RouteError = (StatusCode, Json<serde_json::Value>);

fn recording_error(e: RecordingError) -> RouteError {
    let (status, message) = match &e {
        RecordingError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
        RecordingError::AlreadyRecording | RecordingError::AlreadyExists => {
            (StatusCode::CONFLICT, e.to_string())
        }
        RecordingError::Unauthorized => (StatusCode::FORBIDDEN, e.to_string()),
        RecordingError::InvalidState(_) | RecordingError::TranscriptionNotReady => {
            (StatusCode::CONFLICT, e.to_string())
        }
        _ => {
            log::error!("Meet recording error: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    };
    (status, Json(serde_json::json!({ "error": message })))
}

fn server_error(e: &str) -> RouteError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": botsecurity::error_sanitizer::log_and_sanitize_json(
                e,
                "botmeet::recording_routes",
                None,
            )
        })),
    )
}

fn unauthorized() -> RouteError {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "authentication required" })),
    )
}

fn forbidden(message: &str) -> RouteError {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": message })),
    )
}

fn not_found(message: &str) -> RouteError {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": message })),
    )
}

fn verified_user_id(user: &AuthenticatedUser) -> Result<Uuid, RouteError> {
    if user.user_id.is_nil() {
        return Err(unauthorized());
    }
    Ok(user.user_id)
}

/// Shared `RecordingService` instance.
///
/// The service keeps in-memory active sessions (start → pause/stop must hit
/// the same instance), so a single process-wide instance is created lazily
/// from the first request's pool. All requests share it; the DB row is the
/// source of truth for the recording lifecycle.
static RECORDING_SERVICE: std::sync::OnceLock<RecordingService> = std::sync::OnceLock::new();

fn service_for(state: &Arc<AppState>) -> &'static RecordingService {
    RECORDING_SERVICE
        .get_or_init(|| RecordingService::new(state.conn.clone(), RecordingConfig::default()))
}

/// Resolves the participant role for a room; returns `None` when the user is
/// not (or no longer) a member of the room.
async fn participant_role(
    state: &Arc<AppState>,
    target_room: Uuid,
    target_user: Uuid,
) -> Result<Option<String>, RouteError> {
    use botschema::meeting_participants::dsl::{
        meeting_participants, role as role_col, room_id, user_id,
    };
    use diesel::prelude::*;

    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to acquire connection: {e}"))?;
        meeting_participants
            .filter(room_id.eq(target_room))
            .filter(user_id.eq(target_user))
            .select(role_col)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|e| format!("Failed to query participant role: {e}"))
    })
    .await;

    match result {
        Ok(Ok(role)) => Ok(role),
        Ok(Err(e)) => Err(server_error(&e)),
        Err(e) => Err(server_error(&e.to_string())),
    }
}

async fn load_room_scope(
    state: &Arc<AppState>,
    target_room: Uuid,
) -> Result<Option<RoomScope>, RouteError> {
    use botschema::meeting_rooms::dsl::{branch_id, id, meeting_rooms, org_id};
    use diesel::prelude::*;

    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to acquire connection: {e}"))?;
        meeting_rooms
            .filter(id.eq(target_room))
            .select((id, org_id, branch_id))
            .first::<(Uuid, Uuid, Uuid)>(&mut conn)
            .optional()
            .map_err(|e| format!("Failed to query room scope: {e}"))
    })
    .await;

    let row = match result {
        Ok(Ok(row)) => row,
        Ok(Err(e)) => return Err(server_error(&e)),
        Err(e) => return Err(server_error(&e.to_string())),
    };
    // The DSL column imports above shadow these names in pattern position;
    // bind distinct locals so they are not parsed as unit-struct patterns.
    Ok(row.map(|(room_id, row_org_id, row_branch_id)| RoomScope {
        room_id,
        org_id: row_org_id,
        branch_id: row_branch_id,
    }))
}

/// Verifies the caller is a host/co-host of the room; returns 403 otherwise.
async fn require_host(
    state: &Arc<AppState>,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<(), RouteError> {
    match participant_role(state, room_id, user_id).await? {
        Some(role) if is_host_role(Some(role.as_str())) => Ok(()),
        Some(_) => Err(forbidden("Only the host or co-host can control recording")),
        None => Err(forbidden("Not a participant of this meeting")),
    }
}

async fn require_room_scope(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    user: &AuthenticatedUser,
    room_id: Uuid,
) -> Result<(), RouteError> {
    let caller = CallerScope::from_request(user, headers);
    match load_room_scope(state, room_id).await? {
        Some(room) if room_in_scope(&caller, &room) => Ok(()),
        Some(_) => Err(forbidden("Meeting room is outside your organization")),
        None => Err(not_found("Meeting room not found")),
    }
}

async fn require_membership(
    state: &Arc<AppState>,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<(), RouteError> {
    match participant_role(state, room_id, user_id).await? {
        Some(_) => Ok(()),
        None => Err(forbidden("Not a participant of this meeting")),
    }
}

async fn require_room_member(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    user: &AuthenticatedUser,
    room_id: Uuid,
) -> Result<(), RouteError> {
    let user_id = verified_user_id(user)?;
    require_room_scope(state, headers, user, room_id).await?;
    require_membership(state, room_id, user_id).await
}

/// `WebinarRecording.webinar_id` is mapped from `meeting_recordings.room_id`,
/// so it is the id of the room that owns the recording.
async fn recording_room(state: &Arc<AppState>, recording_id: Uuid) -> Result<Uuid, RouteError> {
    service_for(state)
        .get_recording(recording_id)
        .await
        .map(|recording| recording.webinar_id)
        .map_err(recording_error)
}

/// A host of room A must not be able to control a recording owned by room B.
async fn require_recording_in_room(
    state: &Arc<AppState>,
    room_id: Uuid,
    recording_id: Uuid,
) -> Result<(), RouteError> {
    let owner = recording_room(state, recording_id).await?;
    if owner != room_id {
        return Err(forbidden("Recording belongs to another meeting"));
    }
    Ok(())
}

/// JSON control payload. The legacy `user_id` field, when present, is accepted
/// and dropped by the flattened payload: identity comes only from the verified
/// `AuthenticatedUser` extension (#2732).
#[derive(Debug, Deserialize)]
pub struct RecordingControlRequest<T> {
    #[serde(flatten)]
    pub payload: T,
}

/// `POST /api/meet/rooms/{id}/recording/start`
pub async fn start_recording(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
    Json(body): Json<RecordingControlRequest<StartRecordingRequest>>,
) -> Result<Json<WebinarRecording>, RouteError> {
    let caller = verified_user_id(&user)?;
    require_room_scope(&state, &headers, &user, room_id).await?;
    require_host(&state, room_id, caller).await?;

    let mut request = body.payload;
    request.webinar_id = room_id;
    service_for(&state)
        .start_recording(request)
        .await
        .map(Json)
        .map_err(recording_error)
}

/// `POST /api/meet/rooms/{id}/recording/stop`
pub async fn stop_recording(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<RecordingControlRequest<StopRecordingRequest>>,
) -> Result<Json<WebinarRecording>, RouteError> {
    let caller = verified_user_id(&user)?;
    require_recording_in_room(&state, room_id, body.payload.recording_id).await?;
    require_host(&state, room_id, caller).await?;

    service_for(&state)
        .stop_recording(body.payload)
        .await
        .map(Json)
        .map_err(recording_error)
}

/// `POST /api/meet/rooms/{id}/recording/{recording_id}/pause`
pub async fn pause_recording(
    State(state): State<Arc<AppState>>,
    Path((room_id, recording_id)): Path<(Uuid, Uuid)>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<serde_json::Value>, RouteError> {
    let caller = verified_user_id(&user)?;
    require_recording_in_room(&state, room_id, recording_id).await?;
    require_host(&state, room_id, caller).await?;

    service_for(&state)
        .pause_recording(recording_id)
        .await
        .map_err(recording_error)?;
    Ok(Json(
        serde_json::json!({ "success": true, "message": "Recording paused" }),
    ))
}

/// `POST /api/meet/rooms/{id}/recording/{recording_id}/resume`
pub async fn resume_recording(
    State(state): State<Arc<AppState>>,
    Path((room_id, recording_id)): Path<(Uuid, Uuid)>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<serde_json::Value>, RouteError> {
    let caller = verified_user_id(&user)?;
    require_recording_in_room(&state, room_id, recording_id).await?;
    require_host(&state, room_id, caller).await?;

    service_for(&state)
        .resume_recording(recording_id)
        .await
        .map_err(recording_error)?;
    Ok(Json(
        serde_json::json!({ "success": true, "message": "Recording resumed" }),
    ))
}

/// `GET /api/meet/rooms/{id}/recordings`
pub async fn list_recordings(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
) -> Result<Json<Vec<WebinarRecording>>, RouteError> {
    require_room_member(&state, &headers, &user, room_id).await?;
    service_for(&state)
        .list_recordings(room_id)
        .await
        .map(Json)
        .map_err(recording_error)
}

/// `GET /api/meet/recordings/{recording_id}`
pub async fn get_recording(
    State(state): State<Arc<AppState>>,
    Path(recording_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
) -> Result<Json<WebinarRecording>, RouteError> {
    let recording = service_for(&state)
        .get_recording(recording_id)
        .await
        .map_err(recording_error)?;
    require_room_member(&state, &headers, &user, recording.webinar_id).await?;
    Ok(Json(recording))
}

/// `DELETE /api/meet/recordings/{recording_id}`
pub async fn delete_recording(
    State(state): State<Arc<AppState>>,
    Path(recording_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, RouteError> {
    let caller = verified_user_id(&user)?;
    let room_id = recording_room(&state, recording_id).await?;
    require_room_scope(&state, &headers, &user, room_id).await?;
    require_host(&state, room_id, caller).await?;

    service_for(&state)
        .delete_recording(recording_id)
        .await
        .map_err(recording_error)?;
    Ok(Json(
        serde_json::json!({ "success": true, "message": "Recording deleted" }),
    ))
}

/// `GET /api/meet/recordings/{recording_id}/file`
///
/// Streams the recording file. The object lives in OtterIO drive at
/// `meet/recordings/{recording_id}.webm` (per-org bucket) when the egress has
/// uploaded it; until then the recording is still processing and a 409 is
/// returned so the UI can poll.
pub async fn get_recording_file(
    State(state): State<Arc<AppState>>,
    Path(recording_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
) -> Result<Response, RouteError> {
    let recording = service_for(&state)
        .get_recording(recording_id)
        .await
        .map_err(recording_error)?;
    require_room_member(&state, &headers, &user, recording.webinar_id).await?;

    if recording.status != RecordingStatus::Ready {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Recording is not ready yet" })),
        ));
    }

    let drive = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Drive unavailable" })),
        )
    })?;

    let bucket = state.bucket_name.clone();
    let key = format!("meet/recordings/{recording_id}.webm");
    let bytes = drive.get_object(&bucket, &key).await.map_err(|e| {
        log::error!("Failed to read recording {recording_id} from drive: {e}");
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Recording file not found" })),
        )
    })?;

    let mut response = Response::new(axum::body::Body::from(bytes));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("video/webm"));
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("inline"),
    );
    Ok(response)
}
