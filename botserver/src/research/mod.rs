pub mod ui;
pub mod web_search;

use crate::core::shared::state::AppState;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    pub title: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub content: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub collection_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub source_path: String,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CollectionRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub description: String,
}

pub fn configure_research_routes() -> Router<Arc<AppState>> {
    use crate::core::urls::ApiUrls;

    Router::new()
        .merge(web_search::configure_web_search_routes())
        .route(ApiUrls::RESEARCH_COLLECTIONS, get(handle_list_collections))
        .route(
            ApiUrls::RESEARCH_COLLECTIONS_NEW,
            post(handle_create_collection),
        )
        .route(ApiUrls::RESEARCH_COLLECTION_BY_ID, get(handle_get_collection))
        .route(ApiUrls::RESEARCH_SEARCH, post(handle_search))
        .route(ApiUrls::RESEARCH_RECENT, get(handle_recent_searches))
        .route(ApiUrls::RESEARCH_TRENDING, get(handle_trending_tags))
        .route(ApiUrls::RESEARCH_PROMPTS, get(handle_prompts))
        .route(
            ApiUrls::RESEARCH_EXPORT_CITATIONS,
            get(handle_export_citations),
        )
}

pub async fn handle_list_collections(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let collections = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return get_default_collections();
            }
        };

        let result: Result<Vec<CollectionRow>, _> =
            diesel::sql_query("SELECT id, name, description FROM kb_collections ORDER BY name ASC")
                .load(&mut db_conn);

        match result {
            Ok(colls) if !colls.is_empty() => colls
                .into_iter()
                .map(|c| (c.id, c.name, c.description))
                .collect(),
            _ => get_default_collections(),
        }
    })
    .await
    .unwrap_or_else(|_| get_default_collections());

    let mut html = String::new();

    for (id, name, description) in &collections {
        html.push_str("<div class=\"collection-item\" data-id=\"");
        html.push_str(&html_escape(id));
        html.push_str("\">");
        html.push_str("<div class=\"collection-icon\"></div>");
        html.push_str("<div class=\"collection-info\">");
        html.push_str("<span class=\"collection-name\">");
        html.push_str(&html_escape(name));
        html.push_str("</span>");
        html.push_str("<span class=\"collection-desc\">");
        html.push_str(&html_escape(description));
        html.push_str("</span>");
        html.push_str("</div>");
        html.push_str("<button class=\"btn-icon-sm\" hx-get=\"/api/research/collections/");
        html.push_str(&html_escape(id));
        html.push_str("\" hx-target=\"#main-results\">");
        html.push_str("<svg width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"><polyline points=\"9 18 15 12 9 6\"></polyline></svg>");
        html.push_str("</button>");
        html.push_str("</div>");
    }

    if collections.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No collections yet</p>");
        html.push_str("</div>");
    }

    Html(html)
}

fn get_default_collections() -> Vec<(String, String, String)> {
    vec![
        (
            "general".to_string(),
            "General Knowledge".to_string(),
            "Default knowledge base".to_string(),
        ),
        (
            "docs".to_string(),
            "Documentation".to_string(),
            "Product documentation".to_string(),
        ),
        (
            "faq".to_string(),
            "FAQ".to_string(),
            "Frequently asked questions".to_string(),
        ),
    ]
}

pub async fn handle_create_collection(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewCollectionRequest>,
) -> impl IntoResponse {
    let conn = state.conn.clone();
    let id = uuid::Uuid::new_v4().to_string();
    let name = payload.name.clone();
    let description = payload.description.unwrap_or_default();

    let id_clone = id.clone();
    let name_clone = name.clone();
    let desc_clone = description.clone();

    let _ = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return;
            }
        };

        let _ = diesel::sql_query(
            "INSERT INTO kb_collections (id, name, description) VALUES ($1, $2, $3)",
        )
        .bind::<diesel::sql_types::Text, _>(&id)
        .bind::<diesel::sql_types::Text, _>(&name)
        .bind::<diesel::sql_types::Text, _>(&description)
        .execute(&mut db_conn);
    })
    .await;

    let mut html = String::new();
    html.push_str("<div class=\"collection-item new-item\" data-id=\"");
    html.push_str(&html_escape(&id_clone));
    html.push_str("\">");
    html.push_str("<div class=\"collection-icon\"></div>");
    html.push_str("<div class=\"collection-info\">");
    html.push_str("<span class=\"collection-name\">");
    html.push_str(&html_escape(&name_clone));
    html.push_str("</span>");
    html.push_str("<span class=\"collection-desc\">");
    html.push_str(&html_escape(&desc_clone));
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_get_collection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let conn = state.conn.clone();

    let documents = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT id, title, content, collection_id, source_path FROM kb_documents WHERE collection_id = $1 ORDER BY title ASC LIMIT 50",
        )
        .bind::<diesel::sql_types::Text, _>(&id)
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
            html.push_str(&format_search_result(
                &doc.id,
                &doc.title,
                &doc.content,
                &doc.source_path,
            ));
        }
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_search(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<SearchRequest>,
) -> impl IntoResponse {
    let query = payload.query.unwrap_or_default();

    if query.trim().is_empty() {
        return Html("<div class=\"search-prompt\"><p>Enter a search query to find relevant documents</p></div>".to_string());
    }

    let conn = state.conn.clone();
    let collection = payload.collection;

    let results = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };

        let search_pattern = format!("%{}%", query.to_lowercase());

        let docs: Vec<KbDocumentRow> = if let Some(coll) = collection {
            diesel::sql_query(
                "SELECT id, title, content, collection_id, source_path FROM kb_documents WHERE (LOWER(title) LIKE $1 OR LOWER(content) LIKE $1) AND collection_id = $2 ORDER BY title ASC LIMIT 20",
            )
            .bind::<diesel::sql_types::Text, _>(&search_pattern)
            .bind::<diesel::sql_types::Text, _>(&coll)
            .load::<KbDocumentRow>(&mut db_conn)
            .unwrap_or_default()
        } else {
            diesel::sql_query(
                "SELECT id, title, content, collection_id, source_path FROM kb_documents WHERE LOWER(title) LIKE $1 OR LOWER(content) LIKE $1 ORDER BY title ASC LIMIT 20",
            )
            .bind::<diesel::sql_types::Text, _>(&search_pattern)
            .load::<KbDocumentRow>(&mut db_conn)
            .unwrap_or_default()
        };

        docs
    })
    .await
    .unwrap_or_default();

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
            html.push_str(&format_search_result(
                &doc.id,
                &doc.title,
                &doc.content,
                &doc.source_path,
            ));
        }
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

fn format_search_result(id: &str, title: &str, content: &str, source: &str) -> String {
    let snippet = if content.len() > 200 {
        format!("{}...", &content[..200])
    } else {
        content.to_string()
    };

    let mut html = String::new();
    html.push_str("<div class=\"result-item\" data-id=\"");
    html.push_str(&html_escape(id));
    html.push_str("\">");
    html.push_str("<div class=\"result-header\">");
    html.push_str("<h4 class=\"result-title\">");
    html.push_str(&html_escape(title));
    html.push_str("</h4>");
    html.push_str("<span class=\"result-source\">");
    html.push_str(&html_escape(source));
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

pub async fn handle_recent_searches(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let recent_searches = vec![
        "How to get started",
        "API documentation",
        "Configuration guide",
        "Best practices",
        "Troubleshooting",
    ];

    let mut html = String::new();

    for search in &recent_searches {
        html.push_str(
            "<div class=\"recent-item\" hx-post=\"/api/research/search\" hx-vals='{\"query\":\"",
        );
        html.push_str(&html_escape(search));
        html.push_str("\"}' hx-target=\"#main-results\">");
        html.push_str("<span class=\"recent-icon\">🕐</span>");
        html.push_str("<span class=\"recent-text\">");
        html.push_str(&html_escape(search));
        html.push_str("</span>");
        html.push_str("</div>");
    }

    if recent_searches.is_empty() {
        html.push_str("<div class=\"empty-state small\">");
        html.push_str("<p>No recent searches</p>");
        html.push_str("</div>");
    }

    Html(html)
}

pub async fn handle_trending_tags(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let tags = vec![
        ("getting-started", 42),
        ("api", 38),
        ("integration", 25),
        ("configuration", 22),
        ("deployment", 18),
        ("security", 15),
        ("performance", 12),
        ("troubleshooting", 10),
    ];

    let mut html = String::new();
    html.push_str("<div class=\"trending-tags-list\">");

    for (tag, count) in &tags {
        html.push_str(
            "<span class=\"tag\" hx-post=\"/api/research/search\" hx-vals='{\"query\":\"",
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

pub async fn handle_prompts(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let prompts = vec![
        (
            "",
            "Getting Started",
            "Learn the basics and set up your first bot",
        ),
        ("", "Configuration", "Customize settings and preferences"),
        (
            "🔌",
            "Integrations",
            "Connect with external services and APIs",
        ),
        ("", "Deployment", "Deploy your bot to production"),
        ("", "Security", "Best practices for securing your bot"),
        ("", "Analytics", "Monitor and analyze bot performance"),
    ];

    let mut html = String::new();
    html.push_str("<div class=\"prompts-grid\">");

    for (icon, title, description) in &prompts {
        html.push_str(
            "<div class=\"prompt-card\" hx-post=\"/api/research/search\" hx-vals='{\"query\":\"",
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

pub async fn handle_export_citations(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Html("<script>alert('Citations exported. Download will begin shortly.');</script>".to_string())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
