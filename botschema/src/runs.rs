//! Ordered run event log (#1665).
//!
//! One append-only table. `agent_runs` remains the durable summary; this table
//! holds the frames needed to replay a thread. `seq` is dense per thread and
//! enforced by a unique constraint, so a replay can prove there is no gap.

diesel::table! {
    run_events (id) {
        id -> Uuid,
        v -> Int2,
        thread_id -> Text,
        run_id -> Text,
        parent_run_id -> Nullable<Text>,
        seq -> BigInt,
        event_type -> Varchar,
        event_id -> Varchar,
        source -> Varchar,
        ts -> Timestamptz,
        payload -> Jsonb,
        bot_id -> Uuid,
        user_id -> Nullable<Uuid>,
        org_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}
