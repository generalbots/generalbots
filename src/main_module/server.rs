//! HTTP server initialization and routing

use axum::extract::State;
use axum::{
    routing::{get, post},
    Json, Router,
};
use log::{error, info, trace, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;

use crate::core::shared::state::AppState;
use crate::core::urls::ApiUrls;
use crate::security::{
    build_default_route_permissions, create_cors_layer, create_rate_limit_layer,
    create_security_headers_layer, request_id_middleware, security_headers_middleware,
    AuthConfig, AuthMiddlewareState, AuthProviderBuilder, ApiKeyAuthProvider,
    HttpRateLimitConfig, JwtConfig, JwtKey, JwtManager, PanicHandlerConfig, RbacConfig,
    RbacManager, SecurityHeadersConfig,
};
use botlib::SystemLimits;

use super::{health_check, health_check_simple, receive_client_errors, shutdown_signal};

pub async fn run_axum_server(
    app_state: Arc<AppState>,
    port: u16,
    _worker_count: usize,
) -> std::io::Result<()> {
    // Load CORS allowed origins from bot config database if available
    // Config key: cors-allowed-origins in config.csv
    if let Ok(mut conn) = app_state.conn.get() {
        use crate::core::shared::models::schema::bot_configuration::dsl::*;
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
                crate::security::set_cors_allowed_origins(origins);
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
        .route(ApiUrls::SESSIONS, post(crate::core::session::create_session))
        .route(ApiUrls::SESSIONS, get(crate::core::session::get_sessions))
        .route(ApiUrls::SESSION_HISTORY, get(crate::core::session::get_session_history))
        .route(ApiUrls::SESSION_START, post(crate::core::session::start_session))
        .route(ApiUrls::WS, get(crate::core::bot::websocket_handler));

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
    let use_embedded_ui = !ui_path_exists && crate::embedded_ui::has_embedded_ui();

    if ui_path_exists {
        info!("Serving UI from external folder: {}", ui_path);
    } else if use_embedded_ui {
        info!(
            "External UI folder not found at '{}', using embedded UI",
            ui_path
        );
        let file_count = crate::embedded_ui::list_embedded_files().len();
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
        base_router.merge(crate::embedded_ui::embedded_ui_router())
    } else {
        base_router
    };

    // Clone rbac_manager for use in middleware
    let rbac_manager_for_middleware = Arc::clone(&rbac_manager);

    let app =
        app_with_ui
            // Security middleware stack (order matters - last added is outermost/runs first)
            .layer(axum::middleware::from_fn(security_headers_middleware))
            .layer(security_headers_extension)
            .layer(rate_limit_extension)
            // Request ID tracking for all requests
            .layer(axum::middleware::from_fn(request_id_middleware))
            // RBAC middleware - checks permissions AFTER authentication
            // NOTE: In Axum, layers run in reverse order (last added = first to run)
            // So RBAC is added BEFORE auth, meaning auth runs first, then RBAC
            .layer(axum::middleware::from_fn(
                move |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                    let rbac = Arc::clone(&rbac_manager_for_middleware);
                    async move { crate::security::rbac_middleware_fn(req, next, rbac).await }
                },
            ))
            // Authentication middleware - MUST run before RBAC (so added after)
            .layer(axum::middleware::from_fn(
                move |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                    let state = auth_middleware_state.clone();
                    async move {
                        crate::security::auth_middleware_with_providers(req, next, state).await
                    }
                },
            ))
            // Panic handler catches panics and returns safe 500 responses
            .layer(axum::middleware::from_fn(move |req, next| {
                let config = panic_config.clone();
                async move {
                    crate::security::panic_handler_middleware_with_config(req, next, &config).await
                }
            }))
            .layer(axum::Extension(app_state.clone()))
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
