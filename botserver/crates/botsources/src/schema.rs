use diesel::prelude::*;

table! {
    knowledge_sources (id) {
        id -> Text,
        name -> Text,
        source_type -> Text,
        file_path -> Nullable<Text>,
        url -> Nullable<Text>,
        content_hash -> Text,
        chunk_count -> Integer,
        status -> Text,
        collection -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        indexed_at -> Nullable<Timestamptz>,
    }
}

table! {
    knowledge_chunks (id) {
        id -> Text,
        source_id -> Text,
        chunk_index -> Integer,
        content -> Text,
        token_count -> Integer,
        created_at -> Timestamptz,
    }
}

table! {
    bot_configuration (id) {
        id -> Uuid,
        bot_id -> Uuid,
        config_key -> Text,
        config_value -> Text,
        config_type -> Text,
        is_encrypted -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    connectors (id) {
        id -> Uuid,
        bot_id -> Uuid,
        name -> Text,
        connector_type -> Text,
        description -> Nullable<Text>,
        auth_config -> Jsonb,
        schedule -> Nullable<Text>,
        is_active -> Bool,
        last_sync_at -> Nullable<Timestamptz>,
        last_sync_status -> Nullable<Text>,
        error_log -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    connector_endpoints (id) {
        id -> Uuid,
        connector_id -> Uuid,
        name -> Text,
        method -> Text,
        url -> Text,
        headers -> Nullable<Jsonb>,
        sync_direction -> Text,
        field_mapping -> Jsonb,
        schedule -> Nullable<Text>,
        pagination -> Nullable<Jsonb>,
        last_sync_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

table! {
    connector_sync_logs (id) {
        id -> Uuid,
        connector_id -> Uuid,
        endpoint_id -> Nullable<Uuid>,
        status -> Text,
        records_synced -> BigInt,
        records_failed -> BigInt,
        duration_ms -> BigInt,
        error_message -> Nullable<Text>,
        started_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

allow_tables_to_appear_in_same_query!(
    knowledge_sources,
    knowledge_chunks,
    bot_configuration,
    connectors,
    connector_endpoints,
    connector_sync_logs,
);
