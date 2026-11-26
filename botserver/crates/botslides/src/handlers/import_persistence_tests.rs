//! #2927 — deterministic import persistence tests.
//!
//! An axum router with an in-memory Drive and a stub malware scanner exercises
//! the real import handler: a sanitized VBA deck must produce a stored
//! presentation, a macro-free notice and a macro-free package; a clean deck
//! must stay unchanged; a Drive write failure must be a typed, sanitized error.
//! No server, browser, cargo build of the host crate or live scanner is needed.

use super::*;

#[tokio::test]
async fn a_sanitized_vba_import_persists_the_deck_and_returns_the_notice() {
    let original = pptx_fixture(true);
    let stripped = match macro_strip::strip_executable_parts(&original) {
        Ok(Some(result)) => result,
        Ok(None) => panic!("fixture must contain a VBA part"),
        Err(error) => panic!("fixture strip failed: {}", error.log_detail()),
    };
    assert_eq!(stripped.removed_count, 1);

    // The handler strips the VBA part itself; feeding it the already
    // sanitized bytes would produce no notice and prove nothing.
    let body = import_bytes(&original).await;
    let id = body["id"].as_str().expect("presentation id");
    assert!(!id.is_empty(), "the response must carry a presentation id");
    assert_eq!(
        body["security_notices"][0]["message_key"],
        "file-macros-removed"
    );
    assert_eq!(body["security_notices"][0]["count"], 1);

    let state = slides_state(MemoryDrive::default());
    let drive = match state.drive.clone() {
        Some(drive) => drive,
        None => panic!("test state must carry a Drive"),
    };
    // The same deck the response describes is the one listed and stored.
    let app = import_app(state);
    let response = app
        .oneshot(multipart_request(
            "deck.pptx",
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            &original,
        ))
        .await
        .expect("second import response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 16 * 1024 * 1024)
        .await
        .expect("second import body");
    let body: serde_json::Value = serde_json::from_slice(&body).expect("second import JSON");
    assert_eq!(body["id"], id, "the Drive source id must be stable");

    let prefix = crate::storage::get_user_presentations_path("user-a");
    let key = format!("gbo/{prefix}/{id}.json");
    let raw = drive.get(&key).expect("presentation metadata row");
    let stored: Presentation = serde_json::from_slice(&raw).expect("stored deck JSON");
    assert_eq!(stored.id, id);
    let stored_pptx = drive
        .get(&format!("gbo/{prefix}/{id}.pptx"))
        .expect("sanitized PPTX");
    let mut archive =
        zip::ZipArchive::new(Cursor::new(stored_pptx.as_slice())).expect("stored PPTX is a zip");
    let mut stored_names = Vec::new();
    for index in 0..archive.len() {
        if let Ok(entry) = archive.by_index(index) {
            if let Ok(name) = entry.name() {
                stored_names.push(name.to_ascii_lowercase());
            }
        }
    }
    assert!(
        !stored_names.iter().any(|name| name == "ppt/vbaproject.bin"),
        "the macro-bearing original must never be stored"
    );
    let listed = list_presentations_from_drive(&drive, "user-a")
        .await
        .expect("list");
    assert_eq!(listed.len(), 1, "the imported deck must appear in /list");
    assert_eq!(listed[0].id, id);
}

#[tokio::test]
async fn a_macro_free_import_is_unchanged_and_has_no_security_notice() {
    let body = import_bytes(&pptx_fixture(false)).await;
    assert!(
        body.get("security_notices").is_none(),
        "a clean package must not be sanitized or noticed"
    );
    let id = body["id"].as_str().expect("presentation id");
    assert!(!id.is_empty());
}

#[tokio::test]
async fn a_storage_failure_is_a_typed_sanitized_error() {
    let drive = MemoryDrive::failing();
    let deck = crate::storage::create_new_presentation("user-a");
    let result = persist_import(
        Some(&drive),
        "user-a",
        &deck,
        None,
        vec![crate::import_notice::macro_notice(
            vec!["ppt/vbaproject.bin".to_string()],
            1,
        )],
    )
    .await;
    let (status, Json(body)) = match result {
        Ok(_) => panic!("a failed write must not answer 200"),
        Err(error) => error,
    };
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!body.to_string().contains("disk full"));
    assert!(body.get("error").is_some());
    assert!(drive.keys().is_empty(), "nothing may be stored on failure");
}
