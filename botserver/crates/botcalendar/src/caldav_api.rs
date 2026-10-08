use super::*;


// The DAV router lives in the `caldav` module; it is re-exported above so the
// server keeps mounting it through `botcalendar::create_caldav_router`.

pub(crate) async fn caldav_root() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("DAV", "1, 2, calendar-access")
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
<D:response>
<D:href>/caldav/</D:href>
<D:propstat>
<D:prop>
<D:resourcetype>
<D:collection/>
</D:resourcetype>
<D:displayname>GeneralBots CalDAV Server</D:displayname>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>
</D:multistatus>"#
                .to_string(),
        )
        .unwrap_or_default()
}

pub(crate) async fn caldav_principals() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
<D:response>
<D:href>/caldav/principals/</D:href>
<D:propstat>
<D:prop>
<D:resourcetype>
<D:collection/>
<D:principal/>
</D:resourcetype>
<C:calendar-home-set>
<D:href>/caldav/calendars/</D:href>
</C:calendar-home-set>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>
</D:multistatus>"#
                .to_string(),
        )
        .unwrap_or_default()
}

pub(crate) async fn caldav_calendars(State(state): State<Arc<DbPool>>) -> impl IntoResponse {
    let pool = state.clone();
    let calendars: Vec<CalendarRecord> = match tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        calendars::table
            .filter(calendars::org_id.eq(Uuid::nil()))
            .filter(calendars::bot_id.eq(Uuid::nil()))
            .order(calendars::created_at.desc())
            .load::<CalendarRecord>(&mut conn)
            .ok()
    })
    .await
    {
        Ok(Some(c)) => c,
        _ => Vec::new(),
    };

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
<D:response>
<D:href>/caldav/calendars/</D:href>
<D:propstat>
<D:prop>
<D:resourcetype>
<D:collection/>
</D:resourcetype>
<D:displayname>Calendars</D:displayname>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>"#,
    );

    for cal in &calendars {
        xml.push_str(&format!(
            r#"
<D:response>
<D:href>/caldav/calendars/{id}/</D:href>
<D:propstat>
<D:prop>
<D:resourcetype>
<D:collection/>
<C:calendar/>
</D:resourcetype>
<D:displayname>{name}</D:displayname>
<C:supported-calendar-component-set>
<C:comp name="VEVENT"/>
<C:comp name="VTODO"/>
</C:supported-calendar-component-set>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>"#,
            id = cal.id,
            name = html_escape(&cal.name),
        ));
    }

    xml.push_str("\n</D:multistatus>");

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(xml)
        .unwrap_or_default()
}

pub(crate) async fn caldav_calendar(
    State(state): State<Arc<DbPool>>,
    headers: axum::http::HeaderMap,
    Path(calendar_id_str): Path<String>,
) -> impl IntoResponse {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);
    let id_str = calendar_id_str.clone();
    let calendar_uuid = Uuid::parse_str(&id_str).ok();

    let calendar: Option<CalendarRecord> = match calendar_uuid {
        Some(uuid) => match tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().ok()?;
            calendars::table
                .filter(calendars::org_id.eq(org_id))
                .filter(calendars::bot_id.eq(bot_id))
                .find(uuid)
                .first::<CalendarRecord>(&mut conn)
                .optional()
                .ok()
        })
        .await
        {
            Ok(Some(c)) => c,
            _ => None,
        },
        None => None,
    };

    let (display_id, display_name) = match &calendar {
        Some(c) => (c.id.to_string(), c.name.clone()),
        None => (calendar_id_str.clone(), "Unknown Calendar".to_string()),
    };

    let xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
<D:response>
<D:href>/caldav/calendars/{id}/</D:href>
<D:propstat>
<D:prop>
<D:resourcetype>
<D:collection/>
<C:calendar/>
</D:resourcetype>
<D:displayname>{name}</D:displayname>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>
</D:multistatus>"#,
        id = display_id,
        name = html_escape(&display_name),
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(xml)
        .unwrap_or_default()
}

pub(crate) async fn caldav_event(
    State(state): State<Arc<DbPool>>,
    headers: axum::http::HeaderMap,
    Path((_calendar_id_str, event_id_str)): Path<(String, String)>,
) -> impl IntoResponse {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);
    let event_uuid = Uuid::parse_str(&event_id_str).unwrap_or_else(|_| Uuid::nil());

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .find(event_uuid)
            .first::<CalendarEventRecord>(&mut conn)
            .optional()
            .ok()?
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(record) => {
            let event = record_to_event(record);
            let ical_str = export_to_ical(&[event], "Calendar");
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/calendar; charset=utf-8")
                .body(ical_str)
                .unwrap_or_default()
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response()
    }
}

pub(crate) async fn caldav_put_event(
    State(state): State<Arc<DbPool>>, headers: axum::http::HeaderMap,
    Path((calendar_id_str, event_id_str)): Path<(String, String)>,
    body: String,
) -> impl IntoResponse {
    let pool = state.clone();
    let calendar_id = Uuid::parse_str(&calendar_id_str).unwrap_or_else(|_| Uuid::nil());
    let event_id = Uuid::parse_str(&event_id_str).unwrap_or_else(|_| Uuid::new_v4());

    let parsed_events = import_from_ical(&body, "caldav-client", calendar_id);
    if parsed_events.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let event = parsed_events[0].clone();
    let scope = write_scope_from_headers(&state, &headers);
    let (org_id, bot_id) = (scope.org_id, scope.bot_id);
    let owner_id = Uuid::nil();
    let now = Utc::now();

    let record = CalendarEventRecord {
        id: event_id,
        org_id,
        bot_id,
        branch_id: scope.branch_id,
        calendar_id,
        owner_id,
        title: event.title.clone(),
        description: event.description.clone(),
        location: event.location.clone(),
        start_time: event.start_time,
        end_time: event.end_time,
        all_day: event.all_day,
        recurrence_rule: event.recurrence.clone(),
        recurrence_id: None,
        color: None,
        status: event.status.clone(),
        visibility: "default".to_string(),
        busy_status: "busy".to_string(),
        reminders: serde_json::json!([]),
        attendees: serde_json::json!([]),
        conference_data: None,
        metadata: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        diesel::insert_into(calendar_events::table)
            .values(&record)
            .on_conflict(calendar_events::id)
            .do_update()
            .set(&record)
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok::<_, StatusCode>(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            Response::builder()
                .status(StatusCode::CREATED)
                .header("ETag", format!("\"{}\"", Uuid::new_v4()))
                .body(String::new())
                .unwrap_or_default()
                .into_response()
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
