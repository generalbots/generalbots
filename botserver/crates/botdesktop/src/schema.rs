diesel::table! {
    desktop_connections (id) {
        id -> Uuid,
        user_id -> Uuid,
        name -> Varchar,
        host -> Varchar,
        port -> Int4,
        protocol -> Varchar,
        auth_type -> Nullable<Varchar>,
        auto_connect -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_used_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    desktop_connection_log (id) {
        id -> Uuid,
        connection_id -> Nullable<Uuid>,
        user_id -> Uuid,
        session_id -> Uuid,
        host -> Varchar,
        port -> Int4,
        protocol -> Varchar,
        connected_at -> Timestamptz,
        disconnected_at -> Nullable<Timestamptz>,
        bytes_transferred -> Nullable<Int8>,
        disconnect_reason -> Nullable<Varchar>,
    }
}

diesel::joinable!(desktop_connection_log -> desktop_connections (connection_id));

diesel::allow_tables_to_appear_in_same_query!(desktop_connections, desktop_connection_log);
