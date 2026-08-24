//! Diesel schema for the automation crate. Column definitions mirror
//! migration `6.5.85-agent-schedules` exactly.

diesel::table! {
    agent_schedules (id) {
        id -> Uuid,
        org_id -> Nullable<Uuid>,
        branch_id -> Nullable<Uuid>,
        bot_id -> Uuid,
        title -> Text,
        goal -> Text,
        cron_expr -> Varchar,
        timezone -> Varchar,
        owner_user_id -> Uuid,
        delivery -> Jsonb,
        enabled -> Bool,
        max_runtime_secs -> Int4,
        tool_allowlist -> Nullable<Jsonb>,
        next_run_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    agent_runs (id) {
        id -> Uuid,
        schedule_id -> Nullable<Uuid>,
        bot_id -> Uuid,
        trigger_kind -> Varchar,
        status -> Varchar,
        plan -> Nullable<Jsonb>,
        steps -> Nullable<Jsonb>,
        result_summary -> Nullable<Text>,
        artifacts -> Nullable<Jsonb>,
        verdict -> Nullable<Jsonb>,
        delivery_status -> Nullable<Jsonb>,
        error -> Nullable<Text>,
        started_at -> Nullable<Timestamptz>,
        finished_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    agent_spans (id) {
        id -> Uuid,
        run_id -> Uuid,
        parent_id -> Nullable<Uuid>,
        kind -> Varchar,
        name -> Text,
        input_ref -> Nullable<Text>,
        output_ref -> Nullable<Text>,
        tokens_in -> Nullable<Int4>,
        tokens_out -> Nullable<Int4>,
        vm_seconds -> Nullable<Int4>,
        status -> Varchar,
        error -> Nullable<Text>,
        started_at -> Nullable<Timestamptz>,
        finished_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    compute_usage_hourly (org_id, hour, resource) {
        org_id -> Uuid,
        hour -> Timestamp,
        resource -> Varchar,
        quantity -> Double,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(agent_runs -> agent_schedules (schedule_id));
diesel::joinable!(agent_spans -> agent_runs (run_id));

diesel::allow_tables_to_appear_in_same_query!(
    agent_schedules,
    agent_runs,
    agent_spans,
    compute_usage_hourly,
);
