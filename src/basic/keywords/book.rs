use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct TimeSlot {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    available: bool,
}

pub fn book_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["BOOK", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                // Parse attendees (array or single email)
                let attendees_input = context.eval_expression_tree(&inputs[0])?;
                let mut attendees = Vec::new();

                if attendees_input.is_array() {
                    let arr = attendees_input.cast::<rhai::Array>();
                    for item in arr.iter() {
                        attendees.push(item.to_string());
                    }
                } else {
                    attendees.push(attendees_input.to_string());
                }

                let date_range = context.eval_expression_tree(&inputs[1])?.to_string();
                let duration = context.eval_expression_tree(&inputs[2])?;

                let duration_minutes = if duration.is_int() {
                    duration.as_int().unwrap_or(30)
                } else {
                    duration.to_string().parse::<i64>().unwrap_or(30)
                };

                trace!(
                    "BOOK: attendees={:?}, date_range={}, duration={} for user={}",
                    attendees,
                    date_range,
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
                            execute_booking(
                                &state_for_task,
                                &user_for_task,
                                attendees,
                                &date_range,
                                duration_minutes as i32,
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
                    Ok(Ok(booking_id)) => Ok(Dynamic::from(booking_id)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("BOOK failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "BOOK timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("BOOK thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();

    // Register FIND_SLOT keyword to find available slots
    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    engine
        .register_custom_syntax(
            &["FIND_SLOT", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let attendees_input = context.eval_expression_tree(&inputs[0])?;
                let mut attendees = Vec::new();

                if attendees_input.is_array() {
                    let arr = attendees_input.cast::<rhai::Array>();
                    for item in arr.iter() {
                        attendees.push(item.to_string());
                    }
                } else {
                    attendees.push(attendees_input.to_string());
                }

                let duration = context.eval_expression_tree(&inputs[1])?;
                let preferences = context.eval_expression_tree(&inputs[2])?.to_string();

                let duration_minutes = if duration.is_int() {
                    duration.as_int().unwrap_or(30)
                } else {
                    duration.to_string().parse::<i64>().unwrap_or(30)
                };

                let state_for_task = Arc::clone(&state_clone2);
                let user_for_task = user_clone2.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            find_available_slot(
                                &state_for_task,
                                &user_for_task,
                                attendees,
                                duration_minutes as i32,
                                &preferences,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send FIND_SLOT result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(slot)) => Ok(Dynamic::from(slot)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("FIND_SLOT failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "FIND_SLOT timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

async fn execute_booking(
    state: &AppState,
    user: &UserSession,
    attendees: Vec<String>,
    date_range: &str,
    duration_minutes: i32,
) -> Result<String, String> {
    // Parse date range
    let (start_search, end_search) = parse_date_range(date_range)?;

    // Find available slot
    let available_slot = find_common_availability(
        state,
        &attendees,
        start_search,
        end_search,
        duration_minutes,
    )
    .await?;

    // Create calendar event
    let event_id = create_calendar_event(
        state,
        user,
        &attendees,
        available_slot.start,
        available_slot.end,
        "Meeting",
        None,
    )
    .await?;

    // Send invitations
    for attendee in &attendees {
        send_calendar_invite(state, &event_id, attendee).await?;
    }

    Ok(format!(
        "Meeting booked for {} at {}",
        available_slot.start.format("%Y-%m-%d %H:%M"),
        event_id
    ))
}

async fn find_available_slot(
    state: &AppState,
    _user: &UserSession,
    attendees: Vec<String>,
    duration_minutes: i32,
    preferences: &str,
) -> Result<String, String> {
    // Parse preferences (e.g., "mornings preferred", "afternoons only", "next week")
    let (start_search, end_search) = if preferences.contains("tomorrow") {
        let tomorrow = Utc::now() + Duration::days(1);
        (
            tomorrow
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            tomorrow
                .date_naive()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc(),
        )
    } else if preferences.contains("next week") {
        let now = Utc::now();
        let next_week = now + Duration::days(7);
        (now, next_week)
    } else {
        // Default to next 7 days
        let now = Utc::now();
        (now, now + Duration::days(7))
    };

    let slot = find_common_availability(
        state,
        &attendees,
        start_search,
        end_search,
        duration_minutes,
    )
    .await?;

    Ok(slot.start.format("%Y-%m-%d %H:%M").to_string())
}

async fn find_common_availability(
    state: &AppState,
    attendees: &[String],
    start_search: DateTime<Utc>,
    end_search: DateTime<Utc>,
    duration_minutes: i32,
) -> Result<TimeSlot, String> {
    // This would integrate with actual calendar API
    // For now, simulate finding an available slot

    let mut current = start_search;

    while current < end_search {
        // Skip weekends
        if current.weekday().num_days_from_monday() >= 5 {
            current = current + Duration::days(1);
            continue;
        }

        // Check business hours (9 AM - 5 PM)
        let hour = current.hour();
        if hour >= 9 && hour < 17 {
            // Check if slot is available for all attendees
            let slot_end = current + Duration::minutes(duration_minutes as i64);

            if slot_end.hour() <= 17 {
                // In a real implementation, check each attendee's calendar
                // For now, simulate availability check
                if check_slot_availability(state, attendees, current, slot_end).await? {
                    return Ok(TimeSlot {
                        start: current,
                        end: slot_end,
                        available: true,
                    });
                }
            }
        }

        // Move to next slot (30 minute intervals)
        current = current + Duration::minutes(30);
    }

    Err("No available slot found in the specified date range".to_string())
}

async fn check_slot_availability(
    _state: &AppState,
    _attendees: &[String],
    _start: DateTime<Utc>,
    _end: DateTime<Utc>,
) -> Result<bool, String> {
    // Simulate calendar availability check
    // In real implementation, this would query calendar API

    // For demo, randomly return availability
    let random = (Utc::now().timestamp() % 3) == 0;
    Ok(random)
}

async fn create_calendar_event(
    state: &AppState,
    user: &UserSession,
    attendees: &[String],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    subject: &str,
    description: Option<String>,
) -> Result<String, String> {
    let event_id = Uuid::new_v4().to_string();

    // Store in database
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let user_id_str = user.user_id.to_string();
    let bot_id_str = user.bot_id.to_string();
    let attendees_json = json!(attendees);
    let now = Utc::now();

    let query = diesel::sql_query(
        "INSERT INTO calendar_events (id, user_id, bot_id, subject, description, start_time, end_time, attendees, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind::<diesel::sql_types::Text, _>(&event_id)
    .bind::<diesel::sql_types::Text, _>(&user_id_str)
    .bind::<diesel::sql_types::Text, _>(&bot_id_str)
    .bind::<diesel::sql_types::Text, _>(subject)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&description)
    .bind::<diesel::sql_types::Timestamptz, _>(&start)
    .bind::<diesel::sql_types::Timestamptz, _>(&end)
    .bind::<diesel::sql_types::Jsonb, _>(&attendees_json)
    .bind::<diesel::sql_types::Timestamptz, _>(&now);

    use diesel::RunQueryDsl;
    query.execute(&mut *conn).map_err(|e| {
        error!("Failed to create calendar event: {}", e);
        format!("Failed to create calendar event: {}", e)
    })?;

    trace!("Created calendar event: {}", event_id);
    Ok(event_id)
}

async fn send_calendar_invite(
    _state: &AppState,
    event_id: &str,
    attendee: &str,
) -> Result<(), String> {
    // In real implementation, send actual calendar invite via email or calendar API
    trace!(
        "Sending calendar invite for event {} to {}",
        event_id,
        attendee
    );
    Ok(())
}

fn parse_date_range(date_range: &str) -> Result<(DateTime<Utc>, DateTime<Utc>), String> {
    let range_lower = date_range.to_lowercase();
    let now = Utc::now();

    if range_lower.contains("today") {
        Ok((
            now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
            now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc(),
        ))
    } else if range_lower.contains("tomorrow") {
        let tomorrow = now + Duration::days(1);
        Ok((
            tomorrow
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            tomorrow
                .date_naive()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc(),
        ))
    } else if range_lower.contains("this week") || range_lower.contains("this_week") {
        Ok((
            now,
            now + Duration::days(7 - now.weekday().num_days_from_monday() as i64),
        ))
    } else if range_lower.contains("next week") || range_lower.contains("next_week") {
        let next_monday = now + Duration::days(7 - now.weekday().num_days_from_monday() as i64 + 1);
        Ok((next_monday, next_monday + Duration::days(6)))
    } else if range_lower.contains("2pm") || range_lower.contains("14:00") {
        // Handle specific time
        let target_time = now.date_naive().and_hms_opt(14, 0, 0).unwrap().and_utc();
        Ok((target_time, target_time + Duration::hours(1)))
    } else {
        // Default to next 7 days
        Ok((now, now + Duration::days(7)))
    }
}
