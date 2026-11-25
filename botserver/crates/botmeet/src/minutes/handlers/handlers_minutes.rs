use super::*;
use botsecurity_core::error_sanitizer::{log_and_sanitize_str, SafeErrorResponse};

/// #2753 — log the cause server-side, hand the caller only the generic body.
fn safe_internal(e: impl std::fmt::Display, context: &str) -> SafeErrorResponse {
    log_and_sanitize_str(&e.to_string(), context, None)
}

fn safe_db(e: impl std::fmt::Display) -> SafeErrorResponse {
    safe_internal(e, "botmeet::minutes::db")
}

/// The PDF export keeps its explicit 501, so its error arm carries the status
/// next to the sanitised envelope instead of relying on the envelope's kind.
type PdfError = (StatusCode, Json<SafeErrorResponse>);

fn pdf_internal(e: impl std::fmt::Display, context: &str) -> PdfError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(safe_internal(e, context)),
    )
}

pub async fn handle_get_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MeetingMinute>, SafeErrorResponse> {
    let pool = state.conn.clone();
    let minute = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_minute(&mut conn, id).map_err(|e| format!("Failed to load minutes: {e}"))
    })
    .await
    .map_err(|e| safe_internal(e, "botmeet::minutes::spawn"))?
    .map_err(|_| SafeErrorResponse::not_found("Resource"))?;

    Ok(Json(minute))
}

pub async fn handle_list_minutes(
    State(state): State<Arc<AppState>>,
    Path(bot_id): Path<Uuid>,
    Query(p): Query<PaginationParams>,
) -> Result<Json<Vec<MeetingMinute>>, SafeErrorResponse> {
    let limit = p.limit.unwrap_or(50);
    let offset = p.offset.unwrap_or(0);
    let pool = state.conn.clone();
    let minutes = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::list_minutes(&mut conn, bot_id, limit, offset)
            .map_err(|e| format!("Failed to list minutes: {e}"))
    })
    .await
    .map_err(|e| safe_internal(e, "botmeet::minutes::spawn"))?
    .map_err(safe_db)?;

    Ok(Json(minutes))
}

pub async fn handle_update_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMinutesRequest>,
) -> Result<Json<MeetingMinute>, SafeErrorResponse> {
    let pool = state.conn.clone();
    let updated = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::update_minute(&mut conn, id, &req)
            .map_err(|e| format!("Failed to update minutes: {e}"))?;
        MinuteStorage::get_minute(&mut conn, id)
            .map_err(|e| format!("Failed to reload minutes: {e}"))
    })
    .await
    .map_err(|e| safe_internal(e, "botmeet::minutes::spawn"))?
    .map_err(safe_db)?;

    Ok(Json(updated))
}

pub async fn handle_finalize_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    let pool = state.conn.clone();
    let minute = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::finalize_minute(&mut conn, id)
            .map_err(|e| format!("Failed to finalize minutes: {e}"))?;
        MinuteStorage::get_minute(&mut conn, id)
            .map_err(|e| format!("Failed to reload minutes: {e}"))
    })
    .await
    .map_err(|e| safe_internal(e, "botmeet::minutes::spawn"))?
    .map_err(safe_db)?;

    info!("Finalized minutes {}", minute.id);
    Ok(Json(serde_json::json!({
        "id": minute.id,
        "status": minute.status.to_string(),
        "finalized_at": minute.updated_at,
    })))
}

pub async fn handle_export_pdf(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Vec<u8>, PdfError> {
    // Load the minute so the export endpoint at least validates existence
    let pool = state.conn.clone();
    let _minute = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_minute(&mut conn, id).map_err(|e| format!("Failed to load minutes: {e}"))
    })
    .await
    .map_err(|e| pdf_internal(e, "botmeet::minutes::spawn"))?
    .map_err(|_| (StatusCode::NOT_FOUND, Json(SafeErrorResponse::not_found("Resource"))))?;

    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(SafeErrorResponse::new(
            "not_implemented",
            "PDF export not yet implemented",
        )),
    ))
}

pub async fn handle_get_signatures(
    State(state): State<Arc<AppState>>,
    Path(minute_id): Path<Uuid>,
) -> Result<Json<Vec<MinuteSignature>>, SafeErrorResponse> {
    let pool = state.conn.clone();
    let sigs = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_signatures(&mut conn, minute_id)
            .map_err(|e| format!("Failed to load signatures: {e}"))
    })
    .await
    .map_err(|e| safe_internal(e, "botmeet::minutes::spawn"))?
    .map_err(safe_db)?;

    Ok(Json(sigs))
}

pub async fn handle_sign_minutes(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<SignMinutesRequest>,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    if user.user_id.is_nil() {
        return Err(SafeErrorResponse::unauthorized());
    }
    let pool = state.conn.clone();
    let minute = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_minute(&mut conn, id).map_err(|e| format!("Failed to load minutes: {e}"))
    })
    .await
    .map_err(|e| safe_internal(e, "botmeet::minutes::spawn"))?
    .map_err(|_| SafeErrorResponse::not_found("Resource"))?;

    let signature =
        SignatureService::sign_minute(&minute, user.user_id, req.signature_id, req.ip_address)
            .map_err(|error| {
                log::error!("Failed to sign minute {id}: {error}");
                SafeErrorResponse::new(
                    "service_unavailable",
                    "Meeting-minute signing is unavailable",
                )
            })?;

    let pool = state.conn.clone();
    let sig = signature.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::save_signature(&mut conn, &sig)
            .map_err(|e| format!("Failed to save signature: {e}"))?;
        // Recompute status: signed when enough signatures exist
        let mut sigs = MinuteStorage::get_signatures(&mut conn, id)
            .map_err(|e| format!("Failed to load signatures: {e}"))?;
        sigs.push(sig.clone());
        let mut m = MinuteStorage::get_minute(&mut conn, id)
            .map_err(|e| format!("Failed to load minutes: {e}"))?;
        SignatureService::update_minute_status(&mut m, &sigs);
        let status = m.status.to_string();
        MinuteStorage::set_minute_status(&mut conn, id, &m.status)
            .map_err(|e| format!("Failed to update minute status: {e}"))?;
        Ok::<_, String>((sig, status))
    })
    .await
    .map_err(|e| safe_internal(e, "botmeet::minutes::spawn"))?
    .map_err(safe_db)?;

    let (signature, status) = result;
    info!("Minute {} signed (status: {status})", id);
    Ok(Json(serde_json::json!({
        "success": true,
        "signature": signature,
        "status": status,
    })))
}

pub fn minutes_routes() -> axum::Router<Arc<AppState>> {
    use axum::routing::{get, post, put};

    axum::Router::new()
        .route("/api/meet/recordings", post(handle_create_recording))
        .route("/api/meet/recordings/:id", get(handle_get_recording))
        .route(
            "/api/meet/recordings/bot/:bot_id",
            get(handle_list_recordings),
        )
        .route(
            "/api/meet/recordings/:id/transcribe",
            post(handle_transcribe_recording),
        )
        .route(
            "/api/meet/minutes/from/:recording_id",
            post(handle_generate_minutes),
        )
        .route("/api/meet/minutes/:id", get(handle_get_minutes))
        .route("/api/meet/minutes/bot/:bot_id", get(handle_list_minutes))
        .route("/api/meet/minutes/:id", put(handle_update_minutes))
        .route(
            "/api/meet/minutes/:id/finalize",
            post(handle_finalize_minutes),
        )
        .route("/api/meet/minutes/:id/export/pdf", get(handle_export_pdf))
        .route(
            "/api/meet/minutes/:id/signatures",
            get(handle_get_signatures),
        )
        .route("/api/meet/minutes/:id/sign", post(handle_sign_minutes))
}
