// @generated automatically by script from migration SQL for issue #707.
// Aligned with 9.23-branch-scope-cleanup migration.

diesel::table! {
    compliance_checks (id) {
        id -> Uuid,
        branch_id -> Uuid,
        check_type -> Varchar,
        status -> Nullable<Varchar>,
        target_type -> Nullable<Varchar>,
        target_id -> Nullable<Uuid>,
        result -> Nullable<Jsonb>,
        checked_at -> Nullable<Timestamptz>,
        checked_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_issues (id) {
        id -> Uuid,
        branch_id -> Uuid,
        title -> Varchar,
        severity -> Varchar,
        status -> Nullable<Varchar>,
        description -> Nullable<Text>,
        assigned_to -> Nullable<Uuid>,
        remediation -> Nullable<Text>,
        due_date -> Nullable<Date>,
        resolved_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_audit_log (id) {
        id -> Uuid,
        branch_id -> Uuid,
        action -> Varchar,
        actor_id -> Nullable<Uuid>,
        target_type -> Nullable<Varchar>,
        target_id -> Nullable<Uuid>,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_evidence (id) {
        id -> Uuid,
        branch_id -> Uuid,
        check_id -> Nullable<Uuid>,
        file_path -> Varchar,
        description -> Nullable<Text>,
        uploaded_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_risk_assessments (id) {
        id -> Uuid,
        branch_id -> Uuid,
        title -> Varchar,
        risk_level -> Varchar,
        probability -> Nullable<Int4>,
        impact -> Nullable<Int4>,
        mitigation -> Nullable<Text>,
        status -> Nullable<Varchar>,
        assessed_by -> Nullable<Uuid>,
        assessed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_training_records (id) {
        id -> Uuid,
        branch_id -> Uuid,
        person_id -> Nullable<Uuid>,
        course_name -> Varchar,
        completed -> Nullable<Bool>,
        completed_at -> Nullable<Timestamptz>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    compliance_access_reviews (id) {
        id -> Uuid,
        branch_id -> Uuid,
        reviewer_id -> Nullable<Uuid>,
        reviewed_type -> Varchar,
        reviewed_id -> Uuid,
        decision -> Nullable<Varchar>,
        comments -> Nullable<Text>,
        reviewed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
