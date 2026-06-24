diesel::table! {
    user_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        bot_id -> Uuid,
        title -> Varchar,
        context_data -> Jsonb,
        attendant_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    message_history (id) {
        id -> Uuid,
        session_id -> Uuid,
        user_id -> Uuid,
        role -> Int4,
        content_encrypted -> Text,
        message_type -> Int4,
        message_index -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    bots (id) {
        id -> Uuid,
        org_id -> Nullable<Uuid>,
        name -> Varchar,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        email -> Varchar,
    }
}

diesel::table! {
    attendance_webhooks (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        webhook_url -> Varchar,
        events -> Array<Text>,
        is_active -> Bool,
        secret_key -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    attendance_sla_policies (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        channel -> Nullable<Varchar>,
        priority -> Nullable<Varchar>,
        first_response_minutes -> Int4,
        resolution_minutes -> Int4,
        escalate_on_breach -> Bool,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_sla_events (id) {
        id -> Uuid,
        session_id -> Uuid,
        sla_policy_id -> Uuid,
        event_type -> Varchar,
        due_at -> Timestamptz,
        status -> Varchar,
        breached_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}
