diesel::table! {
    billing_usage_alerts (id) {

    id -> Uuid,
    org_id -> Uuid,
    bot_id -> Uuid,
    #[max_length = 50]
    metric -> Varchar,
    #[max_length = 20]
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
    branch_id -> Uuid,
    }
}


diesel::table! {
    billing_alert_history (id) {

    id -> Uuid,
    org_id -> Uuid,
    bot_id -> Uuid,
    alert_id -> Uuid,
    #[max_length = 50]
    metric -> Varchar,
    #[max_length = 20]
    severity -> Varchar,
    current_usage -> Int8,
    usage_limit -> Int8,
    percentage -> Numeric,
    message -> Text,
    acknowledged_at -> Nullable<Timestamptz>,
    acknowledged_by -> Nullable<Uuid>,
    resolved_at -> Nullable<Timestamptz>,
    #[max_length = 50]
    resolution_type -> Nullable<Varchar>,
    created_at -> Timestamptz,
    branch_id -> Uuid,
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
    #[max_length = 20]
    min_severity -> Varchar,
    quiet_hours_start -> Nullable<Int4>,
    quiet_hours_end -> Nullable<Int4>,
    #[max_length = 50]
    quiet_hours_timezone -> Nullable<Varchar>,
    quiet_hours_days -> Nullable<Jsonb>,
    metric_overrides -> Jsonb,
    created_at -> Timestamptz,
    updated_at -> Timestamptz,
    branch_id -> Uuid,
    }
}


diesel::table! {
    billing_grace_periods (id) {

    id -> Uuid,
    org_id -> Uuid,
    bot_id -> Uuid,
    #[max_length = 50]
    metric -> Varchar,
    started_at -> Timestamptz,
    expires_at -> Timestamptz,
    overage_at_start -> Numeric,
    current_overage -> Numeric,
    max_allowed_overage -> Numeric,
    is_active -> Bool,
    ended_at -> Nullable<Timestamptz>,
    #[max_length = 50]
    end_reason -> Nullable<Varchar>,
    created_at -> Timestamptz,
    updated_at -> Timestamptz,
    branch_id -> Uuid,
    }
}
