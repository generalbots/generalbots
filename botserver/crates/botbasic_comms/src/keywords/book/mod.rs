pub(crate) use botbasic_types::BasicRuntime;
pub(crate) use botbasic_types::UserSession;
pub(crate) use botcore::shared::schema::calendar_events;
pub(crate) use chrono::{DateTime, Duration, Utc};
pub(crate) use log::info;
pub(crate) use rhai::{Dynamic, Engine};

#[derive(Debug)]
pub struct CalendarEngine {
    db: botcore::shared::utils::DbPool,
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

/// Tenant scope written alongside every calendar row. The three identifiers
/// are `NOT NULL` in the database and none of them has a default, so a write
/// that omits any of them is rejected by PostgreSQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EventScope {
    org_id: Uuid,
    bot_id: Uuid,
    branch_id: Uuid,
}

use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;
#[cfg(test)]
mod booking_persistence_tests {
    use super::*;

    /// Builds a session bound to a real branch in the test database so the
    /// branch FK on `calendars`/`calendar_events` is satisfied. This only picks
    /// a fixture workspace — the resolver still derives the scope exclusively
    /// from the session's verified claims.
    fn session_for_scope(conn: &mut diesel::PgConnection) -> Option<UserSession> {
        #[derive(diesel::QueryableByName)]
        struct ScopeRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            branch_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            org_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            bot_id: Uuid,
        }
        let row = diesel::sql_query(
            "SELECT b.id AS branch_id, b.org_id, \
             COALESCE((SELECT id FROM bots WHERE branch_id = b.id \
                       ORDER BY is_default_for_branch DESC, created_at ASC LIMIT 1), \
                      '00000000-0000-0000-0000-000000000000') AS bot_id \
             FROM branches b ORDER BY b.created_at ASC LIMIT 1",
        )
        .get_result::<ScopeRow>(conn)
        .optional()
        .ok()
        .flatten()?;
        let now = Utc::now();
        Some(UserSession {
            id: Uuid::new_v4(),
            user_id: "fixture".to_string(),
            bot_id: row.bot_id,
            title: String::new(),
            answer_mode: 0,
            context_data: serde_json::json!({}),
            current_tool: None,
            message_count: 0,
            total_tokens: 0,
            created_at: now,
            updated_at: now,
            last_activity: now,
            tenant_id: None,
            active_email_account_id: None,
            branch_id: row.branch_id,
            session_id: None,
            data: Some(serde_json::json!({ "org_id": row.org_id.to_string() })),
            expires_at: None,
            locked: false,
            lock_pin_hash: None,
        })
    }

    /// `BOOK` and `BOOK MEETING` report a confirmed booking and an identifier,
    /// so a booking that was never stored produced a false confirmation and the
    /// conflict check could not see the previous attempt. This round trip stores
    /// a booking and then asks for the same slot again, asserting the stored
    /// event is reported as a conflict (issue #1337).
    ///
    /// The test needs a live database, so it is skipped unless `DATABASE_URL`
    /// points at a schema carrying the calendar tables. The fixture row is
    /// removed afterwards.
    #[test]
    fn a_stored_booking_is_reported_as_a_conflict() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            return;
        };
        let Ok(pool) = diesel::r2d2::Pool::builder()
            .max_size(2)
            .build(diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(database_url))
        else {
            log::warn!("Skipping the booking conflict round trip: DATABASE_URL is unreachable");
            return;
        };

        let engine = CalendarEngine::new(pool.clone());
        let mut setup_conn = pool
            .get()
            .expect("the pool must hand out a connection for the fixture");
        let session =
            session_for_scope(&mut setup_conn).expect("a branch fixture must exist for the test");
        // A decade ahead keeps the fixture out of every real schedule.
        let start = Utc::now() + Duration::days(3650);
        let end = start + Duration::hours(1);

        let booking = CalendarEvent {
            id: Uuid::new_v4(),
            title: "Booking persistence fixture".to_string(),
            description: Some("Booking persistence fixture".to_string()),
            start_time: start,
            end_time: end,
            location: None,
            organizer: Uuid::new_v4().to_string(),
            attendees: Vec::new(),
            reminder_minutes: None,
            recurrence_rule: None,
            status: EventStatus::Confirmed,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let stored = engine
            .create_event(booking, &session)
            .expect("a booking must be persisted, not echoed back");

        let conflicts = engine
            .check_conflicts(start, end, "fixture")
            .expect("the conflict check must succeed");
        assert!(
            conflicts.iter().any(|event| event.id == stored.id),
            "a persisted booking must be visible to the conflict check"
        );

        let mut conn = engine
            .db
            .get()
            .expect("the pool must hand out a connection for cleanup");
        diesel::sql_query("DELETE FROM calendar_events WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(stored.id)
            .execute(&mut conn)
            .ok();
    }
}

mod book_keyword;
mod create_event;
mod execute_book_meeting;
mod resolve_event_scope;

#[cfg(test)]
#[path = "time_parse_tests.rs"]
mod time_parse_tests;

#[cfg(test)]
#[path = "recurrence_tests.rs"]
mod recurrence_tests;

pub use book_keyword::*;
use execute_book_meeting::*;
use resolve_event_scope::*;
