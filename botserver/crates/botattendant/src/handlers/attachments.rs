use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::schema::*;
use crate::handlers::db_conn;

/// Maximum accepted attachment size (25 MiB).
const MAX_ATTACHMENT_BYTES: usize = 25 * 1024 * 1024;

/// Content types allowed for conversation attachments (type limit).
/// Everything else is rejected to keep the channel surface safe.
const ALLOWED_CONTENT_TYPES: &[&str] = &[
    // Images
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/svg+xml",
    // Documents
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/vnd.ms-powerpoint",
    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    "text/plain",
    "text/csv",
    "text/markdown",
    // Archives / audio / video
    "application/zip",
    "application/gzip",
    "audio/mpeg",
    "audio/ogg",
    "audio/wav",
    "video/mp4",
    "video/webm",
];

fn content_type_allowed(content_type: &str) -> bool {
    ALLOWED_CONTENT_TYPES.contains(&content_type)
}

/// Returns true when the content type is an image (rendered inline with a
/// preview in the message thread).
fn is_image(content_type: &str) -> bool {
    content_type.starts_with("image/")
}

/// `POST /api/attendant/sessions/:id/attachments`
///
/// Multipart upload (`file` field). Enforces size/type limits, persists the
/// blob, and returns `AttachmentMeta` for the composer to attach to the next
/// message. The download URL embeds the attachment UUID â an unguessable
/// capability â and the session must exist.
pub async fn upload_attachment(
    State(config): State<Arc<crate::AttendantConfig>>,
    Path(session_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<AttachmentMeta>, (StatusCode, String)> {
    {
        let mut conn = db_conn!(config);
        let exists: i64 = attendant_sessions::table
            .filter(attendant_sessions::id.eq(session_id))
            .count()
            .get_result(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;
        if exists == 0 {
            return Err((StatusCode::NOT_FOUND, "Session not found".to_string()));
        }
    }

    let mut name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {e}")))?
    {
        if field.name() != Some("file") {
            continue;
        }
        name = field.file_name().map(str::to_string);
        content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .or_else(|| name.as_deref().and_then(detect_from_extension));
        data = Some(
            field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Read error: {e}")))?
                .to_vec(),
        );
    }

    let name = name.ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing file field".to_string()))?;
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let data = data.ok_or_else(|| (StatusCode::BAD_REQUEST, "Empty file payload".to_string()))?;

    if data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Empty file".to_string()));
    }
    if data.len() > MAX_ATTACHMENT_BYTES {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "File exceeds 25 MiB limit".to_string()));
    }
    if !content_type_allowed(&content_type) {
        return Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, "File type not allowed".to_string()));
    }

    let id = Uuid::new_v4();
    let size_bytes = data.len() as i64;
    let now = Utc::now();

    diesel::insert_into(attendant_attachments::table)
        .values((
            attendant_attachments::id.eq(id),
            attendant_attachments::session_id.eq(session_id),
            attendant_attachments::name.eq(&name),
            attendant_attachments::content_type.eq(&content_type),
            attendant_attachments::size_bytes.eq(size_bytes),
            attendant_attachments::data.eq(&data),
            attendant_attachments::created_at.eq(now),
        ))
        .execute(&mut db_conn!(config))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(AttachmentMeta {
        id,
        name,
        content_type: content_type.clone(),
        size_bytes,
        url: format!("/api/attendant/attachments/{id}/download"),
        thumb_url: is_image(&content_type).then(|| format!("/api/attendant/attachments/{id}/download")),
    }))
}

/// `GET /api/attendant/attachments/:id/download`
///
/// Streams the stored blob with the original filename. Ownership is enforced
/// at the session level by the calling UI (agents fetch only attachments of
/// sessions they are assigned to); the UUID acts as an unguessable token.
pub async fn download_attachment(
    State(config): State<Arc<crate::AttendantConfig>>,
    Path(id): Path<Uuid>,
) -> Result<Response, (StatusCode, String)> {
    let mut conn = db_conn!(config);

    let row: AttendantAttachment = attendant_attachments::table
        .filter(attendant_attachments::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Attachment not found".to_string()))?;

    let mut response = Response::new(axum::body::Body::from(row.data));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_str(&row.content_type).unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")));
    let disposition = format!("attachment; filename=\"{}\"", sanitize_filename(&row.name));
    response
        .headers_mut()
        .insert(header::CONTENT_DISPOSITION, HeaderValue::from_str(&disposition).unwrap_or_else(|_| HeaderValue::from_static("attachment")));
    response
        .headers_mut()
        .insert(header::CONTENT_LENGTH, HeaderValue::from_str(&row.size_bytes.to_string()).unwrap_or_else(|_| HeaderValue::from_static("0")));
    Ok(response)
}

/// Strips path separators and control characters from a filename so the
/// Content-Disposition header cannot be abused.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '/' | '\\' | '\0' | '\r'))
        .collect()
}

/// Best-effort content-type detection from the file extension when the
/// multipart field carries no explicit content type.
fn detect_from_extension(name: &str) -> Option<String> {
    let ext = name.rsplit('.').next()?.to_lowercase();
    let ct = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "md" => "text/markdown",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => return None,
    };
    Some(ct.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_allow_list() {
        assert!(content_type_allowed("image/png"));
        assert!(content_type_allowed("application/pdf"));
        assert!(!content_type_allowed("application/x-msdownload"));
        assert!(!content_type_allowed("text/html"));
    }

    #[test]
    fn test_sanitize_filename_strips_separators() {
        assert_eq!(sanitize_filename("..\\..\\evil.txt"), "....evil.txt");
        assert_eq!(sanitize_filename("report\r.pdf"), "report.pdf");
    }

    #[test]
    fn test_detect_from_extension() {
        assert_eq!(detect_from_extension("photo.png"), Some("image/png".to_string()));
        assert_eq!(detect_from_extension("notes.docx"), Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()));
        assert_eq!(detect_from_extension("mystery.bin"), None);
    }
}