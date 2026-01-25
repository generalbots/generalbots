use crate::core::shared::schema::core::{organizations, bots};

diesel::table! {
    calendars (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        owner_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        color -> Nullable<Varchar>,
        timezone -> Nullable<Varchar>,
        is_primary -> Bool,
        is_visible -> Bool,
        is_shared -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_events (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        calendar_id -> Uuid,
        owner_id -> Uuid,
        title -> Varchar,
        description -> Nullable<Text>,
        location -> Nullable<Varchar>,
        start_time -> Timestamptz,
        end_time -> Timestamptz,
        all_day -> Bool,
        recurrence_rule -> Nullable<Text>,
        recurrence_id -> Nullable<Uuid>,
        color -> Nullable<Varchar>,
        status -> Varchar,
        visibility -> Varchar,
        busy_status -> Varchar,
        reminders -> Jsonb,
        attendees -> Jsonb,
        conference_data -> Nullable<Jsonb>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_event_attendees (id) {
        id -> Uuid,
        event_id -> Uuid,
        email -> Varchar,
        name -> Nullable<Varchar>,
        status -> Varchar,
        role -> Varchar,
        rsvp_time -> Nullable<Timestamptz>,
        comment -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_event_reminders (id) {
        id -> Uuid,
        event_id -> Uuid,
        reminder_type -> Varchar,
        minutes_before -> Int4,
        is_sent -> Bool,
        sent_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_shares (id) {
        id -> Uuid,
        calendar_id -> Uuid,
        shared_with_user_id -> Nullable<Uuid>,
        shared_with_email -> Nullable<Varchar>,
        permission -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(calendars -> organizations (org_id));
diesel::joinable!(calendars -> bots (bot_id));
diesel::joinable!(calendar_events -> organizations (org_id));
diesel::joinable!(calendar_events -> bots (bot_id));
diesel::joinable!(calendar_events -> calendars (calendar_id));
diesel::joinable!(calendar_event_attendees -> calendar_events (event_id));
diesel::joinable!(calendar_event_reminders -> calendar_events (event_id));
diesel::joinable!(calendar_shares -> calendars (calendar_id));
