use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebApp {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub template: WebAppTemplate,
    pub status: WebAppStatus,
    pub config: WebAppConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum WebAppTemplate {
    #[default]
    Blank,
    Landing,
    Dashboard,
    Form,
    Portal,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum WebAppStatus {
    #[default]
    Draft,
    Published,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebAppConfig {
    pub theme: String,
    pub layout: String,
    pub auth_required: bool,
    pub custom_domain: Option<String>,
    pub meta_tags: HashMap<String, String>,
    pub scripts: Vec<String>,
    pub styles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAppPage {
    pub id: Uuid,
    pub app_id: Uuid,
    pub path: String,
    pub title: String,
    pub content: String,
    pub layout: Option<String>,
    pub is_index: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAppComponent {
    pub id: Uuid,
    pub app_id: Uuid,
    pub name: String,
    pub component_type: ComponentType,
    pub props: serde_json::Value,
    pub children: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Container,
    Text,
    Image,
    Button,
    Form,
    Input,
    Table,
    Chart,
    Custom(String),
}

pub struct WebaState {
    apps: RwLock<HashMap<Uuid, WebApp>>,
    pages: RwLock<HashMap<Uuid, WebAppPage>>,
    components: RwLock<HashMap<Uuid, WebAppComponent>>,
}

impl WebaState {
    pub fn new() -> Self {
        Self {
            apps: RwLock::new(HashMap::new()),
            pages: RwLock::new(HashMap::new()),
            components: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for WebaState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAppRequest {
    pub name: String,
    pub description: Option<String>,
    pub template: Option<WebAppTemplate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAppRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<WebAppStatus>,
    pub config: Option<WebAppConfig>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePageRequest {
    pub path: String,
    pub title: String,
    pub content: String,
    pub layout: Option<String>,
    pub is_index: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub status: Option<String>,
}

pub fn configure_routes(state: Arc<WebaState>) -> Router {
    Router::new()
        .route("/apps", get(list_apps).post(create_app))
        .route("/apps/:id", get(get_app).put(update_app).delete(delete_app))
        .route("/apps/:id/pages", get(list_pages).post(create_page))
        .route(
            "/apps/:id/pages/:page_id",
            get(get_page).put(update_page).delete(delete_page),
        )
        .route("/apps/:id/publish", post(publish_app))
        .route("/apps/:id/preview", get(preview_app))
        .route("/render/:slug", get(render_app))
        .route("/render/:slug/*path", get(render_page))
        .with_state(state)
}

async fn list_apps(
    State(state): State<Arc<WebaState>>,
    Query(query): Query<ListQuery>,
) -> Json<Vec<WebApp>> {
    let apps = state.apps.read().await;
    let mut result: Vec<WebApp> = apps.values().cloned().collect();

    if let Some(status) = query.status {
        result.retain(|app| match (&app.status, status.as_str()) {
            (WebAppStatus::Draft, "draft") => true,
            (WebAppStatus::Published, "published") => true,
            (WebAppStatus::Archived, "archived") => true,
            _ => false,
        });
    }

    result.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    let result: Vec<WebApp> = result.into_iter().skip(offset).take(limit).collect();

    Json(result)
}

async fn create_app(
    State(state): State<Arc<WebaState>>,
    Json(req): Json<CreateAppRequest>,
) -> Json<WebApp> {
    let now = chrono::Utc::now();
    let id = Uuid::new_v4();
    let slug = slugify(&req.name);

    let app = WebApp {
        id,
        name: req.name,
        slug,
        description: req.description,
        template: req.template.unwrap_or_default(),
        status: WebAppStatus::Draft,
        config: WebAppConfig::default(),
        created_at: now,
        updated_at: now,
    };

    let mut apps = state.apps.write().await;
    apps.insert(id, app.clone());

    Json(app)
}

async fn get_app(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<WebApp>, axum::http::StatusCode> {
    let apps = state.apps.read().await;
    apps.get(&id)
        .cloned()
        .map(Json)
        .ok_or(axum::http::StatusCode::NOT_FOUND)
}

async fn update_app(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAppRequest>,
) -> Result<Json<WebApp>, axum::http::StatusCode> {
    let mut apps = state.apps.write().await;

    let app = apps.get_mut(&id).ok_or(axum::http::StatusCode::NOT_FOUND)?;

    if let Some(name) = req.name {
        app.name = name.clone();
        app.slug = slugify(&name);
    }
    if let Some(description) = req.description {
        app.description = Some(description);
    }
    if let Some(status) = req.status {
        app.status = status;
    }
    if let Some(config) = req.config {
        app.config = config;
    }
    app.updated_at = chrono::Utc::now();

    Ok(Json(app.clone()))
}

async fn delete_app(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
) -> axum::http::StatusCode {
    let mut apps = state.apps.write().await;
    let mut pages = state.pages.write().await;

    pages.retain(|_, page| page.app_id != id);

    if apps.remove(&id).is_some() {
        axum::http::StatusCode::NO_CONTENT
    } else {
        axum::http::StatusCode::NOT_FOUND
    }
}

async fn list_pages(
    State(state): State<Arc<WebaState>>,
    Path(app_id): Path<Uuid>,
) -> Json<Vec<WebAppPage>> {
    let pages = state.pages.read().await;
    let result: Vec<WebAppPage> = pages
        .values()
        .filter(|p| p.app_id == app_id)
        .cloned()
        .collect();
    Json(result)
}

async fn create_page(
    State(state): State<Arc<WebaState>>,
    Path(app_id): Path<Uuid>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<WebAppPage>, axum::http::StatusCode> {
    let apps = state.apps.read().await;
    if !apps.contains_key(&app_id) {
        return Err(axum::http::StatusCode::NOT_FOUND);
    }
    drop(apps);

    let now = chrono::Utc::now();
    let id = Uuid::new_v4();

    let page = WebAppPage {
        id,
        app_id,
        path: req.path,
        title: req.title,
        content: req.content,
        layout: req.layout,
        is_index: req.is_index,
        created_at: now,
        updated_at: now,
    };

    let mut pages = state.pages.write().await;
    pages.insert(id, page.clone());

    Ok(Json(page))
}

async fn get_page(
    State(state): State<Arc<WebaState>>,
    Path((app_id, page_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<WebAppPage>, axum::http::StatusCode> {
    let pages = state.pages.read().await;
    pages
        .get(&page_id)
        .filter(|p| p.app_id == app_id)
        .cloned()
        .map(Json)
        .ok_or(axum::http::StatusCode::NOT_FOUND)
}

async fn update_page(
    State(state): State<Arc<WebaState>>,
    Path((app_id, page_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<WebAppPage>, axum::http::StatusCode> {
    let mut pages = state.pages.write().await;

    let page = pages
        .get_mut(&page_id)
        .filter(|p| p.app_id == app_id)
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    page.path = req.path;
    page.title = req.title;
    page.content = req.content;
    page.layout = req.layout;
    page.is_index = req.is_index;
    page.updated_at = chrono::Utc::now();

    Ok(Json(page.clone()))
}

async fn delete_page(
    State(state): State<Arc<WebaState>>,
    Path((app_id, page_id)): Path<(Uuid, Uuid)>,
) -> axum::http::StatusCode {
    let mut pages = state.pages.write().await;

    let exists = pages
        .get(&page_id)
        .map(|p| p.app_id == app_id)
        .unwrap_or(false);

    if exists {
        pages.remove(&page_id);
        axum::http::StatusCode::NO_CONTENT
    } else {
        axum::http::StatusCode::NOT_FOUND
    }
}

async fn publish_app(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<WebApp>, axum::http::StatusCode> {
    let mut apps = state.apps.write().await;
    let app = apps.get_mut(&id).ok_or(axum::http::StatusCode::NOT_FOUND)?;

    app.status = WebAppStatus::Published;
    app.updated_at = chrono::Utc::now();

    Ok(Json(app.clone()))
}

async fn preview_app(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
) -> Result<Html<String>, axum::http::StatusCode> {
    let apps = state.apps.read().await;
    let app = apps.get(&id).ok_or(axum::http::StatusCode::NOT_FOUND)?;

    let pages = state.pages.read().await;
    let index_page = pages.values().find(|p| p.app_id == id && p.is_index);

    let content = index_page
        .map(|p| p.content.clone())
        .unwrap_or_else(|| "<p>No content yet</p>".to_string());

    let html = render_html(app, &content);
    Ok(Html(html))
}

async fn render_app(
    State(state): State<Arc<WebaState>>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let apps = state.apps.read().await;
    let app = apps
        .values()
        .find(|a| a.slug == slug && matches!(a.status, WebAppStatus::Published))
        .ok_or(axum::http::StatusCode::NOT_FOUND)?
        .clone();
    drop(apps);

    let pages = state.pages.read().await;
    let index_page = pages.values().find(|p| p.app_id == app.id && p.is_index);

    let content = index_page
        .map(|p| p.content.clone())
        .unwrap_or_else(|| "<p>Page not found</p>".to_string());

    let html = render_html(&app, &content);
    Ok(Html(html))
}

async fn render_page(
    State(state): State<Arc<WebaState>>,
    Path((slug, path)): Path<(String, String)>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let apps = state.apps.read().await;
    let app = apps
        .values()
        .find(|a| a.slug == slug && matches!(a.status, WebAppStatus::Published))
        .ok_or(axum::http::StatusCode::NOT_FOUND)?
        .clone();
    drop(apps);

    let normalized_path = format!("/{}", path.trim_start_matches('/'));

    let pages = state.pages.read().await;
    let page = pages
        .values()
        .find(|p| p.app_id == app.id && p.path == normalized_path);

    let content = page
        .map(|p| p.content.clone())
        .unwrap_or_else(|| "<p>Page not found</p>".to_string());

    let html = render_html(&app, &content);
    Ok(Html(html))
}

fn render_html(app: &WebApp, content: &str) -> String {
    let meta_tags: String = app
        .config
        .meta_tags
        .iter()
        .map(|(k, v)| format!("<meta name=\"{}\" content=\"{}\">", k, v))
        .collect::<Vec<_>>()
        .join("\n    ");

    let scripts: String = app
        .config
        .scripts
        .iter()
        .map(|s| format!("<script src=\"{}\"></script>", s))
        .collect::<Vec<_>>()
        .join("\n    ");

    let styles: String = app
        .config
        .styles
        .iter()
        .map(|s| format!("<link rel=\"stylesheet\" href=\"{}\">", s))
        .collect::<Vec<_>>()
        .join("\n    ");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    {}
    {}
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; }}
    </style>
</head>
<body>
    {}
    {}
</body>
</html>"#,
        app.name, meta_tags, styles, content, scripts
    )
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn init() {
    log::info!("WEBA module initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("My App 123"), "my-app-123");
        assert_eq!(slugify("  Test  App  "), "test-app");
    }

    #[test]
    fn test_webapp_creation() {
        let now = chrono::Utc::now();
        let app = WebApp {
            id: Uuid::new_v4(),
            name: "Test App".to_string(),
            slug: "test-app".to_string(),
            description: None,
            template: WebAppTemplate::Blank,
            status: WebAppStatus::Draft,
            config: WebAppConfig::default(),
            created_at: now,
            updated_at: now,
        };
        assert_eq!(app.name, "Test App");
        assert_eq!(app.slug, "test-app");
    }

    #[tokio::test]
    async fn test_weba_state() {
        let state = WebaState::new();
        let apps = state.apps.read().await;
        assert!(apps.is_empty());
    }
}
