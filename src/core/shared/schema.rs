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
    kb_documents (id) {
        id -> Text,
        bot_id -> Text,
        user_id -> Text,
        collection_name -> Text,
        file_path -> Text,
        file_size -> Integer,
        file_hash -> Text,
        first_published_at -> Text,
        last_modified_at -> Text,
        indexed_at -> Nullable<Text>,
        metadata -> Text,
        created_at -> Text,
        updated_at -> Text,
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
    kb_collections (id) {
        id -> Text,
        bot_id -> Text,
        user_id -> Text,
        name -> Text,
        folder_path -> Text,
        qdrant_collection -> Text,
        document_count -> Integer,
        is_active -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    user_kb_associations (id) {
        id -> Text,
        user_id -> Text,
        bot_id -> Text,
        kb_name -> Text,
        is_website -> Integer,
        website_url -> Nullable<Text>,
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
    user_email_accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        email -> Varchar,
        display_name -> Nullable<Varchar>,
        imap_server -> Varchar,
        imap_port -> Int4,
        smtp_server -> Varchar,
        smtp_port -> Int4,
        username -> Varchar,
        password_encrypted -> Text,
        is_primary -> Bool,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    email_drafts (id) {
        id -> Uuid,
        user_id -> Uuid,
        account_id -> Uuid,
        to_address -> Text,
        cc_address -> Nullable<Text>,
        bcc_address -> Nullable<Text>,
        subject -> Nullable<Varchar>,
        body -> Nullable<Text>,
        attachments -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    email_folders (id) {
        id -> Uuid,
        account_id -> Uuid,
        folder_name -> Varchar,
        folder_path -> Varchar,
        unread_count -> Int4,
        total_count -> Int4,
        last_synced -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
    tasks (id) {
        id -> Uuid,
        title -> Text,
        description -> Nullable<Text>,
        status -> Text,
        priority -> Text,
        assignee_id -> Nullable<Uuid>,
        reporter_id -> Nullable<Uuid>,
        project_id -> Nullable<Uuid>,
        due_date -> Nullable<Timestamptz>,
        tags -> Array<Text>,
        dependencies -> Array<Uuid>,
        estimated_hours -> Nullable<Float8>,
        actual_hours -> Nullable<Float8>,
        progress -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}



diesel::table! {
    global_email_signatures (id) {
        id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        content_html -> Text,
        content_plain -> Text,
        position -> Varchar,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    email_signatures (id) {
        id -> Uuid,
        user_id -> Uuid,
        bot_id -> Nullable<Uuid>,
        name -> Varchar,
        content_html -> Text,
        content_plain -> Text,
        is_default -> Bool,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    scheduled_emails (id) {
        id -> Uuid,
        user_id -> Uuid,
        bot_id -> Uuid,
        to_addresses -> Text,
        cc_addresses -> Nullable<Text>,
        bcc_addresses -> Nullable<Text>,
        subject -> Text,
        body_html -> Text,
        body_plain -> Nullable<Text>,
        attachments_json -> Text,
        scheduled_at -> Timestamptz,
        sent_at -> Nullable<Timestamptz>,
        status -> Varchar,
        retry_count -> Int4,
        error_message -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    email_templates (id) {
        id -> Uuid,
        bot_id -> Uuid,
        user_id -> Nullable<Uuid>,
        name -> Varchar,
        description -> Nullable<Text>,
        subject_template -> Text,
        body_html_template -> Text,
        body_plain_template -> Nullable<Text>,
        variables_json -> Text,
        category -> Nullable<Varchar>,
        is_shared -> Bool,
        usage_count -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    email_auto_responders (id) {
        id -> Uuid,
        user_id -> Uuid,
        bot_id -> Uuid,
        responder_type -> Varchar,
        subject -> Text,
        body_html -> Text,
        body_plain -> Nullable<Text>,
        start_date -> Nullable<Timestamptz>,
        end_date -> Nullable<Timestamptz>,
        send_to_internal_only -> Bool,
        exclude_addresses -> Nullable<Text>,
        is_active -> Bool,
        stalwart_sieve_id -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    email_rules (id) {
        id -> Uuid,
        user_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        priority -> Int4,
        conditions_json -> Text,
        actions_json -> Text,
        stop_processing -> Bool,
        is_active -> Bool,
        stalwart_sieve_id -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    email_labels (id) {
        id -> Uuid,
        user_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        color -> Varchar,
        parent_id -> Nullable<Uuid>,
        is_system -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    email_label_assignments (id) {
        id -> Uuid,
        email_message_id -> Varchar,
        label_id -> Uuid,
        assigned_at -> Timestamptz,
    }
}

diesel::table! {
    distribution_lists (id) {
        id -> Uuid,
        bot_id -> Uuid,
        owner_id -> Uuid,
        name -> Varchar,
        email_alias -> Nullable<Varchar>,
        description -> Nullable<Text>,
        members_json -> Text,
        is_public -> Bool,
        stalwart_principal_id -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    shared_mailboxes (id) {
        id -> Uuid,
        bot_id -> Uuid,
        email_address -> Varchar,
        display_name -> Varchar,
        description -> Nullable<Text>,
        settings_json -> Text,
        stalwart_account_id -> Nullable<Varchar>,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    shared_mailbox_members (id) {
        id -> Uuid,
        mailbox_id -> Uuid,
        user_id -> Uuid,
        permission_level -> Varchar,
        added_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    organizations,
    bots,
    system_automations,
    user_sessions,
    message_history,
    users,
    clicks,
    bot_memories,
    kb_documents,
    basic_tools,
    kb_collections,
    user_kb_associations,
    session_tool_associations,
    bot_configuration,
    user_email_accounts,
    email_drafts,
    email_folders,
    user_preferences,
    user_login_tokens,
    tasks,
    global_email_signatures,
    email_signatures,
    scheduled_emails,
    email_templates,
    email_auto_responders,
    email_rules,
    email_labels,
    email_label_assignments,
    distribution_lists,
    shared_mailboxes,
    shared_mailbox_members,
);
