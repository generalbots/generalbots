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
    email_tracking (id) {
        id -> Uuid,
        recipient_id -> Nullable<Uuid>,
        campaign_id -> Nullable<Uuid>,
        message_id -> Nullable<Varchar>,
        open_token -> Nullable<Uuid>,
        open_tracking_enabled -> Nullable<Bool>,
        opened -> Nullable<Bool>,
        opened_at -> Nullable<Timestamptz>,
        clicked -> Nullable<Bool>,
        clicked_at -> Nullable<Timestamptz>,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    feature_flags (id) {
        id -> Uuid,
        org_id -> Uuid,
        feature -> Varchar,
        enabled -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    email_crm_links (id) {
        id -> Uuid,
        email_id -> Uuid,
        contact_id -> Nullable<Uuid>,
        opportunity_id -> Nullable<Uuid>,
        logged_at -> Timestamp,
    }
}

diesel::table! {
    email_campaign_links (id) {
        id -> Uuid,
        email_id -> Uuid,
        campaign_id -> Nullable<Uuid>,
        list_id -> Nullable<Uuid>,
        sent_at -> Timestamp,
    }
}

diesel::table! {
    email_snooze (id) {
        id -> Uuid,
        email_id -> Uuid,
        snooze_until -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    email_flags (id) {
        id -> Uuid,
        email_id -> Uuid,
        follow_up_date -> Nullable<Date>,
        flag_type -> Nullable<Varchar>,
        completed -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    email_nudges (id) {
        id -> Uuid,
        email_id -> Uuid,
        last_sent -> Nullable<Timestamp>,
        dismissed -> Bool,
        created_at -> Timestamp,
    }
}

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
