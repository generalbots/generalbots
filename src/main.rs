#![recursion_limit = "512"]

// Use jemalloc as the global allocator when the feature is enabled
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// Module declarations
#[cfg(feature = "analytics")]
pub mod analytics;
#[cfg(feature = "attendant")]
pub mod attendant;
#[cfg(feature = "automation")]
pub mod auto_task;
#[cfg(feature = "scripting")]
pub mod basic;
#[cfg(feature = "billing")]
pub mod billing;
pub mod botmodels;
#[cfg(feature = "canvas")]
pub mod canvas;
pub mod channels;
#[cfg(feature = "people")]
pub mod contacts;
pub mod core;
#[cfg(feature = "designer")]
pub mod designer;
#[cfg(feature = "docs")]
pub mod docs;
pub mod embedded_ui;
#[cfg(feature = "learn")]
pub mod learn;
#[cfg(feature = "compliance")]
pub mod legal;
pub mod maintenance;
#[cfg(feature = "monitoring")]
pub mod monitoring;
pub mod multimodal;
#[cfg(feature = "paper")]
pub mod paper;
#[cfg(feature = "people")]
pub mod people;
#[cfg(feature = "player")]
pub mod player;
#[cfg(feature = "billing")]
pub mod products;
#[cfg(feature = "project")]
pub mod project;
#[cfg(feature = "research")]
pub mod research;
pub mod search;
pub mod security;
pub mod settings;
#[cfg(feature = "dashboards")]
pub mod shared;
#[cfg(feature = "sheet")]
pub mod sheet;
#[cfg(feature = "slides")]
pub mod slides;
#[cfg(feature = "social")]
pub mod social;
#[cfg(feature = "sources")]
pub mod sources;
#[cfg(feature = "tickets")]
pub mod tickets;
#[cfg(feature = "video")]
pub mod video;
#[cfg(feature = "workspaces")]
pub mod workspaces;

#[cfg(feature = "attendant")]
pub mod attendance;

#[cfg(feature = "calendar")]
pub mod calendar;

#[cfg(feature = "compliance")]
pub mod compliance;

#[cfg(feature = "console")]
pub mod console;

#[cfg(feature = "directory")]
pub mod directory;

#[cfg(feature = "drive")]
pub mod drive;

#[cfg(feature = "mail")]
pub mod email;

#[cfg(feature = "instagram")]
pub mod instagram;

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "meet")]
pub mod meet;

#[cfg(feature = "msteams")]
pub mod msteams;

#[cfg(feature = "nvidia")]
pub mod nvidia;

#[cfg(feature = "tasks")]
pub mod tasks;

#[cfg(feature = "vectordb")]
#[path = "vector-db/mod.rs"]
pub mod vector_db;

#[cfg(feature = "weba")]
pub mod weba;

#[cfg(feature = "whatsapp")]
pub mod whatsapp;

#[cfg(feature = "telegram")]
pub mod telegram;

pub use core::shared;

#[derive(Debug, Clone)]
pub enum BootstrapProgress {
    StartingBootstrap,
    InstallingComponent(String),
    StartingComponent(String),
    UploadingTemplates,
    ConnectingDatabase,
    StartingLLM,
    BootstrapComplete,
    BootstrapError(String),
}

#[cfg(feature = "drive")]
pub use drive::drive_monitor::DriveMonitor;

#[cfg(feature = "llm")]
pub use llm::cache::{CacheConfig, CachedLLMProvider, CachedResponse, LocalEmbeddingService};
#[cfg(feature = "llm")]
pub use llm::DynamicLLMProvider;

#[cfg(feature = "tasks")]
pub use tasks::TaskEngine;

use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::Json;
use axum::{
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use log::{error, info, trace, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[cfg(feature = "drive")]
async fn ensure_vendor_files_in_minio(drive: &aws_sdk_s3::Client) {
    use aws_sdk_s3::primitives::ByteStream;

    let htmx_paths = [
        "./botui/ui/suite/js/vendor/htmx.min.js",
        "../botui/ui/suite/js/vendor/htmx.min.js",
    ];

    let htmx_content = htmx_paths.iter().find_map(|path| std::fs::read(path).ok());

    let Some(content) = htmx_content else {
        warn!("Could not find htmx.min.js in botui, skipping MinIO upload");
        return;
    };

    let bucket = "default.gbai";
    let key = "default.gblib/vendor/htmx.min.js";

    match drive
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(content))
        .content_type("application/javascript")
        .send()
        .await
    {
        Ok(_) => info!("Uploaded vendor file to MinIO: s3://{}/{}", bucket, key),
        Err(e) => warn!("Failed to upload vendor file to MinIO: {}", e),
    }
}

use crate::security::{
    build_default_route_permissions, create_cors_layer, create_rate_limit_layer,
    create_security_headers_layer, request_id_middleware, security_headers_middleware,
    set_cors_allowed_origins, set_global_panic_hook, ApiKeyAuthProvider, AuthConfig,
    AuthMiddlewareState, AuthProviderBuilder, HttpRateLimitConfig, JwtConfig, JwtKey, JwtManager,
    PanicHandlerConfig, RbacConfig, RbacManager, SecurityHeadersConfig,
};
use botlib::SystemLimits;

use crate::core::shared::memory_monitor::{
    log_process_memory, record_thread_activity, register_thread, start_memory_monitor, MemoryStats,
};

#[cfg(feature = "automation")]
use crate::core::automation;
use crate::core::bootstrap;
use crate::core::bot;
use crate::core::package_manager;
use crate::core::session;

use crate::core::bot::channels::{VoiceAdapter, WebChannelAdapter};
use crate::core::bot::websocket_handler;
use crate::core::bot::BotOrchestrator;
use crate::core::bot_database::BotDatabaseManager;
use crate::core::config::AppConfig;
#[cfg(feature = "automation")]
use automation::AutomationService;
use bootstrap::BootstrapManager;

use crate::shared::state::AppState;
use crate::shared::utils::create_conn;
#[cfg(feature = "drive")]
use crate::shared::utils::create_s3_operator;
use package_manager::InstallMode;
use session::{create_session, get_session_history, get_sessions, start_session};

async fn health_check(State(state): State<Arc<AppState>>) -> (StatusCode, Json<serde_json::Value>) {
    let db_ok = state.conn.get().is_ok();

    let status = if db_ok { "healthy" } else { "degraded" };
    let code = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        code,
        Json(serde_json::json!({
            "status": status,
            "service": "botserver",
            "version": env!("CARGO_PKG_VERSION"),
            "database": db_ok
        })),
    )
}

async fn health_check_simple() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "service": "botserver",
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}

#[derive(serde::Deserialize)]
struct ClientErrorsRequest {
    errors: Vec<ClientErrorData>,
}

#[derive(serde::Deserialize)]
struct ClientErrorData {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    stack: Option<String>,
    #[serde(default)]
    url: String,
    #[serde(default)]
    timestamp: String,
}

async fn receive_client_errors(
    Json(payload): Json<ClientErrorsRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    for error in &payload.errors {
        log::error!(
            "[CLIENT ERROR] {} | {} | {} | URL: {} | Stack: {}",
            error.timestamp,
            error.r#type,
            error.message,
            error.url,
            error.stack.as_deref().unwrap_or("<no stack>")
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "received",
            "count": payload.errors.len()
        })),
    )
}

fn print_shutdown_message() {
    println!();
    println!("Thank you for using General Bots!");
    println!();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Failed to install Ctrl+C handler: {}", e);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => {
                error!("Failed to install SIGTERM handler: {}", e);
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }

    print_shutdown_message();
}

async fn run_axum_server(
    app_state: Arc<AppState>,
    port: u16,
    _worker_count: usize,
) -> std::io::Result<()> {
    // Load CORS allowed origins from bot config database if available
    // Config key: cors-allowed-origins in config.csv
    if let Ok(mut conn) = app_state.conn.get() {
        use crate::shared::models::schema::bot_configuration::dsl::*;
        use diesel::prelude::*;

        if let Ok(origins_str) = bot_configuration
            .filter(config_key.eq("cors-allowed-origins"))
            .select(config_value)
            .first::<String>(&mut conn)
        {
            let origins: Vec<String> = origins_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !origins.is_empty() {
                info!("Loaded {} CORS allowed origins from config", origins.len());
                set_cors_allowed_origins(origins);
            }
        }
    }

    let cors = create_cors_layer();

    let auth_config = Arc::new(
        AuthConfig::from_env()
            .add_anonymous_path("/health")
            .add_anonymous_path("/healthz")
            .add_anonymous_path("/api/health")
            .add_anonymous_path("/api/product")
            .add_anonymous_path("/api/manifest")
            .add_anonymous_path("/api/i18n")
            .add_anonymous_path("/api/auth")
            .add_anonymous_path("/api/auth/login")
            .add_anonymous_path("/api/auth/refresh")
            .add_anonymous_path("/api/auth/bootstrap")
            .add_anonymous_path("/api/bot/config")
            .add_anonymous_path("/api/client-errors")
            .add_anonymous_path("/ws")
            .add_anonymous_path("/auth")
            .add_public_path("/static")
            .add_public_path("/favicon.ico")
            .add_public_path("/suite")
            .add_public_path("/themes"),
    );

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        warn!("JWT_SECRET not set, using default development secret - DO NOT USE IN PRODUCTION");
        "dev-secret-key-change-in-production-minimum-32-chars".to_string()
    });

    let jwt_config = JwtConfig::default();
    let jwt_key = JwtKey::from_secret(&jwt_secret);
    let jwt_manager = match JwtManager::new(jwt_config, jwt_key) {
        Ok(manager) => {
            info!("JWT Manager initialized successfully");
            Some(Arc::new(manager))
        }
        Err(e) => {
            error!("Failed to initialize JWT Manager: {e}");
            None
        }
    };

    let rbac_config = RbacConfig::default();
    let rbac_manager = Arc::new(RbacManager::new(rbac_config));

    let default_permissions = build_default_route_permissions();
    rbac_manager.register_routes(default_permissions).await;
    info!(
        "RBAC Manager initialized with {} default route permissions",
        rbac_manager.config().cache_ttl_seconds
    );

    let auth_provider_registry = {
        let mut builder = AuthProviderBuilder::new()
            .with_api_key_provider(Arc::new(ApiKeyAuthProvider::new()))
            .with_auth_config(Arc::clone(&auth_config));

        if let Some(ref manager) = jwt_manager {
            builder = builder.with_jwt_manager(Arc::clone(manager));
        }

        let zitadel_configured = std::env::var("ZITADEL_ISSUER_URL").is_ok()
            && std::env::var("ZITADEL_CLIENT_ID").is_ok();

        if zitadel_configured {
            info!("Zitadel environment variables detected - external IdP authentication available");
        }

        Arc::new(builder.build().await)
    };

    info!(
        "Auth provider registry initialized with {} providers",
        auth_provider_registry.provider_count().await
    );

    let auth_middleware_state = AuthMiddlewareState::new(
        Arc::clone(&auth_config),
        Arc::clone(&auth_provider_registry),
    );

    use crate::core::product::{get_product_config_json, PRODUCT_CONFIG};
    use crate::core::urls::ApiUrls;

    {
        let config = PRODUCT_CONFIG
            .read()
            .expect("Failed to read product config");
        info!(
            "Product: {} | Theme: {} | Apps: {:?}",
            config.name,
            config.theme,
            config.get_enabled_apps()
        );
    }

    async fn get_product_config() -> Json<serde_json::Value> {
        Json(get_product_config_json())
    }

    async fn get_workspace_manifest() -> Json<serde_json::Value> {
        use crate::core::product::get_workspace_manifest;
        Json(get_workspace_manifest())
    }

    let mut api_router = Router::new()
        .route("/health", get(health_check_simple))
        .route(ApiUrls::HEALTH, get(health_check))
        .route("/api/config/reload", post(crate::core::config_reload::reload_config))
        .route("/api/product", get(get_product_config))
        .route("/api/manifest", get(get_workspace_manifest))
        .route("/api/client-errors", post(receive_client_errors))
        .route("/api/bot/config", get(crate::core::bot::get_bot_config))
        .route(ApiUrls::SESSIONS, post(create_session))
        .route(ApiUrls::SESSIONS, get(get_sessions))
        .route(ApiUrls::SESSION_HISTORY, get(get_session_history))
        .route(ApiUrls::SESSION_START, post(start_session))
        .route(ApiUrls::WS, get(websocket_handler));

    #[cfg(feature = "drive")]
    {
        api_router = api_router.merge(crate::drive::configure());
    }

    #[cfg(feature = "directory")]
    {
        api_router = api_router
            .merge(crate::core::directory::api::configure_user_routes())
            .merge(crate::directory::router::configure())
            .nest(ApiUrls::AUTH, crate::directory::auth_routes::configure());
    }

    #[cfg(feature = "meet")]
    {
        api_router = api_router.merge(crate::meet::configure());
    }

    #[cfg(feature = "mail")]
    {
        api_router = api_router.merge(crate::email::configure());
    }

    #[cfg(all(feature = "calendar", feature = "scripting"))]
    {
        let calendar_engine = Arc::new(crate::basic::keywords::book::CalendarEngine::new(
            app_state.conn.clone(),
        ));

        api_router = api_router.merge(crate::calendar::caldav::create_caldav_router(
            calendar_engine,
        ));
    }

    #[cfg(feature = "tasks")]
    {
        api_router = api_router.merge(crate::tasks::configure_task_routes());
    }

    #[cfg(feature = "calendar")]
    {
        api_router = api_router.merge(crate::calendar::configure_calendar_routes());
        api_router = api_router.merge(crate::calendar::ui::configure_calendar_ui_routes());
    }

    #[cfg(feature = "analytics")]
    {
        api_router = api_router.merge(crate::analytics::configure_analytics_routes());
    }
    api_router = api_router.merge(crate::core::i18n::configure_i18n_routes());
    #[cfg(feature = "docs")]
    {
        api_router = api_router.merge(crate::docs::configure_docs_routes());
    }
    #[cfg(feature = "paper")]
    {
        api_router = api_router.merge(crate::paper::configure_paper_routes());
    }
    #[cfg(feature = "sheet")]
    {
        api_router = api_router.merge(crate::sheet::configure_sheet_routes());
    }
    #[cfg(feature = "slides")]
    {
        api_router = api_router.merge(crate::slides::configure_slides_routes());
    }
    #[cfg(feature = "video")]
    {
        api_router = api_router.merge(crate::video::configure_video_routes());
        api_router = api_router.merge(crate::video::ui::configure_video_ui_routes());
    }
    #[cfg(feature = "research")]
    {
        api_router = api_router.merge(crate::research::configure_research_routes());
        api_router = api_router.merge(crate::research::ui::configure_research_ui_routes());
    }
    #[cfg(feature = "sources")]
    {
        api_router = api_router.merge(crate::sources::configure_sources_routes());
        api_router = api_router.merge(crate::sources::ui::configure_sources_ui_routes());
    }
    #[cfg(feature = "designer")]
    {
        api_router = api_router.merge(crate::designer::configure_designer_routes());
        api_router = api_router.merge(crate::designer::ui::configure_designer_ui_routes());
    }
    #[cfg(feature = "dashboards")]
    {
        api_router = api_router.merge(crate::dashboards::configure_dashboards_routes());
        api_router = api_router.merge(crate::dashboards::ui::configure_dashboards_ui_routes());
    }
    #[cfg(feature = "compliance")]
    {
        api_router = api_router.merge(crate::legal::configure_legal_routes());
        api_router = api_router.merge(crate::legal::ui::configure_legal_ui_routes());
    }
    #[cfg(feature = "compliance")]
    {
        api_router = api_router.merge(crate::compliance::configure_compliance_routes());
        api_router = api_router.merge(crate::compliance::ui::configure_compliance_ui_routes());
    }
    #[cfg(feature = "monitoring")]
    {
        api_router = api_router.merge(crate::monitoring::configure());
    }
    api_router = api_router.merge(crate::security::configure_protection_routes());
    api_router = api_router.merge(crate::settings::configure_settings_routes());
    #[cfg(feature = "scripting")]
    {
        api_router = api_router.merge(crate::basic::keywords::configure_db_routes());
        api_router = api_router.merge(crate::basic::keywords::configure_app_server_routes());
    }
    #[cfg(feature = "automation")]
    {
        api_router = api_router.merge(crate::auto_task::configure_autotask_routes());
    }
    api_router = api_router.merge(crate::core::shared::admin::configure());
    #[cfg(feature = "workspaces")]
    {
        api_router = api_router.merge(crate::workspaces::configure_workspaces_routes());
        api_router = api_router.merge(crate::workspaces::ui::configure_workspaces_ui_routes());
    }
    #[cfg(feature = "project")]
    {
        api_router = api_router.merge(crate::project::configure());
    }
    #[cfg(all(feature = "analytics", feature = "goals"))]
    {
        api_router = api_router.merge(crate::analytics::goals::configure_goals_routes());
        api_router = api_router.merge(crate::analytics::goals_ui::configure_goals_ui_routes());
    }
    #[cfg(feature = "player")]
    {
        api_router = api_router.merge(crate::player::configure_player_routes());
    }
    #[cfg(feature = "canvas")]
    {
        api_router = api_router.merge(crate::canvas::configure_canvas_routes());
        api_router = api_router.merge(crate::canvas::ui::configure_canvas_ui_routes());
    }
    #[cfg(feature = "social")]
    {
        api_router = api_router.merge(crate::social::configure_social_routes());
        api_router = api_router.merge(crate::social::ui::configure_social_ui_routes());
    }
    #[cfg(feature = "learn")]
    {
        api_router = api_router.merge(crate::learn::ui::configure_learn_ui_routes());
    }
    #[cfg(feature = "mail")]
    {
        api_router = api_router.merge(crate::email::ui::configure_email_ui_routes());
    }
    #[cfg(feature = "meet")]
    {
        api_router = api_router.merge(crate::meet::ui::configure_meet_ui_routes());
    }
    #[cfg(feature = "people")]
    {
        api_router = api_router.merge(crate::contacts::crm_ui::configure_crm_routes());
        api_router = api_router.merge(crate::contacts::crm::configure_crm_api_routes());
    }
    #[cfg(feature = "billing")]
    {
        api_router = api_router.merge(crate::billing::billing_ui::configure_billing_routes());
        api_router = api_router.merge(crate::billing::api::configure_billing_api_routes());
        api_router = api_router.merge(crate::products::configure_products_routes());
        api_router = api_router.merge(crate::products::api::configure_products_api_routes());
    }
    #[cfg(feature = "tickets")]
    {
        api_router = api_router.merge(crate::tickets::configure_tickets_routes());
        api_router = api_router.merge(crate::tickets::ui::configure_tickets_ui_routes());
    }
    #[cfg(feature = "people")]
    {
        api_router = api_router.merge(crate::people::configure_people_routes());
        api_router = api_router.merge(crate::people::ui::configure_people_ui_routes());
    }
    #[cfg(feature = "attendant")]
    {
        api_router = api_router.merge(crate::attendant::configure_attendant_routes());
        api_router = api_router.merge(crate::attendant::ui::configure_attendant_ui_routes());
    }

    #[cfg(feature = "whatsapp")]
    {
        api_router = api_router.merge(crate::whatsapp::configure());
    }

    #[cfg(feature = "telegram")]
    {
        api_router = api_router.merge(crate::telegram::configure());
    }

    #[cfg(feature = "attendant")]
    {
        api_router = api_router.merge(crate::attendance::configure_attendance_routes());
    }

    api_router = api_router.merge(crate::core::oauth::routes::configure());

    let site_path = app_state
        .config
        .as_ref()
        .map(|c| c.site_path.clone())
        .unwrap_or_else(|| "./botserver-stack/sites".to_string());

    info!("Serving apps from: {}", site_path);

    // Create rate limiter integrating with botlib's RateLimiter
    let http_rate_config = HttpRateLimitConfig::api();
    let system_limits = SystemLimits::default();
    let (rate_limit_extension, _rate_limiter) =
        create_rate_limit_layer(http_rate_config, system_limits);

    // Create security headers layer
    let security_headers_config = SecurityHeadersConfig::default();
    let security_headers_extension = create_security_headers_layer(security_headers_config.clone());

    // Determine panic handler config based on environment
    let is_production = std::env::var("BOTSERVER_ENV")
        .map(|v| v == "production" || v == "prod")
        .unwrap_or(false);
    let panic_config = if is_production {
        PanicHandlerConfig::production()
    } else {
        PanicHandlerConfig::development()
    };

    info!("Security middleware enabled: rate limiting, security headers, panic handler, request ID tracking, authentication");

    // Path to UI files (botui) - use external folder or fallback to embedded
    let ui_path = std::env::var("BOTUI_PATH").unwrap_or_else(|_| {
        if std::path::Path::new("./botui/ui/suite").exists() {
            "./botui/ui/suite".to_string()
        } else if std::path::Path::new("../botui/ui/suite").exists() {
            "../botui/ui/suite".to_string()
        } else {
            "./botui/ui/suite".to_string()
        }
    });
    let ui_path_exists = std::path::Path::new(&ui_path).exists();
    let use_embedded_ui = !ui_path_exists && embedded_ui::has_embedded_ui();

    if ui_path_exists {
        info!("Serving UI from external folder: {}", ui_path);
    } else if use_embedded_ui {
        info!(
            "External UI folder not found at '{}', using embedded UI",
            ui_path
        );
        let file_count = embedded_ui::list_embedded_files().len();
        info!("Embedded UI contains {} files", file_count);
    } else {
        warn!(
            "No UI available: folder '{}' not found and no embedded UI",
            ui_path
        );
    }

    // Update app_state with auth components
    let mut app_state_with_auth = (*app_state).clone();
    app_state_with_auth.jwt_manager = jwt_manager;
    app_state_with_auth.auth_provider_registry = Some(Arc::clone(&auth_provider_registry));
    app_state_with_auth.rbac_manager = Some(Arc::clone(&rbac_manager));
    let app_state = Arc::new(app_state_with_auth);

    let base_router = Router::new()
        .merge(api_router.with_state(app_state.clone()))
        // Static files fallback for legacy /apps/* paths
        .nest_service("/static", ServeDir::new(&site_path));

    // Add UI routes based on availability
    let app_with_ui = if ui_path_exists {
        base_router
            .nest_service("/auth", ServeDir::new(format!("{}/auth", ui_path)))
            .nest_service("/suite", ServeDir::new(&ui_path))
            .nest_service("/themes", ServeDir::new(format!("{}/../themes", ui_path)))
            .fallback_service(ServeDir::new(&ui_path))
    } else if use_embedded_ui {
        base_router.merge(embedded_ui::embedded_ui_router())
    } else {
        base_router
    };

    // Clone rbac_manager for use in middleware
    let rbac_manager_for_middleware = Arc::clone(&rbac_manager);

    let app =
        app_with_ui
            // Security middleware stack (order matters - last added is outermost/runs first)
            .layer(middleware::from_fn(security_headers_middleware))
            .layer(security_headers_extension)
            .layer(rate_limit_extension)
            // Request ID tracking for all requests
            .layer(middleware::from_fn(request_id_middleware))
            // RBAC middleware - checks permissions AFTER authentication
            // NOTE: In Axum, layers run in reverse order (last added = first to run)
            // So RBAC is added BEFORE auth, meaning auth runs first, then RBAC
            .layer(middleware::from_fn(
                move |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                    let rbac = Arc::clone(&rbac_manager_for_middleware);
                    async move { crate::security::rbac_middleware_fn(req, next, rbac).await }
                },
            ))
            // Authentication middleware - MUST run before RBAC (so added after)
            .layer(middleware::from_fn(
                move |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                    let state = auth_middleware_state.clone();
                    async move {
                        crate::security::auth_middleware_with_providers(req, next, state).await
                    }
                },
            ))
            // Panic handler catches panics and returns safe 500 responses
            .layer(middleware::from_fn(move |req, next| {
                let config = panic_config.clone();
                async move {
                    crate::security::panic_handler_middleware_with_config(req, next, &config).await
                }
            }))
            .layer(Extension(app_state.clone()))
            .layer(cors)
            .layer(TraceLayer::new_for_http());

    let cert_dir = std::path::Path::new("./botserver-stack/conf/system/certificates");
    let cert_path = cert_dir.join("api/server.crt");
    let key_path = cert_dir.join("api/server.key");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let disable_tls = std::env::var("BOTSERVER_DISABLE_TLS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if !disable_tls && cert_path.exists() && key_path.exists() {
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .map_err(std::io::Error::other)?;

        info!("HTTPS server listening on {} with TLS", addr);

        let handle = axum_server::Handle::new();
        let handle_clone = handle.clone();

        tokio::spawn(async move {
            shutdown_signal().await;
            info!("Shutting down HTTPS server...");
            handle_clone.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
        });

        axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(app.into_make_service())
            .await
            .map_err(|e| {
                error!("HTTPS server failed on {}: {}", addr, e);
                e
            })
    } else {
        if disable_tls {
            info!("TLS disabled via BOTSERVER_DISABLE_TLS environment variable");
        } else {
            warn!("TLS certificates not found, using HTTP");
        }

        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                error!(
                    "Failed to bind to {}: {} - is another instance running?",
                    addr, e
                );
                return Err(e);
            }
        };
        info!("HTTP server listening on {}", addr);
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(std::io::Error::other)
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Set global panic hook to log panics that escape async boundaries
    set_global_panic_hook();

    let args: Vec<String> = std::env::args().collect();
    let no_ui = args.contains(&"--noui".to_string());

    #[cfg(feature = "console")]
    let no_console = args.contains(&"--noconsole".to_string());

    #[cfg(not(feature = "console"))]
    let no_console = true;

    let _ = rustls::crypto::ring::default_provider().install_default();

    dotenvy::dotenv().ok();

    let env_path_early = std::path::Path::new("./.env");
    let vault_init_path_early = std::path::Path::new("./botserver-stack/conf/vault/init.json");
    let bootstrap_ready = env_path_early.exists() && vault_init_path_early.exists() && {
        std::fs::read_to_string(env_path_early)
            .map(|content| content.contains("VAULT_TOKEN="))
            .unwrap_or(false)
    };

    if bootstrap_ready {
        if let Err(e) = crate::shared::utils::init_secrets_manager().await {
            warn!(
                "Failed to initialize SecretsManager: {}. Falling back to env vars.",
                e
            );
        } else {
            info!("SecretsManager initialized - fetching secrets from Vault");
        }
    } else {
        trace!("Bootstrap not complete - skipping early SecretsManager init");
    }

    let noise_filters = "vaultrs=off,rustify=off,rustify_derive=off,\
         aws_sigv4=off,aws_smithy_checksums=off,aws_runtime=off,aws_smithy_http_client=off,\
         aws_smithy_runtime=off,aws_smithy_runtime_api=off,aws_sdk_s3=off,aws_config=off,\
         aws_credential_types=off,aws_http=off,aws_sig_auth=off,aws_types=off,\
         mio=off,tokio=off,tokio_util=off,tower=off,tower_http=off,\
         reqwest=off,hyper=off,hyper_util=off,h2=off,\
         rustls=off,rustls_pemfile=off,tokio_rustls=off,\
         tracing=off,tracing_core=off,tracing_subscriber=off,\
         diesel=off,diesel_migrations=off,r2d2=warn,\
         serde=off,serde_json=off,\
         axum=off,axum_core=off,\
         tonic=off,prost=off,\
         lettre=off,imap=off,mailparse=off,\
         crossterm=off,ratatui=off,\
         tauri=off,tauri_runtime=off,tauri_utils=off,\
         notify=off,ignore=off,walkdir=off,\
         want=off,try_lock=off,futures=off,\
         base64=off,bytes=off,encoding_rs=off,\
         url=off,percent_encoding=off,\
         ring=off,webpki=off,\
         hickory_resolver=off,hickory_proto=off";

    let rust_log = match std::env::var("RUST_LOG") {
        Ok(existing) if !existing.is_empty() => format!("{},{}", existing, noise_filters),
        _ => format!("info,{}", noise_filters),
    };

    std::env::set_var("RUST_LOG", &rust_log);

    use crate::core::config::ConfigManager;
    #[cfg(feature = "llm")]
    use crate::llm::local::ensure_llama_servers_running;

    if no_console || no_ui {
        botlib::logging::init_compact_logger_with_style("info");
        println!("Starting General Bots {}...", env!("CARGO_PKG_VERSION"));
    }

    let locales_path = if std::path::Path::new("./locales").exists() {
        "./locales"
    } else if std::path::Path::new("../botlib/locales").exists() {
        "../botlib/locales"
    } else if std::path::Path::new("../locales").exists() {
        "../locales"
    } else {
        "./locales"
    };
    if let Err(e) = crate::core::i18n::init_i18n(locales_path) {
        warn!(
            "Failed to initialize i18n from {}: {}. Translations will show keys.",
            locales_path, e
        );
    } else {
        info!(
            "i18n initialized from {} with locales: {:?}",
            locales_path,
            crate::core::i18n::available_locales()
        );
    }

    let (progress_tx, _progress_rx) = tokio::sync::mpsc::unbounded_channel::<BootstrapProgress>();
    let (state_tx, _state_rx) = tokio::sync::mpsc::channel::<Arc<AppState>>(1);

    if args.len() > 1 {
        let command = &args[1];
        match command.as_str() {
            "install" | "remove" | "list" | "status" | "start" | "stop" | "restart" | "--help"
            | "-h" => match package_manager::cli::run().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("CLI error: {e}");
                    return Err(std::io::Error::other(format!("CLI command failed: {e}")));
                }
            },
            _ => {}
        }
    }

    let ui_handle: Option<std::thread::JoinHandle<()>> = if !no_console && !no_ui {
        #[cfg(feature = "console")]
        {
            let progress_rx = Arc::new(tokio::sync::Mutex::new(_progress_rx));
            let state_rx = Arc::new(tokio::sync::Mutex::new(_state_rx));

            Some(
                std::thread::Builder::new()
                    .name("ui-thread".to_string())
                    .spawn(move || {
                        let mut ui = crate::console::XtreeUI::new();
                        ui.set_progress_channel(progress_rx);
                        ui.set_state_channel(state_rx);

                        if let Err(e) = ui.start_ui() {
                            eprintln!("UI error: {e}");
                        }
                    })
                    .map_err(|e| {
                        std::io::Error::other(format!("Failed to spawn UI thread: {}", e))
                    })?,
            )
        }
        #[cfg(not(feature = "console"))]
        {
            if !no_console {
                eprintln!("Console feature not compiled. Rebuild with --features console or use --noconsole to suppress this message");
            }
            None
        }
    } else {
        None
    };

    let install_mode = if args.contains(&"--container".to_string()) {
        InstallMode::Container
    } else {
        InstallMode::Local
    };

    let tenant = if let Some(idx) = args.iter().position(|a| a == "--tenant") {
        args.get(idx + 1).cloned()
    } else {
        None
    };

    if let Some(idx) = args.iter().position(|a| a == "--stack-path") {
        if let Some(path) = args.get(idx + 1) {
            std::env::set_var("BOTSERVER_STACK_PATH", path);
            info!("Using custom stack path: {}", path);
        }
    }

    trace!("Starting bootstrap process...");
    let progress_tx_clone = progress_tx.clone();
    let cfg = {
        progress_tx_clone
            .send(BootstrapProgress::StartingBootstrap)
            .ok();

        trace!("Creating BootstrapManager...");
        let mut bootstrap = BootstrapManager::new(install_mode.clone(), tenant.clone());

        let env_path = std::path::Path::new("./.env");
        let vault_init_path = std::path::Path::new("./botserver-stack/conf/vault/init.json");
        let bootstrap_completed = env_path.exists() && vault_init_path.exists() && {
            std::fs::read_to_string(env_path)
                .map(|content| content.contains("VAULT_TOKEN="))
                .unwrap_or(false)
        };

        info!(
            "Bootstrap check: .env exists={}, init.json exists={}, bootstrap_completed={}",
            env_path.exists(),
            vault_init_path.exists(),
            bootstrap_completed
        );

        let cfg = if bootstrap_completed {
            info!(">>> BRANCH: bootstrap_completed=TRUE - starting services only");
            trace!("Services already configured, ensuring all are running...");
            info!("Ensuring database and drive services are running...");
            progress_tx_clone
                .send(BootstrapProgress::StartingComponent(
                    "all services".to_string(),
                ))
                .ok();
            trace!("Calling bootstrap.start_all()...");
            bootstrap.start_all().await.map_err(std::io::Error::other)?;
            trace!("bootstrap.start_all() completed");

            trace!("Connecting to database...");
            progress_tx_clone
                .send(BootstrapProgress::ConnectingDatabase)
                .ok();

            trace!("Creating database connection...");
            match create_conn() {
                Ok(pool) => {
                    trace!("Database connection successful, loading config from database");
                    AppConfig::from_database(&pool).unwrap_or_else(|e| {
                        warn!("Failed to load config from database: {}, trying env", e);
                        AppConfig::from_env().unwrap_or_else(|env_e| {
                            error!("Failed to load config from env: {}", env_e);
                            AppConfig::default()
                        })
                    })
                }
                Err(e) => {
                    trace!(
                        "Database connection failed: {:?}, loading config from env",
                        e
                    );
                    AppConfig::from_env().unwrap_or_else(|e| {
                        error!("Failed to load config from env: {}", e);
                        AppConfig::default()
                    })
                }
            }
        } else {
            info!(">>> BRANCH: bootstrap_completed=FALSE - running full bootstrap");
            info!("Bootstrap not complete - running full bootstrap...");
            trace!(".env file not found, running bootstrap.bootstrap()...");
            if let Err(e) = bootstrap.bootstrap().await {
                error!("Bootstrap failed: {}", e);
                return Err(std::io::Error::other(format!("Bootstrap failed: {e}")));
            }
            trace!("bootstrap.bootstrap() completed");
            progress_tx_clone
                .send(BootstrapProgress::StartingComponent(
                    "all services".to_string(),
                ))
                .ok();
            bootstrap.start_all().await.map_err(std::io::Error::other)?;

            match create_conn() {
                Ok(pool) => AppConfig::from_database(&pool).unwrap_or_else(|e| {
                    warn!("Failed to load config from database: {}, trying env", e);
                    AppConfig::from_env().unwrap_or_else(|env_e| {
                        error!("Failed to load config from env: {}", env_e);
                        AppConfig::default()
                    })
                }),
                Err(_) => AppConfig::from_env().unwrap_or_else(|e| {
                    error!("Failed to load config from env: {}", e);
                    AppConfig::default()
                }),
            }
        };

        trace!("Config loaded, syncing templates to database...");
        progress_tx_clone
            .send(BootstrapProgress::UploadingTemplates)
            .ok();

        if let Err(e) = bootstrap.sync_templates_to_database() {
            warn!("Failed to sync templates to database: {}", e);
        } else {
            trace!("Templates synced to database");
        }

        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            bootstrap.upload_templates_to_drive(&cfg),
        )
        .await
        {
            Ok(Ok(_)) => {
                trace!("Templates uploaded to drive successfully");
            }
            Ok(Err(e)) => {
                warn!("Template drive upload error (non-blocking): {}", e);
            }
            Err(_) => {
                warn!("Template drive upload timed out after 30s, continuing startup...");
            }
        }

        Ok::<AppConfig, std::io::Error>(cfg)
    };

    trace!("Bootstrap config phase complete");
    let cfg = cfg?;
    trace!("Reloading dotenv...");
    dotenv().ok();

    trace!("Creating database pool again...");
    progress_tx.send(BootstrapProgress::ConnectingDatabase).ok();

    let pool = match create_conn() {
        Ok(pool) => {
            trace!("Running database migrations...");
            info!("Running database migrations...");
            if let Err(e) = crate::shared::utils::run_migrations(&pool) {
                error!("Failed to run migrations: {}", e);

                warn!("Continuing despite migration errors - database might be partially migrated");
            } else {
                info!("Database migrations completed successfully");
            }
            pool
        }
        Err(e) => {
            error!("Failed to create database pool: {}", e);
            progress_tx
                .send(BootstrapProgress::BootstrapError(format!(
                    "Database pool creation failed: {}",
                    e
                )))
                .ok();
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Database pool creation failed: {}", e),
            ));
        }
    };

    info!("Loading config from database after template sync...");
    let refreshed_cfg = AppConfig::from_database(&pool).unwrap_or_else(|e| {
        warn!(
            "Failed to load config from database: {}, falling back to env",
            e
        );
        AppConfig::from_env().unwrap_or_else(|e| {
            error!("Failed to load config from env: {}", e);
            AppConfig::default()
        })
    });
    let config = std::sync::Arc::new(refreshed_cfg.clone());
    info!(
        "Server configured to listen on {}:{}",
        config.server.host, config.server.port
    );

    #[cfg(feature = "cache")]
    let cache_url = "redis://localhost:6379".to_string();
    #[cfg(feature = "cache")]
    let redis_client = match redis::Client::open(cache_url.as_str()) {
        Ok(client) => Some(Arc::new(client)),
        Err(e) => {
            log::warn!("Failed to connect to Redis: {}", e);
            None
        }
    };
    #[cfg(not(feature = "cache"))]
    let redis_client = None;

    let web_adapter = Arc::new(WebChannelAdapter::new());
    let voice_adapter = Arc::new(VoiceAdapter::new());

    #[cfg(feature = "drive")]
    let drive = create_s3_operator(&config.drive)
        .await
        .map_err(|e| std::io::Error::other(format!("Failed to initialize Drive: {}", e)))?;

    #[cfg(feature = "drive")]
    ensure_vendor_files_in_minio(&drive).await;

    let session_manager = Arc::new(tokio::sync::Mutex::new(session::SessionManager::new(
        pool.get().map_err(|e| {
            std::io::Error::other(format!("Failed to get database connection: {}", e))
        })?,
        #[cfg(feature = "cache")]
        redis_client.clone(),
    )));

    #[cfg(feature = "directory")]
    let zitadel_config = {
        // Try to load from directory_config.json first
        let config_path = "./config/directory_config.json";
        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let base_url = json
                    .get("base_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("http://localhost:8300");
                let client_id = json.get("client_id").and_then(|v| v.as_str()).unwrap_or("");
                let client_secret = json
                    .get("client_secret")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                info!(
                    "Loaded Zitadel config from {}: url={}",
                    config_path, base_url
                );

                crate::directory::ZitadelConfig {
                    issuer_url: base_url.to_string(),
                    issuer: base_url.to_string(),
                    client_id: client_id.to_string(),
                    client_secret: client_secret.to_string(),
                    redirect_uri: format!("{}/callback", base_url),
                    project_id: "default".to_string(),
                    api_url: base_url.to_string(),
                    service_account_key: None,
                }
            } else {
                warn!("Failed to parse directory_config.json, using defaults");
                crate::directory::ZitadelConfig {
                    issuer_url: "http://localhost:8300".to_string(),
                    issuer: "http://localhost:8300".to_string(),
                    client_id: String::new(),
                    client_secret: String::new(),
                    redirect_uri: "http://localhost:8300/callback".to_string(),
                    project_id: "default".to_string(),
                    api_url: "http://localhost:8300".to_string(),
                    service_account_key: None,
                }
            }
        } else {
            warn!("directory_config.json not found, using default Zitadel config");
            crate::directory::ZitadelConfig {
                issuer_url: "http://localhost:8300".to_string(),
                issuer: "http://localhost:8300".to_string(),
                client_id: String::new(),
                client_secret: String::new(),
                redirect_uri: "http://localhost:8300/callback".to_string(),
                project_id: "default".to_string(),
                api_url: "http://localhost:8300".to_string(),
                service_account_key: None,
            }
        }
    };
    #[cfg(feature = "directory")]
    let auth_service = Arc::new(tokio::sync::Mutex::new(
        crate::directory::AuthService::new(zitadel_config.clone())
            .map_err(|e| std::io::Error::other(format!("Failed to create auth service: {}", e)))?,
    ));

    #[cfg(feature = "directory")]
    {
        let pat_path = std::path::Path::new("./botserver-stack/conf/directory/admin-pat.txt");
        let bootstrap_client = if pat_path.exists() {
            match std::fs::read_to_string(pat_path) {
                Ok(pat_token) => {
                    let pat_token = pat_token.trim().to_string();
                    info!("Using admin PAT token for bootstrap authentication");
                    crate::directory::ZitadelClient::with_pat_token(zitadel_config, pat_token)
                        .map_err(|e| {
                            std::io::Error::other(format!(
                                "Failed to create bootstrap client with PAT: {}",
                                e
                            ))
                        })?
                }
                Err(e) => {
                    warn!(
                        "Failed to read admin PAT token: {}, falling back to OAuth2",
                        e
                    );
                    crate::directory::ZitadelClient::new(zitadel_config).map_err(|e| {
                        std::io::Error::other(format!("Failed to create bootstrap client: {}", e))
                    })?
                }
            }
        } else {
            info!("Admin PAT not found, using OAuth2 client credentials for bootstrap");
            crate::directory::ZitadelClient::new(zitadel_config).map_err(|e| {
                std::io::Error::other(format!("Failed to create bootstrap client: {}", e))
            })?
        };

        match crate::directory::bootstrap::check_and_bootstrap_admin(&bootstrap_client).await {
            Ok(Some(_)) => {
                info!("Bootstrap completed - admin credentials displayed in console");
            }
            Ok(None) => {
                info!("Admin user exists, bootstrap skipped");
            }
            Err(e) => {
                warn!("Bootstrap check failed (Zitadel may not be ready): {}", e);
            }
        }
    }
    let config_manager = ConfigManager::new(pool.clone());

    let mut bot_conn = pool
        .get()
        .map_err(|e| std::io::Error::other(format!("Failed to get database connection: {}", e)))?;
    let (default_bot_id, default_bot_name) = crate::bot::get_default_bot(&mut bot_conn);
    info!(
        "Using default bot: {} (id: {})",
        default_bot_name, default_bot_id
    );

    let llm_url = config_manager
        .get_config(&default_bot_id, "llm-url", Some("http://localhost:8081"))
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    info!("LLM URL: {}", llm_url);

    let llm_model = config_manager
        .get_config(&default_bot_id, "llm-model", Some(""))
        .unwrap_or_default();
    if !llm_model.is_empty() {
        info!("LLM Model: {}", llm_model);
    }

    let _llm_key = config_manager
        .get_config(&default_bot_id, "llm-key", Some(""))
        .unwrap_or_default();

    // LLM endpoint path configuration
    let llm_endpoint_path = config_manager
        .get_config(
            &default_bot_id,
            "llm-endpoint-path",
            Some("/v1/chat/completions"),
        )
        .unwrap_or_else(|_| "/v1/chat/completions".to_string());

    #[cfg(feature = "llm")]
    let base_llm_provider = crate::llm::create_llm_provider_from_url(
        &llm_url,
        if llm_model.is_empty() {
            None
        } else {
            Some(llm_model.clone())
        },
        Some(llm_endpoint_path.clone()),
    );

    #[cfg(feature = "llm")]
    let dynamic_llm_provider = Arc::new(crate::llm::DynamicLLMProvider::new(base_llm_provider));

    #[cfg(feature = "llm")]
    {
        // Ensure the DynamicLLMProvider is initialized with the correct config from database
        // This makes the system robust: even if the URL was set before server startup,
        // the provider will use the correct configuration
        info!("Initializing DynamicLLMProvider with config: URL={}, Model={}, Endpoint={}",
              llm_url,
              if llm_model.is_empty() { "(default)" } else { &llm_model },
              llm_endpoint_path.clone());
        dynamic_llm_provider.update_from_config(
            &llm_url,
            if llm_model.is_empty() { None } else { Some(llm_model.clone()) },
            Some(llm_endpoint_path),
        ).await;
        info!("DynamicLLMProvider initialized successfully");
    }

    #[cfg(feature = "llm")]
    let llm_provider: Arc<dyn crate::llm::LLMProvider> = if let Some(ref cache) = redis_client {
        let embedding_url = config_manager
            .get_config(
                &default_bot_id,
                "embedding-url",
                Some("http://localhost:8082"),
            )
            .unwrap_or_else(|_| "http://localhost:8082".to_string());
        let embedding_model = config_manager
            .get_config(&default_bot_id, "embedding-model", Some("all-MiniLM-L6-v2"))
            .unwrap_or_else(|_| "all-MiniLM-L6-v2".to_string());
        info!("Embedding URL: {}", embedding_url);
        info!("Embedding Model: {}", embedding_model);

        let embedding_service = Some(Arc::new(crate::llm::cache::LocalEmbeddingService::new(
            embedding_url,
            embedding_model,
        )) as Arc<dyn crate::llm::cache::EmbeddingService>);

        let cache_config = crate::llm::cache::CacheConfig {
            ttl: 3600,
            semantic_matching: true,
            similarity_threshold: 0.85,
            max_similarity_checks: 100,
            key_prefix: "llm_cache".to_string(),
        };

        Arc::new(crate::llm::cache::CachedLLMProvider::with_db_pool(
            dynamic_llm_provider.clone() as Arc<dyn crate::llm::LLMProvider>,
            cache.clone(),
            cache_config,
            embedding_service,
            pool.clone(),
        ))
    } else {
        dynamic_llm_provider.clone() as Arc<dyn crate::llm::LLMProvider>
    };

    #[cfg(any(feature = "research", feature = "llm"))]
    let kb_manager = Arc::new(crate::core::kb::KnowledgeBaseManager::new("work"));

    #[cfg(feature = "tasks")]
    let task_engine = Arc::new(crate::tasks::TaskEngine::new(pool.clone()));

    let metrics_collector = crate::core::shared::analytics::MetricsCollector::new();

    #[cfg(feature = "tasks")]
    let task_scheduler = None;

    let (attendant_tx, _attendant_rx) =
        tokio::sync::broadcast::channel::<crate::core::shared::state::AttendantNotification>(1000);

    let (task_progress_tx, _task_progress_rx) =
        tokio::sync::broadcast::channel::<crate::core::shared::state::TaskProgressEvent>(1000);

    // Initialize BotDatabaseManager for per-bot database support
    let database_url = crate::shared::utils::get_database_url_sync().unwrap_or_default();
    let bot_database_manager = Arc::new(BotDatabaseManager::new(pool.clone(), &database_url));

    // Sync all bot databases on startup - ensures each bot has its own database
    info!("Syncing bot databases on startup...");
    match bot_database_manager.sync_all_bot_databases() {
        Ok(sync_result) => {
            info!(
                "Bot database sync complete: {} created, {} verified, {} errors",
                sync_result.databases_created,
                sync_result.databases_verified,
                sync_result.errors.len()
            );
            for err in &sync_result.errors {
                warn!("Bot database sync error: {}", err);
            }
        }
        Err(e) => {
            error!("Failed to sync bot databases: {}", e);
        }
    }

    let app_state = Arc::new(AppState {
        #[cfg(feature = "drive")]
        drive: Some(drive),
        config: Some(cfg.clone()),
        conn: pool.clone(),
        database_url: database_url.clone(),
        bot_database_manager: bot_database_manager.clone(),
        bucket_name: "default.gbai".to_string(),
        #[cfg(feature = "cache")]
        cache: redis_client.clone(),
        session_manager: session_manager.clone(),
        metrics_collector,
        #[cfg(feature = "tasks")]
        task_scheduler,
        #[cfg(feature = "llm")]
        llm_provider: llm_provider.clone(),
        #[cfg(feature = "llm")]
        dynamic_llm_provider: Some(dynamic_llm_provider.clone()),
        #[cfg(feature = "directory")]
        auth_service: auth_service.clone(),
        channels: Arc::new(tokio::sync::Mutex::new({
            let mut map = HashMap::new();
            map.insert(
                "web".to_string(),
                web_adapter.clone() as Arc<dyn crate::core::bot::channels::ChannelAdapter>,
            );
            map
        })),
        response_channels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        web_adapter: web_adapter.clone(),
        voice_adapter: voice_adapter.clone(),
        #[cfg(any(feature = "research", feature = "llm"))]
        kb_manager: Some(kb_manager.clone()),
        #[cfg(feature = "tasks")]
        task_engine,
        extensions: {
            let ext = crate::core::shared::state::Extensions::new();
            #[cfg(feature = "llm")]
            ext.insert_blocking(Arc::clone(&dynamic_llm_provider));
            ext
        },
        attendant_broadcast: Some(attendant_tx),
        task_progress_broadcast: Some(task_progress_tx),
        billing_alert_broadcast: None,
        task_manifests: Arc::new(std::sync::RwLock::new(HashMap::new())),
        #[cfg(feature = "project")]
        project_service: Arc::new(tokio::sync::RwLock::new(
            crate::project::ProjectService::new(),
        )),
        #[cfg(feature = "compliance")]
        legal_service: Arc::new(tokio::sync::RwLock::new(crate::legal::LegalService::new())),
        jwt_manager: None,
        auth_provider_registry: None,
        rbac_manager: None,
    });

    // Resume workflows after server restart
    if let Err(e) =
        crate::basic::keywords::orchestration::resume_workflows_on_startup(app_state.clone()).await
    {
        log::warn!("Failed to resume workflows on startup: {}", e);
    }

    #[cfg(feature = "tasks")]
    let task_scheduler = Arc::new(crate::tasks::scheduler::TaskScheduler::new(
        app_state.clone(),
    ));

    #[cfg(feature = "tasks")]
    task_scheduler.start();

    #[cfg(any(feature = "research", feature = "llm"))]
    if let Err(e) = crate::core::kb::ensure_crawler_service_running(app_state.clone()).await {
        log::warn!("Failed to start website crawler service: {}", e);
    }

    // Start memory monitoring - check every 30 seconds, warn if growth > 50MB
    start_memory_monitor(30, 50);
    info!("Memory monitor started");
    log_process_memory();

    let _ = state_tx.try_send(app_state.clone());
    progress_tx.send(BootstrapProgress::BootstrapComplete).ok();

    info!(
        "Starting HTTP server on {}:{}",
        config.server.host, config.server.port
    );

    let worker_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let bot_orchestrator = BotOrchestrator::new(app_state.clone());
    if let Err(e) = bot_orchestrator.mount_all_bots() {
        error!("Failed to mount bots: {}", e);
    }

    #[cfg(feature = "llm")]
    {
        let app_state_for_llm = app_state.clone();
        trace!("ensure_llama_servers_running starting...");
        if let Err(e) = ensure_llama_servers_running(app_state_for_llm).await {
            error!("Failed to start LLM servers: {}", e);
        }
        trace!("ensure_llama_servers_running completed");
    }

    #[cfg(feature = "drive")]
    {
        let drive_monitor_state = app_state.clone();
        let pool_clone = pool.clone();

        tokio::spawn(async move {
            register_thread("drive-monitor", "drive");

            let bots_to_monitor = tokio::task::spawn_blocking(move || {
                use uuid::Uuid;
                let mut conn = match pool_clone.get() {
                    Ok(conn) => conn,
                    Err(_) => return Vec::new(),
                };
                use crate::shared::models::schema::bots::dsl::*;
                use diesel::prelude::*;
                bots.filter(is_active.eq(true))
                    .select((id, name))
                    .load::<(Uuid, String)>(&mut conn)
                    .unwrap_or_default()
            })
            .await
            .unwrap_or_default();

            info!("Found {} active bots to monitor", bots_to_monitor.len());

            for (bot_id, bot_name) in bots_to_monitor {
                // Skip default bot - it's managed locally via ConfigWatcher
                if bot_name == "default" {
                    info!("Skipping DriveMonitor for 'default' bot - managed via ConfigWatcher");
                    continue;
                }

                let bucket_name = format!("{}.gbai", bot_name);
                let monitor_state = drive_monitor_state.clone();
                let bot_id_clone = bot_id;
                let bucket_name_clone = bucket_name.clone();

                tokio::spawn(async move {
                    register_thread(&format!("drive-monitor-{}", bot_name), "drive");
                    trace!("DriveMonitor::new starting for bot: {}", bot_name);
                    let monitor =
                        crate::DriveMonitor::new(monitor_state, bucket_name_clone, bot_id_clone);
                    trace!(
                        "DriveMonitor::new done for bot: {}, calling start_monitoring...",
                        bot_name
                    );
                    info!(
                        "Starting DriveMonitor for bot: {} (bucket: {})",
                        bot_name, bucket_name
                    );
                    if let Err(e) = monitor.start_monitoring().await {
                        error!("DriveMonitor failed for bot {}: {}", bot_name, e);
                    }
                    trace!(
                        "DriveMonitor start_monitoring returned for bot: {}",
                        bot_name
                    );
                });
            }
        });
    }

    #[cfg(feature = "drive")]
    {
        // Start local file monitor for ~/data/*.gbai directories
        let local_monitor_state = app_state.clone();
        tokio::spawn(async move {
            register_thread("local-file-monitor", "drive");
            trace!("Starting LocalFileMonitor for ~/data/*.gbai directories");
            let monitor = crate::drive::local_file_monitor::LocalFileMonitor::new(local_monitor_state);
            if let Err(e) = monitor.start_monitoring().await {
                error!("LocalFileMonitor failed: {}", e);
            } else {
                info!("LocalFileMonitor started - watching ~/data/*.gbai/.gbdialog/*.bas");
            }
        });
    }

    #[cfg(feature = "drive")]
    {
        // Start config file watcher for ~/data/*.gbai/*.gbot/config.csv
        let config_watcher_state = app_state.clone();
        tokio::spawn(async move {
            register_thread("config-file-watcher", "drive");
            trace!("Starting ConfigWatcher for ~/data/*.gbai/*.gbot/config.csv");

            // Determine data directory
            let data_dir = std::env::var("DATA_DIR")
                .or_else(|_| std::env::var("HOME").map(|h| format!("{}/data", h)))
                .unwrap_or_else(|_| "./botserver-stack/data".to_string());
            let data_dir = std::path::PathBuf::from(data_dir);

            let watcher = crate::core::config::watcher::ConfigWatcher::new(
                data_dir,
                config_watcher_state,
            );
            Arc::new(watcher).spawn();

            info!("ConfigWatcher started - watching ~/data/*.gbai/*.gbot/config.csv");
        });
    }

    #[cfg(feature = "automation")]
    {
        let automation_state = app_state.clone();
        tokio::spawn(async move {
            register_thread("automation-service", "automation");
            let automation = AutomationService::new(automation_state);
            trace!(
                "[TASK] AutomationService starting, RSS={}",
                MemoryStats::format_bytes(MemoryStats::current().rss_bytes)
            );
            loop {
                record_thread_activity("automation-service");
                if let Err(e) = automation.check_scheduled_tasks().await {
                    error!("Error checking scheduled tasks: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }

    trace!("Initial data setup task spawned");
    trace!("All system threads started, starting HTTP server...");

    info!("Starting HTTP server on port {}...", config.server.port);
    if let Err(e) = run_axum_server(app_state, config.server.port, worker_count).await {
        error!("Failed to start HTTP server: {}", e);
        std::process::exit(1);
    }
    trace!("run_axum_server returned (should not happen normally)");

    if let Some(handle) = ui_handle {
        handle.join().ok();
    }

    Ok(())
}
