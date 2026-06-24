use diesel::table;

table! {
    user_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        bot_id -> Nullable<Uuid>,
        context_data -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        email -> Nullable<Text>,
        phone_number -> Nullable<Text>,
        display_name -> Nullable<Text>,
        password_hash -> Text,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    message_history (id) {
        id -> Uuid,
        bot_id -> Uuid,
        user_id -> Nullable<Uuid>,
        session_id -> Nullable<Uuid>,
        phone_number -> Nullable<Text>,
        direction -> Text,
        content -> Text,
        message_type -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

table! {
    bots (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
    }
}

table! {
    bot_configuration (id) {
        id -> Uuid,
        bot_id -> Uuid,
        key -> Text,
        value -> Text,
    }
}
