// @generated from the actual migrations (6.0.21-01-compliance + 9.23-branch-scope-cleanup).
// Do not edit by hand; keep in sync with migrations/*/up.sql.

diesel::table! {
    compliance_checks (id) {
        id -> Uuid,
        org_id -> Nullable<Uuid>,
        bot_id -> Nullable<Uuid>,
        check_type -> Varchar,
        target_type -> Nullable<Varchar>,
        status -> Nullable<Varchar>,
        checked_at -> Nullable<Timestamptz>,
        checked_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        branch_id -> Uuid,
        target_id -> Nullable<Uuid>,
        result -> Nullable<Jsonb>,
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
        branch_id -> Uuid,
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
        branch_id -> Uuid,
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
        branch_id -> Uuid,
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
        branch_id -> Uuid,
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
        branch_id -> Uuid,
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
        branch_id -> Uuid,
    }
}

diesel::table! {
    compliance_frameworks (id) {
        id -> Uuid,
        branch_id -> Uuid,
        name -> Varchar,
        version -> Varchar,
        description -> Nullable<Text>,
        framework_key -> Varchar,
        status -> Varchar,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_controls (id) {
        id -> Uuid,
        branch_id -> Uuid,
        framework_id -> Uuid,
        control_id -> Varchar,
        title -> Varchar,
        description -> Nullable<Text>,
        category -> Nullable<Varchar>,
        is_mandatory -> Bool,
        version -> Varchar,
        status -> Varchar,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_control_evidence (id) {
        id -> Uuid,
        branch_id -> Uuid,
        control_id -> Uuid,
        file_path -> Varchar,
        description -> Nullable<Text>,
        evidence_type -> Varchar,
        status -> Varchar,
        owner_id -> Nullable<Uuid>,
        approved_by -> Nullable<Uuid>,
        approved_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    compliance_checks,
    compliance_issues,
    compliance_audit_log,
    compliance_evidence,
    compliance_risk_assessments,
    compliance_training_records,
    compliance_access_reviews,
    compliance_frameworks,
    compliance_controls,
    compliance_control_evidence,
);
