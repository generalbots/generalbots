use diesel::prelude::*;

table! {
    identity_profiles (id) {
        id -> Uuid,
        bot_id -> Uuid,
        person_id -> Uuid,
        legal_name -> Text,
        tax_id -> Text,
        date_of_birth -> Nullable<Date>,
        nationality -> Nullable<Text>,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        address -> Nullable<Jsonb>,
        risk_score -> Nullable<Integer>,
        kyc_status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    identity_faces (id) {
        id -> Uuid,
        bot_id -> Uuid,
        profile_id -> Uuid,
        photo_url -> Text,
        embedding -> Nullable<Jsonb>,
        quality_score -> Nullable<Numeric>,
        is_primary -> Bool,
        created_at -> Timestamptz,
    }
}

table! {
    identity_documents (id) {
        id -> Uuid,
        bot_id -> Uuid,
        profile_id -> Uuid,
        document_type -> Text,
        document_number -> Text,
        issuing_country -> Nullable<Text>,
        issue_date -> Nullable<Date>,
        expiry_date -> Nullable<Date>,
        front_image_url -> Nullable<Text>,
        back_image_url -> Nullable<Text>,
        selfie_image_url -> Nullable<Text>,
        ocr_data -> Nullable<Jsonb>,
        verification_status -> Text,
        created_at -> Timestamptz,
    }
}

table! {
    identity_signatures (id) {
        id -> Uuid,
        bot_id -> Uuid,
        profile_id -> Uuid,
        document_id -> Uuid,
        signature_data -> Text,
        signature_image_url -> Nullable<Text>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        signed_at -> Timestamptz,
    }
}

table! {
    identity_signed_documents (id) {
        id -> Uuid,
        bot_id -> Uuid,
        signature_id -> Uuid,
        document_hash -> Text,
        document_name -> Text,
        signature_algorithm -> Text,
        signed_at -> Timestamptz,
    }
}

table! {
    identity_kyc_workflows (id) {
        id -> Uuid,
        bot_id -> Uuid,
        profile_id -> Uuid,
        workflow_name -> Text,
        current_step -> Text,
        steps_completed -> Jsonb,
        total_steps -> Integer,
        status -> Text,
        started_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

allow_tables_to_appear_in_same_query!(
    identity_profiles,
    identity_faces,
    identity_documents,
    identity_signatures,
    identity_signed_documents,
    identity_kyc_workflows,
);
