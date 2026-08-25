//! Diesel schema for botagent tables (migrations 6.5.82, 6.5.83, 6.5.84).

diesel::table! {
    agent_sessions (id) {
        id -> Uuid,
        session_id -> Varchar,
        user_id -> Uuid,
        org_id -> Nullable<Uuid>,
        branch_id -> Nullable<Uuid>,
        bot_id -> Uuid,
        vm_name -> Varchar,
        status -> Varchar,
        last_active_at -> Timestamptz,
        expires_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    agent_snapshots (id) {
        id -> Uuid,
        agent_session_id -> Uuid,
        label -> Nullable<Varchar>,
        incus_snapshot -> Varchar,
        size_bytes -> Nullable<BigInt>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    org_api_keys (id) {
        id -> Uuid,
        org_id -> Uuid,
        name -> Varchar,
        key_hash -> Varchar,
        key_prefix -> Varchar,
        scopes -> Jsonb,
        last_used_at -> Nullable<Timestamptz>,
        revoked_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    sandbox_runs (id) {
        id -> Uuid,
        org_id -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
        language -> Varchar,
        status -> Varchar,
        exit_code -> Nullable<Int4>,
        stdout_ref -> Nullable<Text>,
        stderr_ref -> Nullable<Text>,
        duration_ms -> Nullable<Int4>,
        created_at -> Timestamptz,
    }
}
