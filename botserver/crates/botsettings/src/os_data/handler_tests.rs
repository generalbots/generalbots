//! Offline tests for the pure response builders of the data-protection surface
//! (#2470). Included as a child of `os_data_protection` via `#[path]`, so
//! `super::*` is that module. No AppState, no I/O, no server needed.

use super::*;

/// A refused/unimplemented decision carries a stable, generic error code.
fn assert_generic_error(spec: &ResponseSpec, code: &str) {
    assert_eq!(spec.body["error"], code);
    assert!(spec.body.get("error").is_some());
}

/// No caller-supplied secret marker may be echoed in a response body.
fn assert_no_secrets(body: &Value, secrets: &[&str]) {
    let rendered = body.to_string().to_ascii_lowercase();
    for secret in secrets {
        let needle = secret.to_ascii_lowercase();
        assert!(
            !rendered.contains(needle.as_str()),
            "secret marker leaked into response: {secret}"
        );
    }
}

#[test]
fn export_accepted_is_200_descriptor_with_zero_bytes_and_no_archive() {
    let spec = export_response(true);
    assert_eq!(spec.status, StatusCode::OK);
    assert_eq!(spec.body["status"], "descriptor_only");
    assert_eq!(spec.body["bytes_read"], 0);
    assert_eq!(spec.body["archive_created"], false);
    assert_eq!(spec.body["destructive"], false);
    assert_eq!(spec.body["source_root"], DATA_ROOT);
    assert_eq!(spec.body["scope"], ARCHIVE_SCOPE_FULL);
    assert_eq!(spec.body["format"], ARCHIVE_FORMAT);
}

#[test]
fn export_rejected_is_400_with_generic_code() {
    let spec = export_response(false);
    assert_eq!(spec.status, StatusCode::BAD_REQUEST);
    assert_generic_error(&spec, "unsupported_export_scope");
}

#[test]
fn snapshot_without_backend_is_503_with_generic_code() {
    // No backend registered (Drive absent) -> fail closed, nothing staged.
    let spec = snapshot_response(true, None);
    assert_eq!(spec.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_generic_error(&spec, "os_backend_unavailable");
    assert_eq!(spec.body["status"], "not_implemented");
    assert_eq!(spec.body["contract"], SNAPSHOT_CONTRACT_PATH);
    assert_eq!(spec.body["adapters"].as_array().map(Vec::len), Some(2));
}

#[test]
fn snapshot_with_drive_descriptor_is_202_non_destructive_without_secrets() {
    let reason = "nightly backup token=sk_live_secret";
    let descriptor = match snapshot_descriptor(&registered_snapshot_backends(true), reason) {
        Some(descriptor) => descriptor,
        None => Value::Null,
    };
    let spec = snapshot_response(true, Some(descriptor));

    assert_eq!(spec.status, StatusCode::ACCEPTED);
    assert_eq!(spec.body["status"], "staged");
    assert_eq!(spec.body["adapter"], "drive-archive");
    assert_eq!(spec.body["medium"], "drive-archive");
    assert_eq!(spec.body["destructive"], false);
    assert_eq!(spec.body["manifest"]["archive_created"], false);
    assert_eq!(spec.body["manifest"]["bytes_copied"], 0);
    // The raw reason (with its secret markers) must never leak into the body.
    assert_no_secrets(&spec.body, &["sk_live_secret", "nightly backup"]);
}

#[test]
fn snapshot_invalid_reason_is_400_with_generic_code() {
    let spec = snapshot_response(false, None);
    assert_eq!(spec.status, StatusCode::BAD_REQUEST);
    assert_generic_error(&spec, "invalid_reason");
}

#[test]
fn reset_mismatch_is_400_with_generic_code_and_no_token_echo() {
    let spec = reset_response(false);
    assert_eq!(spec.status, StatusCode::BAD_REQUEST);
    assert_generic_error(&spec, "confirmation_mismatch");
    assert_no_secrets(&spec.body, &[RESET_CONFIRMATION_TOKEN]);
}

#[test]
fn reset_confirmed_is_501_absent_with_generic_code_and_no_token_echo() {
    let spec = reset_response(true);
    assert_eq!(spec.status, StatusCode::NOT_IMPLEMENTED);
    assert_generic_error(&spec, "reset_backend_absent");
    assert_eq!(spec.body["status"], "not_implemented");
    assert_eq!(spec.body["contract"], RESET_CONTRACT_PATH);
    assert_no_secrets(&spec.body, &[RESET_CONFIRMATION_TOKEN]);
}
