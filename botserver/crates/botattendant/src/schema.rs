diesel::table! {
    attendant_queues (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        priority -> Int4,
        max_wait_minutes -> Int4,
        auto_assign -> Bool,
        working_hours -> Jsonb,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_sessions (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        session_number -> Varchar,
        channel -> Varchar,
        customer_id -> Nullable<Uuid>,
        customer_name -> Nullable<Varchar>,
        customer_email -> Nullable<Varchar>,
        customer_phone -> Nullable<Varchar>,
        status -> Varchar,
        priority -> Int4,
        agent_id -> Nullable<Uuid>,
        queue_id -> Nullable<Uuid>,
        subject -> Nullable<Varchar>,
        initial_message -> Nullable<Text>,
        started_at -> Timestamptz,
        assigned_at -> Nullable<Timestamptz>,
        first_response_at -> Nullable<Timestamptz>,
        ended_at -> Nullable<Timestamptz>,
        wait_time_seconds -> Nullable<Int4>,
        handle_time_seconds -> Nullable<Int4>,
        satisfaction_rating -> Nullable<Int4>,
        satisfaction_comment -> Nullable<Text>,
        tags -> Array<Text>,
        metadata -> Jsonb,
        notes -> Nullable<Text>,
        transfer_count -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_session_messages (id) {
        id -> Uuid,
        session_id -> Uuid,
        sender_type -> Varchar,
        sender_id -> Nullable<Uuid>,
        sender_name -> Nullable<Varchar>,
        content -> Text,
        content_type -> Varchar,
        attachments -> Jsonb,
        is_internal -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_queue_agents (id) {
        id -> Uuid,
        queue_id -> Uuid,
        agent_id -> Uuid,
        max_concurrent -> Int4,
        priority -> Int4,
        skills -> Array<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_agent_status (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        agent_id -> Uuid,
        status -> Varchar,
        status_message -> Nullable<Varchar>,
        current_sessions -> Int4,
        max_sessions -> Int4,
        last_activity_at -> Timestamptz,
        break_started_at -> Nullable<Timestamptz>,
        break_reason -> Nullable<Varchar>,
        available_since -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_transfers (id) {
        id -> Uuid,
        session_id -> Uuid,
        from_agent_id -> Nullable<Uuid>,
        to_agent_id -> Nullable<Uuid>,
        to_queue_id -> Nullable<Uuid>,
        reason -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_canned_responses (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        title -> Varchar,
        content -> Text,
        shortcut -> Nullable<Varchar>,
        category -> Nullable<Varchar>,
        queue_id -> Nullable<Uuid>,
        is_active -> Bool,
        usage_count -> Int4,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_tags (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        color -> Nullable<Varchar>,
        description -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_wrap_up_codes (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        code -> Varchar,
        name -> Varchar,
        description -> Nullable<Text>,
        requires_notes -> Bool,
        is_active -> Bool,
        created_at -> Timestamptz,
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
