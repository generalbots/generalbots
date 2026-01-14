use axum::{
    extract::{Path, Query, State},
    response::Html,
    routing::get,
    Router,
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{calendar_events, calendars};
use crate::shared::state::AppState;

#[derive(Debug, Deserialize, Default)]
pub struct EventsQuery {
    pub calendar_id: Option<Uuid>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub view: Option<String>,
}

pub async fn events_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventsQuery>,
) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let now = Utc::now();
        let start = query.start.unwrap_or(now);
        let end = query.end.unwrap_or(now + Duration::days(30));

        let mut db_query = calendar_events::table
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

pub async fn event_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        calendar_events::table
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

pub async fn calendars_sidebar(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        calendars::table
            .filter(calendars::bot_id.eq(bot_id))
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
                        id,
                        checked,
                        id,
                        !visible,
                        cal_color,
                        name,
                        primary_badge
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

pub async fn upcoming_events(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let now = Utc::now();
        let end = now + Duration::days(7);

        calendar_events::table
            .filter(calendar_events::bot_id.eq(bot_id))
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

pub async fn events_count(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        calendar_events::table
            .filter(calendar_events::bot_id.eq(bot_id))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

pub async fn today_events_count(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let today = Utc::now().date_naive();
        let today_start = today.and_hms_opt(0, 0, 0)?;
        let today_end = today.and_hms_opt(23, 59, 59)?;

        calendar_events::table
            .filter(calendar_events::bot_id.eq(bot_id))
            .filter(calendar_events::start_time.ge(today_start))
            .filter(calendar_events::start_time.le(today_end))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

#[derive(Debug, Deserialize, Default)]
pub struct MonthQuery {
    pub year: Option<i32>,
    pub month: Option<u32>,
}

pub async fn month_view(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MonthQuery>,
) -> Html<String> {
    let pool = state.conn.clone();
    let now = Utc::now();
    let year = query.year.unwrap_or(now.year());
    let month = query.month.unwrap_or(now.month());

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let first_day = NaiveDate::from_ymd_opt(year, month, 1)?;
        let last_day = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)?
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)?
        }
        .pred_opt()?;

        let start = first_day.and_hms_opt(0, 0, 0)?;
        let end = last_day.and_hms_opt(23, 59, 59)?;

        let events = calendar_events::table
            .filter(calendar_events::bot_id.eq(bot_id))
            .filter(calendar_events::start_time.ge(start))
            .filter(calendar_events::start_time.le(end))
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::start_time,
                calendar_events::color,
            ))
            .load::<(Uuid, String, DateTime<Utc>, Option<String>)>(&mut conn)
            .ok()?;

        Some((first_day, last_day, events))
    })
    .await
    .ok()
    .flatten();

    match result {
        Some((first_day, last_day, events)) => {
            let month_name = first_day.format("%B %Y").to_string();
            let start_weekday = first_day.weekday().num_days_from_sunday();

            let mut days_html = String::new();

            for _ in 0..start_weekday {
                days_html.push_str(r#"<div class="calendar-day empty"></div>"#);
            }

            let mut current = first_day;
            while current <= last_day {
                let day_num = current.day();
                let day_events: Vec<_> = events
                    .iter()
                    .filter(|(_, _, start, _)| start.date_naive() == current)
                    .collect();

                let events_dots: String = day_events
                    .iter()
                    .take(3)
                    .map(|(_, _, _, color)| {
                        let c = color.clone().unwrap_or_else(|| "#3b82f6".to_string());
                        format!(r##"<span class="event-dot" style="background: {};"></span>"##, c)
                    })
                    .collect();

                let is_today = current == Utc::now().date_naive();
                let today_class = if is_today { "today" } else { "" };

                days_html.push_str(&format!(
                    r##"<div class="calendar-day {}" data-date="{}"
                        hx-get="/api/ui/calendar/day?date={}" hx-target="#day-events">
                        <span class="day-number">{}</span>
                        <div class="day-events">{}</div>
                    </div>"##,
                    today_class,
                    current,
                    current,
                    day_num,
                    events_dots
                ));

                current = current.succ_opt().unwrap_or(current);
            }

            let prev_month = if month == 1 { 12 } else { month - 1 };
            let prev_year = if month == 1 { year - 1 } else { year };
            let next_month = if month == 12 { 1 } else { month + 1 };
            let next_year = if month == 12 { year + 1 } else { year };

            Html(format!(
                r##"<div class="calendar-month">
                    <div class="month-header">
                        <button class="btn-icon" hx-get="/api/ui/calendar/month?year={}&month={}" hx-target="#calendar-view">&lt;</button>
                        <h3>{}</h3>
                        <button class="btn-icon" hx-get="/api/ui/calendar/month?year={}&month={}" hx-target="#calendar-view">&gt;</button>
                    </div>
                    <div class="weekdays">
                        <div>Sun</div><div>Mon</div><div>Tue</div><div>Wed</div><div>Thu</div><div>Fri</div><div>Sat</div>
                    </div>
                    <div class="days-grid">{}</div>
                </div>"##,
                prev_year, prev_month,
                month_name,
                next_year, next_month,
                days_html
            ))
        }
        None => Html(
            r##"<div class="empty-state">
                <p>Could not load calendar</p>
            </div>"##
                .to_string(),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct DayQuery {
    pub date: NaiveDate,
}

pub async fn day_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DayQuery>,
) -> Html<String> {
    let pool = state.conn.clone();
    let date = query.date;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let start = date.and_hms_opt(0, 0, 0)?;
        let end = date.and_hms_opt(23, 59, 59)?;

        calendar_events::table
            .filter(calendar_events::bot_id.eq(bot_id))
            .filter(calendar_events::start_time.ge(start))
            .filter(calendar_events::start_time.le(end))
            .order(calendar_events::start_time.asc())
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::start_time,
                calendar_events::end_time,
                calendar_events::color,
                calendar_events::all_day,
            ))
            .load::<(Uuid, String, DateTime<Utc>, DateTime<Utc>, Option<String>, bool)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    let date_str = date.format("%A, %B %d, %Y").to_string();

    match result {
        Some(events) if !events.is_empty() => {
            let items: String = events
                .iter()
                .map(|(id, title, start, end, color, all_day)| {
                    let event_color = color.clone().unwrap_or_else(|| "#3b82f6".to_string());
                    let time_str = if *all_day {
                        "All day".to_string()
                    } else {
                        format!("{} - {}", start.format("%H:%M"), end.format("%H:%M"))
                    };

                    format!(
                        r##"<div class="day-event-item" style="border-left: 4px solid {};"
                             hx-get="/api/ui/calendar/events/{}" hx-target="#event-detail">
                            <span class="event-time">{}</span>
                            <span class="event-title">{}</span>
                        </div>"##,
                        event_color, id, time_str, title
                    )
                })
                .collect();

            Html(format!(
                r##"<div class="day-events-panel">
                    <h4>{}</h4>
                    <div class="events-list">{}</div>
                </div>"##,
                date_str, items
            ))
        }
        _ => Html(format!(
            r##"<div class="day-events-panel">
                <h4>{}</h4>
                <div class="empty-state">
                    <p>No events on this day</p>
                </div>
            </div>"##,
            date_str
        )),
    }
}

pub async fn new_event_form() -> Html<String> {
    let now = Utc::now();
    let date = now.format("%Y-%m-%d").to_string();
    let time = now.format("%H:00").to_string();
    let end_time = (now + Duration::hours(1)).format("%H:00").to_string();

    Html(format!(
        r##"<div class="modal-header">
            <h3>New Event</h3>
            <button class="btn-close" onclick="closeModal()">&times;</button>
        </div>
        <form class="event-form" hx-post="/api/calendar/events" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#calendar-view', 'refresh')">
            <div class="form-group">
                <label>Title</label>
                <input type="text" name="title" placeholder="Event title" required />
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Date</label>
                    <input type="date" name="date" value="{}" required />
                </div>
                <div class="form-group">
                    <label>All Day</label>
                    <input type="checkbox" name="all_day" onchange="toggleTimeInputs(this)" />
                </div>
            </div>
            <div class="form-row time-inputs">
                <div class="form-group">
                    <label>Start Time</label>
                    <input type="time" name="start_time" value="{}" />
                </div>
                <div class="form-group">
                    <label>End Time</label>
                    <input type="time" name="end_time" value="{}" />
                </div>
            </div>
            <div class="form-group">
                <label>Location</label>
                <input type="text" name="location" placeholder="Add location" />
            </div>
            <div class="form-group">
                <label>Description</label>
                <textarea name="description" rows="3" placeholder="Add description"></textarea>
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
                <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
                <button type="submit" class="btn btn-primary">Create Event</button>
            </div>
        </form>"##,
        date, time, end_time
    ))
}

pub async fn new_calendar_form() -> Html<String> {
    Html(
        r##"<div class="modal-header">
            <h3>New Calendar</h3>
            <button class="btn-close" onclick="closeModal()">&times;</button>
        </div>
        <form class="calendar-form" hx-post="/api/calendar/calendars" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#calendars-sidebar', 'refresh')">
            <div class="form-group">
                <label>Calendar Name</label>
                <input type="text" name="name" placeholder="My Calendar" required />
            </div>
            <div class="form-group">
                <label>Description</label>
                <textarea name="description" rows="2" placeholder="Calendar description"></textarea>
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
            <div class="form-group">
                <label>Timezone</label>
                <select name="timezone">
                    <option value="UTC">UTC</option>
                    <option value="America/New_York">Eastern Time</option>
                    <option value="America/Chicago">Central Time</option>
                    <option value="America/Denver">Mountain Time</option>
                    <option value="America/Los_Angeles">Pacific Time</option>
                    <option value="Europe/London">London</option>
                    <option value="Europe/Paris">Paris</option>
                    <option value="Asia/Tokyo">Tokyo</option>
                </select>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
                <button type="submit" class="btn btn-primary">Create Calendar</button>
            </div>
        </form>"##
            .to_string(),
    )
}

pub fn configure_calendar_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ui/calendar/events", get(events_list))
        .route("/api/ui/calendar/events/count", get(events_count))
        .route("/api/ui/calendar/events/today", get(today_events_count))
        .route("/api/ui/calendar/events/:id", get(event_detail))
        .route("/api/ui/calendar/calendars", get(calendars_sidebar))
        .route("/api/ui/calendar/upcoming", get(upcoming_events))
        .route("/api/ui/calendar/month", get(month_view))
        .route("/api/ui/calendar/day", get(day_events))
        .route("/api/ui/calendar/new-event", get(new_event_form))
        .route("/api/ui/calendar/new-calendar", get(new_calendar_form))
}
