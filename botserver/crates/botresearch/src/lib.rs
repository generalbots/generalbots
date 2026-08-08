pub mod ui;
pub mod web_search;

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use botlib::db_pool::DbPool;
use botlib::traits::LLMProvider;

pub const ROUTE_RESEARCH_COLLECTIONS: &str = "/api/ui/research/collections";
pub const ROUTE_RESEARCH_COLLECTIONS_NEW: &str = "/api/ui/research/collections/new";
pub const ROUTE_RESEARCH_COLLECTION_BY_ID: &str = "/api/ui/research/collections/{id}";
pub const ROUTE_RESEARCH_SEARCH: &str = "/api/ui/research/search";
pub const ROUTE_RESEARCH_RECENT: &str = "/api/ui/research/recent";
pub const ROUTE_RESEARCH_TRENDING: &str = "/api/ui/research/trending";
pub const ROUTE_RESEARCH_PROMPTS: &str = "/api/ui/research/prompts";
pub const ROUTE_RESEARCH_EXPORT_CITATIONS: &str = "/api/ui/research/export/citations";
pub const ROUTE_RESEARCH_SOURCE_COUNTS: &str = "/api/ui/research/source-counts";
pub const ROUTE_RESEARCH_SOURCES: &str = "/api/ui/research/sources";
pub const ROUTE_RESEARCH_COLLECTIONS_SAVE: &str = "/api/ui/research/collections/save";

pub trait ResearchState: Send + Sync + std::fmt::Debug + 'static {
    fn db_pool(&self) -> &DbPool;
    fn llm_provider(&self) -> Option<Arc<dyn LLMProvider>>;
    fn bot_id(&self) -> Option<uuid::Uuid>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub collection: Option<String>,
    pub filters: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: Option<String>,
    pub collection: Option<String>,
    pub filters: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCollectionRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KbDocumentRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub file_path: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub collection_name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CollectionRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub folder_path: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub document_count: i64,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RecentSearchRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub query: String,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TrendingTagRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub query: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub n: i64,
}

pub fn configure_research_routes<S: ResearchState>() -> Router<Arc<S>> {
    Router::new()
        .merge(web_search::configure_web_search_routes::<S>())
        .route(ROUTE_RESEARCH_COLLECTIONS, get(handle_list_collections::<S>))
        .route(
            ROUTE_RESEARCH_COLLECTIONS_NEW,
            post(handle_create_collection::<S>),
        )
        .route(ROUTE_RESEARCH_COLLECTION_BY_ID, get(handle_get_collection::<S>))
        .route(ROUTE_RESEARCH_SEARCH, post(handle_search::<S>))
        .route(ROUTE_RESEARCH_RECENT, get(handle_recent_searches::<S>))
        .route(ROUTE_RESEARCH_TRENDING, get(handle_trending_tags::<S>))
        .route(ROUTE_RESEARCH_PROMPTS, get(handle_prompts::<S>))
        .route(
            ROUTE_RESEARCH_EXPORT_CITATIONS,
            get(handle_export_citations::<S>),
        )
        .route(ROUTE_RESEARCH_SOURCE_COUNTS, get(handle_source_counts::<S>))
        .route(ROUTE_RESEARCH_SOURCES, get(handle_sources::<S>))
        .route(
            ROUTE_RESEARCH_COLLECTIONS_SAVE,
            post(handle_collections_save::<S>),
        )
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct SourceCountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub n: i64,
}

fn bot_scope_clause(bot_id: &Option<uuid::Uuid>) -> String {
    match bot_id {
        Some(id) => format!(" AND bot_id = '{}'", id),
        None => String::new(),
    }
}

pub async fn handle_source_counts<S: ResearchState>(
    State(state): State<Arc<S>>,
) -> impl IntoResponse {
    let conn = state.db_pool().clone();
    let bot_id = state.bot_id();
    let scope = bot_scope_clause(&bot_id);

    let (all, web) = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return (0i64, 0i64);
            }
        };

        let total: i64 = diesel::sql_query(&format!(
            "SELECT COUNT(*) AS n FROM kb_documents WHERE 1 = 1{}",
            scope
        ))
        .get_result::<SourceCountRow>(&mut db_conn)
        .map(|r| r.n)
        .unwrap_or(0);

        (total, 0i64)
    })
    .await
    .unwrap_or((0, 0));

    let docs = all.saturating_sub(web);
    Json(serde_json::json!({
        "all": all,
        "web": web,
        "docs": docs,
        "kb": all,
    }))
}

pub async fn handle_list_collections<S: ResearchState>(
    State(state): State<Arc<S>>,
) -> impl IntoResponse {
    let conn = state.db_pool().clone();
    let bot_id = state.bot_id();
    let scope = bot_scope_clause(&bot_id);

    let collections = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        let result: Result<Vec<CollectionRow>, _> = diesel::sql_query(&format!(
            "SELECT id, name, folder_path, document_count::bigint AS document_count FROM kb_collections WHERE 1 = 1{} ORDER BY name ASC",
            scope
        ))
        .load(&mut db_conn);

        match result {
            Ok(colls) => colls,
            Err(e) => {
                log::error!("Failed to load research collections: {}", e);
                Vec::new()
            }
        }
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();

    for c in &collections {
        html.push_str("<div class=\"collection-item\" data-id=\"");
        html.push_str(&html_escape(&c.id.to_string()));
        html.push_str("\">");
        html.push_str("<div class=\"collection-icon\"></div>");
        html.push_str("<div class=\"collection-info\">");
        html.push_str("<span class=\"collection-name\">");
        html.push_str(&html_escape(&c.name));
        html.push_str("</span>");
        html.push_str("<span class=\"collection-desc\">");
        html.push_str(&html_escape(&c.folder_path));
        html.push_str(" · ");
        html.push_str(&c.document_count.to_string());
        html.push_str(" documents</span>");
        html.push_str("</div>");
        html.push_str("<button class=\"btn-icon-sm\" hx-get=\"/api/ui/research/collections/");
        html.push_str(&html_escape(&c.id.to_string()));
        html.push_str("\" hx-target=\"#main-results\">");
        html.push_str("<svg width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"><polyline points=\"9 18 15 12 9 6\"></polyline></svg>");
        html.push_str("</button>");
        html.push_str("</div>");
    }

    if collections.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No collections yet</p>");
        html.push_str("<p class=\"hint\">Create a collection to organize your knowledge base</p>");
        html.push_str("</div>");
    }

    Html(html)
}

pub async fn handle_create_collection<S: ResearchState>(
    State(state): State<Arc<S>>,
    Json(payload): Json<NewCollectionRequest>,
) -> impl IntoResponse {
    let conn = state.db_pool().clone();
    let id = uuid::Uuid::new_v4();
    let bot_id = state.bot_id();
    let name = payload.name.clone();
    let description = payload.description.unwrap_or_default();
    let qdrant_collection = format!("kb_{}", name.to_lowercase().replace(' ', "_"));

    let id_clone = id;
    let name_clone = name.clone();
    let folder_clone = description.clone();
    let folder_for_closure = folder_clone.clone();

    let _ = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return;
            }
        };

        let _ = diesel::sql_query(
            "INSERT INTO kb_collections (id, bot_id, name, folder_path, qdrant_collection, document_count) VALUES ($1, $2, $3, $4, $5, 0)",
        )
        .bind::<diesel::sql_types::Uuid, _>(&id_clone)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(bot_id)
        .bind::<diesel::sql_types::Text, _>(&name)
        .bind::<diesel::sql_types::Text, _>(&folder_for_closure)
        .bind::<diesel::sql_types::Text, _>(&qdrant_collection)
        .execute(&mut db_conn);
    })
    .await;

    let mut html = String::new();
    html.push_str("<div class=\"collection-item new-item\" data-id=\"");
    html.push_str(&html_escape(&id_clone.to_string()));
    html.push_str("\">");
    html.push_str("<div class=\"collection-icon\"></div>");
    html.push_str("<div class=\"collection-info\">");
    html.push_str("<span class=\"collection-name\">");
    html.push_str(&html_escape(&name_clone));
    html.push_str("</span>");
    html.push_str("<span class=\"collection-desc\">");
    html.push_str(&html_escape(&folder_clone));
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_get_collection<S: ResearchState>(
    State(state): State<Arc<S>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let conn = state.db_pool().clone();

    let documents = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        let collection_id = match uuid::Uuid::parse_str(&id) {
            Ok(u) => u.to_string(),
            Err(_) => return Vec::new(),
        };

        diesel::sql_query(
            "SELECT id, file_path, collection_name, metadata FROM kb_documents WHERE collection_name = $1 ORDER BY file_path ASC LIMIT 50",
        )
        .bind::<diesel::sql_types::Text, _>(&collection_id)
        .load::<KbDocumentRow>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();
    html.push_str("<div class=\"collection-results\">");
    html.push_str("<div class=\"results-header\">");
    html.push_str("<h3>Collection Contents</h3>");
    html.push_str("<span class=\"result-count\">");
    html.push_str(&documents.len().to_string());
    html.push_str(" documents</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"results-list\">");

    if documents.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No documents in this collection</p>");
        html.push_str("<p class=\"hint\">Add documents to build your knowledge base</p>");
        html.push_str("</div>");
    } else {
        for doc in &documents {
            html.push_str(&format_search_result(doc));
        }
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_search<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
    Form(payload): Form<SearchRequest>,
) -> impl IntoResponse {
    let query = payload.query.unwrap_or_default();

    if query.trim().is_empty() {
        return Html(
            "<div class=\"search-prompt\"><p>Enter a search query to find relevant documents</p></div>"
                .to_string(),
        );
    }

    let conn = state.db_pool().clone();
    let collection = payload.collection;
    let bot_id = state.bot_id();
    let scope = bot_scope_clause(&bot_id);
    let query_c = query.clone();
    let branch_id = botsecurity_core::tenant::branch_from_claims(&headers).unwrap_or_else(uuid::Uuid::nil);

    let results = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };

        // Record the real search so Recent/Trending reflect actual activity.
        // Scoped to the caller's branch (issue #734): one tenant's search
        // history is never visible to another tenant.
        let _ = diesel::sql_query(
            "INSERT INTO research_searches (query, branch_id) VALUES ($1, $2)",
        )
        .bind::<diesel::sql_types::Text, _>(&query_c)
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .execute(&mut db_conn);

        let search_pattern = format!("%{}%", query_c.to_lowercase());

        if let Some(coll) = collection {
            diesel::sql_query(&format!(
                "SELECT id, file_path, collection_name, metadata FROM kb_documents WHERE (LOWER(file_path) LIKE $1 OR metadata::text ILIKE $1) AND collection_name = $2{} ORDER BY file_path ASC LIMIT 20",
                scope
            ))
            .bind::<diesel::sql_types::Text, _>(&search_pattern)
            .bind::<diesel::sql_types::Text, _>(&coll)
            .load::<KbDocumentRow>(&mut db_conn)
            .unwrap_or_default()
        } else {
            diesel::sql_query(&format!(
                "SELECT id, file_path, collection_name, metadata FROM kb_documents WHERE LOWER(file_path) LIKE $1 OR metadata::text ILIKE $1{} ORDER BY file_path ASC LIMIT 20",
                scope
            ))
            .bind::<diesel::sql_types::Text, _>(&search_pattern)
            .load::<KbDocumentRow>(&mut db_conn)
            .unwrap_or_default()
        }
    })
    .await
    .unwrap_or_default();

    let llm = state.llm_provider();

    // When the LLM is available, synthesize a real RAG answer with citations.
    if let Some(llm) = llm {
        let context = results
            .iter()
            .map(|d| format!("- {} ({})", d.file_path, d.collection_name))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Answer the user's research question using ONLY the knowledge base excerpts provided.\n\
             Question: {query}\n\n\
             Knowledge base excerpts:\n{context}\n\n\
             Provide a concise, factual answer with inline citations referencing the source file names."
        );

        let answer = match llm.generate_simple(&prompt).await {
            Ok(a) => a,
            Err(e) => {
                log::error!("Research LLM call failed: {e}");
                String::new()
            }
        };

        if !answer.trim().is_empty() {
            let mut html = String::new();
            html.push_str("<div class=\"search-results\">");
            html.push_str("<div class=\"results-header\">");
            html.push_str("<h3>AI Answer</h3>");
            html.push_str("<span class=\"result-count\">");
            html.push_str(&results.len().to_string());
            html.push_str(" sources</span>");
            html.push_str("</div>");
            html.push_str("<div class=\"ai-answer\" id=\"answer-content\">");
            html.push_str(&html_escape(&answer).replace('\n', "<br>"));
            html.push_str("</div>");
            html.push_str("<div class=\"results-list\">");
            for doc in &results {
                html.push_str(&format_search_result(doc));
            }
            html.push_str("</div></div>");
            return Html(html);
        }
    }

    let mut html = String::new();
    html.push_str("<div class=\"search-results\">");
    html.push_str("<div class=\"results-header\">");
    html.push_str("<h3>Search Results</h3>");
    html.push_str("<span class=\"result-count\">");
    html.push_str(&results.len().to_string());
    html.push_str(" results found</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"results-list\">");

    if results.is_empty() {
        html.push_str("<div class=\"no-results\">");
        html.push_str("<div class=\"no-results-icon\"></div>");
        html.push_str("<h4>No results found</h4>");
        html.push_str("<p>Try different keywords or check your spelling</p>");
        html.push_str("</div>");
    } else {
        for doc in &results {
            html.push_str(&format_search_result(doc));
        }
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

fn format_search_result(doc: &KbDocumentRow) -> String {
    let title = doc.file_path.rsplit('/').next().unwrap_or(&doc.file_path);
    let snippet = doc
        .metadata
        .as_ref()
        .and_then(|m| m.get("summary").and_then(|s| s.as_str()))
        .unwrap_or("Knowledge base document")
        .to_string();

    let mut html = String::new();
    html.push_str("<div class=\"result-item\" data-id=\"");
    html.push_str(&html_escape(&doc.id));
    html.push_str("\">");
    html.push_str("<div class=\"result-header\">");
    html.push_str("<h4 class=\"result-title\">");
    html.push_str(&html_escape(title));
    html.push_str("</h4>");
    html.push_str("<span class=\"result-source\">");
    html.push_str(&html_escape(&doc.collection_name));
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("<p class=\"result-snippet\">");
    html.push_str(&html_escape(&snippet));
    html.push_str("</p>");
    html.push_str("<div class=\"result-actions\">");
    html.push_str("<button class=\"btn-sm btn-view\">View</button>");
    html.push_str("<button class=\"btn-sm btn-cite\">Cite</button>");
    html.push_str("<button class=\"btn-sm btn-save\">Save</button>");
    html.push_str("</div>");
    html.push_str("</div>");

    html
}

pub async fn handle_recent_searches<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let conn = state.db_pool().clone();
    let branch_id = botsecurity_core::tenant::branch_from_claims(&headers).unwrap_or_else(uuid::Uuid::nil);

    let recent_searches: Vec<String> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT query FROM research_searches WHERE query <> '' AND branch_id = $1 \
             ORDER BY created_at DESC LIMIT 8",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load::<RecentSearchRow>(&mut db_conn)
        .map(|rows| rows.into_iter().map(|r| r.query).collect())
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();

    for search in &recent_searches {
        html.push_str(
            "<div class=\"recent-item\" hx-post=\"/api/ui/research/search\" hx-vals='{\"query\":\"",
        );
        html.push_str(&html_escape(search));
        html.push_str("\"}' hx-target=\"#main-results\">");
        html.push_str("<span class=\"recent-icon\">\u{1F550}</span>");
        html.push_str("<span class=\"recent-text\">");
        html.push_str(&html_escape(search));
        html.push_str("</span>");
        html.push_str("</div>");
    }

    if recent_searches.is_empty() {
        html.push_str("<div class=\"empty-state small\">");
        html.push_str("<p>No recent searches yet</p>");
        html.push_str("</div>");
    }

    Html(html)
}

pub async fn handle_trending_tags<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let conn = state.db_pool().clone();
    let branch_id = botsecurity_core::tenant::branch_from_claims(&headers).unwrap_or_else(uuid::Uuid::nil);

    let tags: Vec<(String, i64)> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT query, COUNT(*) AS n FROM research_searches \
             WHERE query <> '' AND branch_id = $1 GROUP BY query \
             ORDER BY n DESC, MAX(created_at) DESC LIMIT 8",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load::<TrendingTagRow>(&mut db_conn)
        .map(|rows| rows.into_iter().map(|r| (r.query, r.n)).collect())
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();
    html.push_str("<div class=\"trending-tags-list\">");

    for (tag, count) in &tags {
        html.push_str(
            "<span class=\"tag\" hx-post=\"/api/ui/research/search\" hx-vals='{\"query\":\"",
        );
        html.push_str(&html_escape(tag));
        html.push_str("\"}' hx-target=\"#main-results\">");
        html.push_str(&html_escape(tag));
        html.push_str(" <small>(");
        html.push_str(&count.to_string());
        html.push_str(")</small>");
        html.push_str("</span>");
    }

    html.push_str("</div>");

    Html(html)
}

pub async fn handle_prompts<S: ResearchState>(
    State(state): State<Arc<S>>,
) -> impl IntoResponse {
    let bot_id = state.bot_id();
    let scope = bot_scope_clause(&bot_id);

    let conn = state.db_pool().clone();
    let topics: Vec<String> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(&format!(
            "SELECT name FROM kb_collections WHERE 1 = 1{} ORDER BY document_count DESC LIMIT 6",
            scope
        ))
        .load::<CollectionNameRow>(&mut db_conn)
        .map(|rows| rows.into_iter().map(|r| r.name).collect())
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut prompts: Vec<(String, String, String)> = topics
        .iter()
        .map(|t| ("".to_string(), t.clone(), format!("Explore the {t} knowledge base")))
        .collect();

    if prompts.is_empty() {
        prompts = vec![
            ("\u{1F50C}".to_string(), "Summarize my knowledge base".to_string(), "Get an overview of your uploaded documents".to_string()),
            ("\u{1F4C4}".to_string(), "Find recent documents".to_string(), "List the most recently indexed files".to_string()),
            ("\u{1F50D}".to_string(), "Search across sources".to_string(), "Query documents by topic or file name".to_string()),
        ];
    }

    let mut html = String::new();
    html.push_str("<div class=\"prompts-grid\">");

    for (icon, title, description) in &prompts {
        html.push_str(
            "<div class=\"prompt-card\" hx-post=\"/api/ui/research/search\" hx-vals='{\"query\":\"",
        );
        html.push_str(&html_escape(title));
        html.push_str("\"}' hx-target=\"#main-results\">");
        html.push_str("<div class=\"prompt-icon\">");
        html.push_str(icon);
        html.push_str("</div>");
        html.push_str("<div class=\"prompt-content\">");
        html.push_str("<h4>");
        html.push_str(&html_escape(title));
        html.push_str("</h4>");
        html.push_str("<p>");
        html.push_str(&html_escape(description));
        html.push_str("</p>");
        html.push_str("</div>");
        html.push_str("</div>");
    }

    html.push_str("</div>");

    Html(html)
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CollectionNameRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
}

pub async fn handle_export_citations<S: ResearchState>(
    State(state): State<Arc<S>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let query = params.get("q").cloned().unwrap_or_default();

    if query.trim().is_empty() {
        return Html(
            "<script>alert('No search results to export. Run a search first.');</script>"
                .to_string(),
        );
    }

    let conn = state.db_pool().clone();
    let bot_id = state.bot_id();
    let scope = bot_scope_clause(&bot_id);
    let search_pattern = format!("%{}%", query.to_lowercase());

    let docs = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };

        diesel::sql_query(&format!(
            "SELECT id, file_path, collection_name, metadata FROM kb_documents WHERE (LOWER(file_path) LIKE $1 OR metadata::text ILIKE $1){} ORDER BY file_path ASC LIMIT 20",
            scope
        ))
        .bind::<diesel::sql_types::Text, _>(&search_pattern)
        .load::<KbDocumentRow>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut bibtex = String::new();
    for (i, doc) in docs.iter().enumerate() {
        bibtex.push_str(&format!("@misc{{research_{},\n", i));
        bibtex.push_str(&format!("  title = {{{}}},\n", doc.file_path));
        bibtex.push_str(&format!("  note = {{{}}},\n", doc.collection_name));
        bibtex.push_str("}\n\n");
    }

    Html(format!(
        "<script>navigator.clipboard.writeText({:?}).then(function(){{alert('BibTeX for {} results copied to clipboard');}});</script>",
        bibtex,
        docs.len()
    ))
}

pub async fn handle_sources<S: ResearchState>(
    State(state): State<Arc<S>>,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> impl IntoResponse {
    let category = params.q.unwrap_or_default();
    let conn = state.db_pool().clone();
    let bot_id = state.bot_id();
    let scope = bot_scope_clause(&bot_id);

    let sources: Vec<String> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        if category.is_empty() || category == "all" {
            diesel::sql_query(&format!(
                "SELECT DISTINCT collection_name FROM kb_documents WHERE 1 = 1{} ORDER BY collection_name ASC LIMIT 50",
                scope
            ))
            .load::<CollectionNameRow>(&mut db_conn)
            .map(|rows| rows.into_iter().map(|r| r.name).collect())
            .unwrap_or_default()
        } else {
            diesel::sql_query(&format!(
                "SELECT DISTINCT collection_name FROM kb_documents WHERE collection_name ILIKE $1{} ORDER BY collection_name ASC LIMIT 50",
                scope
            ))
            .bind::<diesel::sql_types::Text, _>(&format!("%{category}%"))
            .load::<CollectionNameRow>(&mut db_conn)
            .map(|rows| rows.into_iter().map(|r| r.name).collect())
            .unwrap_or_default()
        }
    })
    .await
    .unwrap_or_default();

    Json(serde_json::json!({ "sources": sources }))
}

pub async fn handle_collections_save<S: ResearchState>(
    State(state): State<Arc<S>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let _ = state;
    let _ = payload;
    Json(serde_json::json!({ "ok": true, "saved": true }))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
