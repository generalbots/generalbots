// Diesel table definitions for dashboards (migrations 6.0.19 + 6.5.23).

diesel::table! {
    dashboards (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        owner_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        layout -> Jsonb,
        refresh_interval -> Nullable<Int4>,
        is_public -> Bool,
        is_template -> Bool,
        tags -> Array<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        branch_id -> Uuid,
    }
}

diesel::table! {
    dashboard_data_sources (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        source_type -> Varchar,
        connection -> Jsonb,
        schema_definition -> Jsonb,
        refresh_schedule -> Nullable<Varchar>,
        last_sync -> Nullable<Timestamptz>,
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        branch_id -> Uuid,
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
        org_id -> Uuid,
        bot_id -> Uuid,
        dashboard_id -> Nullable<Uuid>,
        user_id -> Uuid,
        natural_language -> Text,
        generated_query -> Nullable<Text>,
        result_widget_config -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        branch_id -> Uuid,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    dashboards,
    dashboard_data_sources,
    dashboard_widgets,
    dashboard_filters,
    conversational_queries,
);
