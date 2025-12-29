// @generated automatically by Diesel CLI.
// This schema matches the consolidated migration 20250101000000_consolidated_schema

diesel::table! {
    shard_config (shard_id) {
        shard_id -> Int2,
        region_code -> Bpchar,
        datacenter -> Varchar,
        connection_string -> Text,
        is_primary -> Nullable<Bool>,
        is_active -> Nullable<Bool>,
        min_tenant_id -> Int8,
        max_tenant_id -> Int8,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    tenant_shard_map (tenant_id) {
        tenant_id -> Int8,
        shard_id -> Int2,
        region_code -> Bpchar,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    tenants (id) {
        id -> Int8,
        shard_id -> Int2,
        external_id -> Nullable<Uuid>,
        name -> Varchar,
        slug -> Varchar,
        region_code -> Bpchar,
        plan_tier -> Int2,
        settings -> Nullable<Jsonb>,
        limits -> Nullable<Jsonb>,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Nullable<Varchar>,
        phone_number -> Nullable<Varchar>,
        display_name -> Nullable<Varchar>,
        avatar_url -> Nullable<Varchar>,
        locale -> Nullable<Bpchar>,
        timezone -> Nullable<Varchar>,
        is_active -> Nullable<Bool>,
        last_login_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    bots (id) {
        id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        name -> Varchar,
        description -> Nullable<Text>,
        llm_provider -> Int2,
        llm_config -> Nullable<Jsonb>,
        context_provider -> Int2,
        context_config -> Nullable<Jsonb>,
        system_prompt -> Nullable<Text>,
        personality -> Nullable<Jsonb>,
        capabilities -> Nullable<Jsonb>,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    bot_configuration (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        config_key -> Varchar,
        config_value -> Text,
        value_type -> Int2,
        is_secret -> Nullable<Bool>,
        vault_path -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    bot_channels (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        channel_type -> Int2,
        channel_identifier -> Nullable<Varchar>,
        config -> Nullable<Jsonb>,
        credentials_vault_path -> Nullable<Varchar>,
        is_active -> Nullable<Bool>,
        last_activity_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    user_sessions (id) {
        id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        user_id -> Uuid,
        bot_id -> Uuid,
        channel_type -> Int2,
        title -> Nullable<Varchar>,
        context_data -> Nullable<Jsonb>,
        current_tool -> Nullable<Varchar>,
        answer_mode -> Nullable<Int2>,
        message_count -> Nullable<Int4>,
        total_tokens -> Nullable<Int4>,
        last_activity_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    message_history (id) {
        id -> Uuid,
        session_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        user_id -> Uuid,
        role -> Int2,
        message_type -> Int2,
        content_encrypted -> Text,
        content_hash -> Nullable<Bpchar>,
        media_url -> Nullable<Varchar>,
        metadata -> Nullable<Jsonb>,
        token_count -> Nullable<Int4>,
        processing_time_ms -> Nullable<Int4>,
        llm_model -> Nullable<Varchar>,
        message_index -> Int4,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    bot_memories (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        user_id -> Nullable<Uuid>,
        session_id -> Nullable<Uuid>,
        memory_type -> Int2,
        content -> Text,
        embedding_id -> Nullable<Varchar>,
        importance_score -> Nullable<Float4>,
        access_count -> Nullable<Int4>,
        last_accessed_at -> Nullable<Timestamptz>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    auto_tasks (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        session_id -> Nullable<Uuid>,
        title -> Varchar,
        intent -> Text,
        status -> Int2,
        execution_mode -> Int2,
        priority -> Int2,
        plan_id -> Nullable<Uuid>,
        basic_program -> Nullable<Text>,
        current_step -> Nullable<Int4>,
        total_steps -> Nullable<Int4>,
        progress -> Nullable<Float4>,
        step_results -> Nullable<Jsonb>,
        error_message -> Nullable<Text>,
        started_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    execution_plans (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        task_id -> Nullable<Uuid>,
        intent -> Text,
        intent_type -> Nullable<Int2>,
        confidence -> Nullable<Float4>,
        status -> Int2,
        steps -> Jsonb,
        context -> Nullable<Jsonb>,
        basic_program -> Nullable<Text>,
        simulation_result -> Nullable<Jsonb>,
        risk_level -> Nullable<Int2>,
        approved_by -> Nullable<Uuid>,
        approved_at -> Nullable<Timestamptz>,
        executed_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    task_approvals (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        task_id -> Uuid,
        plan_id -> Nullable<Uuid>,
        step_index -> Nullable<Int4>,
        action_type -> Varchar,
        action_description -> Text,
        risk_level -> Nullable<Int2>,
        status -> Int2,
        decision -> Nullable<Int2>,
        decision_reason -> Nullable<Text>,
        decided_by -> Nullable<Uuid>,
        decided_at -> Nullable<Timestamptz>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    task_decisions (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        task_id -> Uuid,
        question -> Text,
        options -> Jsonb,
        context -> Nullable<Jsonb>,
        status -> Int2,
        selected_option -> Nullable<Varchar>,
        decision_reason -> Nullable<Text>,
        decided_by -> Nullable<Uuid>,
        decided_at -> Nullable<Timestamptz>,
        timeout_seconds -> Nullable<Int4>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    safety_audit_log (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        task_id -> Nullable<Uuid>,
        plan_id -> Nullable<Uuid>,
        action_type -> Varchar,
        action_details -> Jsonb,
        constraint_checks -> Nullable<Jsonb>,
        simulation_result -> Nullable<Jsonb>,
        risk_assessment -> Nullable<Jsonb>,
        outcome -> Int2,
        error_message -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    intent_classifications (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        session_id -> Nullable<Uuid>,
        original_text -> Text,
        intent_type -> Int2,
        confidence -> Float4,
        entities -> Nullable<Jsonb>,
        suggested_name -> Nullable<Varchar>,
        was_correct -> Nullable<Bool>,
        corrected_type -> Nullable<Int2>,
        feedback -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    generated_apps (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        name -> Varchar,
        description -> Nullable<Text>,
        domain -> Nullable<Varchar>,
        intent_source -> Nullable<Text>,
        pages -> Nullable<Jsonb>,
        tables_created -> Nullable<Jsonb>,
        tools -> Nullable<Jsonb>,
        schedulers -> Nullable<Jsonb>,
        app_path -> Nullable<Varchar>,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    designer_changes (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        session_id -> Nullable<Uuid>,
        change_type -> Int2,
        description -> Text,
        file_path -> Varchar,
        original_content -> Text,
        new_content -> Text,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    designer_pending_changes (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        session_id -> Nullable<Uuid>,
        analysis_json -> Text,
        instruction -> Text,
        expires_at -> Timestamptz,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    kb_collections (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        name -> Varchar,
        description -> Nullable<Text>,
        folder_path -> Nullable<Varchar>,
        qdrant_collection -> Nullable<Varchar>,
        document_count -> Nullable<Int4>,
        chunk_count -> Nullable<Int4>,
        total_tokens -> Nullable<Int4>,
        last_indexed_at -> Nullable<Timestamptz>,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    kb_documents (id) {
        id -> Uuid,
        collection_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        file_path -> Varchar,
        file_name -> Varchar,
        file_type -> Nullable<Varchar>,
        file_size -> Nullable<Int8>,
        content_hash -> Nullable<Bpchar>,
        chunk_count -> Nullable<Int4>,
        is_indexed -> Nullable<Bool>,
        indexed_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    session_kb_associations (id) {
        id -> Uuid,
        session_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        bot_id -> Uuid,
        kb_name -> Varchar,
        kb_folder_path -> Nullable<Varchar>,
        qdrant_collection -> Nullable<Varchar>,
        added_by_tool -> Nullable<Varchar>,
        is_active -> Nullable<Bool>,
        added_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    kb_sources (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        name -> Varchar,
        source_type -> Varchar,
        connection_config -> Jsonb,
        sync_schedule -> Nullable<Varchar>,
        last_sync_at -> Nullable<Timestamptz>,
        sync_status -> Nullable<Int2>,
        document_count -> Nullable<Int4>,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    tools (id) {
        id -> Uuid,
        bot_id -> Nullable<Uuid>,
        tenant_id -> Int8,
        shard_id -> Int2,
        name -> Varchar,
        description -> Text,
        parameters -> Nullable<Jsonb>,
        script -> Text,
        tool_type -> Nullable<Varchar>,
        is_system -> Nullable<Bool>,
        is_active -> Nullable<Bool>,
        usage_count -> Nullable<Int8>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    system_automations (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        name -> Nullable<Varchar>,
        kind -> Int2,
        target -> Nullable<Varchar>,
        schedule -> Nullable<Varchar>,
        param -> Nullable<Varchar>,
        is_active -> Nullable<Bool>,
        last_triggered -> Nullable<Timestamptz>,
        next_trigger -> Nullable<Timestamptz>,
        run_count -> Nullable<Int8>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    pending_info (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        field_name -> Varchar,
        field_label -> Varchar,
        field_type -> Varchar,
        reason -> Nullable<Text>,
        config_key -> Varchar,
        is_filled -> Nullable<Bool>,
        filled_at -> Nullable<Timestamptz>,
        filled_value -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    usage_analytics (id) {
        id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        user_id -> Uuid,
        bot_id -> Uuid,
        session_id -> Nullable<Uuid>,
        date -> Date,
        session_count -> Nullable<Int4>,
        message_count -> Nullable<Int4>,
        total_tokens -> Nullable<Int4>,
        total_processing_time_ms -> Nullable<Int8>,
        avg_response_time_ms -> Nullable<Int4>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    analytics_events (id) {
        id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        user_id -> Nullable<Uuid>,
        session_id -> Nullable<Uuid>,
        bot_id -> Nullable<Uuid>,
        event_type -> Varchar,
        event_data -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    tasks (id) {
        id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        bot_id -> Nullable<Uuid>,
        title -> Varchar,
        description -> Nullable<Text>,
        assignee_id -> Nullable<Uuid>,
        reporter_id -> Nullable<Uuid>,
        project_id -> Nullable<Uuid>,
        parent_task_id -> Nullable<Uuid>,
        status -> Int2,
        priority -> Int2,
        due_date -> Nullable<Timestamptz>,
        estimated_hours -> Nullable<Float4>,
        actual_hours -> Nullable<Float4>,
        progress -> Nullable<Int2>,
        tags -> Nullable<Array<Nullable<Text>>>,
        dependencies -> Nullable<Array<Nullable<Uuid>>>,
        completed_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    task_comments (id) {
        id -> Uuid,
        task_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        author_id -> Uuid,
        content -> Text,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    connected_accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        provider -> Varchar,
        provider_user_id -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        display_name -> Nullable<Varchar>,
        access_token_vault -> Nullable<Varchar>,
        refresh_token_vault -> Nullable<Varchar>,
        token_expires_at -> Nullable<Timestamptz>,
        scopes -> Nullable<Array<Nullable<Text>>>,
        sync_status -> Nullable<Int2>,
        last_sync_at -> Nullable<Timestamptz>,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    session_account_associations (id) {
        id -> Uuid,
        session_id -> Uuid,
        bot_id -> Uuid,
        account_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        email -> Nullable<Varchar>,
        provider -> Nullable<Varchar>,
        qdrant_collection -> Nullable<Varchar>,
        is_active -> Nullable<Bool>,
        added_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    whatsapp_numbers (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        phone_number -> Varchar,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    clicks (campaign_id, email) {
        campaign_id -> Varchar,
        email -> Varchar,
        tenant_id -> Int8,
        click_count -> Nullable<Int4>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    table_role_access (id) {
        id -> Uuid,
        bot_id -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        table_name -> Varchar,
        role_name -> Varchar,
        can_read -> Nullable<Bool>,
        can_write -> Nullable<Bool>,
        can_delete -> Nullable<Bool>,
        row_filter -> Nullable<Jsonb>,
        column_filter -> Nullable<Array<Nullable<Text>>>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    context_injections (id) {
        id -> Uuid,
        session_id -> Uuid,
        injected_by -> Uuid,
        tenant_id -> Int8,
        shard_id -> Int2,
        context_data -> Jsonb,
        reason -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    organizations (org_id) {
        org_id -> Uuid,
        tenant_id -> Int8,
        name -> Varchar,
        slug -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    user_organizations (id) {
        id -> Uuid,
        user_id -> Uuid,
        org_id -> Uuid,
        role -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
    }
}

// Foreign key relationships
diesel::joinable!(tenant_shard_map -> shard_config (shard_id));
diesel::joinable!(users -> tenants (tenant_id));
diesel::joinable!(bots -> tenants (tenant_id));
diesel::joinable!(bot_configuration -> bots (bot_id));
diesel::joinable!(bot_channels -> bots (bot_id));
diesel::joinable!(user_sessions -> users (user_id));
diesel::joinable!(user_sessions -> bots (bot_id));
diesel::joinable!(user_sessions -> tenants (tenant_id));
diesel::joinable!(message_history -> user_sessions (session_id));
diesel::joinable!(message_history -> users (user_id));
diesel::joinable!(bot_memories -> bots (bot_id));
diesel::joinable!(auto_tasks -> bots (bot_id));
diesel::joinable!(auto_tasks -> user_sessions (session_id));
diesel::joinable!(execution_plans -> bots (bot_id));
diesel::joinable!(execution_plans -> auto_tasks (task_id));
diesel::joinable!(task_approvals -> bots (bot_id));
diesel::joinable!(task_approvals -> auto_tasks (task_id));
diesel::joinable!(task_decisions -> bots (bot_id));
diesel::joinable!(task_decisions -> auto_tasks (task_id));
diesel::joinable!(safety_audit_log -> bots (bot_id));
diesel::joinable!(intent_classifications -> bots (bot_id));
diesel::joinable!(generated_apps -> bots (bot_id));
diesel::joinable!(designer_changes -> bots (bot_id));
diesel::joinable!(designer_pending_changes -> bots (bot_id));
diesel::joinable!(kb_collections -> bots (bot_id));
diesel::joinable!(kb_documents -> kb_collections (collection_id));
diesel::joinable!(session_kb_associations -> user_sessions (session_id));
diesel::joinable!(session_kb_associations -> bots (bot_id));
diesel::joinable!(kb_sources -> bots (bot_id));
diesel::joinable!(system_automations -> bots (bot_id));
diesel::joinable!(pending_info -> bots (bot_id));
diesel::joinable!(usage_analytics -> users (user_id));
diesel::joinable!(usage_analytics -> bots (bot_id));
diesel::joinable!(task_comments -> tasks (task_id));
diesel::joinable!(task_comments -> users (author_id));
diesel::joinable!(connected_accounts -> users (user_id));
diesel::joinable!(session_account_associations -> user_sessions (session_id));
diesel::joinable!(session_account_associations -> bots (bot_id));
diesel::joinable!(session_account_associations -> connected_accounts (account_id));
diesel::joinable!(whatsapp_numbers -> bots (bot_id));
diesel::joinable!(table_role_access -> bots (bot_id));
diesel::joinable!(context_injections -> user_sessions (session_id));
diesel::joinable!(organizations -> tenants (tenant_id));
diesel::joinable!(user_organizations -> users (user_id));
diesel::joinable!(user_organizations -> organizations (org_id));

diesel::allow_tables_to_appear_in_same_query!(
    shard_config,
    tenant_shard_map,
    tenants,
    users,
    bots,
    bot_configuration,
    bot_channels,
    user_sessions,
    message_history,
    bot_memories,
    auto_tasks,
    execution_plans,
    task_approvals,
    task_decisions,
    safety_audit_log,
    intent_classifications,
    generated_apps,
    designer_changes,
    designer_pending_changes,
    kb_collections,
    kb_documents,
    session_kb_associations,
    kb_sources,
    tools,
    system_automations,
    pending_info,
    usage_analytics,
    analytics_events,
    tasks,
    task_comments,
    connected_accounts,
    session_account_associations,
    whatsapp_numbers,
    clicks,
    table_role_access,
    context_injections,
    organizations,
    user_organizations,
);
