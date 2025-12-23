use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{DateTime, Duration, Timelike, Utc};
use diesel::prelude::*;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
// use serde_json::json;  // Commented out - unused import
use std::sync::Arc;
use uuid::Uuid;

// Calendar types - would be from crate::calendar when feature is enabled
#[derive(Debug)]
pub struct CalendarEngine {
    _db: crate::shared::utils::DbPool,
}

#[derive(Debug)]
pub struct CalendarEvent {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub organizer: String,
    pub attendees: Vec<String>,
    pub reminder_minutes: Option<i32>,
    pub recurrence_rule: Option<RecurrenceRule>,
    pub status: EventStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

#[derive(Debug)]
pub struct RecurrenceRule {
    pub frequency: String,
    pub interval: i32,
    pub count: Option<i32>,
    pub until: Option<DateTime<Utc>>,
    pub by_day: Option<Vec<String>>,
}

impl CalendarEngine {
    #[must_use]
    pub fn new(db: crate::shared::utils::DbPool) -> Self {
        Self { _db: db }
    }

    pub async fn create_event(
        &self,
        event: CalendarEvent,
    ) -> Result<CalendarEvent, Box<dyn std::error::Error>> {
        Ok(event)
    }

    pub async fn check_conflicts(
        &self,
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
        _user: &str,
    ) -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }

    pub async fn get_events_range(
        &self,
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
}

/// Register BOOK keyword in BASIC for calendar appointments
pub fn book_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &[
                "BOOK", "$expr$", ",", "$expr$", ",", "$expr$", ",", "$expr$", ",", "$expr$",
            ],
            false,
            move |context, inputs| {
                let title = context.eval_expression_tree(&inputs[0])?.to_string();
                let description = context.eval_expression_tree(&inputs[1])?.to_string();
                let start_time_str = context.eval_expression_tree(&inputs[2])?.to_string();
                let duration_minutes = context
                    .eval_expression_tree(&inputs[3])?
                    .as_int()
                    .unwrap_or(30) as i64;
                let location = context.eval_expression_tree(&inputs[4])?.to_string();

                trace!(
                    "BOOK: title={}, start={}, duration={} min for user={}",
                    title,
                    start_time_str,
                    duration_minutes,
                    user_clone.user_id
                );

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_book(
                                &state_for_task,
                                &user_for_task,
                                &title,
                                &description,
                                &start_time_str,
                                duration_minutes,
                                &location,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send BOOK result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(event_id)) => Ok(Dynamic::from(event_id)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("BOOK failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "BOOK timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();

    // Register BOOK MEETING for more complex meetings
    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    engine
        .register_custom_syntax(
            &["BOOK_MEETING", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let meeting_details = context.eval_expression_tree(&inputs[0])?;
                let attendees_input = context.eval_expression_tree(&inputs[1])?;

                let mut attendees = Vec::new();
                if attendees_input.is_array() {
                    let arr = attendees_input.cast::<rhai::Array>();
                    for item in arr.iter() {
                        attendees.push(item.to_string());
                    }
                }

                trace!(
                    "BOOK_MEETING with {} attendees for user={}",
                    attendees.len(),
                    user_clone2.user_id
                );

                let state_for_task = Arc::clone(&state_clone2);
                let user_for_task = user_clone2.clone();

                // Use tokio's block_in_place to run async code in sync context
                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async move {
                        execute_book_meeting(
                            &state_for_task,
                            &user_for_task,
                            meeting_details.to_string(),
                            attendees,
                        )
                        .await
                    })
                });

                match result {
                    Ok(event_id) => Ok(Dynamic::from(event_id)),
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("BOOK_MEETING failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();

    // Register CHECK_AVAILABILITY keyword
    let state_clone3 = Arc::clone(&state);
    let user_clone3 = user.clone();

    engine
        .register_custom_syntax(
            &["CHECK_AVAILABILITY", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let date_str = context.eval_expression_tree(&inputs[0])?.to_string();
                let duration_minutes = context
                    .eval_expression_tree(&inputs[1])?
                    .as_int()
                    .unwrap_or(30) as i64;

                trace!(
                    "CHECK_AVAILABILITY for {} on {} for user={}",
                    duration_minutes,
                    date_str,
                    user_clone3.user_id
                );

                let state_for_task = Arc::clone(&state_clone3);
                let user_for_task = user_clone3.clone();
                let date_str = date_str.clone();

                // Use tokio's block_in_place to run async code in sync context
                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async move {
                        check_availability(
                            &state_for_task,
                            &user_for_task,
                            &date_str,
                            duration_minutes,
                        )
                        .await
                    })
                });

                match result {
                    Ok(slots) => Ok(Dynamic::from(slots)),
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("CHECK_AVAILABILITY failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

async fn execute_book(
    state: &AppState,
    user: &UserSession,
    title: &str,
    description: &str,
    start_time_str: &str,
    duration_minutes: i64,
    location: &str,
) -> Result<String, String> {
    // Parse start time
    let start_time = parse_time_string(start_time_str)?;
    let end_time = start_time + Duration::minutes(duration_minutes);

    // Get or create calendar engine
    let calendar_engine = get_calendar_engine(state).await?;

    // Check for conflicts
    let conflicts = calendar_engine
        .check_conflicts(start_time, end_time, &user.user_id.to_string())
        .await
        .map_err(|e| format!("Failed to check conflicts: {}", e))?;

    if !conflicts.is_empty() {
        return Err(format!(
            "Time slot conflicts with existing appointment: {}",
            conflicts[0].title
        ));
    }

    // Create calendar event
    let event = CalendarEvent {
        id: Uuid::new_v4(),
        title: title.to_string(),
        description: Some(description.to_string()),
        start_time,
        end_time,
        location: if location.is_empty() {
            None
        } else {
            Some(location.to_string())
        },
        organizer: user.user_id.to_string(),
        attendees: vec![user.user_id.to_string()],
        reminder_minutes: Some(15), // Default 15-minute reminder
        recurrence_rule: None,
        status: EventStatus::Confirmed,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create the event
    let created_event = calendar_engine
        .create_event(event)
        .await
        .map_err(|e| format!("Failed to create appointment: {}", e))?;

    // Log the booking
    log_booking(state, user, &created_event.id.to_string(), title).await?;

    info!(
        "Appointment booked: {} at {} for user {}",
        title, start_time, user.user_id
    );

    Ok(format!(
        "Appointment '{}' booked for {} (ID: {})",
        title,
        start_time.format("%Y-%m-%d %H:%M"),
        created_event.id
    ))
}

async fn execute_book_meeting(
    state: &AppState,
    user: &UserSession,
    meeting_json: String,
    attendees: Vec<String>,
) -> Result<String, String> {
    // Parse meeting details from JSON
    let meeting_data: serde_json::Value = serde_json::from_str(&meeting_json)
        .map_err(|e| format!("Invalid meeting details: {}", e))?;

    let title = meeting_data["title"]
        .as_str()
        .ok_or("Missing meeting title")?;
    let start_time_str = meeting_data["start_time"]
        .as_str()
        .ok_or("Missing start time")?;
    let duration_minutes = meeting_data["duration"].as_i64().unwrap_or(60);
    let description = meeting_data["description"].as_str().unwrap_or("");
    let location = meeting_data["location"].as_str().unwrap_or("");
    let recurring = meeting_data["recurring"].as_bool().unwrap_or(false);

    let start_time = parse_time_string(start_time_str)?;
    let end_time = start_time + Duration::minutes(duration_minutes);

    // Get or create calendar engine
    let calendar_engine = get_calendar_engine(state).await?;

    // Check conflicts for all attendees
    for attendee in &attendees {
        let conflicts = calendar_engine
            .check_conflicts(start_time, end_time, attendee)
            .await
            .map_err(|e| format!("Failed to check conflicts: {}", e))?;

        if !conflicts.is_empty() {
            return Err(format!("Attendee {} has a conflict at this time", attendee));
        }
    }

    // Create recurrence rule if needed
    let recurrence_rule = if recurring {
        Some(RecurrenceRule {
            frequency: "WEEKLY".to_string(),
            interval: 1,
            count: Some(10), // Default to 10 occurrences
            until: None,
            by_day: None,
        })
    } else {
        None
    };

    // Create calendar event
    let event = CalendarEvent {
        id: Uuid::new_v4(),
        title: title.to_string(),
        description: Some(description.to_string()),
        start_time,
        end_time,
        location: if location.is_empty() {
            None
        } else {
            Some(location.to_string())
        },
        organizer: user.user_id.to_string(),
        attendees: attendees.clone(),
        reminder_minutes: Some(30), // 30-minute reminder for meetings
        recurrence_rule,
        status: EventStatus::Confirmed,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create the meeting
    let created_event = calendar_engine
        .create_event(event)
        .await
        .map_err(|e| format!("Failed to create meeting: {}", e))?;

    // Send invites to attendees (would integrate with email system)
    for attendee in &attendees {
        send_meeting_invite(state, &created_event, attendee).await?;
    }

    info!(
        "Meeting booked: {} at {} with {} attendees",
        title,
        start_time,
        attendees.len()
    );

    Ok(format!(
        "Meeting '{}' scheduled for {} with {} attendees (ID: {})",
        title,
        start_time.format("%Y-%m-%d %H:%M"),
        attendees.len(),
        created_event.id
    ))
}

async fn check_availability(
    state: &AppState,
    _user: &UserSession,
    date_str: &str,
    duration_minutes: i64,
) -> Result<String, String> {
    let date = parse_date_string(date_str)?;
    let calendar_engine = get_calendar_engine(state).await?;

    // Define business hours (9 AM to 5 PM)
    let business_start = date.with_hour(9).unwrap().with_minute(0).unwrap();
    let business_end = date.with_hour(17).unwrap().with_minute(0).unwrap();

    // Get all events for the day
    let events = calendar_engine
        .get_events_range(business_start, business_end)
        .await
        .map_err(|e| format!("Failed to get events: {}", e))?;

    // Find available slots
    let mut available_slots = Vec::new();
    let mut current_time = business_start;
    let slot_duration = Duration::minutes(duration_minutes);

    for event in &events {
        // Check if there's a gap before this event
        if current_time + slot_duration <= event.start_time {
            available_slots.push(format!(
                "{} - {}",
                current_time.format("%H:%M"),
                (current_time + slot_duration).format("%H:%M")
            ));
        }
        current_time = event.end_time;
    }

    // Check if there's time after the last event
    if current_time + slot_duration <= business_end {
        available_slots.push(format!(
            "{} - {}",
            current_time.format("%H:%M"),
            (current_time + slot_duration).format("%H:%M")
        ));
    }

    if available_slots.is_empty() {
        Ok("No available slots on this date".to_string())
    } else {
        Ok(format!(
            "Available slots on {}: {}",
            date.format("%Y-%m-%d"),
            available_slots.join(", ")
        ))
    }
}

fn parse_time_string(time_str: &str) -> Result<DateTime<Utc>, String> {
    // Try different date formats
    let formats = vec![
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%d/%m/%Y %H:%M",
        "%Y-%m-%dT%H:%M:%S",
    ];

    for format in formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(time_str, format) {
            return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
        }
    }

    // Try parsing relative times like "tomorrow at 3pm"
    if time_str.contains("tomorrow") {
        let tomorrow = Utc::now() + Duration::days(1);
        if let Some(hour) = extract_hour_from_string(time_str) {
            return Ok(tomorrow
                .with_hour(hour)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap());
        }
    }

    // Try parsing relative times like "in 2 hours"
    if time_str.starts_with("in ") {
        if let Ok(hours) = time_str
            .trim_start_matches("in ")
            .trim_end_matches(" hours")
            .trim_end_matches(" hour")
            .parse::<i64>()
        {
            return Ok(Utc::now() + Duration::hours(hours));
        }
    }

    Err(format!("Could not parse time: {}", time_str))
}

fn parse_date_string(date_str: &str) -> Result<DateTime<Utc>, String> {
    // Handle special cases
    if date_str == "today" {
        return Ok(Utc::now());
    } else if date_str == "tomorrow" {
        return Ok(Utc::now() + Duration::days(1));
    }

    // Try standard date formats
    let formats = vec!["%Y-%m-%d", "%Y/%m/%d", "%d/%m/%Y"];

    for format in formats {
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, format) {
            return Ok(dt.and_hms_opt(0, 0, 0).unwrap().and_utc());
        }
    }

    Err(format!("Could not parse date: {}", date_str))
}

fn extract_hour_from_string(s: &str) -> Option<u32> {
    // Extract hour from strings like "3pm", "15:00", "3 PM"
    let s = s.to_lowercase();

    if s.contains("pm") {
        if let Some(hour_str) = s.split("pm").next() {
            if let Ok(hour) = hour_str.trim().replace(":", "").parse::<u32>() {
                return Some(if hour < 12 { hour + 12 } else { hour });
            }
        }
    } else if s.contains("am") {
        if let Some(hour_str) = s.split("am").next() {
            if let Ok(hour) = hour_str.trim().replace(":", "").parse::<u32>() {
                return Some(if hour == 12 { 0 } else { hour });
            }
        }
    }

    None
}

async fn log_booking(
    state: &AppState,
    user: &UserSession,
    event_id: &str,
    title: &str,
) -> Result<(), String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    diesel::sql_query(
        "INSERT INTO booking_logs (id, user_id, bot_id, event_id, event_title, booked_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(&user.user_id)
    .bind::<diesel::sql_types::Uuid, _>(&user.bot_id)
    .bind::<diesel::sql_types::Text, _>(event_id)
    .bind::<diesel::sql_types::Text, _>(title)
    .execute(&mut *conn)
    .map_err(|e| format!("Failed to log booking: {}", e))?;

    Ok(())
}

async fn get_calendar_engine(state: &AppState) -> Result<Arc<CalendarEngine>, String> {
    // Get or create calendar engine from app state
    // This would normally be initialized at startup
    let calendar_engine = Arc::new(CalendarEngine::new(state.conn.clone()));
    Ok(calendar_engine)
}

async fn send_meeting_invite(
    _state: &AppState,
    event: &CalendarEvent,
    attendee: &str,
) -> Result<(), String> {
    // This would integrate with the email system to send calendar invites
    info!(
        "Would send meeting invite for '{}' to {}",
        event.title, attendee
    );
    Ok(())
}
