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

table! {
    delivery_transactions (id) {
        id -> Uuid,
        bot_id -> Uuid,
        platform -> Text,
        platform_order_id -> Text,
        order_date -> Date,
        customer_name -> Nullable<Text>,
        items -> Nullable<Jsonb>,
        subtotal -> Numeric,
        delivery_fee -> Numeric,
        platform_commission -> Numeric,
        net_value -> Numeric,
        payment_method -> Nullable<Text>,
        status -> Text,
        reconciled -> Bool,
        reconciled_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

table! {
    bank_transactions (id) {
        id -> Uuid,
        bot_id -> Uuid,
        bank -> Nullable<Text>,
        account -> Nullable<Text>,
        transaction_date -> Date,
        description -> Text,
        amount -> Numeric,
        balance -> Nullable<Numeric>,
        category -> Nullable<Text>,
        reconciled -> Bool,
        reconciled_at -> Nullable<Timestamptz>,
        matched_delivery_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

table! {
    reconciliation_rules (id) {
        id -> Uuid,
        bot_id -> Uuid,
        name -> Text,
        match_field -> Text,
        match_operator -> Text,
        match_value -> Text,
        category -> Nullable<Text>,
        auto_reconcile -> Bool,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

table! {
    reconciliation_runs (id) {
        id -> Uuid,
        bot_id -> Uuid,
        status -> Text,
        matched_count -> Integer,
        unmatched_count -> Integer,
        total_amount_matched -> Numeric,
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
    delivery_transactions,
    bank_transactions,
    reconciliation_rules,
    reconciliation_runs,
);
