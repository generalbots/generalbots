use diesel::table;

table! {
    marketing_campaigns (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        deal_id -> Nullable<Uuid>,
        name -> Varchar,
        status -> Varchar,
        channel -> Varchar,
        content_template -> Json,
        scheduled_at -> Nullable<Timestamptz>,
        sent_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        metrics -> Json,
        budget -> Nullable<Float8>,
        sender_email -> Nullable<Varchar>,
        sender_ip -> Nullable<Varchar>,
        list_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

table! {
    marketing_lists (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        list_type -> Varchar,
        query_text -> Nullable<Varchar>,
        contact_count -> Nullable<Int4>,
        last_sent_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

table! {
    marketing_templates (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        channel -> Varchar,
        subject -> Nullable<Varchar>,
        body -> Nullable<Varchar>,
        media_url -> Nullable<Varchar>,
        ai_prompt -> Nullable<Varchar>,
        variables -> Json,
        approved -> Nullable<Bool>,
        meta_template_id -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

table! {
    marketing_recipients (id) {
        id -> Uuid,
        campaign_id -> Nullable<Uuid>,
        contact_id -> Nullable<Uuid>,
        channel -> Varchar,
        status -> Varchar,
        sent_at -> Nullable<Timestamptz>,
        delivered_at -> Nullable<Timestamptz>,
        failed_at -> Nullable<Timestamptz>,
        error_message -> Nullable<Varchar>,
        response -> Nullable<Json>,
        created_at -> Timestamptz,
    }
}

table! {
    email_tracking (id) {
        id -> Uuid,
        recipient_id -> Nullable<Uuid>,
        campaign_id -> Nullable<Uuid>,
        message_id -> Nullable<Varchar>,
        open_token -> Nullable<Uuid>,
        open_tracking_enabled -> Bool,
        opened -> Bool,
        opened_at -> Nullable<Timestamptz>,
        clicked -> Bool,
        clicked_at -> Nullable<Timestamptz>,
        ip_address -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

table! {
    campaign_metrics (campaign_id) {
        campaign_id -> Uuid,
        sent_count -> Int8,
        delivered_count -> Int8,
        bounce_count -> Int8,
        open_count -> Int8,
        click_count -> Int8,
        complaint_count -> Int8,
        unsubscribe_count -> Int8,
        reply_count -> Int8,
    }
}

table! {
    advisor_recommendations (id) {
        id -> Uuid,
        campaign_id -> Uuid,
        check_name -> Varchar,
        severity -> Varchar,
        message -> Varchar,
        details -> Nullable<Text>,
        dismissed -> Bool,
        created_at -> Timestamptz,
    }
}

table! {
    ip_reputation (id) {
        id -> Uuid,
        org_id -> Uuid,
        ip -> Varchar,
        provider -> Varchar,
        delivered -> Int8,
        bounced -> Int8,
        complained -> Int8,
        window_start -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

table! {
    ip_rotations (ip_address) {
        ip_address -> Varchar,
        org_id -> Uuid,
        last_used -> Timestamptz,
    }
}

table! {
    org_ips (id) {
        id -> Uuid,
        org_id -> Uuid,
        ip_address -> Varchar,
        is_active -> Bool,
    }
}

table! {
    warmup_schedules (id) {
        id -> Uuid,
        org_id -> Uuid,
        ip -> Varchar,
        started_at -> Timestamptz,
        current_day -> Int4,
        daily_limit -> Int4,
        status -> Varchar,
        paused_reason -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

table! {
    crm_contacts (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        company -> Nullable<Varchar>,
        status -> Nullable<Varchar>,
    }
}

table! {
    bots (id) {
        id -> Uuid,
        org_id -> Nullable<Uuid>,
        name -> Varchar,
    }
}

table! {
    system_automations (id) {
        id -> Uuid,
        bot_id -> Uuid,
        kind -> Int4,
        is_active -> Bool,
        target -> Nullable<Varchar>,
        param -> Nullable<Varchar>,
    }
}

table! {
    marketing_contacts (email) {
        email -> Varchar,
        list_id -> Uuid,
    }
}

table! {
    marketing_email_opens (id) {
        id -> Uuid,
        email -> Varchar,
        opened_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    marketing_campaigns,
    marketing_lists,
    marketing_templates,
    marketing_recipients,
    email_tracking,
    campaign_metrics,
    advisor_recommendations,
    ip_reputation,
    ip_rotations,
    org_ips,
    warmup_schedules,
    crm_contacts,
    bots,
    system_automations,
    marketing_contacts,
    marketing_email_opens,
);
