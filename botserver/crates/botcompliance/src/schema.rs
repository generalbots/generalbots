use diesel::table;

table! {
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

table! {
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

table! {
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
        user_agent -> Nullable<Varchar>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
    }
}

table! {
    compliance_evidence (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        check_id -> Nullable<Uuid>,
        issue_id -> Nullable<Uuid>,
        evidence_type -> Varchar,
        title -> Varchar,
        description -> Nullable<Text>,
        file_url -> Nullable<Varchar>,
        file_name -> Nullable<Varchar>,
        file_size -> Nullable<Int4>,
        mime_type -> Nullable<Varchar>,
        metadata -> Jsonb,
        collected_at -> Timestamptz,
        collected_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

table! {
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

table! {
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

table! {
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
        certificate_url -> Nullable<Varchar>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
    }
}

table! {
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
