use super::*;

pub async fn create_calendar(
    State(state): State<Arc<DbPool>>, headers: axum::http::HeaderMap,
    Json(input): Json<CreateCalendarRequest>,
) -> Result<Json<CalendarRecord>, StatusCode> {
    let pool = state.clone();
    let scope = write_scope_from_headers(&state, &headers);
    let (org_id, bot_id) = (scope.org_id, scope.bot_id);
    let owner_id = Uuid::nil();
    let now = Utc::now();

    let new_calendar = CalendarRecord {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        branch_id: scope.branch_id,
        owner_id,
        name: input.name,
        description: input.description,
        color: input.color.or(Some("#3b82f6".to_string())),
        timezone: input.timezone.or(Some("UTC".to_string())),
        is_primary: input.is_primary,
        is_visible: true,
        is_shared: false,
        created_at: now,
        updated_at: now,
    };

    let calendar = new_calendar.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        diesel::insert_into(calendars::table)
            .values(&new_calendar)
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok::<_, StatusCode>(())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?;
    info!("Created calendar: {} ({})", calendar.name, calendar.id);
    Ok(Json(calendar))
}

pub async fn list_calendars_db(
    State(state): State<Arc<DbPool>>, headers: axum::http::HeaderMap,
) -> Result<Json<Vec<CalendarRecord>>, StatusCode> {
    let pool = state.clone();
    let scope = write_scope_from_headers(&state, &headers);
    let (org_id, bot_id, branch_id) = (scope.org_id, scope.bot_id, scope.branch_id);

    let result = tokio::task::spawn_blocking(
        move || -> Result<Vec<CalendarRecord>, StatusCode> {
            let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            let calendars = calendars::table
                .filter(calendars::org_id.eq(org_id))
                .filter(calendars::bot_id.eq(bot_id))
                .order(calendars::created_at.desc())
                .load::<CalendarRecord>(&mut conn)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            // #1250 — auto-provision a default calendar when the workspace
            // has none: `create_event` requires a `calendar_id`, and a fresh
            // user who opens Calendar would otherwise see an empty state with
            // no way to create events until they manually create a calendar.
            if calendars.is_empty() {
                let now = Utc::now();
                let default_calendar = CalendarRecord {
                    id: Uuid::new_v4(),
                    org_id,
                    bot_id,
                    branch_id,
                    owner_id: Uuid::nil(),
                    name: "Default".to_string(),
                    description: Some("Your default calendar".to_string()),
                    color: Some("#3b82f6".to_string()),
                    timezone: Some("UTC".to_string()),
                    is_primary: true,
                    is_visible: true,
                    is_shared: false,
                    created_at: now,
                    updated_at: now,
                };
                // Race-safe: two concurrent first lists may both try to
                // insert; a duplicate-key error is harmless (the other won).
                let _ = diesel::insert_into(calendars::table)
                    .values(&default_calendar)
                    .execute(&mut conn);
                return Ok(vec![default_calendar]);
            }
            Ok(calendars)
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result?))
}

pub async fn get_calendar(
        headers: axum::http::HeaderMap,
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CalendarRecord>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);

    let result = tokio::task::spawn_blocking(move || {
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

    result?.ok_or(StatusCode::NOT_FOUND).map(Json)
}

pub async fn update_calendar(
        headers: axum::http::HeaderMap,
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateCalendarRequest>,
) -> Result<Json<CalendarRecord>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

        let mut calendar = calendars::table
            .filter(calendars::org_id.eq(org_id))
            .filter(calendars::bot_id.eq(bot_id))
            .find(id)
            .first::<CalendarRecord>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        if let Some(name) = input.name {
            calendar.name = name;
        }
        if let Some(description) = input.description {
            calendar.description = Some(description);
        }
        if let Some(color) = input.color {
            calendar.color = Some(color);
        }
        if let Some(timezone) = input.timezone {
            calendar.timezone = Some(timezone);
        }
        if let Some(is_visible) = input.is_visible {
            calendar.is_visible = is_visible;
        }
        calendar.updated_at = Utc::now();

        diesel::update(
            calendars::table
                .filter(calendars::org_id.eq(org_id))
                .filter(calendars::bot_id.eq(bot_id))
                .find(id),
        )
        .set(&calendar)
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok::<_, StatusCode>(calendar)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result?))
}

pub async fn delete_calendar(
        headers: axum::http::HeaderMap,
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        let deleted = diesel::delete(
            calendars::table
                .filter(calendars::org_id.eq(org_id))
                .filter(calendars::bot_id.eq(bot_id))
                .find(id),
        )
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if deleted > 0 {
            Ok::<_, StatusCode>(StatusCode::NO_CONTENT)
        } else {
            Ok(StatusCode::NOT_FOUND)
        }
    })
    .await;

    match result {
        Ok(Ok(status)) => status,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn list_events(
    State(state): State<Arc<DbPool>>, headers: axum::http::HeaderMap,
    Query(query): Query<EventQuery>,
) -> Result<Json<Vec<CalendarEvent>>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = match state.get() {
        Ok(mut conn) => resolve_scope(&headers, &mut conn),
        Err(_) => (Uuid::nil(), Uuid::nil()),
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

        let mut db_query = calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(calendar_id) = query.calendar_id {
            db_query = db_query.filter(calendar_events::calendar_id.eq(calendar_id));
        }
        if let Some(start) = query.start {
            db_query = db_query.filter(calendar_events::start_time.ge(start));
        }
        if let Some(end) = query.end {
            db_query = db_query.filter(calendar_events::end_time.le(end));
        }

        db_query = db_query.order(calendar_events::start_time.asc());

        if let Some(limit) = query.limit {
            db_query = db_query.limit(limit);
        }
        if let Some(offset) = query.offset {
            db_query = db_query.offset(offset);
        }

        db_query
            .load::<CalendarEventRecord>(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let records = result?;
    let events: Vec<CalendarEvent> = records.into_iter().map(record_to_event).collect();
    Ok(Json(events))
}

pub async fn get_event(
        headers: axum::http::HeaderMap,
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .find(id)
            .first::<CalendarEventRecord>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?
        .map(record_to_event)
        .ok_or(StatusCode::NOT_FOUND)
        .map(Json)
}

pub async fn create_event(
    State(state): State<Arc<DbPool>>, headers: axum::http::HeaderMap,
    Json(input): Json<CalendarEventInput>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let pool = state.clone();
    let scope = write_scope_from_headers(&state, &headers);
    let (org_id, bot_id) = (scope.org_id, scope.bot_id);
    let owner_id = Uuid::nil();
    let now = Utc::now();

    let calendar_id = input.calendar_id.unwrap_or_else(Uuid::nil);

    let reminders = if let Some(minutes) = input.reminder_minutes {
        serde_json::json!([{"minutes_before": minutes, "type": "notification"}])
    } else {
        serde_json::json!([])
    };

    let new_event = CalendarEventRecord {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        branch_id: scope.branch_id,
        calendar_id,
        owner_id,
        title: input.title.clone(),
        description: input.description.clone(),
        location: input.location.clone(),
        start_time: input.start_time,
        end_time: input.end_time,
        all_day: input.all_day,
        recurrence_rule: input.recurrence.clone(),
        recurrence_id: None,
        color: None,
        status: "confirmed".to_string(),
        visibility: "default".to_string(),
        busy_status: "busy".to_string(),
        reminders,
        attendees: serde_json::to_value(&input.attendees).unwrap_or(serde_json::json!([])),
        conference_data: None,
        metadata: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    };

    let event_record = new_event.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        diesel::insert_into(calendar_events::table)
            .values(&new_event)
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok::<_, StatusCode>(())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?;

    let event = record_to_event(event_record);
    info!("Created calendar event: {} ({})", event.title, event.id);
    Ok(Json(event))
}

pub async fn update_event(
        headers: axum::http::HeaderMap,
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CalendarEventInput>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

        let mut event = calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .find(id)
            .first::<CalendarEventRecord>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        event.title = input.title;
        event.description = input.description;
        event.location = input.location;
        event.start_time = input.start_time;
        event.end_time = input.end_time;
        event.all_day = input.all_day;
        event.recurrence_rule = input.recurrence;
        event.attendees =
            serde_json::to_value(&input.attendees).unwrap_or(serde_json::json!([]));
        if let Some(minutes) = input.reminder_minutes {
            event.reminders =
                serde_json::json!([{"minutes_before": minutes, "type": "notification"}]);
        }
        event.updated_at = Utc::now();

        if let Some(calendar_id) = input.calendar_id {
            event.calendar_id = calendar_id;
        }

        diesel::update(calendar_events::table.find(id))
            .set(&event)
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok::<_, StatusCode>(event)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event = record_to_event(result?);
    info!("Updated calendar event: {} ({})", event.title, event.id);
    Ok(Json(event))
}

pub async fn delete_event(
        headers: axum::http::HeaderMap,
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        let deleted = diesel::delete(
            calendar_events::table
                .filter(calendar_events::org_id.eq(org_id))
                .filter(calendar_events::bot_id.eq(bot_id))
                .find(id),
        )
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if deleted > 0 {
            info!("Deleted calendar event: {id}");
            Ok::<_, StatusCode>(StatusCode::NO_CONTENT)
        } else {
            Ok(StatusCode::NOT_FOUND)
        }
    })
    .await;

    match result {
        Ok(Ok(status)) => status,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
