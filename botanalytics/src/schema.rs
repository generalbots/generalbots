diesel::table! {
    bots (id) {
        id -> Uuid,
        org_id -> Nullable<Uuid>,
        name -> Varchar,
        description -> Nullable<Text>,
        llm_provider -> Varchar,
        llm_config -> Jsonb,
        context_provider -> Varchar,
        context_config -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        is_active -> Nullable<Bool>,
        database_name -> Nullable<Varchar>,
        is_public -> Bool,
    }
}

diesel::table! {
    organizations (org_id) {
        org_id -> Uuid,
        tenant_id -> Uuid,
        name -> Text,
        slug -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        is_active -> Bool,
        is_admin -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    okr_objectives (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        owner_id -> Uuid,
        parent_id -> Nullable<Uuid>,
        title -> Varchar,
        description -> Nullable<Text>,
        period -> Varchar,
        period_start -> Nullable<Date>,
        period_end -> Nullable<Date>,
        status -> Varchar,
        progress -> Numeric,
        visibility -> Varchar,
        weight -> Numeric,
        tags -> Array<Nullable<Text>>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    okr_key_results (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        objective_id -> Uuid,
        owner_id -> Uuid,
        title -> Varchar,
        description -> Nullable<Text>,
        metric_type -> Varchar,
        start_value -> Numeric,
        target_value -> Numeric,
        current_value -> Numeric,
        unit -> Nullable<Varchar>,
        weight -> Numeric,
        status -> Varchar,
        due_date -> Nullable<Date>,
        scoring_type -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    okr_checkins (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        key_result_id -> Uuid,
        user_id -> Uuid,
        previous_value -> Nullable<Numeric>,
        new_value -> Numeric,
        note -> Nullable<Text>,
        confidence -> Nullable<Varchar>,
        blockers -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    okr_templates (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        category -> Nullable<Varchar>,
        objective_template -> Jsonb,
        key_result_templates -> Jsonb,
        is_system -> Bool,
        usage_count -> Int4,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

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

diesel::joinable!(bots -> organizations (org_id));
diesel::joinable!(okr_objectives -> organizations (org_id));
diesel::joinable!(okr_objectives -> bots (bot_id));
diesel::joinable!(okr_key_results -> organizations (org_id));
diesel::joinable!(okr_key_results -> bots (bot_id));
diesel::joinable!(okr_key_results -> okr_objectives (objective_id));
diesel::joinable!(okr_checkins -> organizations (org_id));
diesel::joinable!(okr_checkins -> bots (bot_id));
diesel::joinable!(okr_checkins -> okr_key_results (key_result_id));
diesel::joinable!(okr_templates -> organizations (org_id));
diesel::joinable!(okr_templates -> bots (bot_id));
diesel::joinable!(dashboards -> organizations (org_id));
diesel::joinable!(dashboards -> bots (bot_id));
diesel::joinable!(dashboard_widgets -> dashboards (dashboard_id));
diesel::joinable!(dashboard_data_sources -> organizations (org_id));
diesel::joinable!(dashboard_data_sources -> bots (bot_id));
diesel::joinable!(dashboard_filters -> dashboards (dashboard_id));

diesel::allow_tables_to_appear_in_same_query!(
    bots,
    organizations,
    users,
    okr_objectives,
    okr_key_results,
    okr_checkins,
    okr_templates,
    dashboards,
    dashboard_widgets,
    dashboard_data_sources,
    dashboard_filters,
);
