use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use log::trace;
use rhai::{Dynamic, Engine};
use serde_json::{json, Value};
use std::sync::Arc;

/// KYC and identity BASIC keywords for issue #622.
///
/// Provides: VERIFY FACE, VALIDATE DOCUMENT, CAPTURE SIGNATURE, START KYC.
pub fn register_kyc_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    register_verify_face(state.clone(), user.clone(), engine);
    register_validate_document(state.clone(), user.clone(), engine);
    register_capture_signature(state.clone(), user.clone(), engine);
    register_start_kyc(state.clone(), user.clone(), engine);
}

fn register_verify_face(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["VERIFY", "FACE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let profile_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let selfie_url = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("VERIFY FACE: profile={profile_id} selfie={selfie_url}");
                let result = json!({
                    "kind": "face_verification",
                    "action": "verify",
                    "profile_id": profile_id,
                    "selfie_url": selfie_url,
                    "status": "pending",
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid VERIFY FACE syntax");
}

fn register_validate_document(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["VALIDATE", "DOCUMENT", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let profile_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let document_url = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("VALIDATE DOCUMENT: profile={profile_id} doc={document_url}");
                let result = json!({
                    "kind": "document_validation",
                    "action": "validate",
                    "profile_id": profile_id,
                    "document_url": document_url,
                    "verification_status": "pending",
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid VALIDATE DOCUMENT syntax");
}

fn register_capture_signature(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["CAPTURE", "SIGNATURE", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let profile_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let document_id = context.eval_expression_tree(&inputs[1])?.to_string();
                let signature_data = context.eval_expression_tree(&inputs[2])?.to_string();
                trace!("CAPTURE SIGNATURE: profile={profile_id} doc={document_id}");
                let result = json!({
                    "kind": "signature_capture",
                    "action": "create",
                    "profile_id": profile_id,
                    "document_id": document_id,
                    "signature_data": signature_data,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid CAPTURE SIGNATURE syntax");
}

fn register_start_kyc(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["START", "KYC", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let profile_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let workflow_name = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("START KYC: profile={profile_id} workflow={workflow_name}");
                let result = json!({
                    "kind": "kyc_workflow",
                    "action": "start",
                    "profile_id": profile_id,
                    "workflow_name": workflow_name,
                    "current_step": "document_upload",
                    "status": "in_progress",
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid START KYC syntax");
}

fn serde_json_to_dynamic(v: &Value) -> Dynamic {
    Dynamic::from(v.to_string())
}
