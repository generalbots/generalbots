diesel::table! {
    desktop_connections (id) {
        id -> Uuid,
        user_id -> Uuid,
        target_host -> Varchar,
        target_port -> Int4,
        session_token -> Varchar,
        status -> Varchar,
        created_at -> Timestamptz,
        last_active_at -> Timestamptz,
        disconnected_at -> Nullable<Timestamptz>,
        bytes_sent -> Int8,
        bytes_received -> Int8,
        client_ip -> Varchar,
    }
}

diesel::table! {
    desktop_connection_log (id) {
        id -> Uuid,
        connection_id -> Uuid,
        event_type -> Varchar,
        message -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(desktop_connection_log -> desktop_connections (connection_id));

diesel::allow_tables_to_appear_in_same_query!(desktop_connections, desktop_connection_log);
