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
    crm_contacts (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        mobile -> Nullable<Varchar>,
        company -> Nullable<Varchar>,
        job_title -> Nullable<Varchar>,
        source -> Nullable<Varchar>,
        status -> Varchar,
        tags -> Array<Text>,
        custom_fields -> Jsonb,
        address_line1 -> Nullable<Varchar>,
        address_line2 -> Nullable<Varchar>,
        city -> Nullable<Varchar>,
        state -> Nullable<Varchar>,
        postal_code -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        owner_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    crm_accounts (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        website -> Nullable<Varchar>,
        industry -> Nullable<Varchar>,
        employees_count -> Nullable<Int4>,
        annual_revenue -> Nullable<Float8>,
        phone -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        address_line1 -> Nullable<Varchar>,
        address_line2 -> Nullable<Varchar>,
        city -> Nullable<Varchar>,
        state -> Nullable<Varchar>,
        postal_code -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        description -> Nullable<Text>,
        tags -> Array<Text>,
        custom_fields -> Jsonb,
        owner_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    crm_pipeline_stages (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        stage_order -> Int4,
        probability -> Int4,
        is_won -> Bool,
        is_lost -> Bool,
        color -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    crm_leads (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        contact_id -> Nullable<Uuid>,
        account_id -> Nullable<Uuid>,
        title -> Varchar,
        description -> Nullable<Text>,
        value -> Nullable<Float8>,
        currency -> Nullable<Varchar>,
        stage_id -> Nullable<Uuid>,
        stage -> Varchar,
        probability -> Int4,
        source -> Nullable<Varchar>,
        expected_close_date -> Nullable<Date>,
        owner_id -> Nullable<Uuid>,
        lost_reason -> Nullable<Varchar>,
        tags -> Array<Text>,
        custom_fields -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        closed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    crm_opportunities (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        lead_id -> Nullable<Uuid>,
        account_id -> Nullable<Uuid>,
        contact_id -> Nullable<Uuid>,
        name -> Varchar,
        description -> Nullable<Text>,
        value -> Nullable<Float8>,
        currency -> Nullable<Varchar>,
        stage_id -> Nullable<Uuid>,
        stage -> Varchar,
        probability -> Int4,
        source -> Nullable<Varchar>,
        expected_close_date -> Nullable<Date>,
        actual_close_date -> Nullable<Date>,
        won -> Nullable<Bool>,
        owner_id -> Nullable<Uuid>,
        tags -> Array<Text>,
        custom_fields -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    crm_activities (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        contact_id -> Nullable<Uuid>,
        lead_id -> Nullable<Uuid>,
        opportunity_id -> Nullable<Uuid>,
        account_id -> Nullable<Uuid>,
        activity_type -> Varchar,
        subject -> Nullable<Varchar>,
        description -> Nullable<Text>,
        due_date -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        outcome -> Nullable<Varchar>,
        owner_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    crm_notes (id) {
        id -> Uuid,
        org_id -> Uuid,
        contact_id -> Nullable<Uuid>,
        lead_id -> Nullable<Uuid>,
        opportunity_id -> Nullable<Uuid>,
        account_id -> Nullable<Uuid>,
        content -> Text,
        author_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    support_tickets (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        ticket_number -> Varchar,
        subject -> Varchar,
        description -> Nullable<Text>,
        status -> Varchar,
        priority -> Varchar,
        category -> Nullable<Varchar>,
        source -> Varchar,
        requester_id -> Nullable<Uuid>,
        requester_email -> Nullable<Varchar>,
        requester_name -> Nullable<Varchar>,
        assignee_id -> Nullable<Uuid>,
        team_id -> Nullable<Uuid>,
        due_date -> Nullable<Timestamptz>,
        first_response_at -> Nullable<Timestamptz>,
        resolved_at -> Nullable<Timestamptz>,
        closed_at -> Nullable<Timestamptz>,
        satisfaction_rating -> Nullable<Int4>,
        tags -> Array<Text>,
        custom_fields -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    ticket_comments (id) {
        id -> Uuid,
        ticket_id -> Uuid,
        author_id -> Nullable<Uuid>,
        author_name -> Nullable<Varchar>,
        author_email -> Nullable<Varchar>,
        content -> Text,
        is_internal -> Bool,
        attachments -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    ticket_sla_policies (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        priority -> Varchar,
        first_response_hours -> Int4,
        resolution_hours -> Int4,
        business_hours_only -> Bool,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    ticket_canned_responses (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        title -> Varchar,
        content -> Text,
        category -> Nullable<Varchar>,
        shortcut -> Nullable<Varchar>,
        created_by -> Nullable<Uuid>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    ticket_categories (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        parent_id -> Nullable<Uuid>,
        color -> Nullable<Varchar>,
        icon -> Nullable<Varchar>,
        sort_order -> Int4,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    ticket_tags (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        color -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    billing_invoices (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        invoice_number -> Varchar,
        customer_id -> Nullable<Uuid>,
        customer_name -> Varchar,
        customer_email -> Nullable<Varchar>,
        customer_address -> Nullable<Text>,
        status -> Varchar,
        issue_date -> Date,
        due_date -> Date,
        subtotal -> Numeric,
        tax_rate -> Numeric,
        tax_amount -> Numeric,
        discount_percent -> Numeric,
        discount_amount -> Numeric,
        total -> Numeric,
        amount_paid -> Numeric,
        amount_due -> Numeric,
        currency -> Varchar,
        notes -> Nullable<Text>,
        terms -> Nullable<Text>,
        footer -> Nullable<Text>,
        paid_at -> Nullable<Timestamptz>,
        sent_at -> Nullable<Timestamptz>,
        voided_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    billing_invoice_items (id) {
        id -> Uuid,
        invoice_id -> Uuid,
        product_id -> Nullable<Uuid>,
        description -> Varchar,
        quantity -> Numeric,
        unit_price -> Numeric,
        discount_percent -> Numeric,
        tax_rate -> Numeric,
        amount -> Numeric,
        sort_order -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    billing_payments (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        invoice_id -> Nullable<Uuid>,
        payment_number -> Varchar,
        amount -> Numeric,
        currency -> Varchar,
        payment_method -> Varchar,
        payment_reference -> Nullable<Varchar>,
        status -> Varchar,
        payer_name -> Nullable<Varchar>,
        payer_email -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        paid_at -> Timestamptz,
        refunded_at -> Nullable<Timestamptz>,
        refund_amount -> Nullable<Numeric>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    billing_quotes (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        quote_number -> Varchar,
        customer_id -> Nullable<Uuid>,
        customer_name -> Varchar,
        customer_email -> Nullable<Varchar>,
        customer_address -> Nullable<Text>,
        status -> Varchar,
        issue_date -> Date,
        valid_until -> Date,
        subtotal -> Numeric,
        tax_rate -> Numeric,
        tax_amount -> Numeric,
        discount_percent -> Numeric,
        discount_amount -> Numeric,
        total -> Numeric,
        currency -> Varchar,
        notes -> Nullable<Text>,
        terms -> Nullable<Text>,
        accepted_at -> Nullable<Timestamptz>,
        rejected_at -> Nullable<Timestamptz>,
        converted_invoice_id -> Nullable<Uuid>,
        sent_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    billing_quote_items (id) {
        id -> Uuid,
        quote_id -> Uuid,
        product_id -> Nullable<Uuid>,
        description -> Varchar,
        quantity -> Numeric,
        unit_price -> Numeric,
        discount_percent -> Numeric,
        tax_rate -> Numeric,
        amount -> Numeric,
        sort_order -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    billing_recurring (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        customer_id -> Nullable<Uuid>,
        customer_name -> Varchar,
        customer_email -> Nullable<Varchar>,
        status -> Varchar,
        frequency -> Varchar,
        interval_count -> Int4,
        amount -> Numeric,
        currency -> Varchar,
        description -> Nullable<Text>,
        next_invoice_date -> Date,
        last_invoice_date -> Nullable<Date>,
        last_invoice_id -> Nullable<Uuid>,
        start_date -> Date,
        end_date -> Nullable<Date>,
        invoices_generated -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    billing_tax_rates (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        rate -> Numeric,
        description -> Nullable<Text>,
        region -> Nullable<Varchar>,
        is_default -> Bool,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    products (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        sku -> Nullable<Varchar>,
        name -> Varchar,
        description -> Nullable<Text>,
        category -> Nullable<Varchar>,
        product_type -> Varchar,
        price -> Numeric,
        cost -> Nullable<Numeric>,
        currency -> Varchar,
        tax_rate -> Numeric,
        unit -> Varchar,
        stock_quantity -> Int4,
        low_stock_threshold -> Int4,
        is_active -> Bool,
        images -> Jsonb,
        attributes -> Jsonb,
        weight -> Nullable<Numeric>,
        dimensions -> Nullable<Jsonb>,
        barcode -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        // Campos fiscais e tributários (Stone/Brasil)
        ncm -> Nullable<Varchar>,
        cest -> Nullable<Varchar>,
        cfop -> Nullable<Varchar>,
        origem -> Nullable<Int4>,
        gtin -> Nullable<Varchar>,
        gtin_tributavel -> Nullable<Varchar>,
        // Dimensões detalhadas (frete)
        peso_liquido -> Nullable<Numeric>,
        peso_bruto -> Nullable<Numeric>,
        largura -> Nullable<Numeric>,
        altura -> Nullable<Numeric>,
        comprimento -> Nullable<Numeric>,
        volumes -> Nullable<Int4>,
        // Informações tributárias
        icms_cst -> Nullable<Varchar>,
        icms_aliquota -> Nullable<Numeric>,
        ipi_cst -> Nullable<Varchar>,
        ipi_aliquota -> Nullable<Numeric>,
        pis_cst -> Nullable<Varchar>,
        pis_aliquota -> Nullable<Numeric>,
        cofins_cst -> Nullable<Varchar>,
        cofins_aliquota -> Nullable<Numeric>,
        // Marketplace e e-commerce
        marca -> Nullable<Varchar>,
        modelo -> Nullable<Varchar>,
        cor -> Nullable<Varchar>,
        tamanho -> Nullable<Varchar>,
        material -> Nullable<Varchar>,
        genero -> Nullable<Varchar>,
        // Controle de estoque avançado
        localizacao_estoque -> Nullable<Varchar>,
        lote -> Nullable<Varchar>,
        data_validade -> Nullable<Date>,
        data_fabricacao -> Nullable<Date>,
        estoque_minimo -> Nullable<Int4>,
        estoque_maximo -> Nullable<Int4>,
        ponto_reposicao -> Nullable<Int4>,
        // Preços e custos detalhados
        preco_promocional -> Nullable<Numeric>,
        promocao_inicio -> Nullable<Timestamptz>,
        promocao_fim -> Nullable<Timestamptz>,
        custo_frete -> Nullable<Numeric>,
        margem_lucro -> Nullable<Numeric>,
        // Campos Stone específicos
        stone_item_id -> Nullable<Varchar>,
        stone_category_id -> Nullable<Varchar>,
        stone_metadata -> Nullable<Jsonb>,
        // SEO e busca
        slug -> Nullable<Varchar>,
        meta_title -> Nullable<Varchar>,
        meta_description -> Nullable<Text>,
        tags -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    services (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        category -> Nullable<Varchar>,
        service_type -> Varchar,
        hourly_rate -> Nullable<Numeric>,
        fixed_price -> Nullable<Numeric>,
        currency -> Varchar,
        duration_minutes -> Nullable<Int4>,
        is_active -> Bool,
        attributes -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    product_categories (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        parent_id -> Nullable<Uuid>,
        slug -> Nullable<Varchar>,
        image_url -> Nullable<Text>,
        sort_order -> Int4,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    price_lists (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        currency -> Varchar,
        is_default -> Bool,
        valid_from -> Nullable<Date>,
        valid_until -> Nullable<Date>,
        customer_group -> Nullable<Varchar>,
        discount_percent -> Numeric,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    price_list_items (id) {
        id -> Uuid,
        price_list_id -> Uuid,
        product_id -> Nullable<Uuid>,
        service_id -> Nullable<Uuid>,
        price -> Numeric,
        min_quantity -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    inventory_movements (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        product_id -> Uuid,
        movement_type -> Varchar,
        quantity -> Int4,
        reference_type -> Nullable<Varchar>,
        reference_id -> Nullable<Uuid>,
        notes -> Nullable<Text>,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    product_variants (id) {
        id -> Uuid,
        product_id -> Uuid,
        sku -> Nullable<Varchar>,
        name -> Varchar,
        price_adjustment -> Numeric,
        stock_quantity -> Int4,
        attributes -> Jsonb,
        is_active -> Bool,
        created_at -> Timestamptz,
        // Stone/Brasil fields
        gtin -> Nullable<Varchar>,
        peso_liquido -> Nullable<Numeric>,
        peso_bruto -> Nullable<Numeric>,
        largura -> Nullable<Numeric>,
        altura -> Nullable<Numeric>,
        comprimento -> Nullable<Numeric>,
        cor -> Nullable<Varchar>,
        tamanho -> Nullable<Varchar>,
        images -> Nullable<Jsonb>,
    }
}

diesel::table! {
    people (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Nullable<Uuid>,
        first_name -> Varchar,
        last_name -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        mobile -> Nullable<Varchar>,
        job_title -> Nullable<Varchar>,
        department -> Nullable<Varchar>,
        manager_id -> Nullable<Uuid>,
        office_location -> Nullable<Varchar>,
        hire_date -> Nullable<Date>,
        birthday -> Nullable<Date>,
        avatar_url -> Nullable<Text>,
        bio -> Nullable<Text>,
        skills -> Array<Text>,
        social_links -> Jsonb,
        custom_fields -> Jsonb,
        timezone -> Nullable<Varchar>,
        locale -> Nullable<Varchar>,
        is_active -> Bool,
        last_seen_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    people_teams (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        leader_id -> Nullable<Uuid>,
        parent_team_id -> Nullable<Uuid>,
        color -> Nullable<Varchar>,
        icon -> Nullable<Varchar>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    people_team_members (id) {
        id -> Uuid,
        team_id -> Uuid,
        person_id -> Uuid,
        role -> Nullable<Varchar>,
        is_primary -> Bool,
        joined_at -> Timestamptz,
    }
}

diesel::table! {
    people_org_chart (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        person_id -> Uuid,
        reports_to_id -> Nullable<Uuid>,
        position_title -> Nullable<Varchar>,
        position_level -> Int4,
        position_order -> Int4,
        effective_from -> Nullable<Date>,
        effective_until -> Nullable<Date>,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    people_departments (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        code -> Nullable<Varchar>,
        parent_id -> Nullable<Uuid>,
        head_id -> Nullable<Uuid>,
        cost_center -> Nullable<Varchar>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    people_skills (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        category -> Nullable<Varchar>,
        description -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    people_person_skills (id) {
        id -> Uuid,
        person_id -> Uuid,
        skill_id -> Uuid,
        proficiency_level -> Int4,
        years_experience -> Nullable<Numeric>,
        verified_by -> Nullable<Uuid>,
        verified_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    people_time_off (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        person_id -> Uuid,
        time_off_type -> Varchar,
        status -> Varchar,
        start_date -> Date,
        end_date -> Date,
        hours_requested -> Nullable<Numeric>,
        reason -> Nullable<Text>,
        approved_by -> Nullable<Uuid>,
        approved_at -> Nullable<Timestamptz>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(people -> organizations (org_id));
diesel::joinable!(people -> bots (bot_id));
diesel::joinable!(people_teams -> organizations (org_id));
diesel::joinable!(people_teams -> bots (bot_id));
diesel::joinable!(people_team_members -> people_teams (team_id));
diesel::joinable!(people_team_members -> people (person_id));
diesel::joinable!(people_org_chart -> organizations (org_id));
diesel::joinable!(people_org_chart -> bots (bot_id));
diesel::joinable!(people_departments -> organizations (org_id));
diesel::joinable!(people_departments -> bots (bot_id));
diesel::joinable!(people_skills -> organizations (org_id));
diesel::joinable!(people_skills -> bots (bot_id));
diesel::joinable!(people_person_skills -> people (person_id));
diesel::joinable!(people_person_skills -> people_skills (skill_id));
diesel::joinable!(people_time_off -> organizations (org_id));
diesel::joinable!(people_time_off -> bots (bot_id));

diesel::table! {
    attendant_queues (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        priority -> Int4,
        max_wait_minutes -> Int4,
        auto_assign -> Bool,
        working_hours -> Jsonb,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_sessions (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        session_number -> Varchar,
        channel -> Varchar,
        customer_id -> Nullable<Uuid>,
        customer_name -> Nullable<Varchar>,
        customer_email -> Nullable<Varchar>,
        customer_phone -> Nullable<Varchar>,
        status -> Varchar,
        priority -> Int4,
        agent_id -> Nullable<Uuid>,
        queue_id -> Nullable<Uuid>,
        subject -> Nullable<Varchar>,
        initial_message -> Nullable<Text>,
        started_at -> Timestamptz,
        assigned_at -> Nullable<Timestamptz>,
        first_response_at -> Nullable<Timestamptz>,
        ended_at -> Nullable<Timestamptz>,
        wait_time_seconds -> Nullable<Int4>,
        handle_time_seconds -> Nullable<Int4>,
        satisfaction_rating -> Nullable<Int4>,
        satisfaction_comment -> Nullable<Text>,
        tags -> Array<Text>,
        metadata -> Jsonb,
        notes -> Nullable<Text>,
        transfer_count -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_session_messages (id) {
        id -> Uuid,
        session_id -> Uuid,
        sender_type -> Varchar,
        sender_id -> Nullable<Uuid>,
        sender_name -> Nullable<Varchar>,
        content -> Text,
        content_type -> Varchar,
        attachments -> Jsonb,
        is_internal -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_queue_agents (id) {
        id -> Uuid,
        queue_id -> Uuid,
        agent_id -> Uuid,
        max_concurrent -> Int4,
        priority -> Int4,
        skills -> Array<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_agent_status (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        agent_id -> Uuid,
        status -> Varchar,
        status_message -> Nullable<Varchar>,
        current_sessions -> Int4,
        max_sessions -> Int4,
        last_activity_at -> Timestamptz,
        break_started_at -> Nullable<Timestamptz>,
        break_reason -> Nullable<Varchar>,
        available_since -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_transfers (id) {
        id -> Uuid,
        session_id -> Uuid,
        from_agent_id -> Nullable<Uuid>,
        to_agent_id -> Nullable<Uuid>,
        to_queue_id -> Nullable<Uuid>,
        reason -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_canned_responses (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        title -> Varchar,
        content -> Text,
        shortcut -> Nullable<Varchar>,
        category -> Nullable<Varchar>,
        queue_id -> Nullable<Uuid>,
        is_active -> Bool,
        usage_count -> Int4,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_tags (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        color -> Nullable<Varchar>,
        description -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_wrap_up_codes (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        code -> Varchar,
        name -> Varchar,
        description -> Nullable<Text>,
        requires_notes -> Bool,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendant_session_wrap_up (id) {
        id -> Uuid,
        session_id -> Uuid,
        wrap_up_code_id -> Nullable<Uuid>,
        notes -> Nullable<Text>,
        follow_up_required -> Bool,
        follow_up_date -> Nullable<Date>,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    calendars (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        owner_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        color -> Nullable<Varchar>,
        timezone -> Nullable<Varchar>,
        is_primary -> Bool,
        is_visible -> Bool,
        is_shared -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_events (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        calendar_id -> Uuid,
        owner_id -> Uuid,
        title -> Varchar,
        description -> Nullable<Text>,
        location -> Nullable<Varchar>,
        start_time -> Timestamptz,
        end_time -> Timestamptz,
        all_day -> Bool,
        recurrence_rule -> Nullable<Text>,
        recurrence_id -> Nullable<Uuid>,
        color -> Nullable<Varchar>,
        status -> Varchar,
        visibility -> Varchar,
        busy_status -> Varchar,
        reminders -> Jsonb,
        attendees -> Jsonb,
        conference_data -> Nullable<Jsonb>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_event_attendees (id) {
        id -> Uuid,
        event_id -> Uuid,
        email -> Varchar,
        name -> Nullable<Varchar>,
        status -> Varchar,
        role -> Varchar,
        rsvp_time -> Nullable<Timestamptz>,
        comment -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_event_reminders (id) {
        id -> Uuid,
        event_id -> Uuid,
        reminder_type -> Varchar,
        minutes_before -> Int4,
        is_sent -> Bool,
        sent_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_shares (id) {
        id -> Uuid,
        calendar_id -> Uuid,
        shared_with_user_id -> Nullable<Uuid>,
        shared_with_email -> Nullable<Varchar>,
        permission -> Varchar,
        created_at -> Timestamptz,
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
    okr_alignments (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        child_objective_id -> Uuid,
        parent_objective_id -> Uuid,
        alignment_type -> Varchar,
        weight -> Numeric,
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
    okr_comments (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        objective_id -> Nullable<Uuid>,
        key_result_id -> Nullable<Uuid>,
        user_id -> Uuid,
        content -> Text,
        parent_comment_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    okr_activity_log (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        objective_id -> Nullable<Uuid>,
        key_result_id -> Nullable<Uuid>,
        user_id -> Uuid,
        activity_type -> Varchar,
        description -> Nullable<Text>,
        old_value -> Nullable<Text>,
        new_value -> Nullable<Text>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    canvases (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        width -> Int4,
        height -> Int4,
        background_color -> Nullable<Varchar>,
        thumbnail_url -> Nullable<Text>,
        is_public -> Bool,
        is_template -> Bool,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    canvas_elements (id) {
        id -> Uuid,
        canvas_id -> Uuid,
        element_type -> Varchar,
        x -> Float8,
        y -> Float8,
        width -> Float8,
        height -> Float8,
        rotation -> Float8,
        z_index -> Int4,
        locked -> Bool,
        properties -> Jsonb,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    canvas_collaborators (id) {
        id -> Uuid,
        canvas_id -> Uuid,
        user_id -> Uuid,
        permission -> Varchar,
        added_by -> Nullable<Uuid>,
        added_at -> Timestamptz,
    }
}

diesel::table! {
    canvas_versions (id) {
        id -> Uuid,
        canvas_id -> Uuid,
        version_number -> Int4,
        name -> Nullable<Varchar>,
        elements_snapshot -> Jsonb,
        created_by -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    canvas_comments (id) {
        id -> Uuid,
        canvas_id -> Uuid,
        element_id -> Nullable<Uuid>,
        parent_comment_id -> Nullable<Uuid>,
        author_id -> Uuid,
        content -> Text,
        x_position -> Nullable<Float8>,
        y_position -> Nullable<Float8>,
        resolved -> Bool,
        resolved_by -> Nullable<Uuid>,
        resolved_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        icon_type -> Nullable<Varchar>,
        icon_value -> Nullable<Varchar>,
        cover_image -> Nullable<Text>,
        settings -> Jsonb,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_members (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Uuid,
        role -> Varchar,
        invited_by -> Nullable<Uuid>,
        joined_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_pages (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        parent_id -> Nullable<Uuid>,
        title -> Varchar,
        icon_type -> Nullable<Varchar>,
        icon_value -> Nullable<Varchar>,
        cover_image -> Nullable<Text>,
        content -> Jsonb,
        properties -> Jsonb,
        is_template -> Bool,
        template_id -> Nullable<Uuid>,
        is_public -> Bool,
        public_edit -> Bool,
        position -> Int4,
        created_by -> Uuid,
        last_edited_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_page_versions (id) {
        id -> Uuid,
        page_id -> Uuid,
        version_number -> Int4,
        title -> Varchar,
        content -> Jsonb,
        change_summary -> Nullable<Text>,
        created_by -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_page_permissions (id) {
        id -> Uuid,
        page_id -> Uuid,
        user_id -> Nullable<Uuid>,
        role -> Nullable<Varchar>,
        permission -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_comments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        page_id -> Uuid,
        block_id -> Nullable<Uuid>,
        parent_comment_id -> Nullable<Uuid>,
        author_id -> Uuid,
        content -> Text,
        resolved -> Bool,
        resolved_by -> Nullable<Uuid>,
        resolved_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_comment_reactions (id) {
        id -> Uuid,
        comment_id -> Uuid,
        user_id -> Uuid,
        emoji -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_templates (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        category -> Nullable<Varchar>,
        icon_type -> Nullable<Varchar>,
        icon_value -> Nullable<Varchar>,
        cover_image -> Nullable<Text>,
        content -> Jsonb,
        is_system -> Bool,
        usage_count -> Int4,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    social_communities (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        slug -> Varchar,
        description -> Nullable<Text>,
        cover_image -> Nullable<Text>,
        icon -> Nullable<Text>,
        visibility -> Varchar,
        join_policy -> Varchar,
        owner_id -> Uuid,
        member_count -> Int4,
        post_count -> Int4,
        is_official -> Bool,
        is_featured -> Bool,
        settings -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        archived_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    social_community_members (id) {
        id -> Uuid,
        community_id -> Uuid,
        user_id -> Uuid,
        role -> Varchar,
        notifications_enabled -> Bool,
        joined_at -> Timestamptz,
        last_seen_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    social_posts (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        author_id -> Uuid,
        community_id -> Nullable<Uuid>,
        parent_id -> Nullable<Uuid>,
        content -> Text,
        content_type -> Varchar,
        attachments -> Jsonb,
        mentions -> Jsonb,
        hashtags -> Array<Nullable<Text>>,
        visibility -> Varchar,
        is_announcement -> Bool,
        is_pinned -> Bool,
        poll_id -> Nullable<Uuid>,
        reaction_counts -> Jsonb,
        comment_count -> Int4,
        share_count -> Int4,
        view_count -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        edited_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    social_comments (id) {
        id -> Uuid,
        post_id -> Uuid,
        parent_comment_id -> Nullable<Uuid>,
        author_id -> Uuid,
        content -> Text,
        mentions -> Jsonb,
        reaction_counts -> Jsonb,
        reply_count -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        edited_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    social_reactions (id) {
        id -> Uuid,
        post_id -> Nullable<Uuid>,
        comment_id -> Nullable<Uuid>,
        user_id -> Uuid,
        reaction_type -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    social_polls (id) {
        id -> Uuid,
        post_id -> Uuid,
        question -> Text,
        allow_multiple -> Bool,
        allow_add_options -> Bool,
        anonymous -> Bool,
        total_votes -> Int4,
        ends_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    social_poll_options (id) {
        id -> Uuid,
        poll_id -> Uuid,
        text -> Varchar,
        vote_count -> Int4,
        position -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    social_poll_votes (id) {
        id -> Uuid,
        poll_id -> Uuid,
        option_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    social_announcements (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        author_id -> Uuid,
        title -> Varchar,
        content -> Text,
        priority -> Varchar,
        target_audience -> Jsonb,
        is_pinned -> Bool,
        requires_acknowledgment -> Bool,
        acknowledged_by -> Jsonb,
        starts_at -> Nullable<Timestamptz>,
        ends_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    social_praises (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        from_user_id -> Uuid,
        to_user_id -> Uuid,
        badge_type -> Varchar,
        message -> Nullable<Text>,
        is_public -> Bool,
        post_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    social_bookmarks (id) {
        id -> Uuid,
        user_id -> Uuid,
        post_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    social_hashtags (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        tag -> Varchar,
        post_count -> Int4,
        last_used_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    research_projects (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        status -> Varchar,
        owner_id -> Uuid,
        tags -> Array<Nullable<Text>>,
        settings -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    research_sources (id) {
        id -> Uuid,
        project_id -> Uuid,
        source_type -> Varchar,
        name -> Varchar,
        url -> Nullable<Text>,
        content -> Nullable<Text>,
        summary -> Nullable<Text>,
        metadata -> Jsonb,
        credibility_score -> Nullable<Int4>,
        is_verified -> Bool,
        added_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    research_notes (id) {
        id -> Uuid,
        project_id -> Uuid,
        source_id -> Nullable<Uuid>,
        title -> Nullable<Varchar>,
        content -> Text,
        note_type -> Varchar,
        tags -> Array<Nullable<Text>>,
        highlight_text -> Nullable<Text>,
        highlight_position -> Nullable<Jsonb>,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    research_findings (id) {
        id -> Uuid,
        project_id -> Uuid,
        title -> Varchar,
        content -> Text,
        finding_type -> Varchar,
        confidence_level -> Nullable<Varchar>,
        supporting_sources -> Jsonb,
        related_findings -> Jsonb,
        status -> Varchar,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    research_citations (id) {
        id -> Uuid,
        source_id -> Uuid,
        citation_style -> Varchar,
        formatted_citation -> Text,
        bibtex -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    research_collaborators (id) {
        id -> Uuid,
        project_id -> Uuid,
        user_id -> Uuid,
        role -> Varchar,
        invited_by -> Nullable<Uuid>,
        joined_at -> Timestamptz,
    }
}

diesel::table! {
    research_exports (id) {
        id -> Uuid,
        project_id -> Uuid,
        export_type -> Varchar,
        format -> Varchar,
        file_url -> Nullable<Text>,
        file_size -> Nullable<Int4>,
        status -> Varchar,
        created_by -> Uuid,
        created_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
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

diesel::table! {
    legal_documents (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        slug -> Varchar,
        title -> Varchar,
        content -> Text,
        document_type -> Varchar,
        version -> Varchar,
        effective_date -> Timestamptz,
        is_active -> Bool,
        requires_acceptance -> Bool,
        metadata -> Jsonb,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    legal_document_versions (id) {
        id -> Uuid,
        document_id -> Uuid,
        version -> Varchar,
        content -> Text,
        change_summary -> Nullable<Text>,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    cookie_consents (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Nullable<Uuid>,
        session_id -> Nullable<Varchar>,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        country_code -> Nullable<Varchar>,
        consent_necessary -> Bool,
        consent_analytics -> Bool,
        consent_marketing -> Bool,
        consent_preferences -> Bool,
        consent_functional -> Bool,
        consent_version -> Varchar,
        consent_given_at -> Timestamptz,
        consent_updated_at -> Timestamptz,
        consent_withdrawn_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    consent_history (id) {
        id -> Uuid,
        consent_id -> Uuid,
        action -> Varchar,
        previous_consents -> Jsonb,
        new_consents -> Jsonb,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    legal_acceptances (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Uuid,
        document_id -> Uuid,
        document_version -> Varchar,
        accepted_at -> Timestamptz,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
    }
}

diesel::table! {
    data_deletion_requests (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Uuid,
        request_type -> Varchar,
        status -> Varchar,
        reason -> Nullable<Text>,
        requested_at -> Timestamptz,
        scheduled_for -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        confirmation_token -> Varchar,
        confirmed_at -> Nullable<Timestamptz>,
        processed_by -> Nullable<Uuid>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    data_export_requests (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Uuid,
        status -> Varchar,
        format -> Varchar,
        include_sections -> Jsonb,
        requested_at -> Timestamptz,
        started_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        file_url -> Nullable<Text>,
        file_size -> Nullable<Int4>,
        expires_at -> Nullable<Timestamptz>,
        error_message -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_checks (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        framework -> Varchar,
        control_id -> Varchar,
        control_name -> Varchar,
        status -> Varchar,
        score -> Numeric,
        checked_at -> Timestamptz,
        checked_by -> Nullable<Uuid>,
        evidence -> Jsonb,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_issues (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        check_id -> Nullable<Uuid>,
        severity -> Varchar,
        title -> Varchar,
        description -> Text,
        remediation -> Nullable<Text>,
        due_date -> Nullable<Timestamptz>,
        assigned_to -> Nullable<Uuid>,
        status -> Varchar,
        resolved_at -> Nullable<Timestamptz>,
        resolved_by -> Nullable<Uuid>,
        resolution_notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_audit_log (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        event_type -> Varchar,
        user_id -> Nullable<Uuid>,
        resource_type -> Varchar,
        resource_id -> Varchar,
        action -> Varchar,
        result -> Varchar,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_evidence (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        check_id -> Nullable<Uuid>,
        issue_id -> Nullable<Uuid>,
        evidence_type -> Varchar,
        title -> Varchar,
        description -> Nullable<Text>,
        file_url -> Nullable<Text>,
        file_name -> Nullable<Varchar>,
        file_size -> Nullable<Int4>,
        mime_type -> Nullable<Varchar>,
        metadata -> Jsonb,
        collected_at -> Timestamptz,
        collected_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_risk_assessments (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        title -> Varchar,
        assessor_id -> Uuid,
        methodology -> Varchar,
        overall_risk_score -> Numeric,
        status -> Varchar,
        started_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        next_review_date -> Nullable<Date>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_risks (id) {
        id -> Uuid,
        assessment_id -> Uuid,
        title -> Varchar,
        description -> Nullable<Text>,
        category -> Varchar,
        likelihood_score -> Int4,
        impact_score -> Int4,
        risk_score -> Int4,
        risk_level -> Varchar,
        current_controls -> Jsonb,
        treatment_strategy -> Varchar,
        status -> Varchar,
        owner_id -> Nullable<Uuid>,
        due_date -> Nullable<Date>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_training_records (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Uuid,
        training_type -> Varchar,
        training_name -> Varchar,
        provider -> Nullable<Varchar>,
        score -> Nullable<Int4>,
        passed -> Bool,
        completion_date -> Timestamptz,
        valid_until -> Nullable<Timestamptz>,
        certificate_url -> Nullable<Text>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_access_reviews (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Uuid,
        reviewer_id -> Uuid,
        review_date -> Timestamptz,
        permissions_reviewed -> Jsonb,
        anomalies -> Jsonb,
        recommendations -> Jsonb,
        status -> Varchar,
        approved_at -> Nullable<Timestamptz>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    billing_usage_alerts (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        metric -> Varchar,
        severity -> Varchar,
        current_usage -> Int8,
        usage_limit -> Int8,
        percentage -> Numeric,
        threshold -> Numeric,
        message -> Text,
        acknowledged_at -> Nullable<Timestamptz>,
        acknowledged_by -> Nullable<Uuid>,
        notification_sent -> Bool,
        notification_channels -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    billing_alert_history (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        alert_id -> Uuid,
        metric -> Varchar,
        severity -> Varchar,
        current_usage -> Int8,
        usage_limit -> Int8,
        percentage -> Numeric,
        message -> Text,
        acknowledged_at -> Nullable<Timestamptz>,
        acknowledged_by -> Nullable<Uuid>,
        resolved_at -> Nullable<Timestamptz>,
        resolution_type -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    billing_notification_preferences (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        enabled -> Bool,
        channels -> Jsonb,
        email_recipients -> Jsonb,
        webhook_url -> Nullable<Text>,
        webhook_secret -> Nullable<Text>,
        slack_webhook_url -> Nullable<Text>,
        teams_webhook_url -> Nullable<Text>,
        sms_numbers -> Jsonb,
        min_severity -> Varchar,
        quiet_hours_start -> Nullable<Int4>,
        quiet_hours_end -> Nullable<Int4>,
        quiet_hours_timezone -> Nullable<Varchar>,
        quiet_hours_days -> Nullable<Jsonb>,
        metric_overrides -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    billing_grace_periods (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        metric -> Varchar,
        started_at -> Timestamptz,
        expires_at -> Timestamptz,
        overage_at_start -> Numeric,
        current_overage -> Numeric,
        max_allowed_overage -> Numeric,
        is_active -> Bool,
        ended_at -> Nullable<Timestamptz>,
        end_reason -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    meeting_rooms (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        room_code -> Varchar,
        name -> Varchar,
        description -> Nullable<Text>,
        created_by -> Uuid,
        max_participants -> Int4,
        is_recording -> Bool,
        is_transcribing -> Bool,
        status -> Varchar,
        settings -> Jsonb,
        started_at -> Nullable<Timestamptz>,
        ended_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    meeting_participants (id) {
        id -> Uuid,
        room_id -> Uuid,
        user_id -> Nullable<Uuid>,
        participant_name -> Varchar,
        email -> Nullable<Varchar>,
        role -> Varchar,
        is_bot -> Bool,
        is_active -> Bool,
        has_video -> Bool,
        has_audio -> Bool,
        joined_at -> Timestamptz,
        left_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    meeting_recordings (id) {
        id -> Uuid,
        room_id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        recording_type -> Varchar,
        file_url -> Nullable<Text>,
        file_size -> Nullable<Int8>,
        duration_seconds -> Nullable<Int4>,
        status -> Varchar,
        started_at -> Timestamptz,
        stopped_at -> Nullable<Timestamptz>,
        processed_at -> Nullable<Timestamptz>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    meeting_transcriptions (id) {
        id -> Uuid,
        room_id -> Uuid,
        recording_id -> Nullable<Uuid>,
        org_id -> Uuid,
        bot_id -> Uuid,
        participant_id -> Nullable<Uuid>,
        speaker_name -> Nullable<Varchar>,
        content -> Text,
        start_time -> Numeric,
        end_time -> Numeric,
        confidence -> Nullable<Numeric>,
        language -> Nullable<Varchar>,
        is_final -> Bool,
        metadata -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    meeting_whiteboards (id) {
        id -> Uuid,
        room_id -> Nullable<Uuid>,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        background_color -> Nullable<Varchar>,
        grid_enabled -> Bool,
        grid_size -> Nullable<Int4>,
        elements -> Jsonb,
        version -> Int4,
        created_by -> Uuid,
        last_modified_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    whiteboard_elements (id) {
        id -> Uuid,
        whiteboard_id -> Uuid,
        element_type -> Varchar,
        position_x -> Numeric,
        position_y -> Numeric,
        width -> Nullable<Numeric>,
        height -> Nullable<Numeric>,
        rotation -> Nullable<Numeric>,
        z_index -> Int4,
        properties -> Jsonb,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    whiteboard_exports (id) {
        id -> Uuid,
        whiteboard_id -> Uuid,
        org_id -> Uuid,
        export_format -> Varchar,
        file_url -> Nullable<Text>,
        file_size -> Nullable<Int8>,
        status -> Varchar,
        error_message -> Nullable<Text>,
        requested_by -> Uuid,
        created_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    meeting_chat_messages (id) {
        id -> Uuid,
        room_id -> Uuid,
        participant_id -> Nullable<Uuid>,
        sender_name -> Varchar,
        message_type -> Varchar,
        content -> Text,
        reply_to_id -> Nullable<Uuid>,
        is_system_message -> Bool,
        metadata -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    scheduled_meetings (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        room_id -> Nullable<Uuid>,
        title -> Varchar,
        description -> Nullable<Text>,
        organizer_id -> Uuid,
        scheduled_start -> Timestamptz,
        scheduled_end -> Timestamptz,
        timezone -> Varchar,
        recurrence_rule -> Nullable<Text>,
        attendees -> Jsonb,
        settings -> Jsonb,
        status -> Varchar,
        reminder_sent -> Bool,
        calendar_event_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(attendant_queues -> organizations (org_id));
diesel::joinable!(attendant_queues -> bots (bot_id));
diesel::joinable!(attendant_sessions -> organizations (org_id));
diesel::joinable!(attendant_sessions -> bots (bot_id));
diesel::joinable!(attendant_sessions -> attendant_queues (queue_id));
diesel::joinable!(attendant_session_messages -> attendant_sessions (session_id));
diesel::joinable!(attendant_queue_agents -> attendant_queues (queue_id));
diesel::joinable!(attendant_agent_status -> organizations (org_id));
diesel::joinable!(attendant_agent_status -> bots (bot_id));
diesel::joinable!(attendant_transfers -> attendant_sessions (session_id));
diesel::joinable!(attendant_canned_responses -> organizations (org_id));
diesel::joinable!(attendant_canned_responses -> bots (bot_id));
diesel::joinable!(attendant_tags -> organizations (org_id));
diesel::joinable!(attendant_tags -> bots (bot_id));
diesel::joinable!(attendant_wrap_up_codes -> organizations (org_id));
diesel::joinable!(attendant_wrap_up_codes -> bots (bot_id));
diesel::joinable!(attendant_session_wrap_up -> attendant_sessions (session_id));
diesel::joinable!(attendant_session_wrap_up -> attendant_wrap_up_codes (wrap_up_code_id));

diesel::joinable!(calendars -> organizations (org_id));
diesel::joinable!(calendars -> bots (bot_id));
diesel::joinable!(calendar_events -> organizations (org_id));
diesel::joinable!(calendar_events -> bots (bot_id));
diesel::joinable!(calendar_events -> calendars (calendar_id));
diesel::joinable!(calendar_event_attendees -> calendar_events (event_id));
diesel::joinable!(calendar_event_reminders -> calendar_events (event_id));
diesel::joinable!(calendar_shares -> calendars (calendar_id));

diesel::joinable!(okr_objectives -> organizations (org_id));
diesel::joinable!(okr_objectives -> bots (bot_id));
diesel::joinable!(okr_key_results -> organizations (org_id));
diesel::joinable!(okr_key_results -> bots (bot_id));
diesel::joinable!(okr_key_results -> okr_objectives (objective_id));
diesel::joinable!(okr_checkins -> organizations (org_id));
diesel::joinable!(okr_checkins -> bots (bot_id));
diesel::joinable!(okr_checkins -> okr_key_results (key_result_id));
diesel::joinable!(okr_alignments -> organizations (org_id));
diesel::joinable!(okr_alignments -> bots (bot_id));
diesel::joinable!(okr_templates -> organizations (org_id));
diesel::joinable!(okr_templates -> bots (bot_id));
diesel::joinable!(okr_comments -> organizations (org_id));
diesel::joinable!(okr_comments -> bots (bot_id));
diesel::joinable!(okr_activity_log -> organizations (org_id));
diesel::joinable!(okr_activity_log -> bots (bot_id));

diesel::joinable!(canvases -> organizations (org_id));
diesel::joinable!(canvases -> bots (bot_id));
diesel::joinable!(canvas_elements -> canvases (canvas_id));
diesel::joinable!(canvas_collaborators -> canvases (canvas_id));
diesel::joinable!(canvas_versions -> canvases (canvas_id));
diesel::joinable!(canvas_comments -> canvases (canvas_id));

diesel::joinable!(workspaces -> organizations (org_id));
diesel::joinable!(workspaces -> bots (bot_id));
diesel::joinable!(workspace_members -> workspaces (workspace_id));
diesel::joinable!(workspace_pages -> workspaces (workspace_id));
diesel::joinable!(workspace_page_versions -> workspace_pages (page_id));
diesel::joinable!(workspace_page_permissions -> workspace_pages (page_id));
diesel::joinable!(workspace_comments -> workspaces (workspace_id));
diesel::joinable!(workspace_comments -> workspace_pages (page_id));
diesel::joinable!(workspace_comment_reactions -> workspace_comments (comment_id));
diesel::joinable!(workspace_templates -> organizations (org_id));
diesel::joinable!(workspace_templates -> bots (bot_id));

diesel::joinable!(social_communities -> organizations (org_id));
diesel::joinable!(social_communities -> bots (bot_id));
diesel::joinable!(social_community_members -> social_communities (community_id));
diesel::joinable!(social_posts -> organizations (org_id));
diesel::joinable!(social_posts -> bots (bot_id));
diesel::joinable!(social_comments -> social_posts (post_id));
diesel::joinable!(social_polls -> social_posts (post_id));
diesel::joinable!(social_poll_options -> social_polls (poll_id));
diesel::joinable!(social_poll_votes -> social_polls (poll_id));
diesel::joinable!(social_poll_votes -> social_poll_options (option_id));
diesel::joinable!(social_announcements -> organizations (org_id));
diesel::joinable!(social_announcements -> bots (bot_id));
diesel::joinable!(social_praises -> organizations (org_id));
diesel::joinable!(social_praises -> bots (bot_id));
diesel::joinable!(social_bookmarks -> social_posts (post_id));
diesel::joinable!(social_hashtags -> organizations (org_id));
diesel::joinable!(social_hashtags -> bots (bot_id));

diesel::joinable!(research_projects -> organizations (org_id));
diesel::joinable!(research_projects -> bots (bot_id));
diesel::joinable!(research_sources -> research_projects (project_id));
diesel::joinable!(research_notes -> research_projects (project_id));
diesel::joinable!(research_findings -> research_projects (project_id));
diesel::joinable!(research_citations -> research_sources (source_id));
diesel::joinable!(research_collaborators -> research_projects (project_id));
diesel::joinable!(research_exports -> research_projects (project_id));

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

diesel::joinable!(legal_documents -> organizations (org_id));
diesel::joinable!(legal_documents -> bots (bot_id));
diesel::joinable!(legal_document_versions -> legal_documents (document_id));
diesel::joinable!(cookie_consents -> organizations (org_id));
diesel::joinable!(cookie_consents -> bots (bot_id));
diesel::joinable!(consent_history -> cookie_consents (consent_id));
diesel::joinable!(legal_acceptances -> organizations (org_id));
diesel::joinable!(legal_acceptances -> bots (bot_id));
diesel::joinable!(legal_acceptances -> legal_documents (document_id));
diesel::joinable!(data_deletion_requests -> organizations (org_id));
diesel::joinable!(data_deletion_requests -> bots (bot_id));
diesel::joinable!(data_export_requests -> organizations (org_id));
diesel::joinable!(data_export_requests -> bots (bot_id));

diesel::joinable!(compliance_checks -> organizations (org_id));
diesel::joinable!(compliance_checks -> bots (bot_id));
diesel::joinable!(compliance_issues -> organizations (org_id));
diesel::joinable!(compliance_issues -> bots (bot_id));
diesel::joinable!(compliance_issues -> compliance_checks (check_id));
diesel::joinable!(compliance_audit_log -> organizations (org_id));
diesel::joinable!(compliance_audit_log -> bots (bot_id));
diesel::joinable!(compliance_evidence -> organizations (org_id));
diesel::joinable!(compliance_evidence -> bots (bot_id));
diesel::joinable!(compliance_risk_assessments -> organizations (org_id));
diesel::joinable!(compliance_risk_assessments -> bots (bot_id));
diesel::joinable!(compliance_risks -> compliance_risk_assessments (assessment_id));
diesel::joinable!(compliance_training_records -> organizations (org_id));
diesel::joinable!(compliance_training_records -> bots (bot_id));
diesel::joinable!(compliance_access_reviews -> organizations (org_id));
diesel::joinable!(compliance_access_reviews -> bots (bot_id));

diesel::joinable!(billing_usage_alerts -> organizations (org_id));
diesel::joinable!(billing_usage_alerts -> bots (bot_id));
diesel::joinable!(billing_alert_history -> organizations (org_id));
diesel::joinable!(billing_alert_history -> bots (bot_id));
diesel::joinable!(billing_notification_preferences -> organizations (org_id));
diesel::joinable!(billing_notification_preferences -> bots (bot_id));
diesel::joinable!(billing_grace_periods -> organizations (org_id));
diesel::joinable!(billing_grace_periods -> bots (bot_id));

diesel::joinable!(meeting_rooms -> organizations (org_id));
diesel::joinable!(meeting_rooms -> bots (bot_id));
diesel::joinable!(meeting_participants -> meeting_rooms (room_id));
diesel::joinable!(meeting_recordings -> meeting_rooms (room_id));
diesel::joinable!(meeting_recordings -> organizations (org_id));
diesel::joinable!(meeting_recordings -> bots (bot_id));
diesel::joinable!(meeting_transcriptions -> meeting_rooms (room_id));
diesel::joinable!(meeting_transcriptions -> meeting_recordings (recording_id));
diesel::joinable!(meeting_transcriptions -> meeting_participants (participant_id));
diesel::joinable!(meeting_whiteboards -> meeting_rooms (room_id));
diesel::joinable!(meeting_whiteboards -> organizations (org_id));
diesel::joinable!(meeting_whiteboards -> bots (bot_id));
diesel::joinable!(whiteboard_elements -> meeting_whiteboards (whiteboard_id));
diesel::joinable!(whiteboard_exports -> meeting_whiteboards (whiteboard_id));
diesel::joinable!(whiteboard_exports -> organizations (org_id));
diesel::joinable!(meeting_chat_messages -> meeting_rooms (room_id));
diesel::joinable!(meeting_chat_messages -> meeting_participants (participant_id));
diesel::joinable!(scheduled_meetings -> organizations (org_id));
diesel::joinable!(scheduled_meetings -> bots (bot_id));
diesel::joinable!(scheduled_meetings -> meeting_rooms (room_id));

diesel::joinable!(products -> organizations (org_id));
diesel::joinable!(products -> bots (bot_id));
diesel::joinable!(services -> organizations (org_id));
diesel::joinable!(services -> bots (bot_id));
diesel::joinable!(product_categories -> organizations (org_id));
diesel::joinable!(product_categories -> bots (bot_id));
diesel::joinable!(price_lists -> organizations (org_id));
diesel::joinable!(price_lists -> bots (bot_id));
diesel::joinable!(price_list_items -> price_lists (price_list_id));
diesel::joinable!(price_list_items -> products (product_id));
diesel::joinable!(price_list_items -> services (service_id));
diesel::joinable!(inventory_movements -> organizations (org_id));
diesel::joinable!(inventory_movements -> bots (bot_id));
diesel::joinable!(inventory_movements -> products (product_id));
diesel::joinable!(product_variants -> products (product_id));

diesel::joinable!(billing_invoices -> organizations (org_id));
diesel::joinable!(billing_invoices -> bots (bot_id));
diesel::joinable!(billing_invoice_items -> billing_invoices (invoice_id));
diesel::joinable!(billing_payments -> organizations (org_id));
diesel::joinable!(billing_payments -> bots (bot_id));
diesel::joinable!(billing_payments -> billing_invoices (invoice_id));
diesel::joinable!(billing_quotes -> organizations (org_id));
diesel::joinable!(billing_quotes -> bots (bot_id));
diesel::joinable!(billing_quote_items -> billing_quotes (quote_id));
diesel::joinable!(billing_recurring -> organizations (org_id));
diesel::joinable!(billing_recurring -> bots (bot_id));
diesel::joinable!(billing_tax_rates -> organizations (org_id));
diesel::joinable!(billing_tax_rates -> bots (bot_id));

diesel::joinable!(support_tickets -> organizations (org_id));
diesel::joinable!(support_tickets -> bots (bot_id));
diesel::joinable!(ticket_comments -> support_tickets (ticket_id));
diesel::joinable!(ticket_sla_policies -> organizations (org_id));
diesel::joinable!(ticket_sla_policies -> bots (bot_id));
diesel::joinable!(ticket_canned_responses -> organizations (org_id));
diesel::joinable!(ticket_canned_responses -> bots (bot_id));
diesel::joinable!(ticket_categories -> organizations (org_id));
diesel::joinable!(ticket_categories -> bots (bot_id));
diesel::joinable!(ticket_tags -> organizations (org_id));
diesel::joinable!(ticket_tags -> bots (bot_id));

diesel::joinable!(crm_contacts -> organizations (org_id));
diesel::joinable!(crm_contacts -> bots (bot_id));
diesel::joinable!(crm_accounts -> organizations (org_id));
diesel::joinable!(crm_accounts -> bots (bot_id));
diesel::joinable!(crm_pipeline_stages -> organizations (org_id));
diesel::joinable!(crm_pipeline_stages -> bots (bot_id));
diesel::joinable!(crm_leads -> organizations (org_id));
diesel::joinable!(crm_leads -> bots (bot_id));
diesel::joinable!(crm_leads -> crm_contacts (contact_id));
diesel::joinable!(crm_leads -> crm_accounts (account_id));
diesel::joinable!(crm_opportunities -> organizations (org_id));
diesel::joinable!(crm_opportunities -> bots (bot_id));
diesel::joinable!(crm_activities -> organizations (org_id));
diesel::joinable!(crm_activities -> bots (bot_id));
diesel::joinable!(crm_notes -> organizations (org_id));

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
    rbac_roles,
    rbac_groups,
    rbac_permissions,
    rbac_role_permissions,
    rbac_user_roles,
    rbac_user_groups,
    rbac_group_roles,
    crm_contacts,
    crm_accounts,
    crm_pipeline_stages,
    crm_leads,
    crm_opportunities,
    crm_activities,
    crm_notes,
    support_tickets,
    ticket_comments,
    ticket_sla_policies,
    ticket_canned_responses,
    ticket_categories,
    ticket_tags,
    billing_invoices,
    billing_invoice_items,
    billing_payments,
    billing_quotes,
    billing_quote_items,
    billing_recurring,
    billing_tax_rates,
    products,
    services,
    product_categories,
    price_lists,
    price_list_items,
    inventory_movements,
    product_variants,
    people,
    people_teams,
    people_team_members,
    people_org_chart,
    people_departments,
    people_skills,
    people_person_skills,
    people_time_off,
    attendant_queues,
    attendant_sessions,
    attendant_session_messages,
    attendant_queue_agents,
    attendant_agent_status,
    attendant_transfers,
    attendant_canned_responses,
    attendant_tags,
    attendant_wrap_up_codes,
    attendant_session_wrap_up,
    calendars,
    calendar_events,
    calendar_event_attendees,
    calendar_event_reminders,
    calendar_shares,
    okr_objectives,
    okr_key_results,
    okr_checkins,
    okr_alignments,
    okr_templates,
    okr_comments,
    okr_activity_log,
    canvases,
    canvas_elements,
    canvas_collaborators,
    canvas_versions,
    canvas_comments,
    workspaces,
    workspace_members,
    workspace_pages,
    workspace_page_versions,
    workspace_page_permissions,
    workspace_comments,
    workspace_comment_reactions,
    workspace_templates,
    social_communities,
    social_community_members,
    social_posts,
    social_comments,
    social_reactions,
    social_polls,
    social_poll_options,
    social_poll_votes,
    social_announcements,
    social_praises,
    social_bookmarks,
    social_hashtags,
    research_projects,
    research_sources,
    research_notes,
    research_findings,
    research_citations,
    research_collaborators,
    research_exports,
    dashboards,
    dashboard_widgets,
    dashboard_data_sources,
    dashboard_filters,
    dashboard_widget_data_sources,
    conversational_queries,
    legal_documents,
    legal_document_versions,
    cookie_consents,
    consent_history,
    legal_acceptances,
    data_deletion_requests,
    data_export_requests,
    compliance_checks,
    compliance_issues,
    compliance_audit_log,
    compliance_evidence,
    compliance_risk_assessments,
    compliance_risks,
    compliance_training_records,
    compliance_access_reviews,
    billing_usage_alerts,
    billing_alert_history,
    billing_notification_preferences,
    billing_grace_periods,
    meeting_rooms,
    meeting_participants,
    meeting_recordings,
    meeting_transcriptions,
    meeting_whiteboards,
    whiteboard_elements,
    whiteboard_exports,
    meeting_chat_messages,
    scheduled_meetings,
);
