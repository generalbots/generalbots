use crate::shared::state::AppState;
use axum::{
    extract::{Multipart, Path, Query, State},
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    pub name: String,
    pub source_type: SourceType,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub content_hash: String,
    pub chunk_count: i32,
    pub status: SourceStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub indexed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Pdf,
    Docx,
    Txt,
    Markdown,
    Html,
    Csv,
    Xlsx,
    Url,
    Custom,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pdf => write!(f, "pdf"),
            Self::Docx => write!(f, "docx"),
            Self::Txt => write!(f, "txt"),
            Self::Markdown => write!(f, "markdown"),
            Self::Html => write!(f, "html"),
            Self::Csv => write!(f, "csv"),
            Self::Xlsx => write!(f, "xlsx"),
            Self::Url => write!(f, "url"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl From<&str> for SourceType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pdf" => Self::Pdf,
            "docx" | "doc" => Self::Docx,
            "txt" | "text" => Self::Txt,
            "md" | "markdown" => Self::Markdown,
            "html" | "htm" => Self::Html,
            "csv" => Self::Csv,
            "xlsx" | "xls" => Self::Xlsx,
            "url" | "web" => Self::Url,
            _ => Self::Custom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceStatus {
    Pending,
    Processing,
    Indexed,
    Failed,
    Reindexing,
}

impl std::fmt::Display for SourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Processing => write!(f, "processing"),
            Self::Indexed => write!(f, "indexed"),
            Self::Failed => write!(f, "failed"),
            Self::Reindexing => write!(f, "reindexing"),
        }
    }
}

impl From<&str> for SourceStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "processing" => Self::Processing,
            "indexed" => Self::Indexed,
            "failed" => Self::Failed,
            "reindexing" => Self::Reindexing,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub source_id: String,
    pub chunk_index: i32,
    pub content: String,
    pub token_count: i32,
    pub embedding: Option<Vec<f32>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub success: bool,
    pub source_id: Option<String>,
    pub message: String,
    pub chunks_created: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    pub collection: Option<String>,
    pub top_k: Option<usize>,
    pub min_score: Option<f32>,
    pub include_metadata: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub content: String,
    pub source_name: String,
    pub source_id: String,
    pub chunk_index: i32,
    pub score: f32,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub results: Vec<QueryResult>,
    pub query: String,
    pub total_results: usize,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSourcesQuery {
    pub status: Option<String>,
    pub source_type: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReindexRequest {
    pub source_ids: Option<Vec<String>>,
    pub force: Option<bool>,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct KnowledgeSourceRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    source_type: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    _file_path: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    _url: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    _content_hash: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    chunk_count: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    created_at: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    _updated_at: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    indexed_at: Option<String>,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct _ChunkRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    _id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    _source_id: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    _chunk_index: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    _content: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    _token_count: i32,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct SearchResultRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    _chunk_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    content: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    source_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    source_name: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    chunk_index: i32,
    #[diesel(sql_type = diesel::sql_types::Float)]
    score: f32,
}

pub fn configure_knowledge_base_routes() -> Router<Arc<AppState>> {
    use crate::core::urls::ApiUrls;

    Router::new()
        .route(ApiUrls::SOURCES_KB_UPLOAD, post(handle_upload_document))
        .route(ApiUrls::SOURCES_KB_LIST, get(handle_list_sources))
        .route(ApiUrls::SOURCES_KB_QUERY, post(handle_query_knowledge_base))
        .route(&ApiUrls::SOURCES_KB_BY_ID.replace(":id", "{id}"), get(handle_get_source))
        .route(&ApiUrls::SOURCES_KB_BY_ID.replace(":id", "{id}"), delete(handle_delete_source))
        .route(ApiUrls::SOURCES_KB_REINDEX, post(handle_reindex_sources))
        .route(ApiUrls::SOURCES_KB_STATS, get(handle_get_stats))
}

pub async fn handle_upload_document(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut file_name = String::new();
    let mut file_data: Vec<u8> = Vec::new();
    let mut collection = "default".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                file_name = field.file_name().unwrap_or("unknown").to_string();
                if let Ok(data) = field.bytes().await {
                    file_data = data.to_vec();
                }
            }
            "collection" => {
                if let Ok(text) = field.text().await {
                    collection = text;
                }
            }
            _ => {}
        }
    }

    if file_data.is_empty() {
        return Json(UploadResponse {
            success: false,
            source_id: None,
            message: "No file provided".to_string(),
            chunks_created: None,
        });
    }

    let extension = file_name.rsplit('.').next().unwrap_or("txt").to_lowercase();
    let source_type = SourceType::from(extension.as_str());

    let content = match extract_text_content(&file_data, &source_type) {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to extract text from {}: {}", file_name, e);
            return Json(UploadResponse {
                success: false,
                source_id: None,
                message: format!("Failed to extract text: {}", e),
                chunks_created: None,
            });
        }
    };

    let content_hash = compute_content_hash(&content);
    let source_id = Uuid::new_v4().to_string();

    let chunks = chunk_text(&content, 512, 50);
    let chunk_count = chunks.len() as i32;

    let conn = state.conn.clone();
    let source_id_clone = source_id.clone();
    let file_name_clone = file_name.clone();
    let source_type_str = source_type.to_string();
    let content_hash_clone = content_hash.clone();
    let collection_clone = collection.clone();
    let chunks_clone = chunks.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                error!("DB connection error: {}", e);
                return Err(format!("Database connection error: {}", e));
            }
        };

        let insert_result = diesel::sql_query(
            "INSERT INTO knowledge_sources (id, name, source_type, content_hash, chunk_count, status, collection, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'processing', $6, NOW(), NOW())
             ON CONFLICT (id) DO NOTHING"
        )
        .bind::<diesel::sql_types::Text, _>(&source_id_clone)
        .bind::<diesel::sql_types::Text, _>(&file_name_clone)
        .bind::<diesel::sql_types::Text, _>(&source_type_str)
        .bind::<diesel::sql_types::Text, _>(&content_hash_clone)
        .bind::<diesel::sql_types::Integer, _>(chunk_count)
        .bind::<diesel::sql_types::Text, _>(&collection_clone)
        .execute(&mut db_conn);

        if let Err(e) = insert_result {
            error!("Failed to insert source: {}", e);
            return Err(format!("Failed to insert source: {}", e));
        }

        for (idx, chunk_content) in chunks_clone.iter().enumerate() {
            let chunk_id = Uuid::new_v4().to_string();
            let token_count = estimate_tokens(chunk_content);

            let chunk_result = diesel::sql_query(
                "INSERT INTO knowledge_chunks (id, source_id, chunk_index, content, token_count, created_at)
                 VALUES ($1, $2, $3, $4, $5, NOW())
                 ON CONFLICT (id) DO NOTHING"
            )
            .bind::<diesel::sql_types::Text, _>(&chunk_id)
            .bind::<diesel::sql_types::Text, _>(&source_id_clone)
            .bind::<diesel::sql_types::Integer, _>(idx as i32)
            .bind::<diesel::sql_types::Text, _>(chunk_content)
            .bind::<diesel::sql_types::Integer, _>(token_count)
            .execute(&mut db_conn);

            if let Err(e) = chunk_result {
                warn!("Failed to insert chunk {}: {}", idx, e);
            }
        }

        let _ = diesel::sql_query(
            "UPDATE knowledge_sources SET status = 'indexed', indexed_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind::<diesel::sql_types::Text, _>(&source_id_clone)
        .execute(&mut db_conn);

        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!(
                "Successfully ingested {} with {} chunks",
                file_name, chunk_count
            );
            Json(UploadResponse {
                success: true,
                source_id: Some(source_id),
                message: format!(
                    "Successfully ingested '{}' with {} chunks",
                    file_name, chunk_count
                ),
                chunks_created: Some(chunk_count),
            })
        }
        Ok(Err(e)) => Json(UploadResponse {
            success: false,
            source_id: None,
            message: e,
            chunks_created: None,
        }),
        Err(e) => Json(UploadResponse {
            success: false,
            source_id: None,
            message: format!("Task error: {}", e),
            chunks_created: None,
        }),
    }
}

pub async fn handle_list_sources(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListSourcesQuery>,
) -> impl IntoResponse {
    let conn = state.conn.clone();
    let status_filter = params.status.clone();
    let type_filter = params.source_type.clone();
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let sources = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        let mut query = String::from(
            "SELECT id, name, source_type, file_path, url, content_hash, chunk_count, status,
                    created_at::text, updated_at::text, indexed_at::text
             FROM knowledge_sources WHERE 1=1",
        );

        if let Some(ref status) = status_filter {
            let _ = write!(query, " AND status = '{}'", status.replace('\'', "''"));
        }
        if let Some(ref stype) = type_filter {
            let _ = write!(query, " AND source_type = '{}'", stype.replace('\'', "''"));
        }

        let _ = write!(query, " ORDER BY created_at DESC LIMIT {per_page} OFFSET {offset}");

        diesel::sql_query(&query)
            .load::<KnowledgeSourceRow>(&mut db_conn)
            .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();
    html.push_str("<div class=\"sources-list\">");

    if sources.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No knowledge sources found</p>");
        html.push_str("<p class=\"hint\">Upload documents to build your knowledge base</p>");
        html.push_str("</div>");
    } else {
        for source in &sources {
            let status_class = match source.status.as_str() {
                "indexed" => "status-success",
                "processing" => "status-processing",
                "failed" => "status-error",
                "pending" => "status-pending",
                _ => "status-unknown",
            };

            let type_icon = match source.source_type.as_str() {
                "pdf" => "📄",
                "docx" | "doc" => "📝",
                "txt" | "text" => "📃",
                "markdown" | "md" => "📋",
                "html" => "🌐",
                "csv" => "📊",
                "xlsx" | "xls" => "📈",
                "url" => "🔗",
                _ => "📁",
            };

            html.push_str("<div class=\"source-item\" data-id=\"");
            html.push_str(&html_escape(&source.id));
            html.push_str("\">");

            html.push_str("<div class=\"source-icon\">");
            html.push_str(type_icon);
            html.push_str("</div>");

            html.push_str("<div class=\"source-info\">");
            html.push_str("<h4 class=\"source-name\">");
            html.push_str(&html_escape(&source.name));
            html.push_str("</h4>");
            html.push_str("<div class=\"source-meta\">");
            html.push_str("<span class=\"source-type\">");
            html.push_str(&html_escape(&source.source_type));
            html.push_str("</span>");
            html.push_str("<span class=\"source-chunks\">");
            html.push_str(&source.chunk_count.to_string());
            html.push_str(" chunks</span>");
            html.push_str("<span class=\"source-status ");
            html.push_str(status_class);
            html.push_str("\">");
            html.push_str(&html_escape(&source.status));
            html.push_str("</span>");
            html.push_str("</div>");
            html.push_str("</div>");

            html.push_str("<div class=\"source-actions\">");
            html.push_str("<button class=\"btn-icon-sm\" title=\"Reindex\" ");
            html.push_str("hx-post=\"/api/sources/kb/reindex\" ");
            html.push_str("hx-vals='{\"source_ids\":[\"");
            html.push_str(&html_escape(&source.id));
            html.push_str("\"]}' hx-swap=\"none\">");
            html.push_str("<svg width=\"14\" height=\"14\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\">");
            html.push_str("<polyline points=\"23 4 23 10 17 10\"></polyline>");
            html.push_str("<path d=\"M20.49 15a9 9 0 1 1-2.12-9.36L23 10\"></path>");
            html.push_str("</svg></button>");

            html.push_str("<button class=\"btn-icon-sm btn-danger\" title=\"Delete\" ");
            html.push_str("hx-delete=\"/api/sources/kb/");
            html.push_str(&html_escape(&source.id));
            html.push_str("\" hx-confirm=\"Delete this source?\" hx-target=\"closest .source-item\" hx-swap=\"outerHTML\">");
            html.push_str("<svg width=\"14\" height=\"14\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\">");
            html.push_str("<polyline points=\"3 6 5 6 21 6\"></polyline>");
            html.push_str("<path d=\"M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2\"></path>");
            html.push_str("</svg></button>");
            html.push_str("</div>");

            html.push_str("</div>");
        }
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_query_knowledge_base(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QueryRequest>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();

    if payload.query.trim().is_empty() {
        return Json(QueryResponse {
            results: Vec::new(),
            query: payload.query,
            total_results: 0,
            processing_time_ms: 0,
        });
    }

    let conn = state.conn.clone();
    let query = payload.query.clone();
    let top_k = payload.top_k.unwrap_or(5).min(20);
    let min_score = payload.min_score.unwrap_or(0.0);
    let collection = payload.collection.clone();

    let results = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        let search_pattern = format!("%{}%", query.to_lowercase());

        let mut sql = String::from(
            "SELECT
                kc.id as chunk_id,
                kc.content,
                kc.source_id,
                ks.name as source_name,
                kc.chunk_index,
                CAST(ts_rank(to_tsvector('english', kc.content), plainto_tsquery('english', $1)) AS FLOAT4) as score
             FROM knowledge_chunks kc
             JOIN knowledge_sources ks ON kc.source_id = ks.id
             WHERE ks.status = 'indexed'
               AND (LOWER(kc.content) LIKE $2
                    OR to_tsvector('english', kc.content) @@ plainto_tsquery('english', $1))",
        );

        if collection.is_some() {
            sql.push_str(" AND ks.collection = $3");
        }

        let _ = write!(sql, " ORDER BY score DESC, kc.chunk_index ASC LIMIT {top_k}");

        let search_results: Vec<SearchResultRow> = if let Some(ref coll) = collection {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Text, _>(&query)
                .bind::<diesel::sql_types::Text, _>(&search_pattern)
                .bind::<diesel::sql_types::Text, _>(coll)
                .load(&mut db_conn)
                .unwrap_or_default()
        } else {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Text, _>(&query)
                .bind::<diesel::sql_types::Text, _>(&search_pattern)
                .load(&mut db_conn)
                .unwrap_or_default()
        };

        search_results
            .into_iter()
            .filter(|r| r.score >= min_score)
            .map(|r| QueryResult {
                content: r.content,
                source_name: r.source_name,
                source_id: r.source_id,
                chunk_index: r.chunk_index,
                score: r.score,
                metadata: serde_json::json!({}),
            })
            .collect::<Vec<_>>()
    })
    .await
    .unwrap_or_default();

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    Json(QueryResponse {
        total_results: results.len(),
        results,
        query: payload.query,
        processing_time_ms,
    })
}

pub async fn handle_get_source(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let conn = state.conn.clone();

    let source = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                error!("DB connection error: {}", e);
                return None;
            }
        };

        let sources: Vec<KnowledgeSourceRow> = diesel::sql_query(
            "SELECT id, name, source_type, file_path, url, content_hash, chunk_count, status,
                    created_at::text, updated_at::text, indexed_at::text
             FROM knowledge_sources WHERE id = $1",
        )
        .bind::<diesel::sql_types::Text, _>(&id)
        .load(&mut db_conn)
        .unwrap_or_default();

        sources.into_iter().next()
    })
    .await
    .unwrap_or(None);

    match source {
        Some(s) => {
            let mut html = String::new();
            html.push_str("<div class=\"source-detail\">");
            html.push_str("<h3>");
            html.push_str(&html_escape(&s.name));
            html.push_str("</h3>");

            html.push_str("<div class=\"detail-grid\">");
            html.push_str("<div class=\"detail-item\"><label>Type:</label><span>");
            html.push_str(&html_escape(&s.source_type));
            html.push_str("</span></div>");

            html.push_str("<div class=\"detail-item\"><label>Status:</label><span>");
            html.push_str(&html_escape(&s.status));
            html.push_str("</span></div>");

            html.push_str("<div class=\"detail-item\"><label>Chunks:</label><span>");
            html.push_str(&s.chunk_count.to_string());
            html.push_str("</span></div>");

            html.push_str("<div class=\"detail-item\"><label>Created:</label><span>");
            html.push_str(&html_escape(&s.created_at));
            html.push_str("</span></div>");

            if let Some(indexed) = &s.indexed_at {
                html.push_str("<div class=\"detail-item\"><label>Indexed:</label><span>");
                html.push_str(&html_escape(indexed));
                html.push_str("</span></div>");
            }

            html.push_str("</div></div>");
            Html(html)
        }
        None => Html("<div class=\"error\">Source not found</div>".to_string()),
    }
}

pub async fn handle_delete_source(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let conn = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                error!("DB connection error: {}", e);
                return false;
            }
        };

        let _ = diesel::sql_query("DELETE FROM knowledge_chunks WHERE source_id = $1")
            .bind::<diesel::sql_types::Text, _>(&id)
            .execute(&mut db_conn);

        let delete_result = diesel::sql_query("DELETE FROM knowledge_sources WHERE id = $1")
            .bind::<diesel::sql_types::Text, _>(&id)
            .execute(&mut db_conn);

        delete_result.is_ok()
    })
    .await
    .unwrap_or(false);

    if result {
        Html("".to_string())
    } else {
        Html("<div class=\"error\">Failed to delete source</div>".to_string())
    }
}

pub async fn handle_reindex_sources(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReindexRequest>,
) -> impl IntoResponse {
    let conn = state.conn.clone();
    let source_ids = payload.source_ids.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                error!("DB connection error: {}", e);
                return 0;
            }
        };

        let sql = if let Some(ids) = source_ids {
            if ids.is_empty() {
                return 0;
            }
            let placeholders: Vec<String> = ids.iter().map(|id| format!("'{}'", id.replace('\'', "''"))).collect();
            format!(
                "UPDATE knowledge_sources SET status = 'reindexing', updated_at = NOW() WHERE id IN ({})",
                placeholders.join(",")
            )
        } else {
            "UPDATE knowledge_sources SET status = 'reindexing', updated_at = NOW() WHERE status = 'indexed'".to_string()
        };

        diesel::sql_query(&sql)
            .execute(&mut db_conn)
            .unwrap_or(0)
    })
    .await
    .unwrap_or(0);

    Json(serde_json::json!({
        "success": true,
        "sources_queued": result,
        "message": format!("{} sources queued for reindexing", result)
    }))
}

pub async fn handle_get_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let stats = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                error!("DB connection error: {}", e);
                return serde_json::json!({
                    "total_sources": 0,
                    "total_chunks": 0,
                    "indexed_sources": 0,
                    "pending_sources": 0,
                    "failed_sources": 0
                });
            }
        };

        #[derive(Debug, QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let total_sources: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM knowledge_sources")
            .load::<CountRow>(&mut db_conn)
            .map(|v| v.first().map(|r| r.count).unwrap_or(0))
            .unwrap_or(0);

        let total_chunks: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM knowledge_chunks")
            .load::<CountRow>(&mut db_conn)
            .map(|v| v.first().map(|r| r.count).unwrap_or(0))
            .unwrap_or(0);

        let indexed: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM knowledge_sources WHERE status = 'indexed'")
            .load::<CountRow>(&mut db_conn)
            .map(|v| v.first().map(|r| r.count).unwrap_or(0))
            .unwrap_or(0);

        let pending: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM knowledge_sources WHERE status IN ('pending', 'processing', 'reindexing')")
            .load::<CountRow>(&mut db_conn)
            .map(|v| v.first().map(|r| r.count).unwrap_or(0))
            .unwrap_or(0);

        let failed: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM knowledge_sources WHERE status = 'failed'")
            .load::<CountRow>(&mut db_conn)
            .map(|v| v.first().map(|r| r.count).unwrap_or(0))
            .unwrap_or(0);

        serde_json::json!({
            "total_sources": total_sources,
            "total_chunks": total_chunks,
            "indexed_sources": indexed,
            "pending_sources": pending,
            "failed_sources": failed
        })
    })
    .await
    .unwrap_or_else(|_| serde_json::json!({
        "error": "Failed to fetch stats"
    }));

    Json(stats)
}

fn extract_text_content(
    data: &[u8],
    source_type: &SourceType,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match source_type {
        SourceType::Txt | SourceType::Markdown | SourceType::Csv => {
            Ok(String::from_utf8_lossy(data).to_string())
        }
        SourceType::Html => {
            let html = String::from_utf8_lossy(data);
            Ok(strip_html_tags(&html))
        }
        SourceType::Pdf => {
            #[cfg(feature = "drive")]
            {
                match pdf_extract::extract_text_from_mem(data) {
                    Ok(text) => Ok(text),
                    Err(e) => {
                        warn!("PDF extraction failed: {}", e);
                        Ok(String::new())
                    }
                }
            }
            #[cfg(not(feature = "drive"))]
            {
                Err("PDF extraction not available without 'drive' feature".into())
            }
        }
        SourceType::Docx => extract_docx_text(data),
        SourceType::Xlsx => extract_xlsx_text(data),
        _ => Ok(String::from_utf8_lossy(data).to_string()),
    }
}

fn extract_docx_text(
    data: &[u8],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::{Cursor, Read};

    let cursor = Cursor::new(data);
    let mut archive = match zip::ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => return Err(format!("Failed to open DOCX: {e}").into()),
    };

    let Ok(mut file) = archive.by_name("word/document.xml") else {
        return Ok(String::new());
    };

    let mut xml = String::new();
    file.read_to_string(&mut xml)?;

    let mut in_text = false;
    let mut result = String::new();

    for part in xml.split('<') {
        if part.starts_with("w:t") || part.starts_with("w:t ") {
            in_text = true;
            continue;
        }
        if part.starts_with("/w:t") {
            in_text = false;
            result.push(' ');
            continue;
        }
        if part.starts_with("w:p") || part.starts_with("w:p ") {
            result.push('\n');
            continue;
        }
        if in_text {
            if let Some(pos) = part.find('>') {
                result.push_str(&part[pos + 1..]);
            } else {
                result.push_str(part);
            }
        }
    }

    Ok(result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n"))
}

fn extract_xlsx_text(
    data: &[u8],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::{Cursor, Read};

    let cursor = Cursor::new(data);
    let mut archive = match zip::ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => return Err(format!("Failed to open XLSX: {e}").into()),
    };

    let mut shared_strings: Vec<String> = Vec::new();

    if let Ok(mut file) = archive.by_name("xl/sharedStrings.xml") {
        let mut xml = String::new();
        file.read_to_string(&mut xml)?;

        for part in xml.split("<t") {
            if let Some(start) = part.find('>') {
                if let Some(end) = part[start..].find("</t>") {
                    let text = &part[start + 1..start + end];
                    shared_strings.push(text.to_string());
                }
            }
        }
    }

    let mut content = String::new();

    for i in 1..=10 {
        let sheet_name = format!("xl/worksheets/sheet{i}.xml");
        let Ok(mut file) = archive.by_name(&sheet_name) else {
            break;
        };

        let mut xml = String::new();
        file.read_to_string(&mut xml)?;

        for part in xml.split("<v>") {
            if let Some(end) = part.find("</v>") {
                let value = &part[..end];
                if let Ok(idx) = value.parse::<usize>() {
                    if let Some(text) = shared_strings.get(idx) {
                        content.push_str(text);
                        content.push('\t');
                    }
                } else {
                    content.push_str(value);
                    content.push('\t');
                }
            }
        }
        content.push('\n');
    }

    Ok(content)
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn compute_content_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return Vec::new();
    }

    if words.len() <= chunk_size {
        return vec![words.join(" ")];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let chunk: String = words[start..end].join(" ");
        chunks.push(chunk);

        if end >= words.len() {
            break;
        }

        start = if overlap < chunk_size {
            end - overlap
        } else {
            end
        };
    }

    chunks
}

fn estimate_tokens(text: &str) -> i32 {
    (text.split_whitespace().count() as f32 * 1.3) as i32
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
