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
        password_hash -> Text,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
