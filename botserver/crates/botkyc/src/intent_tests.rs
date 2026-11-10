//! Canonical KYC request payloads: one spelling per keyword.

use serde_json::json;

use super::{KycRequest, VerificationKind};

#[test]
fn every_verification_kind_has_a_wire_name() {
    let expected = [
        (VerificationKind::FaceVerification, "face_verification"),
        (VerificationKind::DocumentValidation, "document_validation"),
        (VerificationKind::SignatureCapture, "signature_capture"),
        (VerificationKind::KycWorkflow, "kyc_workflow"),
    ];
    for (kind, wire) in expected {
        assert_eq!(kind.as_str(), wire);
        assert_eq!(serde_json::to_value(kind).expect("serializes"), json!(wire));
        let back: VerificationKind =
            serde_json::from_value(json!(wire)).expect("deserializes");
        assert_eq!(back, kind);
    }
    assert!(serde_json::from_value::<VerificationKind>(json!("face")).is_err());
}

#[test]
fn a_face_request_carries_the_profile_and_the_selfie() {
    let json = serde_json::to_value(KycRequest::face("p-1", "selfie.jpg")).expect("serializes");
    assert_eq!(
        json,
        json!({
            "kind": "face_verification",
            "action": "verify",
            "profile_id": "p-1",
            "selfie_url": "selfie.jpg",
            "status": "pending",
        })
    );
}

#[test]
fn a_document_request_carries_the_document_url() {
    let json =
        serde_json::to_value(KycRequest::document("p-2", "passport.jpg")).expect("serializes");
    assert_eq!(
        json,
        json!({
            "kind": "document_validation",
            "action": "validate",
            "profile_id": "p-2",
            "document_url": "passport.jpg",
            "verification_status": "pending",
        })
    );
}

#[test]
fn a_signature_request_carries_the_document_and_the_signature() {
    let json = serde_json::to_value(KycRequest::signature("p-3", "doc-9", "data:image/png;base64,AA"))
        .expect("serializes");
    assert_eq!(
        json,
        json!({
            "kind": "signature_capture",
            "action": "create",
            "profile_id": "p-3",
            "document_id": "doc-9",
            "signature_data": "data:image/png;base64,AA",
        })
    );
}

#[test]
fn a_workflow_request_starts_the_onboarding_step() {
    let json =
        serde_json::to_value(KycRequest::workflow("p-4", "onboarding")).expect("serializes");
    assert_eq!(
        json,
        json!({
            "kind": "kyc_workflow",
            "action": "start",
            "profile_id": "p-4",
            "workflow_name": "onboarding",
            "current_step": "document_upload",
            "status": "in_progress",
        })
    );
}

#[test]
fn optional_fields_are_omitted_rather_than_sent_as_null() {
    for request in [
        KycRequest::face("p", "s"),
        KycRequest::document("p", "d"),
        KycRequest::signature("p", "d", "s"),
        KycRequest::workflow("p", "w"),
    ] {
        let json = serde_json::to_value(&request).expect("serializes");
        for (key, value) in json.as_object().expect("object") {
            assert!(!value.is_null(), "field {key} must be omitted, not null");
        }
    }
}

#[test]
fn requests_are_cloneable_and_debuggable() {
    let request = KycRequest::workflow("p-5", "kyc-default");
    let clone = request.clone();
    assert_eq!(clone.kind, request.kind);
    assert_eq!(clone.profile_id.as_deref(), Some("p-5"));
    assert!(format!("{request:?}").contains("kyc-default"));
}
