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
    dashboard_widget_data_sources (id) {
        id -> Uuid,
        widget_id -> Uuid,
        data_source_id -> Uuid,
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
    }
}

diesel::joinable!(dashboards -> organizations (org_id));
diesel::joinable!(dashboards -> bots (bot_id));
diesel::joinable!(dashboard_widgets -> dashboards (dashboard_id));
diesel::joinable!(dashboard_data_sources -> organizations (org_id));
diesel::joinable!(dashboard_data_sources -> bots (bot_id));
diesel::joinable!(dashboard_filters -> dashboards (dashboard_id));
diesel::joinable!(dashboard_widget_data_sources -> dashboard_widgets (widget_id));
diesel::joinable!(dashboard_widget_data_sources -> dashboard_data_sources (data_source_id));
diesel::joinable!(conversational_queries -> organizations (org_id));
diesel::joinable!(conversational_queries -> bots (bot_id));
