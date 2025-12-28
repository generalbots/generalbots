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
    _components: RwLock<HashMap<Uuid, WebAppComponent>>,
}

impl WebaState {
    pub fn new() -> Self {
        Self {
            apps: RwLock::new(HashMap::new()),
            pages: RwLock::new(HashMap::new()),
            _components: RwLock::new(HashMap::new()),
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
        result.retain(|app| {
            matches!(
                (&app.status, status.as_str()),
                (WebAppStatus::Draft, "draft")
                    | (WebAppStatus::Published, "published")
                    | (WebAppStatus::Archived, "archived")
            )
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
        app.name.clone_from(&name);
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
        .map(|(k, v)| format!("<meta name=\"{k}\" content=\"{v}\">"))
        .collect::<Vec<_>>()
        .join("\n    ");

    let scripts: String = app
        .config
        .scripts
        .iter()
        .map(|s| format!("<script src=\"{s}\"></script>"))
        .collect::<Vec<_>>()
        .join("\n    ");

    let styles: String = app
        .config
        .styles
        .iter()
        .map(|s| format!("<link rel=\"stylesheet\" href=\"{s}\">"))
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

pub fn slugify(s: &str) -> String {
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
    use std::time::Duration;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum BrowserType {
        #[default]
        Chrome,
        Firefox,
        Safari,
        Edge,
    }

    impl BrowserType {
        pub const fn browser_name(self) -> &'static str {
            match self {
                Self::Chrome => "chrome",
                Self::Firefox => "firefox",
                Self::Safari => "safari",
                Self::Edge => "MicrosoftEdge",
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct BrowserConfig {
        pub browser_type: BrowserType,
        pub debug_port: u16,
        pub headless: bool,
        pub window_width: u32,
        pub window_height: u32,
        pub timeout: Duration,
    }

    impl Default for BrowserConfig {
        fn default() -> Self {
            Self {
                browser_type: BrowserType::Chrome,
                debug_port: 9222,
                headless: true,
                window_width: 1920,
                window_height: 1080,
                timeout: Duration::from_secs(30),
            }
        }
    }

    impl BrowserConfig {
        pub fn new() -> Self {
            Self::default()
        }

        #[cfg(test)]
        pub const fn with_browser(mut self, browser: BrowserType) -> Self {
            self.browser_type = browser;
            self
        }

        pub const fn with_debug_port(mut self, port: u16) -> Self {
            self.debug_port = port;
            self
        }

        pub const fn headless(mut self, headless: bool) -> Self {
            self.headless = headless;
            self
        }

        pub const fn with_window_size(mut self, width: u32, height: u32) -> Self {
            self.window_width = width;
            self.window_height = height;
            self
        }

        pub const fn with_timeout(mut self, timeout: Duration) -> Self {
            self.timeout = timeout;
            self
        }
    }

    #[derive(Debug, Clone)]
    pub struct E2EConfig {
        browser: BrowserType,
        headless: bool,
        timeout: Duration,
        pub window_width: u32,
        pub window_height: u32,
        pub screenshot_on_failure: bool,
        screenshot_dir: String,
    }

    impl E2EConfig {
        pub fn browser(&self) -> BrowserType {
            self.browser
        }
        pub fn headless(&self) -> bool {
            self.headless
        }
        pub fn timeout(&self) -> Duration {
            self.timeout
        }
        pub fn screenshot_dir(&self) -> &str {
            &self.screenshot_dir
        }
    }

    impl Default for E2EConfig {
        fn default() -> Self {
            Self {
                browser: BrowserType::Chrome,
                headless: true,
                timeout: Duration::from_secs(30),
                window_width: 1920,
                window_height: 1080,
                screenshot_on_failure: true,
                screenshot_dir: "./test-screenshots".to_string(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum Locator {
        Css(String),
        XPath(String),
        Id(String),
    }

    impl Locator {
        pub fn css(selector: &str) -> Self {
            Self::Css(selector.to_string())
        }

        pub fn xpath(expr: &str) -> Self {
            Self::XPath(expr.to_string())
        }

        pub fn id(id: &str) -> Self {
            Self::Id(id.to_string())
        }

        pub fn as_selector(&self) -> &str {
            match self {
                Self::Css(s) | Self::XPath(s) | Self::Id(s) => s,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum Action {
        Click(Locator),
        SendKeys(String),
        Pause(Duration),
    }

    impl Action {
        pub fn description(&self) -> String {
            match self {
                Self::Click(loc) => format!("click on {}", loc.as_selector()),
                Self::SendKeys(text) => format!("send keys: {text}"),
                Self::Pause(dur) => format!("pause for {dur:?}"),
            }
        }
    }

    pub struct ActionChain {
        actions: Vec<Action>,
    }

    impl ActionChain {
        pub const fn new() -> Self {
            Self {
                actions: Vec::new(),
            }
        }

        pub fn click(mut self, locator: Locator) -> Self {
            self.actions.push(Action::Click(locator));
            self
        }

        pub fn send_keys(mut self, text: &str) -> Self {
            self.actions.push(Action::SendKeys(text.to_string()));
            self
        }

        pub fn pause(mut self, duration: Duration) -> Self {
            self.actions.push(Action::Pause(duration));
            self
        }

        pub fn actions(&self) -> &[Action] {
            &self.actions
        }
    }

    impl Default for ActionChain {
        fn default() -> Self {
            Self::new()
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Cookie {
        pub name: String,
        pub value: String,
        pub domain: Option<String>,
        pub path: Option<String>,
        pub secure: Option<bool>,
        pub http_only: Option<bool>,
    }

    impl Cookie {
        pub fn new(name: &str, value: &str) -> Self {
            Self {
                name: name.to_string(),
                value: value.to_string(),
                domain: None,
                path: None,
                secure: None,
                http_only: None,
            }
        }

        pub fn with_domain(mut self, domain: &str) -> Self {
            self.domain = Some(domain.to_string());
            self
        }

        pub fn with_path(mut self, path: &str) -> Self {
            self.path = Some(path.to_string());
            self
        }

        pub const fn secure(mut self) -> Self {
            self.secure = Some(true);
            self
        }

        pub const fn http_only(mut self) -> Self {
            self.http_only = Some(true);
            self
        }
    }

    pub struct LoginPage {
        base_url: String,
    }

    impl LoginPage {
        pub fn new(base_url: &str) -> Self {
            Self {
                base_url: base_url.to_string(),
            }
        }

        pub fn base_url(&self) -> &str {
            &self.base_url
        }

        pub fn url_pattern() -> &'static str {
            "/login"
        }

        pub fn email_input() -> Locator {
            Locator::id("email")
        }

        pub fn password_input() -> Locator {
            Locator::id("password")
        }

        pub fn login_button() -> Locator {
            Locator::css("button[type='submit']")
        }

        pub fn error_message() -> Locator {
            Locator::css(".error-message")
        }
    }

    pub struct DashboardPage {
        base_url: String,
    }

    impl DashboardPage {
        pub fn new(base_url: &str) -> Self {
            Self {
                base_url: base_url.to_string(),
            }
        }

        pub fn base_url(&self) -> &str {
            &self.base_url
        }

        pub fn url_pattern() -> &'static str {
            "/dashboard"
        }
    }

    pub struct ChatPage {
        base_url: String,
        bot_name: String,
    }

    impl ChatPage {
        pub fn new(base_url: &str, bot_name: &str) -> Self {
            Self {
                base_url: base_url.to_string(),
                bot_name: bot_name.to_string(),
            }
        }

        pub fn base_url(&self) -> &str {
            &self.base_url
        }

        pub fn bot_name(&self) -> &str {
            &self.bot_name
        }

        pub fn url_pattern() -> &'static str {
            "/chat/"
        }

        pub fn chat_input() -> Locator {
            Locator::id("chat-input")
        }

        pub fn send_button() -> Locator {
            Locator::css("button.send-message")
        }

        pub fn bot_message() -> Locator {
            Locator::css(".message.bot")
        }

        pub fn typing_indicator() -> Locator {
            Locator::css(".typing-indicator")
        }
    }

    pub struct QueuePage {
        base_url: String,
    }

    impl QueuePage {
        pub fn new(base_url: &str) -> Self {
            Self {
                base_url: base_url.to_string(),
            }
        }

        pub fn base_url(&self) -> &str {
            &self.base_url
        }

        pub fn url_pattern() -> &'static str {
            "/queue"
        }

        pub fn queue_panel() -> Locator {
            Locator::css(".queue-panel")
        }

        pub fn queue_count() -> Locator {
            Locator::css(".queue-count")
        }

        pub fn take_next_button() -> Locator {
            Locator::css("button.take-next")
        }
    }

    pub struct BotManagementPage {
        base_url: String,
    }

    impl BotManagementPage {
        pub fn new(base_url: &str) -> Self {
            Self {
                base_url: base_url.to_string(),
            }
        }

        pub fn base_url(&self) -> &str {
            &self.base_url
        }

        pub fn url_pattern() -> &'static str {
            "/admin/bots"
        }
    }

    #[test]
    fn test_e2e_config_default() {
        let config = E2EConfig::default();
        assert_eq!(config.window_width, 1920);
        assert_eq!(config.window_height, 1080);
        assert!(config.screenshot_on_failure);
        assert_eq!(config.browser(), BrowserType::Chrome);
        assert!(config.headless());
        assert_eq!(config.timeout(), Duration::from_secs(30));
        assert_eq!(config.screenshot_dir(), "./test-screenshots");
    }

    #[test]
    fn test_browser_config_default() {
        let config = BrowserConfig::default();
        assert_eq!(config.browser_type, BrowserType::Chrome);
        assert_eq!(config.debug_port, 9222);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_browser_config_builder() {
        let config = BrowserConfig::new()
            .with_browser(BrowserType::Firefox)
            .with_debug_port(9333)
            .headless(false)
            .with_window_size(1280, 720)
            .with_timeout(Duration::from_secs(60));

        assert_eq!(config.browser_type, BrowserType::Firefox);
        assert_eq!(config.debug_port, 9333);
        assert!(!config.headless);
        assert_eq!(config.window_width, 1280);
        assert_eq!(config.window_height, 720);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_browser_type_browser_name() {
        assert_eq!(BrowserType::Chrome.browser_name(), "chrome");
        assert_eq!(BrowserType::Firefox.browser_name(), "firefox");
        assert_eq!(BrowserType::Safari.browser_name(), "safari");
        assert_eq!(BrowserType::Edge.browser_name(), "MicrosoftEdge");
    }

    #[test]
    fn test_locator_constructors() {
        let css = Locator::css(".my-class");
        assert!(matches!(css, Locator::Css(_)));
        assert_eq!(css.as_selector(), ".my-class");

        let xpath = Locator::xpath("//div[@id='test']");
        assert!(matches!(xpath, Locator::XPath(_)));
        assert_eq!(xpath.as_selector(), "//div[@id='test']");

        let id = Locator::id("my-id");
        assert!(matches!(id, Locator::Id(_)));
        assert_eq!(id.as_selector(), "my-id");
    }

    #[test]
    fn test_action_chain() {
        let chain = ActionChain::new()
            .click(Locator::id("button"))
            .send_keys("Hello")
            .pause(Duration::from_millis(500));

        assert_eq!(chain.actions().len(), 3);
        for action in chain.actions() {
            let _ = action.description();
        }
    }

    #[test]
    fn test_cookie_builder() {
        let cookie = Cookie::new("session", "abc123")
            .with_domain("example.com")
            .with_path("/")
            .secure()
            .http_only();

        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert!(cookie.secure.unwrap());
        assert!(cookie.http_only.unwrap());
    }

    #[test]
    fn test_login_page_locators() {
        let _ = LoginPage::email_input();
        let _ = LoginPage::password_input();
        let _ = LoginPage::login_button();
        let _ = LoginPage::error_message();
    }

    #[test]
    fn test_chat_page_locators() {
        let _ = ChatPage::chat_input();
        let _ = ChatPage::send_button();
        let _ = ChatPage::bot_message();
        let _ = ChatPage::typing_indicator();
    }

    #[test]
    fn test_queue_page_locators() {
        let _ = QueuePage::queue_panel();
        let _ = QueuePage::queue_count();
        let _ = QueuePage::take_next_button();
    }

    #[test]
    fn test_page_url_patterns() {
        let login = LoginPage::new("http://localhost:4242");
        assert_eq!(LoginPage::url_pattern(), "/login");
        assert_eq!(login.base_url(), "http://localhost:4242");

        let dashboard = DashboardPage::new("http://localhost:4242");
        assert_eq!(DashboardPage::url_pattern(), "/dashboard");
        assert_eq!(dashboard.base_url(), "http://localhost:4242");

        let chat = ChatPage::new("http://localhost:4242", "test-bot");
        assert_eq!(ChatPage::url_pattern(), "/chat/");
        assert_eq!(chat.base_url(), "http://localhost:4242");
        assert_eq!(chat.bot_name(), "test-bot");

        let queue = QueuePage::new("http://localhost:4242");
        assert_eq!(QueuePage::url_pattern(), "/queue");
        assert_eq!(queue.base_url(), "http://localhost:4242");

        let bots = BotManagementPage::new("http://localhost:4242");
        assert_eq!(BotManagementPage::url_pattern(), "/admin/bots");
        assert_eq!(bots.base_url(), "http://localhost:4242");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("My  App  Name"), "my-app-name");
        assert_eq!(slugify("Test123"), "test123");
        assert_eq!(slugify("  Spaces  "), "spaces");
    }

    #[test]
    fn test_web_app_template_default() {
        let template = WebAppTemplate::default();
        assert!(matches!(template, WebAppTemplate::Blank));
    }

    #[test]
    fn test_web_app_status_default() {
        let status = WebAppStatus::default();
        assert!(matches!(status, WebAppStatus::Draft));
    }

    #[test]
    fn test_web_app_config_default() {
        let config = WebAppConfig::default();
        assert!(!config.auth_required);
        assert!(config.custom_domain.is_none());
        assert!(config.meta_tags.is_empty());
        assert!(config.scripts.is_empty());
        assert!(config.styles.is_empty());
    }

    #[test]
    fn test_component_types() {
        let container = ComponentType::Container;
        let text = ComponentType::Text;
        let button = ComponentType::Button;
        let custom = ComponentType::Custom("MyWidget".to_string());

        assert!(matches!(container, ComponentType::Container));
        assert!(matches!(text, ComponentType::Text));
        assert!(matches!(button, ComponentType::Button));
        assert!(matches!(custom, ComponentType::Custom(_)));
    }

    #[test]
    fn test_create_app_request() {
        let request = CreateAppRequest {
            name: "My Test App".to_string(),
            description: Some("A test application".to_string()),
            template: Some(WebAppTemplate::Dashboard),
        };

        assert_eq!(request.name, "My Test App");
        assert!(request.description.is_some());
        assert!(matches!(request.template, Some(WebAppTemplate::Dashboard)));
    }

    #[test]
    fn test_create_page_request() {
        let request = CreatePageRequest {
            path: "/about".to_string(),
            title: "About Us".to_string(),
            content: "<h1>About</h1>".to_string(),
            layout: Some("default".to_string()),
            is_index: false,
        };

        assert_eq!(request.path, "/about");
        assert!(!request.is_index);
    }

    #[test]
    fn test_render_html_basic() {
        let app = WebApp {
            id: Uuid::new_v4(),
            name: "Test App".to_string(),
            slug: "test-app".to_string(),
            description: None,
            template: WebAppTemplate::Blank,
            status: WebAppStatus::Published,
            config: WebAppConfig::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let html = render_html(&app, "<p>Hello</p>");
        assert!(html.contains("Test App"));
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_render_html_with_meta_tags() {
        let mut config = WebAppConfig::default();
        config
            .meta_tags
            .insert("description".to_string(), "A test page".to_string());
        config
            .meta_tags
            .insert("author".to_string(), "Test Author".to_string());

        let app = WebApp {
            id: Uuid::new_v4(),
            name: "Meta Test".to_string(),
            slug: "meta-test".to_string(),
            description: None,
            template: WebAppTemplate::Blank,
            status: WebAppStatus::Published,
            config,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let html = render_html(&app, "");
        assert!(html.contains("meta name="));
    }

    #[test]
    fn test_weba_state_creation() {
        let state = WebaState::new();
        let _ = &state;
    }

    #[test]
    fn test_list_query_defaults() {
        let query = ListQuery {
            limit: None,
            offset: None,
            status: None,
        };

        assert!(query.limit.is_none());
        assert!(query.offset.is_none());
        assert!(query.status.is_none());
    }

    #[test]
    fn test_list_query_with_values() {
        let query = ListQuery {
            limit: Some(10),
            offset: Some(20),
            status: Some("published".to_string()),
        };

        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(20));
        assert_eq!(query.status, Some("published".to_string()));
    }
}
