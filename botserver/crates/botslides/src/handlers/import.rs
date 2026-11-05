use crate::storage::{get_current_user_id, save_presentation_to_drive, DriveOps};
use crate::types::Presentation;
use crate::utils::{create_default_theme, slides_from_markdown};
use crate::SlidesState;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Deterministic deck id for a Drive source object (FNV-1a 64-bit). A
/// cryptographic hash buys nothing here: the id only needs to be stable and
/// collision-resistant enough for one user's presentations.
fn stable_source_id(bucket: &str, path: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in format!("gb-slides:{bucket}/{path}").bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("source-{hash:016x}")
}

pub async fn handle_import_presentation<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = "import.pptx".to_string();
    let mut source_bucket = String::new();
    let mut source_path = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("file") => {
                filename = field.file_name().unwrap_or("import.pptx").to_string();
                if let Ok(bytes) = field.bytes().await {
                    file_bytes = Some(bytes.to_vec());
                }
            }
            // The Drive opener sends the object identity so repeated opens of
            // the same file resolve to the same deck (#1710).
            Some("bucket") => {
                source_bucket = field.text().await.unwrap_or_default();
            }
            Some("path") => {
                source_path = field.text().await.unwrap_or_default();
            }
            _ => {}
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No file uploaded" })),
        )
    })?;

    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();
    let theme = create_default_theme();

    let slides = match ext.as_str() {
        "md" | "markdown" => {
            let content = String::from_utf8_lossy(&bytes);
            slides_from_markdown(&content)
        }
        "json" => {
            let pres: Result<Presentation, _> = serde_json::from_slice(&bytes);
            match pres {
                Ok(p) => p.slides,
                Err(e) => {
                    // A parse error carries the reader's offsets and internals;
                    // the client is told which input was rejected. Detail: log.
                    log::warn!("slides import: JSON parse failed: {e}");
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({ "error": "Invalid JSON" })),
                    ))
                }
            }
        }
        "pptx" => crate::pptx::load_pptx(&bytes)
            .map_err(|e| {
                log::warn!("slides import: PPTX parse failed: {e}");
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "Invalid PPTX" })),
                )
            })?
            .slides,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unsupported format: {ext}") })),
            ))
        }
    };

    let name = filename
        .rsplit('/')
        .next()
        .unwrap_or(&filename)
        .rsplit('.')
        .next_back()
        .unwrap_or(&filename)
        .to_string();

    let user_id = get_current_user_id();
    // Stable per-source identity: opening the same Drive object twice must
    // not create a duplicate deck (and must keep edits made in the first).
    let source_id = if source_bucket.is_empty() || source_path.is_empty() {
        None
    } else {
        Some(stable_source_id(&source_bucket, &source_path))
    };
    if let (Some(id), Some(drive)) = (source_id.as_ref(), state.drive.as_ref()) {
        if let Ok(existing) = crate::storage::load_presentation_by_id(drive, &user_id, id).await {
            return Ok(Json(existing));
        }
    }
    let presentation = Presentation {
        id: source_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name,
        owner_id: user_id.clone(),
        slides,
        theme,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    if let Some(drive) = &state.drive {
        if let Err(e) = save_presentation_to_drive(drive, &user_id, &presentation).await {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botslides::handlers::import", None)})),
            ));
        }
    }

    Ok(Json(presentation))
}
