// @generated automatically by script from migration SQL for issue #707.
// Diesel table definitions for branch-scope cleanup.

diesel::table! {
    user_sessions (id) {
        id -> Uuid,
        branch_id -> Uuid,
        bot_id -> Uuid,
        session_id -> Varchar,
        user_id -> Nullable<Varchar>,
        data -> Nullable<Jsonb>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}


diesel::allow_tables_to_appear_in_same_query!(
    user_sessions,
);
