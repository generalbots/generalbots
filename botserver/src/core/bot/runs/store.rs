//! Persistence for the append-only run log (#1665).
//!
//! Writes are idempotent on `event_id`; reads are tenant-scoped by `bot_id`.
//! `agent_runs` keeps the summary, `message_history` keeps only a pointer.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use botevents::{Envelope, EventSource, EventType};
use botschema::{message_history, run_events};
use botlib::db_pool::DbPool;

/// The durable projection of one envelope. Column order matches the table
/// definition in `botschema::runs`.
#[derive(Debug, Queryable)]
pub struct RunEventRow {
    pub id: Uuid,
    pub v: i16,
    pub thread_id: String,
    pub run_id: String,
    pub parent_run_id: Option<String>,
    pub seq: i64,
    pub event_type: String,
    pub event_id: String,
    pub source: String,
    pub ts: DateTime<Utc>,
    pub payload: Value,
    pub bot_id: Uuid,
    pub user_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Appends one frame. A duplicate `event_id` is ignored, which makes transport
/// retries safe.
pub fn append(
    conn: &mut PgConnection,
    envelope: &Envelope,
    bot_id: Uuid,
    user_id: Option<Uuid>,
    org_id: Option<Uuid>,
) -> Result<(), String> {
    let seq = i64::try_from(envelope.seq).map_err(|_| "sequence does not fit i64".to_string())?;
    diesel::insert_into(run_events::table)
        .values((
            run_events::id.eq(Uuid::new_v4()),
            run_events::v.eq(i16::from(envelope.v)),
            run_events::thread_id.eq(&envelope.thread_id),
            run_events::run_id.eq(&envelope.run_id),
            run_events::parent_run_id.eq(envelope.parent_run_id.as_deref()),
            run_events::seq.eq(seq),
            run_events::event_type.eq(envelope.event_type.as_str()),
            run_events::event_id.eq(&envelope.event_id),
            run_events::source.eq(envelope.source.as_str()),
            run_events::ts.eq(envelope.ts),
            run_events::payload.eq(&envelope.data),
            run_events::bot_id.eq(bot_id),
            run_events::user_id.eq(user_id),
            run_events::org_id.eq(org_id),
            run_events::created_at.eq(Utc::now()),
        ))
        .on_conflict(run_events::event_id)
        .do_nothing()
        .execute(conn)
        .map(|_| ())
        .map_err(|error| format!("run_events insert failed: {error}"))
}

/// Highest sequence persisted for a thread, 0 when the log is empty.
pub fn last_seq(conn: &mut PgConnection, bot_id: Uuid, thread_id: &str) -> Result<u64, String> {
    let max: Option<i64> = run_events::table
        .filter(run_events::bot_id.eq(bot_id))
        .filter(run_events::thread_id.eq(thread_id))
        .select(diesel::dsl::max(run_events::seq))
        .first(conn)
        .map_err(|error| format!("run_events max(seq) failed: {error}"))?;
    Ok(max.and_then(|value| u64::try_from(value).ok()).unwrap_or(0))
}

/// Loads frames after `after_seq`, ordered by `seq`.
pub fn load_thread(
    conn: &mut PgConnection,
    bot_id: Uuid,
    thread_id: &str,
    after_seq: u64,
) -> Result<Vec<Envelope>, String> {
    let after = i64::try_from(after_seq).unwrap_or(i64::MAX);
    let rows: Vec<RunEventRow> = run_events::table
        .filter(run_events::bot_id.eq(bot_id))
        .filter(run_events::thread_id.eq(thread_id))
        .filter(run_events::seq.gt(after))
        .order(run_events::seq.asc())
        .load(conn)
        .map_err(|error| format!("run_events load failed: {error}"))?;
    rows.into_iter().map(row_to_envelope).collect()
}

/// Points the newest message of a session at the run that produced it.
pub fn attach_message_run(
    conn: &mut PgConnection,
    session_id: Uuid,
    run_id: Uuid,
    thread_id: &str,
) -> Result<usize, String> {
    let target = message_history::table
        .filter(message_history::session_id.eq(session_id))
        .select(message_history::id)
        .order(message_history::message_index.desc())
        .limit(1);
    diesel::update(message_history::table.filter(message_history::id.eq_any(target)))
        .set((
            message_history::run_id.eq(Some(run_id)),
            message_history::thread_id.eq(Some(thread_id.to_string())),
        ))
        .execute(conn)
        .map_err(|error| format!("message_history run pointer failed: {error}"))
}

/// Deletes frames older than the configured retention window. The summary in
/// `agent_runs` is never touched.
pub fn cleanup_expired(pool: &DbPool) -> Result<usize, String> {
    let days: i64 = std::env::var("GB_RUN_EVENTS_RETENTION_DAYS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(30);
    if days <= 0 {
        return Ok(0);
    }
    let cutoff = Utc::now() - chrono::Duration::days(days);
    let mut conn = pool
        .get()
        .map_err(|error| format!("pool error: {error}"))?;
    diesel::delete(run_events::table.filter(run_events::created_at.lt(cutoff)))
        .execute(&mut conn)
        .map_err(|error| format!("run_events retention failed: {error}"))
}

fn row_to_envelope(row: RunEventRow) -> Result<Envelope, String> {
    let event_type: EventType = serde_json::from_value(Value::String(row.event_type.clone()))
        .map_err(|error| format!("unknown event type {}: {error}", row.event_type))?;
    let source: EventSource = serde_json::from_value(Value::String(row.source.clone()))
        .map_err(|error| format!("unknown event source {}: {error}", row.source))?;
    let seq = u64::try_from(row.seq).map_err(|_| "sequence is negative".to_string())?;
    Ok(Envelope {
        v: u8::try_from(row.v).unwrap_or(1),
        event_type,
        thread_id: row.thread_id,
        run_id: row.run_id,
        parent_run_id: row.parent_run_id,
        seq,
        event_id: row.event_id,
        ts: row.ts,
        source,
        data: row.payload,
    })
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    use botevents::{Envelope, EventSource, EventType};

    use super::{row_to_envelope, RunEventRow};

    #[test]
    fn row_round_trips_to_envelope() {
        let row = RunEventRow {
            id: Uuid::new_v4(),
            v: 1,
            thread_id: "thread-1".to_string(),
            run_id: "run-1".to_string(),
            parent_run_id: None,
            seq: 3,
            event_type: "STEP_STARTED".to_string(),
            event_id: "a".repeat(64),
            source: "runtime".to_string(),
            ts: Utc::now(),
            payload: json!({ "step_id": "s1", "name": "prepare" }),
            bot_id: Uuid::new_v4(),
            user_id: None,
            org_id: None,
            created_at: Utc::now(),
        };
        let envelope = row_to_envelope(row).expect("row converts");
        assert_eq!(envelope.event_type, EventType::StepStarted);
        assert_eq!(envelope.source, EventSource::Runtime);
        assert_eq!(envelope.seq, 3);
        assert_eq!(envelope.thread_id, "thread-1");
    }

    #[test]
    fn unknown_event_type_is_rejected() {
        let row = RunEventRow {
            id: Uuid::new_v4(),
            v: 1,
            thread_id: "t".to_string(),
            run_id: "r".to_string(),
            parent_run_id: None,
            seq: 1,
            event_type: "NOT_A_FRAME".to_string(),
            event_id: "a".repeat(64),
            source: "runtime".to_string(),
            ts: Utc::now(),
            payload: json!({}),
            bot_id: Uuid::new_v4(),
            user_id: None,
            org_id: None,
            created_at: Utc::now(),
        };
        assert!(row_to_envelope(row).is_err());
    }

    #[test]
    fn envelope_header_stays_stable() {
        let envelope = Envelope {
            v: 1,
            event_type: EventType::RunStarted,
            thread_id: "t".to_string(),
            run_id: "r".to_string(),
            parent_run_id: None,
            seq: 1,
            event_id: "b".repeat(64),
            ts: Utc::now(),
            source: EventSource::Runtime,
            data: json!({}),
        };
        let value = serde_json::to_value(&envelope).expect("serializes");
        assert_eq!(value["type"], "RUN_STARTED");
    }
}
