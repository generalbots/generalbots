use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictCheckRequest {
    pub calendar_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub exclude_event_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictCheckResponse {
    pub has_conflicts: bool,
    pub conflicts: Vec<CalendarEvent>,
}

pub async fn check_conflicts_api(
    State(state): State<Arc<DbPool>>, headers: axum::http::HeaderMap,
    Json(req): Json<ConflictCheckRequest>,
) -> Result<Json<ConflictCheckResponse>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = match state.get() {
        Ok(mut conn) => resolve_scope(&headers, &mut conn),
        Err(_) => (Uuid::nil(), Uuid::nil()),
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        let records = detect_conflicts(
            &mut conn,
            org_id,
            bot_id,
            req.calendar_id,
            req.start_time,
            req.end_time,
            req.exclude_event_id,
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let events: Vec<CalendarEvent> =
            records.into_iter().map(record_to_event).collect();
        Ok::<_, StatusCode>(events)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let conflicts = result?;
    Ok(Json(ConflictCheckResponse {
        has_conflicts: !conflicts.is_empty(),
        conflicts,
    }))
}

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub(crate) fn detect_conflicts(
    conn: &mut diesel::PgConnection,
    org_id: Uuid,
    bot_id: Uuid,
    calendar_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    exclude_event_id: Option<Uuid>,
) -> Result<Vec<CalendarEventRecord>, diesel::result::Error> {
    use diesel::ExpressionMethods;
    let mut q = calendar_events::table
        .filter(calendar_events::org_id.eq(org_id))
        .filter(calendar_events::bot_id.eq(bot_id))
        .filter(calendar_events::calendar_id.eq(calendar_id))
        .filter(calendar_events::start_time.lt(end))
        .filter(calendar_events::end_time.gt(start))
        .into_boxed();
    if let Some(eid) = exclude_event_id {
        q = q.filter(calendar_events::id.ne(eid));
    }
    q.order(calendar_events::start_time.asc())
        .load::<CalendarEventRecord>(conn)
}
