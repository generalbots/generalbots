diesel::table! {
    bots (id) {
        id -> Uuid,
        branch_id -> Uuid,
        name -> Varchar,
        slug -> Varchar,
        org_id -> Uuid,
        tenant_id -> Nullable<Uuid>,
        is_default_for_branch -> Nullable<Bool>,
        description -> Nullable<Text>,
        is_public -> Nullable<Bool>,
        is_active -> Nullable<Bool>,
        avatar_url -> Nullable<Varchar>,
        settings -> Nullable<Jsonb>,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        llm_provider -> Varchar,
        llm_config -> Jsonb,
        context_provider -> Varchar,
        context_config -> Jsonb,
        database_name -> Nullable<Varchar>,
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
        branch_id -> Uuid,
        title -> Varchar,
        description -> Nullable<Text>,
        owner_id -> Nullable<Uuid>,
        cycle -> Nullable<Varchar>,
        status -> Nullable<Varchar>,
        progress -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        parent_id -> Nullable<Uuid>,
        period -> Varchar,
        period_start -> Nullable<Date>,
        period_end -> Nullable<Date>,
        visibility -> Varchar,
        weight -> Numeric,
        tags -> Text,
    }
}

diesel::table! {
    okr_key_results (id) {
        id -> Uuid,
        branch_id -> Uuid,
        objective_id -> Uuid,
        title -> Varchar,
        start_value -> Nullable<Numeric>,
        target_value -> Numeric,
        current_value -> Nullable<Numeric>,
        unit -> Nullable<Varchar>,
        owner_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        description -> Nullable<Text>,
        metric_type -> Varchar,
        weight -> Numeric,
        status -> Varchar,
        due_date -> Nullable<Date>,
        scoring_type -> Varchar,
    }
}

diesel::table! {
    okr_checkins (id) {
        id -> Uuid,
        branch_id -> Uuid,
        key_result_id -> Uuid,
        value -> Numeric,
        confidence -> Nullable<Int4>,
        notes -> Nullable<Text>,
        checked_in_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        user_id -> Uuid,
        previous_value -> Nullable<Numeric>,
        new_value -> Numeric,
        note -> Nullable<Text>,
        blockers -> Nullable<Text>,
    }
}

diesel::table! {
    okr_templates (id) {
        id -> Uuid,
        branch_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        structure -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        category -> Nullable<Varchar>,
        objective_template -> Jsonb,
        key_result_templates -> Jsonb,
        is_system -> Bool,
        usage_count -> Int4,
        created_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    dashboards (id) {
        id -> Uuid,
        branch_id -> Uuid,
        name -> Varchar,
        slug -> Varchar,
        description -> Nullable<Text>,
        config -> Nullable<Jsonb>,
        is_default -> Nullable<Bool>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        owner_id -> Uuid,
        layout -> Jsonb,
        refresh_interval -> Nullable<Int4>,
        is_public -> Bool,
        is_template -> Bool,
        tags -> Text,
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
        branch_id -> Uuid,
        name -> Varchar,
        source_type -> Varchar,
        config -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        description -> Nullable<Text>,
        connection -> Jsonb,
        schema_definition -> Jsonb,
        refresh_schedule -> Nullable<Varchar>,
        last_sync -> Nullable<Timestamptz>,
        status -> Varchar,
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
