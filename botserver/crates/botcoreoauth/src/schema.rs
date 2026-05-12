diesel::table! {
    bots (id) {
        id -> Uuid,
        name -> Varchar,
        is_active -> Bool,
    }
}

diesel::table! {
    bot_configuration (bot_id, config_key) {
        bot_id -> Uuid,
        config_key -> Varchar,
        config_value -> Text,
        is_encrypted -> Bool,
        config_type -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Text,
        is_active -> Bool,
        is_admin -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        bot_id -> Uuid,
        started_at -> Timestamptz,
        is_active -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    bots,
    bot_configuration,
    users,
    user_sessions,
);
