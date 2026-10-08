use super::*;

pub async fn list_calendars_api(
    State(state): State<Arc<DbPool>>,
    headers: axum::http::HeaderMap,
) -> Json<serde_json::Value> {
    let pool = state.clone();
    let (org_id, bot_id) = match state.get() {
        Ok(mut conn) => resolve_scope(&headers, &mut conn),
        Err(_) => (Uuid::nil(), Uuid::nil()),
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        calendars::table
            .filter(calendars::org_id.eq(org_id))
            .filter(calendars::bot_id.eq(bot_id))
            .load::<CalendarRecord>(&mut conn)
            .ok()
    })
    .await;

    match result {
        Ok(Some(cals)) => {
            let calendar_list: Vec<serde_json::Value> = cals
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "id": c.id,
                        "name": c.name,
                        "color": c.color,
                        "visible": c.is_visible
                    })
                })
                .collect();
            Json(serde_json::json!({ "calendars": calendar_list }))
        }
        _ => Json(serde_json::json!({
            "calendars": [{
                "id": "default",
                "name": "My Calendar",
                "color": "#3b82f6",
                "visible": true
            }]
        })),
    }
}

pub async fn list_calendars_html(
    State(state): State<Arc<DbPool>>,
    headers: axum::http::HeaderMap,
) -> Html<String> {
    let pool = state.clone();
    let (org_id, bot_id) = match state.get() {
        Ok(mut conn) => resolve_scope(&headers, &mut conn),
        Err(_) => (Uuid::nil(), Uuid::nil()),
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        calendars::table
            .filter(calendars::org_id.eq(org_id))
            .filter(calendars::bot_id.eq(bot_id))
            .load::<CalendarRecord>(&mut conn)
            .ok()
    })
    .await;

    match result {
        Ok(Some(cals)) if !cals.is_empty() => {
            let html: String = cals
                .iter()
                .map(|c| {
                    let color = c.color.as_deref().unwrap_or("#3b82f6");
                    let checked = if c.is_visible { "checked" } else { "" };
                    format!(
                        r#"<div class="calendar-item" data-calendar-id="{}">
<span class="calendar-checkbox {}" style="background: {};" onclick="toggleCalendar(this)"></span>
<span class="calendar-name">{}</span>
</div>"#,
                        c.id, checked, color, c.name
                    )
                })
                .collect();
            Html(html)
        }
        _ => Html(
            r#"
<div class="calendar-item" data-calendar-id="default">
<span class="calendar-checkbox checked" style="background: #3b82f6;" onclick="toggleCalendar(this)"></span>
<span class="calendar-name">My Calendar</span>
</div>
"#
            .to_string(),
        ),
    }
}

pub async fn upcoming_events_api(
    State(state): State<Arc<DbPool>>,
    headers: axum::http::HeaderMap,
) -> Json<serde_json::Value> {
    let pool = state.clone();
    let (org_id, bot_id) = match state.get() {
        Ok(mut conn) => resolve_scope(&headers, &mut conn),
        Err(_) => (Uuid::nil(), Uuid::nil()),
    };
    let now = Utc::now();
    let end = now + chrono::Duration::days(7);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .filter(calendar_events::start_time.ge(now))
            .filter(calendar_events::start_time.le(end))
            .order(calendar_events::start_time.asc())
            .limit(10)
            .load::<CalendarEventRecord>(&mut conn)
            .ok()
    })
    .await;

    match result {
        Ok(Some(events)) => {
            let event_list: Vec<serde_json::Value> = events
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "title": e.title,
                        "start_time": e.start_time,
                        "end_time": e.end_time,
                        "location": e.location
                    })
                })
                .collect();
            Json(serde_json::json!({ "events": event_list }))
        }
        _ => Json(serde_json::json!({
            "events": [],
            "message": "No upcoming events"
        })),
    }
}

pub async fn upcoming_events_html(
    State(state): State<Arc<DbPool>>,
    headers: axum::http::HeaderMap,
) -> Html<String> {
    let pool = state.clone();
    let (org_id, bot_id) = match state.get() {
        Ok(mut conn) => resolve_scope(&headers, &mut conn),
        Err(_) => (Uuid::nil(), Uuid::nil()),
    };
    let now = Utc::now();
    let end = now + chrono::Duration::days(7);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .filter(calendar_events::start_time.ge(now))
            .filter(calendar_events::start_time.le(end))
            .order(calendar_events::start_time.asc())
            .limit(5)
            .load::<CalendarEventRecord>(&mut conn)
            .ok()
    })
    .await;

    match result {
        Ok(Some(events)) if !events.is_empty() => {
            let html: String = events
                .iter()
                .map(|e| {
                    let color = e.color.as_deref().unwrap_or("#3b82f6");
                    let time = e.start_time.format("%b %d, %H:%M").to_string();
                    format!(
                        r#"<div class="upcoming-event">
<div class="upcoming-color" style="background: {};"></div>
<div class="upcoming-info">
<span class="upcoming-title">{}</span>
<span class="upcoming-time">{}</span>
</div>
</div>"#,
                        color, e.title, time
                    )
                })
                .collect();
            Html(html)
        }
        _ => Html(
            r#"
<div class="upcoming-event">
<div class="upcoming-color" style="background: #3b82f6;"></div>
<div class="upcoming-info">
<span class="upcoming-title">No upcoming events</span>
<span class="upcoming-time">Create your first event</span>
</div>
</div>
"#
            .to_string(),
        ),
    }
}

pub async fn new_event_form() -> Html<String> {
    Html(
        r#"
<div class="event-form-content">
<p>Create a new event using the form on the right panel.</p>
</div>
"#
        .to_string(),
    )
}

pub async fn new_calendar_form() -> Html<String> {
    Html(
        r##"<form class="calendar-form" hx-post="/api/calendar/calendars" hx-swap="none">
<div class="form-group">
<label>Calendar Name</label>
<input type="text" name="name" placeholder="My Calendar" required />
</div>
<div class="form-group">
<label>Color</label>
<div class="color-options">
<label><input type="radio" name="color" value="#3b82f6" checked /><span class="color-dot" style="background:#3b82f6"></span></label>
<label><input type="radio" name="color" value="#22c55e" /><span class="color-dot" style="background:#22c55e"></span></label>
<label><input type="radio" name="color" value="#f59e0b" /><span class="color-dot" style="background:#f59e0b"></span></label>
<label><input type="radio" name="color" value="#ef4444" /><span class="color-dot" style="background:#ef4444"></span></label>
<label><input type="radio" name="color" value="#8b5cf6" /><span class="color-dot" style="background:#8b5cf6"></span></label>
</div>
</div>
<div class="form-actions">
<button type="button" class="btn-secondary" onclick="this.closest('.modal').classList.add('hidden')">Cancel</button>
<button type="submit" class="btn-primary">Create Calendar</button>
</div>
</form>"##
            .to_string(),
    )
}

pub fn configure_calendar_routes() -> Router<Arc<DbPool>> {
    Router::new()
        .route("/api/calendar/calendars", get(list_calendars_db).post(create_calendar))
        .route(
            "/api/calendar/calendars/:id",
            get(get_calendar).put(update_calendar).delete(delete_calendar),
        )
        .route("/api/calendar/calendars/:id/share", post(share_calendar))
        .route("/api/calendar/calendars/:id/export", get(export_ical))
        .route("/api/calendar/calendars/:id/import", post(import_ical))
        .route(API_CALENDAR_EVENTS, get(list_events).post(create_event))
        .route(
            API_CALENDAR_EVENT_BY_ID,
            get(get_event).put(update_event).delete(delete_event),
        )
        .route(API_CALENDAR_UPCOMING_JSON, get(upcoming_events_api))
        .route("/api/calendar/conflicts", post(check_conflicts_api))
}
