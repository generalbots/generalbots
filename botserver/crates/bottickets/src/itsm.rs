use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::TicketsState;

#[derive(Debug, Deserialize)]
pub struct NewCiRequest {
    pub name: String,
    pub ci_type: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewKbArticleRequest {
    pub title: String,
    pub body: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CiRow {
    pub id: Uuid,
    pub name: String,
    pub ci_type: Option<String>,
    pub description: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct KbArticleRow {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub is_published: bool,
}

pub fn configure_itsm_routes() -> Router<Arc<TicketsState>> {
    Router::new()
        .route("/api/tickets/cis", get(list_cis).post(create_ci))
        .route("/api/tickets/cis/:id", get(get_ci).put(update_ci).delete(delete_ci))
        .route("/api/tickets/kb", get(list_kb_articles).post(create_kb_article))
        .route("/api/tickets/kb/:id", get(get_kb_article).put(update_kb_article).delete(delete_kb_article))
}

fn resolve_branch_id(state: &Arc<TicketsState>) -> Uuid {
    let Ok(mut conn) = state.pool.get() else {
        return Uuid::nil();
    };
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BranchRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }
    diesel::sql_query(
        "SELECT branch_id FROM bots WHERE is_default_for_branch = TRUE ORDER BY created_at ASC LIMIT 1",
    )
    .get_result::<BranchRow>(&mut conn)
    .map(|r| r.branch_id)
    .unwrap_or(Uuid::nil())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

async fn list_cis(State(state): State<Arc<TicketsState>>) -> impl IntoResponse {
    let branch_id = resolve_branch_id(&state);
    let conn = state.pool.clone();

    let rows: Vec<CiRow> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };

        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            ci_type: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            description: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
        }

        diesel::sql_query(
            "SELECT id, name, ci_type, description, status FROM ticket_cis WHERE branch_id = $1 ORDER BY name ASC LIMIT 100",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load::<Row>(&mut db_conn)
        .map(|rows| {
            rows.into_iter()
                .map(|r| CiRow {
                    id: r.id,
                    name: r.name,
                    ci_type: r.ci_type,
                    description: r.description,
                    status: r.status,
                })
                .collect()
        })
        .unwrap_or_else(|e| {
            log::error!("Failed to list CIs: {e}");
            Vec::new()
        })
    })
    .await
    .unwrap_or_default();

    Json(rows)
}

async fn create_ci(
    State(state): State<Arc<TicketsState>>,
    Json(payload): Json<NewCiRequest>,
) -> Result<Json<CiRow>, (axum::http::StatusCode, String)> {
    if payload.name.trim().is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "name is required".to_string()));
    }

    let branch_id = resolve_branch_id(&state);
    let conn = state.pool.clone();
    let id = Uuid::new_v4();
    let name = payload.name.clone();
    let ci_type = payload.ci_type.clone();
    let description = payload.description.clone();
    let status = payload.status.clone().unwrap_or_else(|| "operational".to_string());

    let (name_c, ci_type_c, description_c, status_c) = (
        name.clone(),
        ci_type.clone(),
        description.clone(),
        status.clone(),
    );

    tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return;
            }
        };
        let _ = diesel::sql_query(
            "INSERT INTO ticket_cis (id, org_id, branch_id, name, ci_type, description, status) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind::<diesel::sql_types::Uuid, _>(&id)
        .bind::<diesel::sql_types::Uuid, _>(&Uuid::nil())
        .bind::<diesel::sql_types::Uuid, _>(&branch_id)
        .bind::<diesel::sql_types::Text, _>(&name_c)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&ci_type_c)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&description_c)
        .bind::<diesel::sql_types::Text, _>(&status_c)
        .execute(&mut db_conn);
    })
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Task error: {e}")))?;

    Ok(Json(CiRow {
        id,
        name,
        ci_type,
        description,
        status,
    }))
}

async fn get_ci(State(state): State<Arc<TicketsState>>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.pool.clone();

    let row: Option<CiRow> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return None;
            }
        };
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            ci_type: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            description: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
        }
        diesel::sql_query("SELECT id, name, ci_type, description, status FROM ticket_cis WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(&id)
            .get_result::<Row>(&mut db_conn)
            .ok()
            .map(|r| CiRow {
                id: r.id,
                name: r.name,
                ci_type: r.ci_type,
                description: r.description,
                status: r.status,
            })
    })
    .await
    .unwrap_or(None);

    match row {
        Some(r) => Json(r).into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "CI not found").into_response(),
    }
}

async fn update_ci(
    State(state): State<Arc<TicketsState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<NewCiRequest>,
) -> Result<Json<CiRow>, (axum::http::StatusCode, String)> {
    let conn = state.pool.clone();
    let name = payload.name.clone();
    let ci_type = payload.ci_type.clone();
    let description = payload.description.clone();
    let status = payload.status.clone().unwrap_or_else(|| "operational".to_string());

    let (name_c, ci_type_c, description_c, status_c) = (
        name.clone(),
        ci_type.clone(),
        description.clone(),
        status.clone(),
    );

    tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return;
            }
        };
        let _ = diesel::sql_query(
            "UPDATE ticket_cis SET name = $1, ci_type = $2, description = $3, status = $4, updated_at = NOW() WHERE id = $5",
        )
        .bind::<diesel::sql_types::Text, _>(&name_c)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&ci_type_c)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&description_c)
        .bind::<diesel::sql_types::Text, _>(&status_c)
        .bind::<diesel::sql_types::Uuid, _>(&id)
        .execute(&mut db_conn);
    })
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Task error: {e}")))?;

    Ok(Json(CiRow {
        id,
        name,
        ci_type,
        description,
        status,
    }))
}

async fn delete_ci(State(state): State<Arc<TicketsState>>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.pool.clone();
    match tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return;
            }
        };
        let _ = diesel::sql_query("DELETE FROM ticket_cis WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(&id)
            .execute(&mut db_conn);
    })
    .await
    {
        Ok(()) => (),
        Err(e) => log::error!("Delete CI task error: {e}"),
    }

    axum::http::StatusCode::NO_CONTENT.into_response()
}

async fn list_kb_articles(State(state): State<Arc<TicketsState>>) -> impl IntoResponse {
    let branch_id = resolve_branch_id(&state);
    let conn = state.pool.clone();

    let rows: Vec<KbArticleRow> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            body: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            category: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Array<diesel::sql_types::Text>)]
            tags: Vec<String>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_published: bool,
        }
        diesel::sql_query(
            "SELECT id, title, body, category, tags, is_published FROM ticket_kb_articles WHERE branch_id = $1 ORDER BY title ASC LIMIT 100",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load::<Row>(&mut db_conn)
        .map(|rows| {
            rows.into_iter()
                .map(|r| KbArticleRow {
                    id: r.id,
                    title: r.title,
                    body: r.body,
                    category: r.category,
                    tags: r.tags,
                    is_published: r.is_published,
                })
                .collect()
        })
        .unwrap_or_else(|e| {
            log::error!("Failed to list KB articles: {e}");
            Vec::new()
        })
    })
    .await
    .unwrap_or_default();

    Json(rows)
}

async fn create_kb_article(
    State(state): State<Arc<TicketsState>>,
    Json(payload): Json<NewKbArticleRequest>,
) -> Result<Json<KbArticleRow>, (axum::http::StatusCode, String)> {
    if payload.title.trim().is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "title is required".to_string()));
    }

    let branch_id = resolve_branch_id(&state);
    let conn = state.pool.clone();
    let id = Uuid::new_v4();
    let title = payload.title.clone();
    let body = payload.body.clone();
    let category = payload.category.clone();
    let tags = payload.tags.clone().unwrap_or_default();
    let is_published = payload.is_published.unwrap_or(true);

    let (title_c, body_c, category_c, tags_c) = (
        title.clone(),
        body.clone(),
        category.clone(),
        tags.clone(),
    );

    tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return;
            }
        };
        let _ = diesel::sql_query(
            "INSERT INTO ticket_kb_articles (id, org_id, branch_id, title, body, category, tags, is_published) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind::<diesel::sql_types::Uuid, _>(&id)
        .bind::<diesel::sql_types::Uuid, _>(&Uuid::nil())
        .bind::<diesel::sql_types::Uuid, _>(&branch_id)
        .bind::<diesel::sql_types::Text, _>(&title_c)
        .bind::<diesel::sql_types::Text, _>(&body_c)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&category_c)
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(&tags_c)
        .bind::<diesel::sql_types::Bool, _>(&is_published)
        .execute(&mut db_conn);
    })
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Task error: {e}")))?;

    Ok(Json(KbArticleRow {
        id,
        title,
        body,
        category,
        tags,
        is_published,
    }))
}

async fn get_kb_article(State(state): State<Arc<TicketsState>>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.pool.clone();

    let row: Option<KbArticleRow> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return None;
            }
        };
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            body: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            category: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Array<diesel::sql_types::Text>)]
            tags: Vec<String>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_published: bool,
        }
        diesel::sql_query("SELECT id, title, body, category, tags, is_published FROM ticket_kb_articles WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(&id)
            .get_result::<Row>(&mut db_conn)
            .ok()
            .map(|r| KbArticleRow {
                id: r.id,
                title: r.title,
                body: r.body,
                category: r.category,
                tags: r.tags,
                is_published: r.is_published,
            })
    })
    .await
    .unwrap_or(None);

    match row {
        Some(r) => Json(r).into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "KB article not found").into_response(),
    }
}

async fn update_kb_article(
    State(state): State<Arc<TicketsState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<NewKbArticleRequest>,
) -> Result<Json<KbArticleRow>, (axum::http::StatusCode, String)> {
    let conn = state.pool.clone();
    let title = payload.title.clone();
    let body = payload.body.clone();
    let category = payload.category.clone();
    let tags = payload.tags.clone().unwrap_or_default();
    let is_published = payload.is_published.unwrap_or(true);

    let (title_c, body_c, category_c, tags_c) = (
        title.clone(),
        body.clone(),
        category.clone(),
        tags.clone(),
    );

    tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return;
            }
        };
        let _ = diesel::sql_query(
            "UPDATE ticket_kb_articles SET title = $1, body = $2, category = $3, tags = $4, is_published = $5, updated_at = NOW() WHERE id = $6",
        )
        .bind::<diesel::sql_types::Text, _>(&title_c)
        .bind::<diesel::sql_types::Text, _>(&body_c)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&category_c)
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(&tags_c)
        .bind::<diesel::sql_types::Bool, _>(&is_published)
        .bind::<diesel::sql_types::Uuid, _>(&id)
        .execute(&mut db_conn);
    })
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Task error: {e}")))?;

    Ok(Json(KbArticleRow {
        id,
        title,
        body,
        category,
        tags,
        is_published,
    }))
}

async fn delete_kb_article(State(state): State<Arc<TicketsState>>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.pool.clone();
    match tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return;
            }
        };
        let _ = diesel::sql_query("DELETE FROM ticket_kb_articles WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(&id)
            .execute(&mut db_conn);
    })
    .await
    {
        Ok(()) => (),
        Err(e) => log::error!("Delete KB article task error: {e}"),
    }

    axum::http::StatusCode::NO_CONTENT.into_response()
}

// UI fragments for the ITSM-style tabs in the tickets app.
async fn render_ci_list(state: &Arc<TicketsState>) -> String {
    let branch_id = resolve_branch_id(state);
    let conn = state.pool.clone();

    let rows = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            ci_type: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
        }
        diesel::sql_query(
            "SELECT id, name, ci_type, status FROM ticket_cis WHERE branch_id = $1 ORDER BY name ASC LIMIT 100",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load::<Row>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return "<div class=\"empty-state\"><p>No CIs configured yet</p></div>".to_string();
    }

    let mut html = String::from("<table class=\"tickets-table\"><thead><tr><th>Name</th><th>Type</th><th>Status</th></tr></thead><tbody>");
    for r in &rows {
        html.push_str(&format!(
            "<tr data-ci-id=\"{id}\"><td>{name}</td><td>{ci_type}</td><td>{status}</td></tr>",
            id = r.id,
            name = html_escape(&r.name),
            ci_type = html_escape(r.ci_type.as_deref().unwrap_or("-")),
            status = html_escape(&r.status),
        ));
    }
    html.push_str("</tbody></table>");
    html
}

async fn render_kb_list(state: &Arc<TicketsState>) -> String {
    let branch_id = resolve_branch_id(state);
    let conn = state.pool.clone();

    let rows = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            category: Option<String>,
        }
        diesel::sql_query(
            "SELECT id, title, category FROM ticket_kb_articles WHERE branch_id = $1 AND is_published = TRUE ORDER BY title ASC LIMIT 100",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load::<Row>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return "<div class=\"empty-state\"><p>No KB articles yet</p></div>".to_string();
    }

    let mut html = String::from("<div class=\"kb-articles-list\">");
    for r in &rows {
        html.push_str(&format!(
            "<div class=\"kb-article-item\" data-article-id=\"{id}\"><h4>{title}</h4><span class=\"badge\">{category}</span></div>",
            id = r.id,
            title = html_escape(&r.title),
            category = html_escape(r.category.as_deref().unwrap_or("General")),
        ));
    }
    html.push_str("</div>");
    html
}

pub fn configure_itsm_ui_routes() -> Router<Arc<TicketsState>> {
    Router::new()
        .route("/api/ui/tickets/cis", get(handle_ci_fragment))
        .route("/api/ui/tickets/kb", get(handle_kb_fragment))
}

async fn handle_ci_fragment(State(state): State<Arc<TicketsState>>) -> impl IntoResponse {
    Html(render_ci_list(&state).await)
}

async fn handle_kb_fragment(State(state): State<Arc<TicketsState>>) -> impl IntoResponse {
    Html(render_kb_list(&state).await)
}
