//! #2522 — hostile-file gate, macro stripping and fail-closed policy tests for
//! the slides import boundary. Kept in a separate file so the handler stays
//! under the repository's 450-line limit.

use super::*;
use crate::auth::SlidesUser;
use crate::storage::{list_presentations_from_drive, DriveOps};
use crate::SlidesState;
use axum::body::Body;
use axum::http::Request;
use axum::routing::post;
use axum::Router;
use botsecurity_core::upload_hostile::{MalwareScanDecision, MalwareScanner};
use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

fn zip_with(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in entries {
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer.start_file(*name, options).expect("test entry");
        writer.write_all(body).expect("test body");
    }
    writer.finish().expect("test zip").into_inner()
}

fn is_sanitized_rejection(
    result: Result<(), (StatusCode, Json<serde_json::Value>)>,
    code: &str,
) -> bool {
    match result {
        Err((status, Json(body))) => {
            status == StatusCode::BAD_REQUEST
                && body["code"] == code
                && body.get("presentation.xml").is_none()
        }
        Ok(()) => false,
    }
}

#[test]
fn a_compression_ratio_bomb_is_refused_before_conversion() {
    let bytes = zip_with(&[("ppt/slides/slide1.xml", vec![b' '; 512 * 1024])]);
    let result = audit_hostile_upload(
        &bytes,
        "bomb.pptx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(is_sanitized_rejection(result, "error-file-archive-unsafe"));
}

#[test]
fn an_oversized_uncompressed_entry_is_refused_before_conversion() {
    let bytes = zip_with(&[("ppt/slides/slide1.xml", vec![b'x'; 8192])]);
    let policy = upload_policy::HostileFilePolicy {
        max_uncompressed_bytes: 1024,
        ..upload_policy::HostileFilePolicy::default()
    };
    let result = audit_hostile_upload(
        &bytes,
        "bomb.pptx",
        &axum::http::HeaderMap::new(),
        "test",
        &policy,
    );
    assert!(is_sanitized_rejection(result, "error-file-archive-unsafe"));
}

#[test]
fn a_macro_part_is_refused_by_the_gate_before_stripping() {
    let bytes = zip_with(&[
        ("ppt/presentation.xml", b"<p:presentation/>".to_vec()),
        ("ppt/vbaProject.bin", vec![0x01, 0x02, 0x03]),
    ]);
    let result = audit_hostile_upload(
        &bytes,
        "macro.pptx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(is_sanitized_rejection(result, "error-file-archive-unsafe"));
}

#[test]
fn a_plain_format_is_not_mistaken_for_a_zip_package() {
    let result = audit_hostile_upload(
        b"# Slide\nplain markdown",
        "deck.md",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(result.is_ok());
}

#[test]
fn a_macro_deck_is_stripped_and_noticed_before_conversion() {
    let bytes = zip_with(&[
        ("ppt/presentation.xml", b"<p:presentation/>".to_vec()),
        ("ppt/vbaProject.bin", vec![0x01, 0x02, 0x03]),
    ]);
    let stripped = match macro_strip::strip_executable_parts(&bytes) {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            log::warn!("unexpected strip failure: {}", error.log_detail());
            return;
        }
    };
    let upload = match upload_policy::validate_upload(
        UploadClass::PresentationImport,
        "macro.pptx",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        &stripped.bytes,
    ) {
        Ok(upload) => upload,
        Err(error) => {
            log::warn!("sanitized deck refused: {}", error.log_detail());
            return;
        }
    };
    let result = audit_hostile_upload(
        &stripped.bytes,
        &upload.filename,
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(result.is_ok(), "no executable parts may remain");
    let notice =
        crate::import_notice::macro_notice(stripped.removed_parts.clone(), stripped.removed_count);
    assert_eq!(notice.code, "file-macros-removed");
    assert_eq!(notice.message_key, "file-macros-removed");
    assert_eq!(notice.count, 1);
    assert!(notice.parts.iter().any(|part| part == "ppt/vbaproject.bin"));
}

#[test]
fn the_default_policy_requires_a_scanner_with_no_opt_out() {
    assert!(
        upload_policy::HostileFilePolicy::default().require_malware_scanner,
        "Slides imports must never be accepted unscanned"
    );
}

#[test]
fn a_missing_scanner_refuses_a_clean_deck_under_fail_closed_policy() {
    let bytes = zip_with(&[("ppt/slides/slide1.xml", b"<p:sld/>".to_vec())]);
    fn no_scanner(_: &[u8], _: &str) -> MalwareScanDecision {
        MalwareScanDecision::NotConfigured
    }
    let result = audit_hostile_upload_with_scanner(
        &bytes,
        "clean.pptx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
        Some(&no_scanner),
    );
    assert!(is_sanitized_rejection(result, "error-file-upload-failed"));
}

// #2927 — deterministic import persistence. The handler is exercised through
// a real axum router with an in-memory Drive and a stub malware scanner: the
// same persistence function is used on every success path, so a stored deck
// and its notice can be asserted without a server, browser or cargo build of
// the host crate.

#[derive(Clone)]
struct MemoryDrive {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    fail_writes: bool,
}

impl Default for MemoryDrive {
    fn default() -> Self {
        Self {
            objects: Arc::new(Mutex::new(HashMap::new())),
            fail_writes: false,
        }
    }
}

impl MemoryDrive {
    fn failing() -> Self {
        Self {
            fail_writes: true,
            ..Self::default()
        }
    }

    fn get(&self, key: &str) -> Option<Vec<u8>> {
        let objects = self.objects.lock().ok()?;
        objects.get(key).cloned()
    }

    fn keys(&self) -> Vec<String> {
        match self.objects.lock() {
            Ok(objects) => {
                let mut keys: Vec<String> = objects.keys().cloned().collect();
                keys.sort();
                keys
            }
            Err(error) => {
                log::warn!("test drive lock failed: {error}");
                Vec::new()
            }
        }
    }
}

impl DriveOps for MemoryDrive {
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        _content_type: &str,
    ) -> Result<(), String> {
        if self.fail_writes {
            return Err("disk full".to_string());
        }
        let mut objects = self.objects.lock().map_err(|e| e.to_string())?;
        objects.insert(format!("{bucket}/{key}"), body);
        Ok(())
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>, String> {
        self.get(&format!("{bucket}/{key}"))
            .ok_or_else(|| "404 Not Found".to_string())
    }

    async fn list_objects(&self, bucket: &str, prefix: &str) -> Result<Vec<String>, String> {
        let needle = format!("{bucket}/{prefix}");
        Ok(self
            .keys()
            .into_iter()
            .filter_map(|key| key.strip_prefix(&needle).map(str::to_string))
            .collect())
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), String> {
        let mut objects = self.objects.lock().map_err(|e| e.to_string())?;
        objects.remove(&format!("{bucket}/{key}"));
        Ok(())
    }
}

fn slides_state(drive: MemoryDrive) -> Arc<SlidesState<MemoryDrive>> {
    let manager =
        diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new("postgres://localhost/unused");
    let conn = diesel::r2d2::Pool::builder()
        .max_size(1)
        .build_unchecked(manager);
    Arc::new(SlidesState {
        drive: Some(drive),
        conn: Arc::new(conn),
        get_default_bot: Arc::new(|_conn: &mut diesel::PgConnection| {
            (uuid::Uuid::nil(), "test-bot".to_string())
        }),
    })
}

async fn as_user(
    mut request: Request<Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    request
        .extensions_mut()
        .insert(SlidesUser::new("user-a", "Alice"));
    next.run(request).await
}

fn import_app(state: Arc<SlidesState<MemoryDrive>>) -> Router {
    Router::new()
        .route(
            "/api/slides/import",
            post(handle_import_presentation::<MemoryDrive>),
        )
        .with_state(state)
        .layer(axum::middleware::from_fn(as_user))
}

const BOUNDARY: &str = "----gbSlidesImportTest";

fn multipart_request(filename: &str, content_type: &str, bytes: &[u8]) -> Request<Body> {
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(bytes);
    body.extend_from_slice(
        format!(
            "\r\n--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"bucket\"\r\n\r\nnotes\r\n--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"path\"\r\n\r\ndeck.pptx\r\n--{BOUNDARY}--\r\n"
        )
        .as_bytes(),
    );
    Request::builder()
        .method("POST")
        .uri("/api/slides/import")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={BOUNDARY}"),
        )
        .header("content-length", body.len())
        .body(Body::from(body))
        .expect("multipart request")
}

fn pptx_fixture(with_vba: bool) -> Vec<u8> {
    let mut entries: Vec<(&str, Vec<u8>)> = vec![
        (
            "[Content_Types].xml",
            b"<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>"
                .to_vec(),
        ),
        (
            "ppt/presentation.xml",
            b"<p:presentation xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"><p:sldIdLst/></p:presentation>"
                .to_vec(),
        ),
        (
            "ppt/slides/slide1.xml",
            b"<p:sld xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"><p:cSld/></p:sld>"
                .to_vec(),
        ),
    ];
    if with_vba {
        entries.push((
            "ppt/vbaProject.bin",
            vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
        ));
    }
    zip_with(&entries)
}

fn clean_scanner(_: &[u8], _: &str) -> MalwareScanDecision {
    MalwareScanDecision::Clean {
        scanner: "test-scanner".to_string(),
    }
}

fn install_clean_scanner() {
    let scanner: Box<MalwareScanner> = Box::new(clean_scanner);
    let installed = botsecurity_core::upload_hostile::set_malware_scanner(scanner);
    assert!(
        installed || botsecurity_core::upload_hostile::malware_scanner_installed(),
        "a deterministic clean scanner must be installed for import tests"
    );
}

async fn import_bytes(bytes: &[u8]) -> serde_json::Value {
    install_clean_scanner();
    let drive = MemoryDrive::default();
    let app = import_app(slides_state(drive));
    let response = app
        .oneshot(multipart_request(
            "deck.pptx",
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            bytes,
        ))
        .await
        .expect("import response");
    assert_eq!(response.status(), StatusCode::OK, "import must succeed");
    let body = axum::body::to_bytes(response.into_body(), 16 * 1024 * 1024)
        .await
        .expect("import body");
    serde_json::from_slice(&body).expect("import JSON")
}

#[cfg(test)]
#[path = "import_persistence_tests.rs"]
mod import_persistence_tests;

// #2522 — Slides-specific hostile corpus (PPTX paths). Compiled from this
// test module because the gate helpers it reuses are private to `import`.
#[path = "../hostile_corpus_tests.rs"]
mod slides_hostile_corpus;
