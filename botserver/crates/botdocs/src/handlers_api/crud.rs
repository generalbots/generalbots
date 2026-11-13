use crate::state::DocState;
use crate::storage::{
    create_document_with, create_new_document, delete_document_from_drive,
    list_documents_from_drive, load_document_from_drive, save_document_to_drive,
};
use crate::types::{CreateDocumentRequest, DocsSaveRequest, DocsSaveResponse, Document, DocumentMetadata, SearchQuery};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::auth::DocUser;

/// Resolution of the title stored with a save. The editor autosave sends only
/// `{id, content}`; an empty title must keep the existing one instead of
/// renaming the document to "Untitled Document" on every keystroke.
async fn effective_title(
    state: &DocState,
    user_id: &str,
    doc_id: &str,
    requested: &str,
) -> String {
    if !requested.trim().is_empty() {
        return requested.to_string();
    }
    match load_document_from_drive(state, user_id, doc_id).await {
        Ok(Some(doc)) if !doc.title.trim().is_empty() => doc.title,
        _ => "Untitled Document".to_string(),
    }
}

/// Resolution of the body stored with a save. `append` keeps the existing body
/// and adds the new content after it; any other mode replaces it (#2143).
async fn effective_content(
    state: &DocState,
    user_id: &str,
    doc_id: &str,
    requested: &str,
    mode: Option<&str>,
) -> String {
    if mode != Some("append") {
        return requested.to_string();
    }
    match load_document_from_drive(state, user_id, doc_id).await {
        Ok(Some(doc)) => format!("{}{}", doc.content, requested),
        _ => requested.to_string(),
    }
}

pub async fn handle_docs_save(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(req): Json<DocsSaveRequest>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();
    let doc_id = req.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    let title = effective_title(&state, &user_id, &doc_id, &req.title).await;
    let content = effective_content(&state, &user_id, &doc_id, &req.content, req.mode.as_deref()).await;

    // #1138: archive the previous version before overwriting it.
    super::history::capture_snapshot(&state, &doc_id, &title, &content).await;

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc_id, &title, &content).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::crud", None)})),
        ));
    }

    Ok(Json(DocsSaveResponse {
        id: doc_id,
        success: true,
    }))
}

pub async fn handle_docs_get_by_id(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Path(doc_id): Path<String>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();

    match load_document_from_drive(&state, &user_id, &doc_id).await {
        Ok(Some(doc)) => {
            // Seed undo history with the loaded version so a later in-editor
            // transformation (e.g. translate) is undoable in one step.
            super::history::capture_snapshot(&state, &doc.id, &doc.title, &doc.content).await;
            Ok(Json(doc))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Document not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::crud", None)})),
        )),
    }
}

pub async fn handle_new_document(
    user: DocUser,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(create_new_document(user.id())))
}

/// `POST /api/docs/new` — create a document from `{title, content}` and return
/// it with the id the deep link opens (#2143).
pub async fn handle_create_document(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();
    let doc = create_document_with(&user_id, req.title.as_deref(), req.content.as_deref());

    if let Err(e) =
        save_document_to_drive(&state, &user_id, &doc.id, &doc.title, &doc.content).await
    {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::crud", None)})),
        ));
    }

    Ok(Json(doc))
}

pub async fn handle_list_documents(
    State(state): State<Arc<DocState>>,
    user: DocUser,
) -> Result<Json<Vec<DocumentMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();

    match list_documents_from_drive(&state, &user_id).await {
        Ok(docs) => Ok(Json(docs)),
        Err(e) => {
            log::error!("Failed to list documents: {}", e);
            Ok(Json(Vec::new()))
        }
    }
}

pub async fn handle_search_documents(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<DocumentMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();

    let docs = list_documents_from_drive(&state, &user_id)
        .await
        .unwrap_or_default();

    let filtered = if let Some(q) = query.q {
        let q_lower = q.to_lowercase();
        docs.into_iter()
            .filter(|d| d.title.to_lowercase().contains(&q_lower))
            .collect()
    } else {
        docs
    };

    Ok(Json(filtered))
}

pub async fn handle_get_document(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Query(query): Query<crate::types::LoadQuery>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();

    let doc_id = query.id.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Document ID required" })),
        )
    })?;

    match load_document_from_drive(&state, &user_id, &doc_id).await {
        Ok(Some(doc)) => {
            super::history::capture_snapshot(&state, &doc.id, &doc.title, &doc.content).await;
            Ok(Json(doc))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Document not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::crud", None)})),
        )),
    }
}

pub async fn handle_save_document(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(req): Json<DocsSaveRequest>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();
    let doc_id = req.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    let title = effective_title(&state, &user_id, &doc_id, &req.title).await;
    let content = effective_content(&state, &user_id, &doc_id, &req.content, req.mode.as_deref()).await;

    // Autosave is the path the in-editor translate/format flows use; without
    // this snapshot the previous version never reached the undo stack and
    // `POST /api/docs/history` answered 204 (#2256).
    super::history::capture_snapshot(&state, &doc_id, &title, &content).await;

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc_id, &title, &content).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::crud", None)})),
        ));
    }

    Ok(Json(DocsSaveResponse {
        id: doc_id,
        success: true,
    }))
}

pub async fn handle_autosave(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(req): Json<DocsSaveRequest>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    handle_save_document(State(state), user, Json(req)).await
}

pub async fn handle_delete_document(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(req): Json<crate::types::LoadQuery>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();

    let doc_id = req.id.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Document ID required" })),
        )
    })?;

    if let Err(e) = delete_document_from_drive(&state, &user_id, &doc_id).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::crud", None)})),
        ));
    }

    Ok(Json(DocsSaveResponse {
        id: doc_id,
        success: true,
    }))
}

/// Open an arbitrary file from the Drive (any bucket) inside the docs editor.
///
/// Reads the object bytes from MinIO/S3 and converts binary office formats
/// (.docx/.doc/.odt/.rtf) to HTML so the content is editable. Plain text
/// files (.txt/.md/.html) are used as-is.
pub async fn handle_open_from_drive(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(req): Json<crate::types::OpenFromDriveRequest>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    if req.bucket.is_empty() || req.path.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "bucket and path are required" })),
        ));
    }

    let bytes = match state.drive.get_object(&req.bucket, &req.path).await {
        Ok(b) => b,
        Err(e) => {
            // The drive error text names the bucket and object path; the client
            // only needs the lookup result. The detail goes to the log.
            log::warn!("docs editor: drive read of {}/{} failed: {}", req.bucket, req.path, e);
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "File not found in drive" })),
            ))
        }
    };

    let file_name = req
        .path
        .rsplit('/')
        .next()
        .unwrap_or(&req.path)
        .to_string();
    let ext = file_name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    let content = match ext.as_str() {
        // #2254 — a PDF opened from Drive becomes editable paragraphs.
        "pdf" => match crate::import_pdf::pdf_to_editor_html(&bytes) {
            Ok(html) => html,
            Err(e) => {
                log::warn!("docs editor: PDF import of {} failed: {e}", req.path);
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({ "error": e })),
                ));
            }
        },
        "docx" | "doc" | "odt" | "rtf" => match crate::storage::convert_docx_to_html(&bytes) {
            Ok(html) => html,
            Err(e) => {
                log::error!("Failed to convert {} to HTML: {}", req.path, e);
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::crud", None) })),
                ));
            }
        },
        "txt" | "md" | "html" | "htm" | "json" | "xml" | "csv" => {
            String::from_utf8_lossy(&bytes).into_owned()
        }
        _ => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": format!("Unsupported file type for docs editor: .{ext}") })),
            ))
        }
    };

    let now = chrono::Utc::now();
    let title = file_name
        .rsplit('.')
        .next_back()
        .map(|stem| file_name.trim_end_matches(&format!(".{stem}")).to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(file_name.clone());

    Ok(Json(Document {
        id: format!("drive-{}", req.path.replace(['/', '.', ':'], "-")),
        title,
        content,
        owner_id: user.id().to_string(),
        storage_path: format!("{}/{}", req.bucket, req.path),
        created_at: now,
        updated_at: now,
        collaborators: Vec::new(),
        version: 1,
        track_changes: None,
        comments: None,
        footnotes: None,
        endnotes: None,
        styles: None,
        toc: None,
        track_changes_enabled: false,
    }))
}
