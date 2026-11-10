//! Cookie/legal document helpers: the document-type label round trip that the
//! REST layer relies on, the default cookie policy, and the two DB→API
//! converters (including the unknown-type fallback).
use super::*;

use axum::http::StatusCode;

fn db_consent() -> DbCookieConsent {
    DbCookieConsent {
        id: Uuid::from_u128(1),
        org_id: Uuid::nil(),
        bot_id: Uuid::nil(),
        user_id: None,
        session_id: Some("session-1".to_string()),
        ip_address: None,
        user_agent: None,
        country_code: Some("BR".to_string()),
        consent_necessary: true,
        consent_analytics: true,
        consent_marketing: false,
        consent_preferences: true,
        consent_functional: false,
        consent_version: "1.0.0".to_string(),
        consent_given_at: Utc::now(),
        consent_updated_at: Utc::now(),
        consent_withdrawn_at: None,
        created_at: Utc::now(),
        branch_id: Uuid::nil(),
    }
}

fn db_document(document_type: &str) -> DbLegalDocument {
    let now = Utc::now();
    DbLegalDocument {
        id: Uuid::from_u128(2),
        org_id: Uuid::nil(),
        bot_id: Uuid::nil(),
        slug: "privacy".to_string(),
        title: "Privacy".to_string(),
        content: "text".to_string(),
        document_type: document_type.to_string(),
        version: "2".to_string(),
        effective_date: now,
        is_active: true,
        requires_acceptance: true,
        metadata: serde_json::json!({}),
        created_by: None,
        created_at: now,
        updated_at: now,
        branch_id: Uuid::nil(),
    }
}

#[test]
fn every_document_type_survives_a_label_round_trip() {
    let types = [
        LegalDocumentType::PrivacyPolicy,
        LegalDocumentType::TermsOfService,
        LegalDocumentType::CookiePolicy,
        LegalDocumentType::AcceptableUsePolicy,
        LegalDocumentType::DataProcessingAgreement,
        LegalDocumentType::GdprCompliance,
        LegalDocumentType::CcpaCompliance,
    ];

    for document_type in types {
        let label = document_type.to_string();
        let parsed: LegalDocumentType = label.parse().expect("label must parse back");
        assert_eq!(parsed, document_type);
    }
}

#[test]
fn an_unknown_document_type_label_is_rejected_with_the_value() {
    let error = "sla".parse::<LegalDocumentType>().expect_err("must be rejected");

    assert!(error.contains("sla"));
}

#[test]
fn error_statuses_map_to_the_right_http_code() {
    let cases = [
        (LegalError::NotFound("x".into()), StatusCode::NOT_FOUND),
        (LegalError::Validation("x".into()), StatusCode::BAD_REQUEST),
        (
            LegalError::Database("x".into()),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            LegalError::Internal("x".into()),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ];

    for (error, expected) in cases {
        let response = error.into_response();
        assert_eq!(response.status(), expected);
    }
}

#[test]
fn a_consent_row_becomes_a_category_map() {
    let consent = db_consent_to_consent(db_consent());

    assert_eq!(consent.id, Uuid::from_u128(1));
    assert_eq!(consent.consents.get(&CookieCategory::Necessary), Some(&true));
    assert_eq!(consent.consents.get(&CookieCategory::Analytics), Some(&true));
    assert_eq!(consent.consents.get(&CookieCategory::Marketing), Some(&false));
    assert_eq!(
        consent.consents.get(&CookieCategory::Preferences),
        Some(&true)
    );
    assert_eq!(
        consent.consents.get(&CookieCategory::Functional),
        Some(&false)
    );
    assert_eq!(consent.consents.len(), 5);
}

#[test]
fn the_default_policy_marks_only_necessary_as_required() {
    let policy = get_default_cookie_policy();

    assert_eq!(policy.version, "1.0.0");
    assert_eq!(policy.categories.len(), 5);

    for category in &policy.categories {
        let expected = category.category == CookieCategory::Necessary;
        assert_eq!(category.is_required, expected, "{:?}", category.category);
    }

    let necessary = policy
        .categories
        .iter()
        .find(|category| category.category == CookieCategory::Necessary)
        .expect("necessary category");
    assert_eq!(necessary.cookies.len(), 1);
    assert_eq!(necessary.cookies[0].name, "session_id");
    assert_eq!(necessary.cookies[0].cookie_type, CookieType::Session);
}

#[test]
fn a_document_row_maps_its_type_and_timestamps() {
    let row = db_document("terms_of_service");
    let updated_at = row.updated_at;
    let document = db_document_to_document(row);

    assert_eq!(document.document_type, LegalDocumentType::TermsOfService);
    assert_eq!(document.id, Uuid::from_u128(2).to_string());
    assert_eq!(document.version, "2");
    assert_eq!(document.last_updated, updated_at);
}

#[test]
fn an_unknown_stored_document_type_falls_back_to_the_privacy_policy() {
    let document = db_document_to_document(db_document("legacy_unknown"));

    assert_eq!(document.document_type, LegalDocumentType::PrivacyPolicy);
}
