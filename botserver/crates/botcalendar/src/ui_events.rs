use super::*;

#[derive(Debug, Deserialize, Default)]
pub struct EventsQuery {
    pub calendar_id: Option<Uuid>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub view: Option<String>,
}

pub async fn ui_events_list(
        headers: axum::http::HeaderMap,
    State(state): State<Arc<DbPool>>,
    Query(query): Query<EventsQuery>,
) -> Html<String> {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        let now = Utc::now();
        let start = query.start.unwrap_or(now);
        let end = query.end.unwrap_or(now + Duration::days(30));

        let mut db_query = calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .filter(calendar_events::start_time.ge(start))
            .filter(calendar_events::start_time.le(end))
            .into_boxed();

        if let Some(calendar_id) = query.calendar_id {
            db_query = db_query.filter(calendar_events::calendar_id.eq(calendar_id));
        }

        db_query = db_query.order(calendar_events::start_time.asc());

        db_query
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::description,
                calendar_events::location,
                calendar_events::start_time,
                calendar_events::end_time,
                calendar_events::all_day,
                calendar_events::color,
                calendar_events::status,
            ))
            .load::<(
                Uuid,
                String,
                Option<String>,
                Option<String>,
                DateTime<Utc>,
                DateTime<Utc>,
                bool,
                Option<String>,
                String,
            )>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(events) if !events.is_empty() => {
            let items: String = events
                .iter()
                .map(|(id, title, _desc, location, start, end, all_day, color, _status)| {
                    let event_color = color.clone().unwrap_or_else(|| "#3b82f6".to_string());
                    let location_text = location.clone().unwrap_or_default();
                    let time_str = if *all_day {
                        "All day".to_string()
                    } else {
                        format!("{} - {}", start.format("%H:%M"), end.format("%H:%M"))
                    };
                    let date_str = start.format("%b %d").to_string();

                    format!(
                        r##"<div class="event-item" data-id="{}" style="border-left: 4px solid {};"
hx-get="/api/ui/calendar/events/{}" hx-target="#event-detail" hx-swap="innerHTML">
<div class="event-date">{}</div>
<div class="event-content">
<span class="event-title">{}</span>
<span class="event-time">{}</span>
{}</div>
</div>"##,
                        id,
                        event_color,
                        id,
                        date_str,
                        title,
                        time_str,
                        if location_text.is_empty() {
                            String::new()
                        } else {
                            format!(r##"<span class="event-location">{}</span>"##, location_text)
                        }
                    )
                })
                .collect();

            Html(format!(r##"<div class="events-list">{}</div>"##, items))
        }
        _ => Html(
            r##"<div class="empty-state">
<p>No events found</p>
<button class="btn btn-primary" hx-get="/api/ui/calendar/new-event" hx-target="#modal-content">
Create Event
</button>
</div>"##
            .to_string(),
        ),
    }
}

pub async fn ui_event_detail(
        headers: axum::http::HeaderMap,
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Html<String> {
    let pool = state.clone();
    let (org_id, bot_id) = scope_from_headers(&state, &headers);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .find(id)
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::description,
                calendar_events::location,
                calendar_events::start_time,
                calendar_events::end_time,
                calendar_events::all_day,
                calendar_events::color,
                calendar_events::status,
                calendar_events::attendees,
            ))
            .first::<(
                Uuid,
                String,
                Option<String>,
                Option<String>,
                DateTime<Utc>,
                DateTime<Utc>,
                bool,
                Option<String>,
                String,
                serde_json::Value,
            )>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some((id, title, desc, location, start, end, all_day, color, status, attendees)) => {
            let description = desc.unwrap_or_else(|| "No description".to_string());
            let location_text = location.unwrap_or_else(|| "No location".to_string());
            let event_color = color.unwrap_or_else(|| "#3b82f6".to_string());

            let time_str = if all_day {
                format!("{} (All day)", start.format("%B %d, %Y"))
            } else {
                format!(
                    "{} - {}",
                    start.format("%B %d, %Y %H:%M"),
                    end.format("%H:%M")
                )
            };

            let attendees_list: Vec<String> =
                serde_json::from_value(attendees).unwrap_or_default();
            let attendees_html = if attendees_list.is_empty() {
                "<p>No attendees</p>".to_string()
            } else {
                attendees_list
                    .iter()
                    .map(|a| format!(r##"<span class="attendee-badge">{}</span>"##, a))
                    .collect::<Vec<_>>()
                    .join("")
            };

            Html(format!(
                r##"<div class="event-detail-card">
<div class="detail-header" style="border-left: 4px solid {};">
<h3>{}</h3>
<span class="status-badge status-{}">{}</span>
</div>
<div class="detail-section">
<div class="detail-item">
<label>When</label>
<span>{}</span>
</div>
<div class="detail-item">
<label>Where</label>
<span>{}</span>
</div>
</div>
<div class="detail-section">
<h4>Description</h4>
<p>{}</p>
</div>
<div class="detail-section">
<h4>Attendees</h4>
<div class="attendees-list">{}</div>
</div>
<div class="detail-actions">
<button class="btn btn-primary" hx-get="/api/ui/calendar/events/{}/edit" hx-target="#modal-content">Edit</button>
<button class="btn btn-danger" hx-delete="/api/calendar/events/{}" hx-swap="none" hx-confirm="Delete this event?">Delete</button>
</div>
</div>"##,
                event_color,
                title,
                status,
                status,
                time_str,
                location_text,
                description,
                attendees_html,
                id,
                id
            ))
        }
        None => Html(
            r##"<div class="empty-state">
<p>Event not found</p>
</div>"##
            .to_string(),
        ),
    }
}

pub async fn ui_calendars_sidebar(State(state): State<Arc<DbPool>>) -> Html<String> {
    let pool = state.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        calendars::table
            .order(calendars::is_primary.desc())
            .select((
                calendars::id,
                calendars::name,
                calendars::color,
                calendars::is_visible,
                calendars::is_primary,
            ))
            .load::<(Uuid, String, Option<String>, bool, bool)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(cals) if !cals.is_empty() => {
            let items: String = cals
                .iter()
                .map(|(id, name, color, visible, primary)| {
                    let cal_color = color.clone().unwrap_or_else(|| "#3b82f6".to_string());
                    let checked = if *visible { "checked" } else { "" };
                    let primary_badge = if *primary {
                        r##"<span class="primary-badge">Primary</span>"##
                    } else {
                        ""
                    };

                    format!(
                        r##"<div class="calendar-item" data-calendar-id="{}">
<input type="checkbox" class="calendar-checkbox" {}
hx-put="/api/calendar/calendars/{}"
hx-vals='{{"is_visible": {}}}'
hx-swap="none" />
<span class="calendar-color" style="background: {};"></span>
<span class="calendar-name">{}</span>
{}
</div>"##,
                        id, checked, id, !visible, cal_color, name, primary_badge
                    )
                })
                .collect();

            Html(format!(
                r##"<div class="calendars-sidebar">
<div class="sidebar-header">
<h4>My Calendars</h4>
<button class="btn-icon" hx-get="/api/ui/calendar/new-calendar" hx-target="#modal-content">+</button>
</div>
<div class="calendars-list">{}</div>
</div>"##,
                items
            ))
        }
        _ => Html(
            r##"<div class="calendars-sidebar">
<div class="sidebar-header">
<h4>My Calendars</h4>
<button class="btn-icon" hx-get="/api/ui/calendar/new-calendar" hx-target="#modal-content">+</button>
</div>
<div class="empty-state">
<p>No calendars yet</p>
</div>
</div>"##
            .to_string(),
        ),
    }
}

pub async fn ui_upcoming_events(State(state): State<Arc<DbPool>>) -> Html<String> {
    let pool = state.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        let now = Utc::now();
        let end = now + Duration::days(7);

        calendar_events::table
            .filter(calendar_events::start_time.ge(now))
            .filter(calendar_events::start_time.le(end))
            .order(calendar_events::start_time.asc())
            .limit(5)
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::start_time,
                calendar_events::color,
            ))
            .load::<(Uuid, String, DateTime<Utc>, Option<String>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(events) if !events.is_empty() => {
            let items: String = events
                .iter()
                .map(|(id, title, start, color)| {
                    let event_color = color.clone().unwrap_or_else(|| "#3b82f6".to_string());
                    let time_str = start.format("%b %d, %H:%M").to_string();

                    format!(
                        r##"<div class="upcoming-event" hx-get="/api/ui/calendar/events/{}" hx-target="#event-detail">
<div class="upcoming-color" style="background: {};"></div>
<div class="upcoming-info">
<span class="upcoming-title">{}</span>
<span class="upcoming-time">{}</span>
</div>
</div>"##,
                        id, event_color, title, time_str
                    )
                })
                .collect();

            Html(format!(r##"<div class="upcoming-list">{}</div>"##, items))
        }
        _ => Html(
            r##"<div class="upcoming-event">
<div class="upcoming-color" style="background: #94a3b8;"></div>
<div class="upcoming-info">
<span class="upcoming-title">No upcoming events</span>
<span class="upcoming-time">Create your first event</span>
</div>
</div>"##
            .to_string(),
        ),
    }
}

pub async fn ui_events_count(State(state): State<Arc<DbPool>>) -> Html<String> {
    let pool = state.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        calendar_events::table
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(result.unwrap_or(0).to_string())
}
