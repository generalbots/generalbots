diesel::table! {
    organizations (org_id) {
        org_id -> Uuid,
        name -> Text,
        slug -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    bots (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        llm_provider -> Varchar,
        llm_config -> Jsonb,
        context_provider -> Varchar,
        context_config -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        is_active -> Nullable<Bool>,
        tenant_id -> Nullable<Uuid>,
        database_name -> Nullable<Varchar>,
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
        message_index -> Int8,
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
    clicks (id) {
        id -> Uuid,
        campaign_id -> Text,
        email -> Text,
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
    rbac_roles (id) {
        id -> Uuid,
        name -> Varchar,
        display_name -> Varchar,
        description -> Nullable<Text>,
        is_system -> Bool,
        is_active -> Bool,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    rbac_groups (id) {
        id -> Uuid,
        name -> Varchar,
        display_name -> Varchar,
        description -> Nullable<Text>,
        parent_group_id -> Nullable<Uuid>,
        is_active -> Bool,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    rbac_permissions (id) {
        id -> Uuid,
        name -> Varchar,
        display_name -> Varchar,
        description -> Nullable<Text>,
        resource_type -> Varchar,
        action -> Varchar,
        category -> Varchar,
        is_system -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    rbac_role_permissions (id) {
        id -> Uuid,
        role_id -> Uuid,
        permission_id -> Uuid,
        granted_by -> Nullable<Uuid>,
        granted_at -> Timestamptz,
    }
}

diesel::table! {
    rbac_user_roles (id) {
        id -> Uuid,
        user_id -> Uuid,
        role_id -> Uuid,
        granted_by -> Nullable<Uuid>,
        granted_at -> Timestamptz,
        expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    rbac_user_groups (id) {
        id -> Uuid,
        user_id -> Uuid,
        group_id -> Uuid,
        added_by -> Nullable<Uuid>,
        added_at -> Timestamptz,
    }
}

diesel::table! {
    rbac_group_roles (id) {
        id -> Uuid,
        group_id -> Uuid,
        role_id -> Uuid,
        granted_by -> Nullable<Uuid>,
        granted_at -> Timestamptz,
    }
}

diesel::joinable!(rbac_role_permissions -> rbac_roles (role_id));
diesel::joinable!(rbac_role_permissions -> rbac_permissions (permission_id));
diesel::joinable!(rbac_user_roles -> users (user_id));
diesel::joinable!(rbac_user_roles -> rbac_roles (role_id));
diesel::joinable!(rbac_user_groups -> users (user_id));
diesel::joinable!(rbac_user_groups -> rbac_groups (group_id));
diesel::joinable!(rbac_group_roles -> rbac_groups (group_id));
diesel::joinable!(rbac_group_roles -> rbac_roles (role_id));

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

diesel::allow_tables_to_appear_in_same_query!(
    rbac_roles,
    rbac_groups,
    rbac_permissions,
    rbac_role_permissions,
    rbac_user_roles,
    rbac_user_groups,
    rbac_group_roles,
    users,
);
