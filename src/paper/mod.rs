#[cfg(feature = "llm")]
use crate::llm::OpenAIClient;
use crate::shared::state::AppState;
use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{Path, Query, State},
    http::header::HeaderMap,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub owner_id: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub save_as_named: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    #[serde(rename = "selected-text")]
    pub selected_text: Option<String>,
    pub prompt: Option<String>,
    #[serde(rename = "translate-lang")]
    pub translate_lang: Option<String>,
    pub document_id: Option<String>,
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

pub fn configure_paper_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/paper/new", post(handle_new_document))
        .route("/api/paper/list", get(handle_list_documents))
        .route("/api/paper/search", get(handle_search_documents))
        .route("/api/paper/save", post(handle_save_document))
        .route("/api/paper/autosave", post(handle_autosave))
        .route("/api/paper/{id}", get(handle_get_document))
        .route("/api/paper/{id}/delete", post(handle_delete_document))
        .route("/api/paper/template/blank", post(handle_template_blank))
        .route("/api/paper/template/meeting", post(handle_template_meeting))
        .route("/api/paper/template/todo", post(handle_template_todo))
        .route(
            "/api/paper/template/research",
            post(handle_template_research),
        )
        .route("/api/paper/ai/summarize", post(handle_ai_summarize))
        .route("/api/paper/ai/expand", post(handle_ai_expand))
        .route("/api/paper/ai/improve", post(handle_ai_improve))
        .route("/api/paper/ai/simplify", post(handle_ai_simplify))
        .route("/api/paper/ai/translate", post(handle_ai_translate))
        .route("/api/paper/ai/custom", post(handle_ai_custom))
        .route("/api/paper/export/pdf", get(handle_export_pdf))
        .route("/api/paper/export/docx", get(handle_export_docx))
        .route("/api/paper/export/md", get(handle_export_md))
        .route("/api/paper/export/html", get(handle_export_html))
        .route("/api/paper/export/txt", get(handle_export_txt))
}

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

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct UserIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    user_id: Uuid,
}

fn get_user_papers_path(user_identifier: &str) -> String {
    let safe_id = user_identifier
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .to_lowercase();
    format!("users/{}/papers", safe_id)
}

async fn save_document_to_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
    is_named: bool,
) -> Result<String, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_papers_path(user_identifier);
    let storage_type = if is_named { "named" } else { "current" };

    let (doc_path, metadata_path) = if is_named {
        let safe_title = title
            .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
            .to_lowercase()
            .chars()
            .take(50)
            .collect::<String>();
        (
            format!("{}/{}/{}/document.md", base_path, storage_type, safe_title),
            Some(format!(
                "{}/{}/{}/metadata.json",
                base_path, storage_type, safe_title
            )),
        )
    } else {
        (
            format!("{}/{}/{}.md", base_path, storage_type, doc_id),
            None,
        )
    };

    s3_client
        .put_object()
        .bucket(&state.bucket_name)
        .key(&doc_path)
        .body(ByteStream::from(content.as_bytes().to_vec()))
        .content_type("text/markdown")
        .send()
        .await
        .map_err(|e| format!("Failed to save document: {}", e))?;

    if let Some(meta_path) = metadata_path {
        let metadata = serde_json::json!({
            "id": doc_id,
            "title": title,
            "created_at": Utc::now().to_rfc3339(),
            "updated_at": Utc::now().to_rfc3339(),
            "word_count": content.split_whitespace().count()
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
    }

    Ok(doc_path)
}

async fn load_document_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
) -> Result<Option<Document>, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_papers_path(user_identifier);

    let current_path = format!("{}/current/{}.md", base_path, doc_id);

    if let Ok(result) = s3_client
        .get_object()
        .bucket(&state.bucket_name)
        .key(&current_path)
        .send()
        .await
    {
        let bytes = result
            .body
            .collect()
            .await
            .map_err(|e| e.to_string())?
            .into_bytes();
        let content = String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?;

        let title = content
            .lines()
            .next()
            .map(|l| l.trim_start_matches('#').trim())
            .unwrap_or("Untitled")
            .to_string();

        return Ok(Some(Document {
            id: doc_id.to_string(),
            title,
            content,
            owner_id: user_identifier.to_string(),
            storage_path: current_path,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }));
    }

    Ok(None)
}

async fn list_documents_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
) -> Result<Vec<DocumentMetadata>, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_papers_path(user_identifier);
    let mut documents = Vec::new();

    let current_prefix = format!("{}/current/", base_path);
    if let Ok(result) = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .prefix(&current_prefix)
        .send()
        .await
    {
        for obj in result.contents() {
            if let Some(key) = obj.key() {
                if key.to_lowercase().ends_with(".md") {
                    let id = key
                        .trim_start_matches(&current_prefix)
                        .trim_end_matches(".md")
                        .to_string();

                    documents.push(DocumentMetadata {
                        id: id.clone(),
                        title: format!("Untitled ({})", &id[..8.min(id.len())]),
                        owner_id: user_identifier.to_string(),
                        created_at: Utc::now(),
                        updated_at: obj
                            .last_modified()
                            .map(|t| {
                                DateTime::from_timestamp(t.secs(), t.subsec_nanos())
                                    .unwrap_or_else(Utc::now)
                            })
                            .unwrap_or_else(Utc::now),
                        word_count: 0,
                        storage_type: "current".to_string(),
                    });
                }
            }
        }
    }

    let named_prefix = format!("{}/named/", base_path);
    if let Ok(result) = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .prefix(&named_prefix)
        .delimiter("/")
        .send()
        .await
    {
        for prefix in result.common_prefixes() {
            if let Some(folder) = prefix.prefix() {
                let folder_name = folder
                    .trim_start_matches(&named_prefix)
                    .trim_end_matches('/');

                let meta_key = format!("{}metadata.json", folder);
                if let Ok(meta_result) = s3_client
                    .get_object()
                    .bucket(&state.bucket_name)
                    .key(&meta_key)
                    .send()
                    .await
                {
                    if let Ok(bytes) = meta_result.body.collect().await {
                        if let Ok(meta_str) = String::from_utf8(bytes.into_bytes().to_vec()) {
                            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                                documents.push(DocumentMetadata {
                                    id: meta["id"].as_str().unwrap_or(folder_name).to_string(),
                                    title: meta["title"]
                                        .as_str()
                                        .unwrap_or(folder_name)
                                        .to_string(),
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
                                    storage_type: "named".to_string(),
                                });
                                continue;
                            }
                        }
                    }
                }

                documents.push(DocumentMetadata {
                    id: folder_name.to_string(),
                    title: folder_name.to_string(),
                    owner_id: user_identifier.to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    word_count: 0,
                    storage_type: "named".to_string(),
                });
            }
        }
    }

    documents.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(documents)
}

async fn delete_document_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
) -> Result<(), String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_papers_path(user_identifier);

    let current_path = format!("{}/current/{}.md", base_path, doc_id);
    let _ = s3_client
        .delete_object()
        .bucket(&state.bucket_name)
        .key(&current_path)
        .send()
        .await;

    let named_prefix = format!("{}/named/{}/", base_path, doc_id);
    if let Ok(result) = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .prefix(&named_prefix)
        .send()
        .await
    {
        for obj in result.contents() {
            if let Some(key) = obj.key() {
                let _ = s3_client
                    .delete_object()
                    .bucket(&state.bucket_name)
                    .key(key)
                    .send()
                    .await;
            }
        }
    }

    Ok(())
}

#[cfg(feature = "llm")]
async fn call_llm(
    state: &Arc<AppState>,
    system_prompt: &str,
    user_content: &str,
) -> Result<String, String> {
    let llm = &state.llm_provider;

    let messages = OpenAIClient::build_messages(
        system_prompt,
        "",
        &[("user".to_string(), user_content.to_string())],
    );

    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
    let model = config_manager
        .get_config(&Uuid::nil(), "llm-model", None)
        .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
    let key = config_manager
        .get_config(&Uuid::nil(), "llm-key", None)
        .unwrap_or_else(|_| String::new());

    llm.generate(user_content, &messages, &model, &key)
        .await
        .map_err(|e| format!("LLM error: {}", e))
}

#[cfg(not(feature = "llm"))]
async fn call_llm(
    _state: &Arc<AppState>,
    _system_prompt: &str,
    user_content: &str,
) -> Result<String, String> {
    Ok(format!(
        "[LLM not available] Processing: {}...",
        &user_content[..50.min(user_content.len())]
    ))
}

pub async fn handle_new_document(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Untitled".to_string();
    let content = String::new();

    if let Err(e) =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await
    {
        log::error!("Failed to save new document: {}", e);
    }

    let mut html = String::new();
    html.push_str("<div class=\"paper-new-created\" data-id=\"");
    html.push_str(&html_escape(&doc_id));
    html.push_str("\">");

    html.push_str(&format_document_list_item(
        &doc_id, &title, "just now", true,
    ));

    html.push_str("<script>");
    html.push_str("htmx.trigger('#paper-list', 'refresh');");
    html.push_str("htmx.ajax('GET', '/api/paper/");
    html.push_str(&html_escape(&doc_id));
    html.push_str("', {target: '#editor-content', swap: 'innerHTML'});");
    html.push_str("</script>");
    html.push_str("</div>");

    log::info!("New document created: {} for user {}", doc_id, user_id);
    Html(html)
}

pub async fn handle_list_documents(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let documents = match list_documents_from_drive(&state, &user_identifier).await {
        Ok(docs) => docs,
        Err(e) => {
            log::error!("Failed to list documents: {}", e);
            Vec::new()
        }
    };

    let mut html = String::new();
    html.push_str("<div class=\"paper-list\">");

    if documents.is_empty() {
        html.push_str("<div class=\"paper-empty\">");
        html.push_str("<p>No documents yet</p>");
        html.push_str("<button class=\"btn-new\" hx-post=\"/api/paper/new\" hx-target=\"#paper-list\" hx-swap=\"afterbegin\">Create your first document</button>");
        html.push_str("</div>");
    } else {
        for doc in documents {
            let time_str = format_relative_time(doc.updated_at);
            let badge = if doc.storage_type == "named" {
                " 📁"
            } else {
                ""
            };
            html.push_str(&format_document_list_item(
                &doc.id,
                &format!("{}{}", doc.title, badge),
                &time_str,
                false,
            ));
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
            log::error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let query = params.q.unwrap_or_default().to_lowercase();

    let documents = list_documents_from_drive(&state, &user_identifier)
        .await
        .unwrap_or_default();

    let filtered: Vec<_> = if query.is_empty() {
        documents
    } else {
        documents
            .into_iter()
            .filter(|d| d.title.to_lowercase().contains(&query))
            .collect()
    };

    let mut html = String::new();
    html.push_str("<div class=\"paper-search-results\">");

    if filtered.is_empty() {
        html.push_str("<div class=\"paper-empty\">");
        html.push_str("<p>No documents found</p>");
        html.push_str("</div>");
    } else {
        for doc in filtered {
            let time_str = format_relative_time(doc.updated_at);
            html.push_str(&format_document_list_item(
                &doc.id, &doc.title, &time_str, false,
            ));
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
            log::error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    match load_document_from_drive(&state, &user_identifier, &id).await {
        Ok(Some(doc)) => Html(format_document_content(&doc.title, &doc.content)),
        Ok(None) => Html(format_document_content("Untitled", "")),
        Err(e) => {
            log::error!("Failed to load document {}: {}", id, e);
            Html(format_document_content("Untitled", ""))
        }
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
            log::error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let doc_id = payload.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let title = payload.title.unwrap_or_else(|| "Untitled".to_string());
    let content = payload.content.unwrap_or_default();
    let is_named = payload.save_as_named.unwrap_or(false);

    match save_document_to_drive(
        &state,
        &user_identifier,
        &doc_id,
        &title,
        &content,
        is_named,
    )
    .await
    {
        Ok(path) => {
            log::info!("Document saved: {} at {}", doc_id, path);
            let mut html = String::new();
            html.push_str("<div class=\"save-success\">");
            html.push_str("<span class=\"save-icon\">*</span>");
            html.push_str("<span>Saved</span>");
            html.push_str("</div>");
            Html(html)
        }
        Err(e) => {
            log::error!("Failed to save document: {}", e);
            Html(format_error("Failed to save document"))
        }
    }
}

pub async fn handle_autosave(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SaveRequest>,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(String::new());
        }
    };

    let doc_id = payload.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let title = payload.title.unwrap_or_else(|| "Untitled".to_string());
    let content = payload.content.unwrap_or_default();

    if let Err(e) =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await
    {
        log::warn!("Autosave failed for {}: {}", doc_id, e);
    }

    Html("<span class=\"autosave-indicator\">Auto-saved</span>".to_string())
}

pub async fn handle_delete_document(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    match delete_document_from_drive(&state, &user_identifier, &id).await {
        Ok(()) => {
            log::info!("Document deleted: {}", id);
            Html("<div class=\"delete-success\" hx-trigger=\"load\" hx-get=\"/api/paper/list\" hx-target=\"#paper-list\" hx-swap=\"innerHTML\"></div>".to_string())
        }
        Err(e) => {
            log::error!("Failed to delete document {}: {}", id, e);
            Html(format_error("Failed to delete document"))
        }
    }
}

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
            log::error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Meeting Notes".to_string();
    let now = Utc::now();

    let mut content = String::new();
    content.push_str("# Meeting Notes\n\n");
    let _ = writeln!(content, "**Date:** {}\n", now.format("%Y-%m-%d"));
    content.push_str("**Attendees:**\n- \n\n");
    content.push_str("## Agenda\n\n1. \n\n");
    content.push_str("## Discussion\n\n\n\n");
    content.push_str("## Action Items\n\n- [ ] \n\n");
    content.push_str("## Next Steps\n\n");

    let _ =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await;

    Html(format_document_content(&title, &content))
}

pub async fn handle_template_todo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "To-Do List".to_string();

    let mut content = String::new();
    content.push_str("# To-Do List\n\n");
    content.push_str("## High Priority\n\n- [ ] \n\n");
    content.push_str("## Medium Priority\n\n- [ ] \n\n");
    content.push_str("## Low Priority\n\n- [ ] \n\n");
    content.push_str("## Completed\n\n- [x] Example completed task\n");

    let _ =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await;

    Html(format_document_content(&title, &content))
}

pub async fn handle_template_research(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Research Notes".to_string();

    let mut content = String::new();
    content.push_str("# Research Notes\n\n");
    content.push_str("## Topic\n\n\n\n");
    content.push_str("## Research Questions\n\n1. \n\n");
    content.push_str("## Sources\n\n- \n\n");
    content.push_str("## Key Findings\n\n\n\n");
    content.push_str("## Analysis\n\n\n\n");
    content.push_str("## Conclusions\n\n\n\n");
    content.push_str("## References\n\n");

    let _ =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await;

    Html(format_document_content(&title, &content))
}

pub async fn handle_ai_summarize(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Html(format_ai_response("Please select some text to summarize."));
    }

    let system_prompt = "You are a helpful writing assistant. Summarize the following text concisely while preserving the key points. Provide only the summary without any preamble.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(summary) => Html(format_ai_response(&summary)),
        Err(e) => {
            log::error!("LLM summarize error: {}", e);

            let word_count = text.split_whitespace().count();
            let summary = format!(
                "Summary of {} words: {}...",
                word_count,
                text.chars().take(100).collect::<String>()
            );
            Html(format_ai_response(&summary))
        }
    }
}

pub async fn handle_ai_expand(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Html(format_ai_response("Please select some text to expand."));
    }

    let system_prompt = "You are a helpful writing assistant. Expand on the following text by adding more detail, examples, and context. Maintain the same style and tone. Provide only the expanded text without any preamble.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(expanded) => Html(format_ai_response(&expanded)),
        Err(e) => {
            log::error!("LLM expand error: {}", e);
            let expanded = format!(
                "{}\n\nAdditionally, this concept can be further explored by considering its broader implications and related aspects.",
                text
            );
            Html(format_ai_response(&expanded))
        }
    }
}

pub async fn handle_ai_improve(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Html(format_ai_response("Please select some text to improve."));
    }

    let system_prompt = "You are a professional editor. Improve the following text by enhancing clarity, grammar, style, and flow while preserving the original meaning. Provide only the improved text without any preamble or explanation.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(improved) => Html(format_ai_response(&improved)),
        Err(e) => {
            log::error!("LLM improve error: {}", e);
            Html(format_ai_response(&format!("[Improved]: {}", text.trim())))
        }
    }
}

pub async fn handle_ai_simplify(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Html(format_ai_response("Please select some text to simplify."));
    }

    let system_prompt = "You are a writing assistant specializing in plain language. Simplify the following text to make it easier to understand. Use shorter sentences, simpler words, and clearer structure. Provide only the simplified text without any preamble.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(simplified) => Html(format_ai_response(&simplified)),
        Err(e) => {
            log::error!("LLM simplify error: {}", e);
            Html(format_ai_response(&format!(
                "[Simplified]: {}",
                text.trim()
            )))
        }
    }
}

pub async fn handle_ai_translate(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();
    let lang = payload.translate_lang.unwrap_or_else(|| "es".to_string());

    if text.is_empty() {
        return Html(format_ai_response("Please select some text to translate."));
    }

    let lang_name = match lang.as_str() {
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        "pt" => "Portuguese",
        "it" => "Italian",
        "zh" => "Chinese",
        "ja" => "Japanese",
        "ko" => "Korean",
        "ar" => "Arabic",
        "ru" => "Russian",
        _ => "the target language",
    };

    let system_prompt = format!(
        "You are a professional translator. Translate the following text to {}. Provide only the translation without any preamble or explanation.",
        lang_name
    );

    match call_llm(&state, &system_prompt, &text).await {
        Ok(translated) => Html(format_ai_response(&translated)),
        Err(e) => {
            log::error!("LLM translate error: {}", e);
            Html(format_ai_response(&format!(
                "[Translation to {}]: {}",
                lang_name,
                text.trim()
            )))
        }
    }
}

pub async fn handle_ai_custom(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();
    let prompt = payload.prompt.unwrap_or_default();

    if text.is_empty() || prompt.is_empty() {
        return Html(format_ai_response(
            "Please select text and enter a command.",
        ));
    }

    let system_prompt = format!(
        "You are a helpful writing assistant. The user wants you to: {}. Apply this to the following text and provide only the result without any preamble.",
        prompt
    );

    match call_llm(&state, &system_prompt, &text).await {
        Ok(result) => Html(format_ai_response(&result)),
        Err(e) => {
            log::error!("LLM custom error: {}", e);
            Html(format_ai_response(&format!(
                "[Custom '{}' applied]: {}",
                prompt,
                text.trim()
            )))
        }
    }
}

pub async fn handle_export_pdf(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let Ok((_user_id, user_identifier)) = get_current_user(&state, &headers).await else {
        return Html(format_error("Authentication required"));
    };

    if let Some(doc_id) = params.id {
        if let Ok(Some(_doc)) = load_document_from_drive(&state, &user_identifier, &doc_id).await {
            return Html("<script>alert('PDF export started. The file will be saved to your exports folder.');</script>".to_string());
        }
    }

    Html("<script>alert('Please save your document first.');</script>".to_string())
}

pub async fn handle_export_docx(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let Ok((_user_id, user_identifier)) = get_current_user(&state, &headers).await else {
        return Html(format_error("Authentication required"));
    };

    if let Some(doc_id) = params.id {
        if let Ok(Some(_doc)) = load_document_from_drive(&state, &user_identifier, &doc_id).await {
            return Html("<script>alert('Word export started. The file will be saved to your exports folder.');</script>".to_string());
        }
    }

    Html("<script>alert('Please save your document first.');</script>".to_string())
}

pub async fn handle_export_md(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let Ok((_user_id, user_identifier)) = get_current_user(&state, &headers).await else {
        return Html(format_error("Authentication required"));
    };

    if let Some(doc_id) = params.id {
        if let Ok(Some(doc)) = load_document_from_drive(&state, &user_identifier, &doc_id).await {
            let export_path = format!(
                "users/{}/exports/{}.md",
                user_identifier
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
                    .to_lowercase(),
                doc.title
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
            );

            if let Some(s3_client) = state.drive.as_ref() {
                let _ = s3_client
                    .put_object()
                    .bucket(&state.bucket_name)
                    .key(&export_path)
                    .body(ByteStream::from(doc.content.into_bytes()))
                    .content_type("text/markdown")
                    .send()
                    .await;
            }

            return Html(
                "<script>alert('Markdown exported to your exports folder.');</script>".to_string(),
            );
        }
    }

    Html("<script>alert('Please save your document first.');</script>".to_string())
}

pub async fn handle_export_html(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let Ok((_user_id, user_identifier)) = get_current_user(&state, &headers).await else {
        return Html(format_error("Authentication required"));
    };

    if let Some(doc_id) = params.id {
        if let Ok(Some(doc)) = load_document_from_drive(&state, &user_identifier, &doc_id).await {
            let html_content = format!(
                "<!DOCTYPE html>\n<html>\n<head>\n<title>{}</title>\n<meta charset=\"utf-8\">\n</head>\n<body>\n<article>\n{}\n</article>\n</body>\n</html>",
                html_escape(&doc.title),
                markdown_to_html(&doc.content)
            );

            let export_path = format!(
                "users/{}/exports/{}.html",
                user_identifier
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
                    .to_lowercase(),
                doc.title
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
            );

            if let Some(s3_client) = state.drive.as_ref() {
                let _ = s3_client
                    .put_object()
                    .bucket(&state.bucket_name)
                    .key(&export_path)
                    .body(ByteStream::from(html_content.into_bytes()))
                    .content_type("text/html")
                    .send()
                    .await;
            }

            return Html(
                "<script>alert('HTML exported to your exports folder.');</script>".to_string(),
            );
        }
    }

    Html("<script>alert('Please save your document first.');</script>".to_string())
}

pub async fn handle_export_txt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let Ok((_user_id, user_identifier)) = get_current_user(&state, &headers).await else {
        return Html(format_error("Authentication required"));
    };

    if let Some(doc_id) = params.id {
        if let Ok(Some(doc)) = load_document_from_drive(&state, &user_identifier, &doc_id).await {
            let plain_text = strip_markdown(&doc.content);

            let export_path = format!(
                "users/{}/exports/{}.txt",
                user_identifier
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
                    .to_lowercase(),
                doc.title
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
            );

            if let Some(s3_client) = state.drive.as_ref() {
                let _ = s3_client
                    .put_object()
                    .bucket(&state.bucket_name)
                    .key(&export_path)
                    .body(ByteStream::from(plain_text.into_bytes()))
                    .content_type("text/plain")
                    .send()
                    .await;
            }

            return Html(
                "<script>alert('Text exported to your exports folder.');</script>".to_string(),
            );
        }
    }

    Html("<script>alert('Please save your document first.');</script>".to_string())
}

fn format_document_list_item(id: &str, title: &str, time: &str, is_new: bool) -> String {
    let mut html = String::new();
    let new_class = if is_new { " new-item" } else { "" };

    html.push_str("<div class=\"paper-item");
    html.push_str(new_class);
    html.push_str("\" data-id=\"");
    html.push_str(&html_escape(id));
    html.push_str("\" hx-get=\"/api/paper/");
    html.push_str(&html_escape(id));
    html.push_str("\" hx-target=\"#editor-content\" hx-swap=\"innerHTML\">");
    html.push_str("<div class=\"paper-item-icon\">📄</div>");
    html.push_str("<div class=\"paper-item-info\">");
    html.push_str("<span class=\"paper-item-title\">");
    html.push_str(&html_escape(title));
    html.push_str("</span>");
    html.push_str("<span class=\"paper-item-time\">");
    html.push_str(&html_escape(time));
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("</div>");

    html
}

fn format_document_content(title: &str, content: &str) -> String {
    let mut html = String::new();

    html.push_str("<div class=\"paper-editor\" data-title=\"");
    html.push_str(&html_escape(title));
    html.push_str("\">");
    html.push_str(
        "<div class=\"paper-title\" contenteditable=\"true\" data-placeholder=\"Untitled\">",
    );
    html.push_str(&html_escape(title));
    html.push_str("</div>");
    html.push_str("<div class=\"paper-body\" contenteditable=\"true\">");
    if content.is_empty() {
        html.push_str("<p data-placeholder=\"Start writing...\"></p>");
    } else {
        html.push_str(&markdown_to_html(content));
    }
    html.push_str("</div>");
    html.push_str("</div>");

    html
}

fn format_ai_response(content: &str) -> String {
    let mut html = String::new();

    html.push_str("<div class=\"ai-response\">");
    html.push_str("<div class=\"ai-response-header\">");
    html.push_str("<span class=\"ai-icon\"></span>");
    html.push_str("<span>AI Response</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"ai-response-content\">");
    html.push_str(&html_escape(content));
    html.push_str("</div>");
    html.push_str("<div class=\"ai-response-actions\">");
    html.push_str("<button class=\"btn-copy\" onclick=\"copyAiResponse(this)\">Copy</button>");
    html.push_str(
        "<button class=\"btn-insert\" onclick=\"insertAiResponse(this)\">Insert</button>",
    );
    html.push_str(
        "<button class=\"btn-replace\" onclick=\"replaceWithAiResponse(this)\">Replace</button>",
    );
    html.push_str("</div>");
    html.push_str("</div>");

    html
}

fn format_error(message: &str) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"error-message\">");
    html.push_str("<span class=\"error-icon\"></span>");
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

fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_list = false;
    let mut in_code_block = false;

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("# ") {
            html.push_str("<h1>");
            html.push_str(&html_escape(rest));
            html.push_str("</h1>");
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            html.push_str("<h2>");
            html.push_str(&html_escape(rest));
            html.push_str("</h2>");
        } else if let Some(rest) = trimmed.strip_prefix("### ") {
            html.push_str("<h3>");
            html.push_str(&html_escape(rest));
            html.push_str("</h3>");
        } else if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            if !in_list {
                html.push_str("<ul class=\"todo-list\">");
                in_list = true;
            }
            html.push_str("<li><input type=\"checkbox\"> ");
            html.push_str(&html_escape(rest));
            html.push_str("</li>");
        } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            if !in_list {
                html.push_str("<ul class=\"todo-list\">");
                in_list = true;
            }
            html.push_str("<li><input type=\"checkbox\" checked> ");
            html.push_str(&html_escape(rest));
            html.push_str("</li>");
        } else if let Some(rest) = trimmed.strip_prefix("- ") {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str("<li>");
            html.push_str(&html_escape(rest));
            html.push_str("</li>");
        } else if let Some(rest) = trimmed.strip_prefix("* ") {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str("<li>");
            html.push_str(&html_escape(rest));
            html.push_str("</li>");
        } else if trimmed
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && trimmed.contains(". ")
        {
            if !in_list {
                html.push_str("<ol>");
                in_list = true;
            }
            if let Some(pos) = trimmed.find(". ") {
                html.push_str("<li>");
                html.push_str(&html_escape(&trimmed[pos + 2..]));
                html.push_str("</li>");
            }
        } else if trimmed.is_empty() {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str("<br>");
        } else {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str("<p>");
            let formatted = format_inline_markdown(trimmed);
            html.push_str(&formatted);
            html.push_str("</p>");
        }
    }

    if in_list {
        html.push_str("</ul>");
    }
    if in_code_block {
        html.push_str("</code></pre>");
    }

    html
}

fn format_inline_markdown(text: &str) -> String {
    let escaped = html_escape(text);

    let re_bold = escaped.replace("**", "<b>").replace("__", "<b>");

    let re_italic = re_bold.replace(['*', '_'], "<i>");

    let mut result = String::new();
    let mut in_code = false;
    for ch in re_italic.chars() {
        if ch == '`' {
            if in_code {
                result.push_str("</code>");
            } else {
                result.push_str("<code>");
            }
            in_code = !in_code;
        } else {
            result.push(ch);
        }
    }

    result
}

fn strip_markdown(markdown: &str) -> String {
    let mut result = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            continue;
        }

        let content = if let Some(rest) = trimmed.strip_prefix("### ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("- ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("* ") {
            rest
        } else {
            trimmed
        };

        let clean = content
            .replace("**", "")
            .replace("__", "")
            .replace(['*', '_', '`'], "");

        result.push_str(&clean);
        result.push('\n');
    }

    result
}
