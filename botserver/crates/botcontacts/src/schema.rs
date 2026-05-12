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

diesel::joinable!(bots -> organizations (org_id));
diesel::joinable!(calendars -> organizations (org_id));
diesel::joinable!(calendars -> bots (bot_id));
diesel::joinable!(calendar_events -> organizations (org_id));
diesel::joinable!(calendar_events -> bots (bot_id));
diesel::joinable!(calendar_events -> calendars (calendar_id));
diesel::joinable!(calendar_event_attendees -> calendar_events (event_id));
diesel::joinable!(calendar_event_reminders -> calendar_events (event_id));
diesel::joinable!(calendar_shares -> calendars (calendar_id));
diesel::joinable!(crm_deals -> people_departments (department_id));
