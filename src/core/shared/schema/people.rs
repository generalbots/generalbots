use crate::core::shared::schema::core::{bots, organizations};

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
diesel::joinable!(crm_deals -> people_departments (department_id));
diesel::joinable!(attendance_sla_events -> attendance_sla_policies (sla_policy_id));

diesel::table! {
    crm_deals (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        contact_id -> Nullable<Uuid>,
        account_id -> Nullable<Uuid>,
        am_id -> Nullable<Uuid>,
        owner_id -> Nullable<Uuid>,
        lead_id -> Nullable<Uuid>,
        title -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        description -> Nullable<Text>,
        value -> Nullable<Float8>,
        currency -> Nullable<Varchar>,
        stage_id -> Nullable<Uuid>,
        stage -> Nullable<Varchar>,
        probability -> Int4,
        source -> Nullable<Varchar>,
        segment_id -> Nullable<Uuid>,
        department_id -> Nullable<Uuid>,
        expected_close_date -> Nullable<Date>,
        actual_close_date -> Nullable<Date>,
        period -> Nullable<Int4>,
        deal_date -> Nullable<Date>,
        closed_at -> Nullable<Timestamptz>,
        lost_reason -> Nullable<Varchar>,
        won -> Nullable<Bool>,
        notes -> Nullable<Text>,
        tags -> Array<Text>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        custom_fields -> Jsonb,
    }
}

diesel::table! {
    crm_deal_segments (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    marketing_campaigns (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        deal_id -> Nullable<Uuid>,
        name -> Varchar,
        status -> Varchar,
        channel -> Varchar,
        content_template -> Jsonb,
        scheduled_at -> Nullable<Timestamptz>,
        sent_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        metrics -> Jsonb,
        budget -> Nullable<Float8>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    marketing_lists (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        list_type -> Varchar,
        query_text -> Nullable<Text>,
        contact_count -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    marketing_list_contacts (list_id, contact_id) {
        list_id -> Uuid,
        contact_id -> Uuid,
        added_at -> Timestamptz,
    }
}

diesel::table! {
    marketing_recipients (id) {
        id -> Uuid,
        campaign_id -> Nullable<Uuid>,
        contact_id -> Nullable<Uuid>,
        deal_id -> Nullable<Uuid>,
        channel -> Varchar,
        status -> Varchar,
        sent_at -> Nullable<Timestamptz>,
        delivered_at -> Nullable<Timestamptz>,
        failed_at -> Nullable<Timestamptz>,
        error_message -> Nullable<Text>,
        response -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    marketing_templates (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        channel -> Varchar,
        subject -> Nullable<Varchar>,
        body -> Nullable<Text>,
        media_url -> Nullable<Varchar>,
        ai_prompt -> Nullable<Text>,
        variables -> Jsonb,
        approved -> Nullable<Bool>,
        meta_template_id -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
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
    whatsapp_business (id) {
        id -> Uuid,
        bot_id -> Uuid,
        phone_number_id -> Nullable<Varchar>,
        business_account_id -> Nullable<Varchar>,
        access_token -> Nullable<Text>,
        webhooks_verified -> Nullable<Bool>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    attendance_sla_policies (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        channel -> Nullable<Varchar>,
        priority -> Nullable<Varchar>,
        first_response_minutes -> Nullable<Int4>,
        resolution_minutes -> Nullable<Int4>,
        escalate_on_breach -> Nullable<Bool>,
        is_active -> Nullable<Bool>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_sla_events (id) {
        id -> Uuid,
        session_id -> Uuid,
        sla_policy_id -> Uuid,
        event_type -> Varchar,
        due_at -> Timestamptz,
        met_at -> Nullable<Timestamptz>,
        breached_at -> Nullable<Timestamptz>,
        status -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_webhooks (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        webhook_url -> Varchar,
        events -> Nullable<Array<Text>>,
        is_active -> Nullable<Bool>,
        secret_key -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}
