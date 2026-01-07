//! GB Docs - Word Processor Module
//!
//! This module provides a Word-like document editor with:
//! - Rich text document management
//! - Real-time multi-user collaboration via WebSocket
//! - Templates (blank, meeting, report, letter)
//! - AI-powered writing assistance
//! - Export to multiple formats (PDF, DOCX, HTML, TXT, MD)


use crate::core::urls::ApiUrls;
use crate::shared::state::AppState;
use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::header::HeaderMap,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

// =============================================================================
// COLLABORATION TYPES
// =============================================================================

type CollaborationChannels =
    Arc<tokio::sync::RwLock<HashMap<String, broadcast::Sender<CollabMessage>>>>;

static COLLAB_CHANNELS: std::sync::OnceLock<CollaborationChannels> = std::sync::OnceLock::new();

fn get_collab_channels() -> &'static CollaborationChannels {
    COLLAB_CHANNELS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabMessage {
    pub msg_type: String,         // "cursor", "edit", "format", "join", "leave"
    pub doc_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<usize>,  // Cursor position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,    // Selection length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,  // Inserted/changed content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,   // Format command (bold, italic, etc.)
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    pub id: String,
    pub name: String,
    pub color: String,
    pub cursor_position: Option<usize>,
    pub selection_length: Option<usize>,
    pub connected_at: DateTime<Utc>,
}

// =============================================================================
// DOCUMENT TYPES
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,           // HTML content for rich text
    pub owner_id: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub collaborators: Vec<String>, // User IDs with access
    #[serde(default)]
    pub version: u64,               // For conflict resolution
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: String,
    pub title: String,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub word_count: usize,
    pub storage_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResponse {
    pub id: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    #[serde(rename = "selected-text", alias = "text")]
    pub selected_text: Option<String>,
    pub prompt: Option<String>,
    pub action: Option<String>,
    #[serde(rename = "translate-lang")]
    pub translate_lang: Option<String>,
    pub document_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub result: Option<String>,
    pub content: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportQuery {
    pub id: Option<String>,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub email: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub username: String,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct UserIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    user_id: Uuid,
}

// =============================================================================
// ROUTE CONFIGURATION
// =============================================================================

pub fn configure_docs_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiUrls::DOCS_NEW, post(handle_new_document))
        .route(ApiUrls::DOCS_LIST, get(handle_list_documents))
        .route(ApiUrls::DOCS_SEARCH, get(handle_search_documents))
        .route(ApiUrls::DOCS_SAVE, post(handle_save_document))
        .route(ApiUrls::DOCS_AUTOSAVE, post(handle_autosave))
        .route(ApiUrls::DOCS_BY_ID, get(handle_get_document))
        .route(ApiUrls::DOCS_DELETE, post(handle_delete_document))
        .route(ApiUrls::DOCS_TEMPLATE_BLANK, post(handle_template_blank))
        .route(ApiUrls::DOCS_TEMPLATE_MEETING, post(handle_template_meeting))
        .route(ApiUrls::DOCS_TEMPLATE_REPORT, post(handle_template_report))
        .route(ApiUrls::DOCS_TEMPLATE_LETTER, post(handle_template_letter))
        .route(ApiUrls::DOCS_AI_SUMMARIZE, post(handle_ai_summarize))
        .route(ApiUrls::DOCS_AI_EXPAND, post(handle_ai_expand))
        .route(ApiUrls::DOCS_AI_IMPROVE, post(handle_ai_improve))
        .route(ApiUrls::DOCS_AI_SIMPLIFY, post(handle_ai_simplify))
        .route(ApiUrls::DOCS_AI_TRANSLATE, post(handle_ai_translate))
        .route(ApiUrls::DOCS_AI_CUSTOM, post(handle_ai_custom))
        .route(ApiUrls::DOCS_EXPORT_PDF, get(handle_export_pdf))
        .route(ApiUrls::DOCS_EXPORT_DOCX, get(handle_export_docx))
        .route(ApiUrls::DOCS_EXPORT_MD, get(handle_export_md))
        .route(ApiUrls::DOCS_EXPORT_HTML, get(handle_export_html))
        .route(ApiUrls::DOCS_EXPORT_TXT, get(handle_export_txt))
        .route(ApiUrls::DOCS_WS, get(handle_docs_websocket))
}

// =============================================================================
// AUTHENTICATION HELPERS
// =============================================================================

async fn get_current_user(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<(Uuid, String), String> {
    let session_id = headers
        .get("x-session-id")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers
                .get("cookie")
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies
                        .split(';')
                        .find(|c| c.trim().starts_with("session_id="))
                        .map(|c| c.trim().trim_start_matches("session_id="))
                })
        });

    if let Some(sid) = session_id {
        if let Ok(session_uuid) = Uuid::parse_str(sid) {
            let conn = state.conn.clone();
            let result = tokio::task::spawn_blocking(move || {
                let mut db_conn = conn.get().map_err(|e| e.to_string())?;

                let user_id: Option<Uuid> =
                    diesel::sql_query("SELECT user_id FROM user_sessions WHERE id = $1")
                        .bind::<diesel::sql_types::Uuid, _>(session_uuid)
                        .get_result::<UserIdRow>(&mut db_conn)
                        .optional()
                        .map_err(|e| e.to_string())?
                        .map(|r| r.user_id);

                if let Some(uid) = user_id {
                    let user: Option<UserRow> =
                        diesel::sql_query("SELECT id, email, username FROM users WHERE id = $1")
                            .bind::<diesel::sql_types::Uuid, _>(uid)
                            .get_result(&mut db_conn)
                            .optional()
                            .map_err(|e| e.to_string())?;

                    if let Some(u) = user {
                        return Ok((u.id, u.email));
                    }
                }
                Err("User not found".to_string())
            })
            .await
            .map_err(|e| e.to_string())?;

            return result;
        }
    }

    // Fallback to anonymous user
    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        let anon_email = "anonymous@local";
        let user: Option<UserRow> = diesel::sql_query(
            "SELECT id, email, username FROM users WHERE email = $1",
        )
        .bind::<diesel::sql_types::Text, _>(anon_email)
        .get_result(&mut db_conn)
        .optional()
        .map_err(|e| e.to_string())?;

        if let Some(u) = user {
            Ok((u.id, u.email))
        } else {
            let new_id = Uuid::new_v4();
            let now = Utc::now();
            diesel::sql_query(
                "INSERT INTO users (id, username, email, password_hash, is_active, created_at, updated_at)
                 VALUES ($1, $2, $3, '', true, $4, $4)"
            )
            .bind::<diesel::sql_types::Uuid, _>(new_id)
            .bind::<diesel::sql_types::Text, _>("anonymous")
            .bind::<diesel::sql_types::Text, _>(anon_email)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut db_conn)
            .map_err(|e| e.to_string())?;

            Ok((new_id, anon_email.to_string()))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

// =============================================================================
// STORAGE HELPERS
// =============================================================================

fn get_user_docs_path(user_identifier: &str) -> String {
    let safe_id = user_identifier
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .to_lowercase();
    format!("users/{}/docs", safe_id)
}

async fn save_document_to_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
) -> Result<String, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_docs_path(user_identifier);
    let doc_path = format!("{}/{}.html", base_path, doc_id);
    let meta_path = format!("{}/{}.meta.json", base_path, doc_id);

    // Save document content as HTML
    s3_client
        .put_object()
        .bucket(&state.bucket_name)
        .key(&doc_path)
        .body(ByteStream::from(content.as_bytes().to_vec()))
        .content_type("text/html")
        .send()
        .await
        .map_err(|e| format!("Failed to save document: {}", e))?;

    // Save metadata
    let word_count = content
        .split_whitespace()
        .filter(|w| !w.starts_with('<') && !w.ends_with('>'))
        .count();

    let metadata = serde_json::json!({
        "id": doc_id,
        "title": title,
        "created_at": Utc::now().to_rfc3339(),
        "updated_at": Utc::now().to_rfc3339(),
        "word_count": word_count,
        "version": 1
    });

    s3_client
        .put_object()
        .bucket(&state.bucket_name)
        .key(&meta_path)
        .body(ByteStream::from(metadata.to_string().into_bytes()))
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to save metadata: {}", e))?;

    Ok(doc_path)
}

async fn load_document_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
) -> Result<Option<Document>, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_docs_path(user_identifier);
    let doc_path = format!("{}/{}.html", base_path, doc_id);
    let meta_path = format!("{}/{}.meta.json", base_path, doc_id);

    // Load content
    let content = match s3_client
        .get_object()
        .bucket(&state.bucket_name)
        .key(&doc_path)
        .send()
        .await
    {
        Ok(result) => {
            let bytes = result
                .body
                .collect()
                .await
                .map_err(|e| e.to_string())?
                .into_bytes();
            String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?
        }
        Err(_) => return Ok(None),
    };

    // Load metadata
    let (title, created_at, updated_at) = match s3_client
        .get_object()
        .bucket(&state.bucket_name)
        .key(&meta_path)
        .send()
        .await
    {
        Ok(result) => {
            let bytes = result
                .body
                .collect()
                .await
                .map_err(|e| e.to_string())?
                .into_bytes();
            let meta_str = String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?;
            let meta: serde_json::Value = serde_json::from_str(&meta_str).unwrap_or_default();
            (
                meta["title"].as_str().unwrap_or("Untitled").to_string(),
                meta["created_at"]
                    .as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                meta["updated_at"]
                    .as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
            )
        }
        Err(_) => ("Untitled".to_string(), Utc::now(), Utc::now()),
    };

    Ok(Some(Document {
        id: doc_id.to_string(),
        title,
        content,
        owner_id: user_identifier.to_string(),
        storage_path: doc_path,
        created_at,
        updated_at,
        collaborators: Vec::new(),
        version: 1,
    }))
}

async fn list_documents_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
) -> Result<Vec<DocumentMetadata>, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_docs_path(user_identifier);
    let prefix = format!("{}/", base_path);
    let mut documents = Vec::new();

    if let Ok(result) = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .prefix(&prefix)
        .send()
        .await
    {
        for obj in result.contents() {
            if let Some(key) = obj.key() {
                if key.ends_with(".meta.json") {
                    // Load metadata
                    if let Ok(meta_result) = s3_client
                        .get_object()
                        .bucket(&state.bucket_name)
                        .key(key)
                        .send()
                        .await
                    {
                        if let Ok(bytes) = meta_result.body.collect().await {
                            if let Ok(meta_str) = String::from_utf8(bytes.into_bytes().to_vec()) {
                                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                                    documents.push(DocumentMetadata {
                                        id: meta["id"].as_str().unwrap_or("").to_string(),
                                        title: meta["title"].as_str().unwrap_or("Untitled").to_string(),
                                        owner_id: user_identifier.to_string(),
                                        created_at: meta["created_at"]
                                            .as_str()
                                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                            .map(|d| d.with_timezone(&Utc))
                                            .unwrap_or_else(Utc::now),
                                        updated_at: meta["updated_at"]
                                            .as_str()
                                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                            .map(|d| d.with_timezone(&Utc))
                                            .unwrap_or_else(Utc::now),
                                        word_count: meta["word_count"].as_u64().unwrap_or(0) as usize,
                                        storage_type: "docs".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by updated_at descending
    documents.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(documents)
}

async fn delete_document_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
) -> Result<(), String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_docs_path(user_identifier);
    let doc_path = format!("{}/{}.html", base_path, doc_id);
    let meta_path = format!("{}/{}.meta.json", base_path, doc_id);

    // Delete document
    let _ = s3_client
        .delete_object()
        .bucket(&state.bucket_name)
        .key(&doc_path)
        .send()
        .await;

    // Delete metadata
    let _ = s3_client
        .delete_object()
        .bucket(&state.bucket_name)
        .key(&meta_path)
        .send()
        .await;

    Ok(())
}

// =============================================================================
// LLM HELPERS
// =============================================================================

async fn call_llm(_state: &Arc<AppState>, _system_prompt: &str, _user_text: &str) -> Result<String, String> {
    // TODO: Integrate with LLM provider when available
    Err("LLM not available".to_string())
}

// =============================================================================
// DOCUMENT HANDLERS
// =============================================================================

pub async fn handle_new_document(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Untitled Document".to_string();
    let content = "<p></p>".to_string();

    if let Err(e) = save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content).await {
        error!("Failed to save new document: {}", e);
    }

    let mut html = String::new();
    html.push_str("<div class=\"doc-new-created\" data-id=\"");
    html.push_str(&html_escape(&doc_id));
    html.push_str("\">");
    html.push_str(&format_document_list_item(&doc_id, &title, "just now", true));
    html.push_str("<script>");
    html.push_str("htmx.trigger('#docs-list', 'refresh');");
    html.push_str(&format!(
        "htmx.ajax('GET', '/api/ui/docs/{}', {{target: '#editor-content', swap: 'innerHTML'}});",
        &html_escape(&doc_id)
    ));
    html.push_str("</script>");
    html.push_str("</div>");

    info!("New document created: {} for user {}", doc_id, user_id);
    Html(html)
}

pub async fn handle_list_documents(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let documents = match list_documents_from_drive(&state, &user_identifier).await {
        Ok(docs) => docs,
        Err(e) => {
            error!("Failed to list documents: {}", e);
            Vec::new()
        }
    };

    let mut html = String::new();
    html.push_str("<div class=\"docs-list-items\">");

    if documents.is_empty() {
        html.push_str("<div class=\"docs-empty\">");
        html.push_str("<p>No documents yet</p>");
        html.push_str("<button class=\"btn-new\" hx-post=\"/api/ui/docs/new\" hx-target=\"#docs-list\" hx-swap=\"afterbegin\">Create your first document</button>");
        html.push_str("</div>");
    } else {
        for doc in documents {
            let time_str = format_relative_time(doc.updated_at);
            html.push_str(&format_document_list_item(&doc.id, &doc.title, &time_str, false));
        }
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_search_documents(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let query = params.q.unwrap_or_default().to_lowercase();

    let documents = match list_documents_from_drive(&state, &user_identifier).await {
        Ok(docs) => docs,
        Err(e) => {
            error!("Failed to list documents: {}", e);
            Vec::new()
        }
    };

    let filtered: Vec<_> = if query.is_empty() {
        documents
    } else {
        documents
            .into_iter()
            .filter(|d| d.title.to_lowercase().contains(&query))
            .collect()
    };

    let mut html = String::new();
    html.push_str("<div class=\"docs-list-items\">");

    if filtered.is_empty() {
        html.push_str("<div class=\"docs-empty\">");
        html.push_str("<p>No documents found</p>");
        html.push_str("</div>");
    } else {
        for doc in filtered {
            let time_str = format_relative_time(doc.updated_at);
            html.push_str(&format_document_list_item(&doc.id, &doc.title, &time_str, false));
        }
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_get_document(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            error!("Auth error: {}", e);
            return Json(serde_json::json!({"error": "Authentication required"})).into_response();
        }
    };

    match load_document_from_drive(&state, &user_identifier, &id).await {
        Ok(Some(doc)) => Json(serde_json::json!({
            "id": doc.id,
            "title": doc.title,
            "content": doc.content,
            "created_at": doc.created_at.to_rfc3339(),
            "updated_at": doc.updated_at.to_rfc3339()
        })).into_response(),
        Ok(None) => Json(serde_json::json!({"error": "Document not found"})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_save_document(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SaveRequest>,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            error!("Auth error: {}", e);
            return Json(SaveResponse {
                id: String::new(),
                success: false,
                message: Some("Authentication required".to_string()),
            });
        }
    };

    let doc_id = payload.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let title = payload.title.unwrap_or_else(|| "Untitled Document".to_string());
    let content = payload.content.unwrap_or_default();

    match save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content).await {
        Ok(_) => {
            // Broadcast to collaborators
            let channels = get_collab_channels();
            if let Some(sender) = channels.read().await.get(&doc_id) {
                let msg = CollabMessage {
                    msg_type: "save".to_string(),
                    doc_id: doc_id.clone(),
                    user_id: user_identifier.clone(),
                    user_name: user_identifier.clone(),
                    user_color: "#4285f4".to_string(),
                    position: None,
                    length: None,
                    content: None,
                    format: None,
                    timestamp: Utc::now(),
                };
                let _ = sender.send(msg);
            }

            Json(SaveResponse {
                id: doc_id,
                success: true,
                message: None,
            })
        }
        Err(e) => Json(SaveResponse {
            id: doc_id,
            success: false,
            message: Some(e),
        }),
    }
}

pub async fn handle_autosave(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SaveRequest>,
) -> impl IntoResponse {
    handle_save_document(State(state), headers, Json(payload)).await
}

pub async fn handle_delete_document(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    match delete_document_from_drive(&state, &user_identifier, &id).await {
        Ok(_) => {
            Html("<div class=\"doc-deleted\">Document deleted</div>".to_string())
        }
        Err(e) => Html(format_error(&e)),
    }
}

// =============================================================================
// TEMPLATE HANDLERS
// =============================================================================

pub async fn handle_template_blank(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    handle_new_document(State(state), headers).await
}

pub async fn handle_template_meeting(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Meeting Notes".to_string();
    let now = Utc::now();

    let content = format!(
        r#"<h1>Meeting Notes</h1>
<p><strong>Date:</strong> {}</p>
<p><strong>Attendees:</strong></p>
<ul><li></li></ul>
<h2>Agenda</h2>
<ol><li></li></ol>
<h2>Discussion</h2>
<p></p>
<h2>Action Items</h2>
<ul><li>☐ </li></ul>
<h2>Next Steps</h2>
<p></p>"#,
        now.format("%Y-%m-%d")
    );

    let _ = save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content).await;

    Html(format_document_content(&doc_id, &title, &content))
}

pub async fn handle_template_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Report".to_string();
    let now = Utc::now();

    let content = format!(
        r#"<h1>Report</h1>
<p><strong>Date:</strong> {}</p>
<p><strong>Author:</strong> </p>
<hr>
<h2>Executive Summary</h2>
<p></p>
<h2>Introduction</h2>
<p></p>
<h2>Background</h2>
<p></p>
<h2>Findings</h2>
<h3>Key Finding 1</h3>
<p></p>
<h3>Key Finding 2</h3>
<p></p>
<h2>Analysis</h2>
<p></p>
<h2>Recommendations</h2>
<ol><li></li><li></li><li></li></ol>
<h2>Conclusion</h2>
<p></p>
<h2>Appendix</h2>
<p></p>"#,
        now.format("%Y-%m-%d")
    );

    let _ = save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content).await;

    Html(format_document_content(&doc_id, &title, &content))
}

pub async fn handle_template_letter(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Letter".to_string();
    let now = Utc::now();

    let content = format!(
        r#"<p>[Your Name]<br>
[Your Address]<br>
[City, State ZIP]<br>
[Your Email]</p>
<p>{}</p>
<p>[Recipient Name]<br>
[Recipient Title]<br>
[Company/Organization]<br>
[Address]<br>
[City, State ZIP]</p>
<p>Dear [Recipient Name],</p>
<p>[Opening paragraph - State the purpose of your letter]</p>
<p>[Body paragraph(s) - Provide details, explanations, or supporting information]</p>
<p>[Closing paragraph - Summarize, request action, or express appreciation]</p>
<p>Sincerely,</p>
<p><br><br>[Your Signature]<br>
[Your Typed Name]</p>"#,
        now.format("%B %d, %Y")
    );

    let _ = save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content).await;

    Html(format_document_content(&doc_id, &title, &content))
}

// =============================================================================
// AI HANDLERS
// =============================================================================

pub async fn handle_ai_summarize(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Json(AiResponse {
            result: None,
            content: Some("Please select some text to summarize.".to_string()),
            error: None,
        });
    }

    let system_prompt = "You are a helpful writing assistant. Summarize the following text concisely while preserving the key points. Provide only the summary without any preamble.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(summary) => Json(AiResponse {
            result: Some(summary),
            content: None,
            error: None,
        }),
        Err(e) => {
            error!("LLM summarize error: {}", e);
            let word_count = text.split_whitespace().count();
            Json(AiResponse {
                result: Some(format!("Summary of {} words: {}...", word_count, text.chars().take(100).collect::<String>())),
                content: None,
                error: None,
            })
        }
    }
}

pub async fn handle_ai_expand(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Json(AiResponse {
            result: None,
            content: Some("Please select some text to expand.".to_string()),
            error: None,
        });
    }

    let system_prompt = "You are a helpful writing assistant. Expand on the following text by adding more details, examples, and explanations. Maintain the original tone and style.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(expanded) => Json(AiResponse {
            result: Some(expanded),
            content: None,
            error: None,
        }),
        Err(e) => {
            error!("LLM expand error: {}", e);
            Json(AiResponse {
                result: None,
                content: None,
                error: Some("AI processing failed".to_string()),
            })
        }
    }
}

pub async fn handle_ai_improve(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Json(AiResponse {
            result: None,
            content: Some("Please select some text to improve.".to_string()),
            error: None,
        });
    }

    let system_prompt = "You are a helpful writing assistant. Improve the following text by enhancing clarity, grammar, and style while preserving the original meaning.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(improved) => Json(AiResponse {
            result: Some(improved),
            content: None,
            error: None,
        }),
        Err(e) => {
            error!("LLM improve error: {}", e);
            Json(AiResponse {
                result: None,
                content: None,
                error: Some("AI processing failed".to_string()),
            })
        }
    }
}

pub async fn handle_ai_simplify(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Json(AiResponse {
            result: None,
            content: Some("Please select some text to simplify.".to_string()),
            error: None,
        });
    }

    let system_prompt = "You are a helpful writing assistant. Simplify the following text to make it easier to understand while keeping the essential meaning.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(simplified) => Json(AiResponse {
            result: Some(simplified),
            content: None,
            error: None,
        }),
        Err(e) => {
            error!("LLM simplify error: {}", e);
            Json(AiResponse {
                result: None,
                content: None,
                error: Some("AI processing failed".to_string()),
            })
        }
    }
}

pub async fn handle_ai_translate(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();
    let target_lang = payload.translate_lang.unwrap_or_else(|| "English".to_string());

    if text.is_empty() {
        return Json(AiResponse {
            result: None,
            content: Some("Please select some text to translate.".to_string()),
            error: None,
        });
    }

    let system_prompt = format!("You are a translator. Translate the following text to {}. Provide only the translation without any preamble.", target_lang);

    match call_llm(&state, &system_prompt, &text).await {
        Ok(translated) => Json(AiResponse {
            result: Some(translated),
            content: None,
            error: None,
        }),
        Err(e) => {
            error!("LLM translate error: {}", e);
            Json(AiResponse {
                result: None,
                content: None,
                error: Some("AI processing failed".to_string()),
            })
        }
    }
}

pub async fn handle_ai_custom(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();
    let prompt = payload.prompt.unwrap_or_default();

    if prompt.is_empty() {
        return Json(AiResponse {
            result: None,
            content: Some("Please provide a prompt.".to_string()),
            error: None,
        });
    }

    let system_prompt = format!("You are a helpful writing assistant. {}", prompt);

    match call_llm(&state, &system_prompt, &text).await {
        Ok(result) => Json(AiResponse {
            result: Some(result),
            content: None,
            error: None,
        }),
        Err(e) => {
            error!("LLM custom error: {}", e);
            Json(AiResponse {
                result: None,
                content: None,
                error: Some("AI processing failed".to_string()),
            })
        }
    }
}

// =============================================================================
// EXPORT HANDLERS
// =============================================================================

pub async fn handle_export_pdf(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<ExportQuery>,
) -> impl IntoResponse {
    // PDF export would require a library like printpdf or wkhtmltopdf
    Html("<p>PDF export coming soon</p>".to_string())
}

pub async fn handle_export_docx(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<ExportQuery>,
) -> impl IntoResponse {
    // DOCX export would require a library like docx-rs
    Html("<p>DOCX export coming soon</p>".to_string())
}

pub async fn handle_export_md(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(_) => return Html("Authentication required".to_string()),
    };

    let doc_id = match params.id {
        Some(id) => id,
        None => return Html("Document ID required".to_string()),
    };

    match load_document_from_drive(&state, &user_identifier, &doc_id).await {
        Ok(Some(doc)) => {
            let md = html_to_markdown(&doc.content);
            Html(md)
        }
        _ => Html("Document not found".to_string()),
    }
}

pub async fn handle_export_html(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(_) => return Html("Authentication required".to_string()),
    };

    let doc_id = match params.id {
        Some(id) => id,
        None => return Html("Document ID required".to_string()),
    };

    match load_document_from_drive(&state, &user_identifier, &doc_id).await {
        Ok(Some(doc)) => {
            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>{}</title>
<style>
body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
</style>
</head>
<body>
{}
</body>
</html>"#,
                html_escape(&doc.title),
                doc.content
            );
            Html(html)
        }
        _ => Html("Document not found".to_string()),
    }
}

pub async fn handle_export_txt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(_) => return Html("Authentication required".to_string()),
    };

    let doc_id = match params.id {
        Some(id) => id,
        None => return Html("Document ID required".to_string()),
    };

    match load_document_from_drive(&state, &user_identifier, &doc_id).await {
        Ok(Some(doc)) => {
            let txt = strip_html(&doc.content);
            Html(txt)
        }
        _ => Html("Document not found".to_string()),
    }
}

// =============================================================================
// WEBSOCKET COLLABORATION
// =============================================================================

pub async fn handle_docs_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(doc_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_docs_connection(socket, state, doc_id))
}

async fn handle_docs_connection(socket: WebSocket, _state: Arc<AppState>, doc_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let channels = get_collab_channels();

    // Get or create channel for this document
    let rx = {
        let mut channels_write = channels.write().await;
        let tx = channels_write
            .entry(doc_id.clone())
            .or_insert_with(|| broadcast::channel(100).0);
        tx.subscribe()
    };

    let user_id = Uuid::new_v4().to_string();
    let user_color = get_random_color();

    // Send join message
    {
        let channels_read = channels.read().await;
        if let Some(tx) = channels_read.get(&doc_id) {
            let msg = CollabMessage {
                msg_type: "join".to_string(),
                doc_id: doc_id.clone(),
                user_id: user_id.clone(),
                user_name: format!("User {}", &user_id[..8]),
                user_color: user_color.clone(),
                position: None,
                length: None,
                content: None,
                format: None,
                timestamp: Utc::now(),
            };
            let _ = tx.send(msg);
        }
    }

    // Spawn task to forward broadcast messages to this client
    let mut rx = rx;
    let user_id_clone = user_id.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Don't send messages back to the sender
            if msg.user_id != user_id_clone {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Handle incoming messages
    let channels_clone = channels.clone();
    let doc_id_clone = doc_id.clone();
    let user_id_clone2 = user_id.clone();
    let user_color_clone = user_color.clone();

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(mut collab_msg) = serde_json::from_str::<CollabMessage>(&text) {
                collab_msg.user_id = user_id_clone2.clone();
                collab_msg.user_color = user_color_clone.clone();
                collab_msg.timestamp = Utc::now();

                let channels_read = channels_clone.read().await;
                if let Some(tx) = channels_read.get(&doc_id_clone) {
                    let _ = tx.send(collab_msg);
                }
            }
        }
    }

    // Send leave message
    {
        let channels_read = channels.read().await;
        if let Some(tx) = channels_read.get(&doc_id) {
            let msg = CollabMessage {
                msg_type: "leave".to_string(),
                doc_id: doc_id.clone(),
                user_id: user_id.clone(),
                user_name: format!("User {}", &user_id[..8]),
                user_color,
                position: None,
                length: None,
                content: None,
                format: None,
                timestamp: Utc::now(),
            };
            let _ = tx.send(msg);
        }
    }

    send_task.abort();
    info!("WebSocket connection closed for doc {}", doc_id);
}

fn get_random_color() -> String {
    let colors = [
        "#4285f4", "#ea4335", "#fbbc05", "#34a853",
        "#ff6d01", "#46bdc6", "#7b1fa2", "#c2185b",
    ];
    let mut rng = rand::rng();
    let idx = rng.random_range(0..colors.len());
    colors[idx].to_string()
}

// =============================================================================
// FORMATTING HELPERS
// =============================================================================

fn format_document_list_item(id: &str, title: &str, time: &str, is_new: bool) -> String {
    let new_class = if is_new { " new-item" } else { "" };
    let mut html = String::new();
    html.push_str("<div class=\"doc-item");
    html.push_str(new_class);
    html.push_str("\" data-id=\"");
    html.push_str(&html_escape(id));
    html.push_str("\" hx-get=\"/api/ui/docs/");
    html.push_str(&html_escape(id));
    html.push_str("\" hx-target=\"#editor-content\" hx-swap=\"innerHTML\">");
    html.push_str("<div class=\"doc-item-icon\">📄</div>");
    html.push_str("<div class=\"doc-item-info\">");
    html.push_str("<span class=\"doc-item-title\">");
    html.push_str(&html_escape(title));
    html.push_str("</span>");
    html.push_str("<span class=\"doc-item-time\">");
    html.push_str(&html_escape(time));
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("</div>");
    html
}

fn format_document_content(id: &str, title: &str, content: &str) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"doc-editor\" data-id=\"");
    html.push_str(&html_escape(id));
    html.push_str("\" data-title=\"");
    html.push_str(&html_escape(title));
    html.push_str("\">");
    html.push_str("<div class=\"doc-title\" contenteditable=\"true\" data-placeholder=\"Untitled Document\">");
    html.push_str(&html_escape(title));
    html.push_str("</div>");
    html.push_str("<div class=\"doc-body\" contenteditable=\"true\">");
    html.push_str(content);
    html.push_str("</div>");
    html.push_str("</div>");
    html.push_str("<script>");
    html.push_str("if (typeof gbDocs !== 'undefined') {");
    html.push_str("gbDocs.state.docId = '");
    html.push_str(&html_escape(id));
    html.push_str("';");
    html.push_str("gbDocs.state.docTitle = '");
    html.push_str(&html_escape(title));
    html.push_str("';");
    html.push_str("}");
    html.push_str("</script>");
    html
}

fn format_error(message: &str) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"error-message\">");
    html.push_str("<span class=\"error-icon\">⚠️</span>");
    html.push_str("<span>");
    html.push_str(&html_escape(message));
    html.push_str("</span>");
    html.push_str("</div>");
    html
}

fn format_relative_time(time: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(time);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else {
        time.format("%b %d").to_string()
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                result.push(' ');
            }
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    // Clean up whitespace
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn html_to_markdown(html: &str) -> String {
    let mut md = html.to_string();

    // Basic HTML to Markdown conversion
    md = md.replace("<h1>", "# ").replace("</h1>", "\n\n");
    md = md.replace("<h2>", "## ").replace("</h2>", "\n\n");
    md = md.replace("<h3>", "### ").replace("</h3>", "\n\n");
    md = md.replace("<p>", "").replace("</p>", "\n\n");
    md = md.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
    md = md.replace("<strong>", "**").replace("</strong>", "**");
    md = md.replace("<b>", "**").replace("</b>", "**");
    md = md.replace("<em>", "*").replace("</em>", "*");
    md = md.replace("<i>", "*").replace("</i>", "*");
    md = md.replace("<ul>", "").replace("</ul>", "\n");
    md = md.replace("<ol>", "").replace("</ol>", "\n");
    md = md.replace("<li>", "- ").replace("</li>", "\n");
    md = md.replace("<hr>", "\n---\n").replace("<hr/>", "\n---\n");

    // Strip remaining HTML tags
    strip_html(&md)
}
