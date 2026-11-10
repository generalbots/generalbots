//! Upload tests for #2047 — capability, request construction and the
//! honest-error contract. Split out of `upload.rs` so that file stays under
//! the size limit.

use super::*;

fn request(provider: Provider, name: &str) -> UploadRequest {
    UploadRequest {
        provider,
        path: "artifacts".to_string(),
        name: name.to_string(),
        mime_type: "text/plain".to_string(),
        content: b"hello".to_vec(),
        replace_remote_id: None,
        parent_id: None,
    }
}

#[test]
fn oversized_content_is_refused_before_building_a_request() {
    let mut req = request(Provider::GoogleDrive, "big.txt");
    req.content = vec![0; MAX_UPLOAD_BYTES + 1];
    assert_eq!(
        build_target(&req, ""),
        Err(UploadError::TooLarge(MAX_UPLOAD_BYTES + 1))
    );
    assert_eq!(UploadError::TooLarge(1).kind(), "too_large");
}

#[test]
fn google_refuses_above_its_multipart_limit_instead_of_failing_upstream() {
    // Drive's multipart upload is documented for files up to 5 MB; a larger
    // body would come back as a provider error, which hides the real cause.
    let google_limit = max_upload_bytes(Provider::GoogleDrive);
    assert!(google_limit < max_upload_bytes(Provider::OneDrive));
    let mut req = request(Provider::GoogleDrive, "big.csv");
    req.content = vec![0; google_limit + 1];
    assert_eq!(
        build_target(&req, ""),
        Err(UploadError::TooLarge(google_limit + 1))
    );
    // The same payload is fine for OneDrive's simple PUT.
    req.provider = Provider::OneDrive;
    assert!(build_target(&req, "Files.ReadWrite.AppFolder").is_ok());
}

#[test]
fn the_mirror_path_cannot_climb_out_of_the_target_folder() {
    assert_eq!(
        mirror_path("artifacts/../private", "notes.txt"),
        "artifacts/private/notes.txt"
    );
    assert_eq!(mirror_path("", "notes.txt"), "notes.txt");
    assert_eq!(mirror_path("/", "a/b.txt"), "a-b.txt");
    assert_eq!(mirror_path("..", ".."), "artifact");
}

#[test]
fn error_kinds_are_the_contract_the_ui_reads() {
    assert_eq!(UploadError::NotConfigured.kind(), "not_configured");
    assert_eq!(UploadError::NotConnected.kind(), "not_connected");
    assert_eq!(
        UploadError::MissingWriteScope.kind(),
        "missing_write_scope"
    );
    assert_eq!(UploadError::Provider("x".into()).kind(), "provider_error");
}

#[test]
fn write_capability_comes_from_the_recorded_scope() {
    assert!(allows_write(
        Provider::GoogleDrive,
        "https://www.googleapis.com/auth/drive.readonly https://www.googleapis.com/auth/drive.file"
    ));
    assert!(!allows_write(
        Provider::GoogleDrive,
        "https://www.googleapis.com/auth/drive.readonly"
    ));
    assert!(!allows_write(Provider::OneDrive, ""));
    assert!(allows_write(
        Provider::OneDrive,
        "offline_access Files.Read.All Files.ReadWrite.AppFolder User.Read"
    ));
}

#[test]
fn onedrive_uses_the_app_folder_unless_a_broad_write_scope_was_granted() {
    assert!(onedrive_app_folder_only(
        "offline_access Files.Read.All Files.ReadWrite.AppFolder"
    ));
    assert!(!onedrive_app_folder_only("Files.ReadWrite"));
    assert!(!onedrive_app_folder_only("Files.ReadWrite.All"));
}

#[test]
fn google_target_is_multipart_related_with_metadata() {
    let mut req = request(Provider::GoogleDrive, "pib.csv");
    req.parent_id = Some("folder-9".to_string());
    let prepared = build_target(&req, "").expect("target");
    assert_eq!(prepared.method, "POST");
    assert!(prepared.url.contains("uploadType=multipart"));
    assert!(prepared
        .content_type
        .starts_with("multipart/related; boundary="));
    let body = String::from_utf8_lossy(&prepared.body);
    assert!(body.contains(r#""name":"pib.csv""#));
    assert!(body.contains(r#""parents":["folder-9"]"#));
    assert!(body.contains("hello"));
}

#[test]
fn onedrive_target_puts_the_bytes_to_the_path() {
    let prepared = build_target(
        &request(Provider::OneDrive, "pib.csv"),
        "Files.ReadWrite.AppFolder",
    )
    .expect("target");
    assert_eq!(prepared.method, "PUT");
    assert!(prepared
        .url
        .contains("/me/drive/special/approot:/artifacts/pib.csv:/content"));
    assert_eq!(prepared.body, b"hello");
}
