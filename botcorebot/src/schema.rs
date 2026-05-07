use diesel::table;

diesel::table! {
    bots (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        llm_provider -> Text,
        llm_config -> Jsonb,
        context_provider -> Text,
        context_config -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        is_active -> Nullable<Bool>,
        org_id -> Nullable<Uuid>,
        database_name -> Nullable<Text>,
        is_public -> Bool,
    }
}

diesel::table! {
    user_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        bot_id -> Uuid,
        title -> Text,
        context_data -> Jsonb,
        current_tool -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    message_history (id) {
        id -> Uuid,
        session_id -> Uuid,
        user_id -> Uuid,
        role -> Int4,
        content_encrypted -> Text,
        message_type -> Int4,
        message_index -> Int4,
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
    bot_memories (id) {
        id -> Uuid,
        bot_id -> Uuid,
        key -> Text,
        value -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    bot_shared_memory (id) {
        id -> Uuid,
        source_bot_id -> Uuid,
        target_bot_id -> Uuid,
        memory_key -> Text,
        memory_value -> Text,
        shared_at -> Timestamptz,
    }
}

diesel::table! {
    basic_tools (id) {
        id -> Text,
        bot_id -> Text,
        tool_name -> Text,
        file_path -> Text,
        ast_path -> Text,
        file_hash -> Text,
        mcp_json -> Nullable<Text>,
        tool_json -> Nullable<Text>,
        compiled_at -> Text,
        is_active -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    session_tool_associations (id) {
        id -> Text,
        session_id -> Text,
        tool_name -> Text,
        added_at -> Text,
    }
}

diesel::table! {
    bot_configuration (id) {
        id -> Uuid,
        bot_id -> Uuid,
        config_key -> Text,
        config_value -> Text,
        is_encrypted -> Bool,
        config_type -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    system_automations (id) {
        id -> Uuid,
        bot_id -> Uuid,
        kind -> Int4,
        target -> Nullable<Text>,
        schedule -> Nullable<Text>,
        param -> Text,
        is_active -> Bool,
        last_triggered -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    workflow_executions (id) {
        id -> Uuid,
        bot_id -> Uuid,
        workflow_name -> Text,
        current_step -> Int4,
        state_json -> Text,
        status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workflow_events (id) {
        id -> Uuid,
        workflow_id -> Nullable<Uuid>,
        event_name -> Text,
        event_data_json -> Nullable<Text>,
        processed -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    organizations (org_id) {
        org_id -> Uuid,
        tenant_id -> Uuid,
        name -> Varchar,
        slug -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    tenants (id) {
        id -> Uuid,
        name -> Varchar,
        slug -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    clicks (id) {
        id -> Uuid,
        campaign_id -> Text,
        email -> Text,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    organization_invitations (id) {
        id -> Uuid,
        org_id -> Uuid,
        email -> Varchar,
        role -> Varchar,
        status -> Varchar,
        message -> Nullable<Text>,
        invited_by -> Uuid,
        token -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        expires_at -> Nullable<Timestamptz>,
        accepted_at -> Nullable<Timestamptz>,
        accepted_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    user_organizations (id) {
        id -> Uuid,
        user_id -> Uuid,
        org_id -> Uuid,
        role -> Varchar,
        is_default -> Bool,
        joined_at -> Timestamptz,
    }
}

diesel::table! {
    user_preferences (id) {
        id -> Uuid,
        user_id -> Uuid,
        preference_key -> Varchar,
        preference_value -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_login_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        token_hash -> Varchar,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        last_used -> Timestamptz,
        user_agent -> Nullable<Text>,
        ip_address -> Nullable<Varchar>,
        is_active -> Bool,
    }
}

diesel::table! {
    website_crawls (id) {
        id -> Uuid,
        bot_id -> Uuid,
        url -> Text,
        last_crawled -> Nullable<Timestamptz>,
        next_crawl -> Nullable<Timestamptz>,
        expires_policy -> Varchar,
        max_depth -> Nullable<Int4>,
        max_pages -> Nullable<Int4>,
        crawl_status -> Nullable<Int2>,
        pages_crawled -> Nullable<Int4>,
        error_message -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        refresh_policy -> Nullable<Varchar>,
    }
}

diesel::joinable!(user_organizations -> users (user_id));
diesel::joinable!(user_organizations -> organizations (org_id));
diesel::joinable!(website_crawls -> bots (bot_id));
diesel::joinable!(organization_invitations -> organizations (org_id));

diesel::allow_tables_to_appear_in_same_query!(
    bots,
    user_sessions,
    message_history,
    users,
    bot_memories,
    bot_shared_memory,
    basic_tools,
    session_tool_associations,
    bot_configuration,
    system_automations,
    workflow_executions,
    workflow_events,
    organizations,
    tenants,
    clicks,
    organization_invitations,
    user_organizations,
    user_preferences,
    user_login_tokens,
    website_crawls,
);
