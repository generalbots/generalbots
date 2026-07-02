// @generated automatically by script from migration SQL for issue #707.
// Diesel table definitions for branch-scope cleanup.

diesel::table! {
    dashboards (id) {
        id -> Uuid,
        branch_id -> Uuid,
        name -> Varchar,
        slug -> Varchar,
        description -> Nullable<Text>,
        config -> Nullable<Jsonb>,
        is_default -> Nullable<Bool>,
        owner_id -> Uuid,
        layout -> Jsonb,
        refresh_interval -> Nullable<Int4>,
        is_public -> Bool,
        is_template -> Bool,
        tags -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dashboard_data_sources (id) {
        id -> Uuid,
        branch_id -> Uuid,
        name -> Varchar,
        source_type -> Varchar,
        config -> Nullable<Jsonb>,
        description -> Nullable<Text>,
        schema_definition -> Nullable<Jsonb>,
        refresh_schedule -> Nullable<Varchar>,
        last_sync -> Nullable<Timestamptz>,
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dashboard_widgets (id) {
        id -> Uuid,
        dashboard_id -> Uuid,
        widget_type -> Varchar,
        title -> Varchar,
        position_x -> Int4,
        position_y -> Int4,
        width -> Int4,
        height -> Int4,
        config -> Jsonb,
        data_query -> Nullable<Jsonb>,
        style -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dashboard_filters (id) {
        id -> Uuid,
        dashboard_id -> Uuid,
        name -> Varchar,
        field -> Varchar,
        filter_type -> Varchar,
        default_value -> Nullable<Jsonb>,
        options -> Jsonb,
        linked_widgets -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    conversational_queries (id) {
        id -> Uuid,
        branch_id -> Uuid,
        dashboard_id -> Nullable<Uuid>,
        user_id -> Uuid,
        query_text -> Text,
        result -> Nullable<Jsonb>,
        generated_query -> Nullable<Text>,
        executed_at -> Timestamptz,
        execution_ms -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    dashboards,
    dashboard_data_sources,
    dashboard_widgets,
    dashboard_filters,
    conversational_queries,
);
