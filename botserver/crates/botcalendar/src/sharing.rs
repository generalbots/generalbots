use super::*;

pub async fn share_calendar(
        headers: axum::http::HeaderMap,
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(input): Json<ShareCalendarRequest>,
) -> Result<Json<CalendarShareRecord>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);

    // The calendar being shared must belong to the caller's scope.
    let owned = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        calendars::table
            .filter(calendars::org_id.eq(org_id))
            .filter(calendars::bot_id.eq(bot_id))
            .find(id)
            .first::<CalendarRecord>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if owned?.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let pool = state.clone();
    let new_share = CalendarShareRecord {
        id: Uuid::new_v4(),
        calendar_id: id,
        shared_with_user_id: input.user_id,
        shared_with_email: input.email,
        permission: input.permission,
        created_at: Utc::now(),
    };

    let share = new_share.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        diesel::insert_into(calendar_shares::table)
            .values(&new_share)
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok::<_, StatusCode>(())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?;
    Ok(Json(share))
}

pub async fn export_ical(
        headers: axum::http::HeaderMap,
    State(state): State<Arc<DbPool>>,
    Path(calendar_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        let calendar = calendars::table
            .filter(calendars::org_id.eq(org_id))
            .filter(calendars::bot_id.eq(bot_id))
            .find(calendar_id)
            .first::<CalendarRecord>(&mut conn)
            .optional()
            .ok()??;

        let events = calendar_events::table
            .filter(calendar_events::calendar_id.eq(calendar_id))
            .load::<CalendarEventRecord>(&mut conn)
            .ok()?;

        let event_list: Vec<CalendarEvent> = events.into_iter().map(record_to_event).collect();
        Some(export_to_ical(&event_list, &calendar.name))
    })
    .await;

    match result {
        Ok(Some(ical)) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/calendar; charset=utf-8")],
            ical,
        )
            .into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn import_ical(
    State(state): State<Arc<DbPool>>, headers: axum::http::HeaderMap,
    Path(calendar_id): Path<Uuid>,
    body: String,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let pool = state.clone();
    let scope = write_scope_from_headers(&state, &headers);
    let (org_id, bot_id, branch_id) = (scope.org_id, scope.bot_id, scope.branch_id);
    let owner_id = Uuid::nil();

    let events = import_from_ical(&body, &owner_id.to_string(), calendar_id);
    let count = events.len();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        let now = Utc::now();

        for event in events {
            let record = CalendarEventRecord {
                id: event.id,
                org_id,
                bot_id,
                branch_id,
                calendar_id,
                owner_id,
                title: event.title,
                description: event.description,
                location: event.location,
                start_time: event.start_time,
                end_time: event.end_time,
                all_day: event.all_day,
                recurrence_rule: event.recurrence,
                recurrence_id: None,
                color: event.color,
                status: event.status,
                visibility: "default".to_string(),
                busy_status: "busy".to_string(),
                reminders: serde_json::json!([]),
                attendees: serde_json::to_value(&event.attendees)
                    .unwrap_or(serde_json::json!([])),
                conference_data: None,
                metadata: serde_json::json!({}),
                created_at: now,
                updated_at: now,
            };

            diesel::insert_into(calendar_events::table)
                .values(&record)
                .execute(&mut conn)
                .ok();
        }

        Ok::<_, StatusCode>(())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?;
    Ok(Json(serde_json::json!({ "imported": count })))
}
