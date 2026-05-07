use diesel::table;

table! {
    tasks (id) {
        id -> Uuid,
        bot_id -> Uuid,
        title -> Text,
        description -> Nullable<Text>,
        status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        user_id -> Nullable<Uuid>,
    }
}

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
    auto_tasks (id) {
        id -> Uuid,
        bot_id -> Uuid,
        title -> Text,
        description -> Nullable<Text>,
        schedule -> Nullable<Text>,
        enabled -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
