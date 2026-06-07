//! HTTP server initialization and routing

use axum::{
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use std::net::SocketAddr;
use log::{error, info, warn};
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;
use crate::security::{
    build_default_route_permissions, create_cors_layer, create_rate_limit_layer,
    create_security_headers_layer, request_id_middleware, security_headers_middleware, csrf_middleware,
    AuthConfig, AuthMiddlewareState, AuthProviderBuilder, ApiKeyAuthProvider,
    CsrfConfig, CsrfManager,
    HttpRateLimitConfig, JwtConfig, JwtKey, JwtManager, PanicHandlerConfig, RbacConfig,
    RbacManager, SecurityHeadersConfig,
};
use botcore::shared::state::AppState;
use botcore::urls::ApiUrls;
use botlib::SystemLimits;
use diesel::prelude::*;
use diesel::QueryDsl;
use botcore::shared::models::schema::bot_configuration::dsl::*;

use super::{health_check, health_check_simple, receive_client_errors, shutdown_signal};

pub async fn run_axum_server(
    app_state: Arc<AppState>,
    port: u16,
    _worker_count: usize,
) -> std::io::Result<()> {
    // Load CORS allowed origins from bot config database if available
    // Config key: cors-allowed-origins in config.csv
    if let Ok(mut conn) = app_state.conn.get() {

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
            .add_anonymous_path("/api/setup/status")
            .add_anonymous_path("/api/bot/config")
            .add_anonymous_path("/api/suggestions")
            .add_anonymous_path("/api/client-errors")
            .add_anonymous_path("/ws")
            .add_anonymous_path("/auth")
            .add_anonymous_path("/webhook/whatsapp") // WhatsApp webhook for Meta verification
            .add_public_path("/static")
            .add_public_path("/favicon.ico")
            .add_public_path("/suite")
            .add_public_path("/themes")
            .add_public_path("/api/product") // For desktop UI initialization
            .add_public_path("/") // Allow all bot routes (fallback to UI)
    );

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        info!("JWT_SECRET not set, using default development secret");
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
            .unwrap_or_else(|e| {
                error!("Product config RwLock poisoned: {}", e);
                e.into_inner()
            });
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
        .route("/api/config/reload", post(botcore::config_reload::reload_config))
        .route("/api/product", get(get_product_config))
        .route("/api/manifest", get(get_workspace_manifest))
        .route("/api/client-errors", post(receive_client_errors))
        // TODO: fix handler signature
        .route("/api/bot/config", get(crate::core::bot::get_bot_config))
        .route("/api/bots/:bot_name/access", get(crate::core::bot::check_access_handler))
        // Gateway spawns on its own port 5860 below, no nesting needed here
        .route(ApiUrls::SESSIONS, post(crate::core::session::create_session))
        // TODO: fix handler signature
        //.route(ApiUrls::SESSIONS, get(crate::core::session::get_sessions))
        // TODO: fix handler signature
        //.route(ApiUrls::SESSION_HISTORY, get(crate::core::session::get_session_history))
        .route(ApiUrls::SESSION_START, post(crate::core::session::start_session))
        .route(ApiUrls::WS, get(crate::core::bot::websocket_handler))
        .route("/ws/:bot_name", get(crate::core::bot::websocket_handler_with_bot));

    #[cfg(feature = "drive")]
    {
        use axum::routing::{get as axum_get, post as axum_post};
        api_router = api_router
            .route("/api/files/list", axum_get(crate::drive::drive_handlers::list_files))
            .route("/api/files/buckets", axum_get(crate::drive::drive_handlers::list_buckets))
            .route("/api/files/quota", axum_get(crate::drive::drive_handlers::quota))
            .route("/api/files/recent", axum_get(crate::drive::drive_handlers::recent_files))
            .route("/api/files/search", axum_get(crate::drive::drive_handlers::search_files))
            .route(
                "/api/files/write",
                axum_post(crate::drive::drive_handlers::upload_file_to_drive),
            )
            .route(
                "/api/files/download",
                axum_post(crate::drive::drive_handlers::download_file),
            )
            .route(
                "/api/files/download-binary",
                axum_post(crate::drive::drive_handlers::download_file_binary),
            )
            .route(
                "/api/files/delete",
                axum_post(crate::drive::drive_handlers::delete_file),
            )
            .route(
                "/api/files/createFolder",
                axum_post(crate::drive::drive_handlers::create_folder),
            )
            .route(
                "/api/files/copy",
                axum_post(crate::drive::drive_handlers::copy_file),
            )
            .route(
                "/api/files/move",
                axum_post(crate::drive::drive_handlers::move_file),
            )
            .route(
                "/api/files/open",
                axum_post(crate::drive::drive_handlers::open_file),
            )
            .route(
                "/api/files/ai/chat",
                axum_post(crate::drive::drive_handlers::ai_chat_handler),
            )
            .route(
                "/api/files/favorite",
                axum_get(crate::drive::drive_handlers::list_favorites),
            )
            .route(
                "/api/files/shared",
                axum_get(crate::drive::drive_handlers::list_shared),
            );
    }

    // Anonymous auth fallback — available regardless of directory feature
    {
        use axum::extract::State;
        use axum::response::IntoResponse;
        use std::collections::HashMap;

        async fn anonymous_auth_handler(
            State(state): State<Arc<AppState>>,
            axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
        ) -> impl IntoResponse {
            let bot_name = params.get("bot_name").cloned().unwrap_or_default();
            let existing_session_id = params.get("session_id").cloned();
            let existing_user_id = params.get("user_id").cloned();

            let user_id = existing_user_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let session_id = existing_session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let session_uuid = match uuid::Uuid::parse_str(&session_id) {
                Ok(uuid) => uuid,
                Err(_) => uuid::Uuid::new_v4(),
            };
            let user_uuid = match uuid::Uuid::parse_str(&user_id) {
                Ok(uuid) => uuid,
                Err(_) => uuid::Uuid::new_v4(),
            };

            let found_bot_id = {
                let conn = state.conn.get().ok();
                if let Some(mut db_conn) = conn {
                    use botcore::shared::models::schema::bots::dsl::*;
                    use diesel::prelude::*;
                    bots.filter(name.eq(&bot_name))
                        .select(id)
                        .first::<uuid::Uuid>(&mut db_conn)
                        .ok()
                        .unwrap_or_else(uuid::Uuid::nil)
                } else {
                    uuid::Uuid::nil()
                }
            };

            let mut final_session_id = session_id.clone();
            {
                let mut sm = state.session_manager.lock().await;
                sm.get_or_create_anonymous_user(Some(user_uuid)).ok();
                let session = sm.get_or_create_session_by_id(
                    session_uuid, user_uuid, found_bot_id, "Anonymous Chat"
                );
                if let Ok(sess) = session {
                    final_session_id = sess.id.to_string();
                }
            }

            info!("Anonymous auth for bot: {}, session: {}", bot_name, final_session_id);

            (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({
                    "user_id": user_id,
                    "session_id": final_session_id,
                    "bot_id": found_bot_id,
                    "bot_name": bot_name,
                    "status": "anonymous"
                })),
        )
    }

    api_router = api_router.route(ApiUrls::AUTH, get(anonymous_auth_handler));
    }

    api_router = crate::apps::register(api_router);

    #[cfg(feature = "meet")]
    {
        api_router = api_router.merge(crate::meet::configure());
    }

// email routes moved to base_router (crate state adapter)

#[cfg(feature = "tasks")]
{
    api_router = api_router.merge(crate::tasks::configure_task_routes(app_state.clone()));
}

    #[cfg(feature = "analytics")]
    {
        api_router = api_router.merge(crate::analytics::configure_analytics_routes(&app_state));
    }
    api_router = api_router.merge(crate::core::i18n::configure_i18n_routes());
    #[cfg(feature = "docs")]
    {
        api_router = api_router.merge(crate::docs::configure_docs_routes(&app_state));
    }
    #[cfg(feature = "paper")]
    {
        api_router = api_router.merge(crate::paper::configure_paper_routes(app_state.clone()));
    }
    // sheet routes moved to base_router (crate state adapter)
    // slides routes moved to base_router (crate state adapter)
    // video routes moved to base_router (crate state adapter)
    #[cfg(feature = "research")]
    {
        api_router = api_router.merge(crate::research::configure_research_routes(&app_state));
        api_router = api_router.merge(crate::research::configure_research_ui_routes(&app_state));
    }
    #[cfg(any(feature = "research", feature = "llm"))]
    {
        api_router = api_router.route(
            "/api/website/force-recrawl",
            post(crate::core::kb::website_crawler_service::handle_force_recrawl)
        );
    }
    // sources routes moved to base_router (crate state adapter)
    #[cfg(feature = "designer")]
    {
        api_router = api_router.merge(crate::designer::configure_designer_routes(&app_state));
        api_router = api_router.merge(crate::designer::configure_designer_ui_routes(&app_state));
    }
    #[cfg(feature = "dashboards")]
    {
        api_router = api_router.merge(crate::dashboards::configure_dashboards_routes(&app_state));
        api_router = api_router.merge(crate::dashboards::configure_dashboards_ui_routes(&app_state));
    }
    #[cfg(feature = "legal")]
    {
        let legal_pool = app_state.conn.clone();
        api_router = api_router.merge(
            crate::legal::configure_legal_routes().with_state(Arc::new(legal_pool))
        );
        api_router = api_router.merge(
            crate::legal::configure_legal_ui_routes().with_state(Arc::new(legal_pool))
        );
    }
    #[cfg(feature = "compliance")]
    {
        let compliance_pool = app_state.conn.clone();
        api_router = api_router.merge(
            crate::compliance::configure_compliance_routes().with_state(Arc::new(compliance_pool))
        );
        api_router = api_router.merge(
            crate::compliance::configure_compliance_ui_routes().with_state(Arc::new(compliance_pool))
        );
    }
    #[cfg(feature = "monitoring")]
    {
        api_router = api_router.merge(crate::monitoring::configure(&app_state));
    }
            api_router = api_router.merge(crate::security::configure_protection_routes());
        api_router = api_router.merge(crate::settings::configure_settings_routes());
#[cfg(feature = "scripting")]
{
api_router = api_router.merge(crate::basic::keywords::configure_app_server_routes());
}
#[cfg(feature = "people")]
{
api_router = api_router.merge(crate::basic::keywords::configure_db_routes());
}
#[cfg(feature = "vibe")]
{
api_router = api_router.merge(crate::vibe::configure_vibe_routes(&app_state));
}
    api_router = api_router.merge(botcore::shared::admin::configure());
    
    #[cfg(feature = "project")]
    {
        api_router = api_router.merge(crate::project::configure());
    }
    #[cfg(all(feature = "analytics", feature = "goals"))]
    {
api_router = api_router.merge(crate::analytics::goals::configure_goals_routes(&app_state));
    api_router = api_router.merge(crate::analytics::goals_ui::configure_goals_ui_routes(&app_state));
    }
    #[cfg(feature = "player")]
    {
        api_router = api_router.merge(crate::player::configure_player_routes());
    }
    #[cfg(feature = "canvas")]
    {
        api_router = api_router.merge(crate::canvas::configure_canvas_routes(&app_state));
        api_router = api_router.merge(crate::canvas::ui::configure_canvas_ui_routes(&app_state));
    }
    #[cfg(feature = "desktop")]
    {
        let desktop_state = Arc::new(crate::desktop::AppState::new());
        api_router = api_router.merge(crate::desktop::configure_routes().with_state(desktop_state));
    }
    api_router = api_router.nest("/api/directory", crate::directory::router::configure());
    api_router = api_router.nest("/api/auth", crate::directory::auth_routes::configure());
    api_router = api_router.merge(crate::directory::scim::server::configure_scim_routes());

    api_router = api_router
        .route("/api/organizations/current", get(handle_get_organization).put(handle_update_organization).post(handle_update_organization).delete(handle_delete_organization))
        .route("/api/organizations/current/settings", get(handle_get_org_settings))
        .route("/api/organizations/current/stats", get(handle_get_org_stats))
        .route("/api/organizations/current/contact", post(handle_update_organization_contact))
        .route("/api/organizations/current/branding", post(handle_update_organization_branding))
        .route("/api/organizations/current/audit", get(handle_get_org_audit))
        .route("/api/organizations/current/export", get(handle_export_org_data))
        .route("/api/admin/migrate/office365", post(handle_office365_migration));

    #[cfg(feature = "social")]
    {
        api_router = api_router.merge(crate::social::configure_social_routes(&app_state));
        api_router = api_router.merge(crate::social::ui::configure_social_ui_routes(&app_state));
    }
    #[cfg(feature = "learn")]
    {
        api_router = api_router.merge(crate::learn::ui::configure_learn_ui_routes());
    }
    // email UI routes moved to base_router (crate state adapter)
    #[cfg(feature = "meet")]
    {
        api_router = api_router.merge(crate::meet::ui::configure_meet_ui_routes());
    }
    // contacts routes moved to base_router (crate state adapter)
    #[cfg(feature = "billing")]
    {
        api_router = api_router.merge(crate::billing::billing_ui::configure_billing_routes(&app_state));
        api_router = api_router.merge(crate::billing::api::configure_billing_api_routes(&app_state));
        
    }
    
    #[cfg(feature = "whatsapp")]
    {
        api_router = api_router.merge(crate::whatsapp::configure(app_state.clone()));
    }

    #[cfg(feature = "marketing")]
    {
        api_router = api_router.merge(crate::marketing::configure_marketing_routes(&app_state));
    }

    #[cfg(feature = "telegram")]
    {
        api_router = api_router.merge(crate::telegram::configure(app_state.clone()));
    }

 #[cfg(feature = "attendant")]
 {
 api_router = api_router.merge(crate::attendance::configure_attendance_routes(&app_state));
 }

    #[cfg(feature = "deployment")]
    {
        tokio::spawn(async {
            let gateway_state = std::sync::Arc::new(crate::deployment::GatewayState::default());
            let gateway_router = crate::deployment::configure_gateway_routes(gateway_state);
            let gateway_addr = SocketAddr::from(([0, 0, 0, 0], 5860));
            match tokio::net::TcpListener::bind(gateway_addr).await {
                Ok(listener) => {
                    log::info!("Deploy Gateway Server listening on http://0.0.0.0:5860");
                    if let Err(e) = axum::serve(listener, gateway_router.into_make_service()).await {
                        log::error!("Deploy Gateway Server execution failed: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Failed to bind Deploy Gateway Server to port 5860: {}", e);
                }
            }
        });
    }

 // BotCoder IDE APIs
    api_router = api_router.merge(crate::api::editor::configure_editor_routes());
    api_router = api_router.merge(crate::api::database::configure_database_routes());
    api_router = api_router.merge(crate::api::git::configure_git_routes());
    api_router = api_router.merge(crate::api::system::configure_system_routes());
    #[cfg(feature = "browser")]
    {
        api_router = api_router.merge(
            crate::browser::api::configure_browser_routes::<crate::browser::AppStateBrowserState>()
                .with_state(Arc::new(crate::browser::AppStateBrowserState(Arc::clone(&app_state)))),
        );
    }
    #[cfg(feature = "terminal")]
    {
        api_router = api_router.merge(crate::api::terminal::configure_terminal_routes());
    }

    let site_path = app_state
        .config
        .as_ref()
        .map(|c| c.site_path.clone())
        .unwrap_or_else(|| format!("{}/sites", botcore::shared::utils::get_stack_path()));

    #[cfg(not(feature = "vibe"))]
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

    let csrf_secret = std::env::var("CSRF_SECRET").unwrap_or_else(|_| {
        info!("CSRF_SECRET not set, using default development secret");
        "dev-csrf-secret-change-in-production-minimum-32-bytes".to_string()
    });
    let csrf_config = CsrfConfig {
        exempt_paths: vec![
            "/api/health".into(),
            "/api/auth".into(),
            "/api/auth/login".into(),
            "/api/auth/refresh".into(),
            "/api/auth/bootstrap".into(),
            "/api/setup/status".into(),
            "/api/product".into(),
            "/api/manifest".into(),
            "/api/i18n".into(),
            "/api/client-errors".into(),
            "/ws".into(),
            "/ws/".into(),
            "/webhook/whatsapp".into(),
            "/webhook".into(),
        ],
        ..Default::default()
    };
    let csrf_manager = Arc::new(
        CsrfManager::new(csrf_config, csrf_secret.as_bytes())
            .expect("Failed to create CSRF manager"),
    );

    info!("Security middleware enabled: rate limiting, CSRF, security headers, panic handler, request ID tracking, authentication");

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
    if let Some(jwt_mgr) = jwt_manager {
        app_state_with_auth.jwt_manager = Some(jwt_mgr as std::sync::Arc<dyn botlib::traits::JwtService>);
    }
    app_state_with_auth.auth_provider_registry = Some(auth_provider_registry as std::sync::Arc<dyn std::any::Any + Send + Sync>);
    { let rbac: std::sync::Arc<dyn botlib::traits::RbacService> = rbac_manager.clone(); app_state_with_auth.rbac_manager = Some(rbac); }
    let app_state = Arc::new(app_state_with_auth);

    let oauth_state = std::sync::Arc::new(botcoreoauth::routes::OAuthState_ {
        conn: app_state.conn.clone(),
        base_url: format!("http://localhost:{}", port),
    });

    #[cfg(feature = "deployment")]
    let base_router = {
        let dep_pool = app_state.conn.clone();
        let dep_router: axum::Router<()> = crate::deployment::configure_deployment_routes(dep_pool);
        Router::new()
            .merge(api_router.with_state(app_state.clone()))
            .merge(dep_router)
            .nest("/", botcoreoauth::routes::configure(oauth_state))
    };
    #[cfg(not(feature = "deployment"))]
    let base_router = Router::new()
        .merge(api_router.with_state(app_state.clone()))
        .nest("/", botcoreoauth::routes::configure(oauth_state));

    #[cfg(not(feature = "vibe"))]
    let base_router = base_router
        // Static files fallback for legacy /apps/* paths
        .nest_service("/static", ServeDir::new(&site_path));

    let base_router = {
        let r = base_router;
        #[cfg(feature = "calendar")]
        {
            r.merge(crate::calendar::configure_calendar_routes(&app_state))
                .merge(crate::calendar::configure_calendar_ui_routes(&app_state))
                .merge(crate::calendar::create_caldav_router(&app_state))
        }
        #[cfg(not(feature = "calendar"))]
        r
    };

let base_router = {
    let r = base_router;
    #[cfg(feature = "mail")]
    {
        r.merge(crate::email::routes::configure(app_state.clone()))
    }
    #[cfg(not(feature = "mail"))]
    r
};

let base_router = {
    let r = base_router;
    #[cfg(feature = "sheet")]
    {
        r.merge(crate::sheet::configure_sheet_routes(app_state.clone()))
    }
    #[cfg(not(feature = "sheet"))]
    r
};

let base_router = {
    let r = base_router;
    #[cfg(feature = "slides")]
    {
        r.merge(crate::slides::configure_slides_routes(app_state.clone()))
    }
    #[cfg(not(feature = "slides"))]
    r
};

let base_router = {
    let r = base_router;
    #[cfg(feature = "plan")]
    {
        r.merge(crate::plan::configure_plan_routes())
    }
    #[cfg(not(feature = "plan"))]
    r
};

let base_router = {
    let r = base_router;
    #[cfg(feature = "video")]
    {
        r.merge(crate::video::configure_video_routes(app_state.clone()))
        .merge(crate::video::configure_video_ui_routes(app_state.clone()))
    }
    #[cfg(not(feature = "video"))]
    r
};

let base_router = {
    let r = base_router;
    #[cfg(feature = "sources")]
    {
        r.merge(crate::sources::configure_sources_routes(app_state.clone()))
        .merge(crate::sources::configure_sources_ui_routes(app_state.clone()))
    }
    #[cfg(not(feature = "sources"))]
    r
};

let base_router = {
    let r = base_router;
    #[cfg(feature = "people")]
    {
        r.merge(crate::contacts::configure_crm_routes(app_state.clone()))
        .merge(crate::contacts::configure_crm_api_routes(app_state.clone()))
    }
    #[cfg(not(feature = "people"))]
    r
};

let base_router = {
    let r = base_router;
    #[cfg(feature = "workspaces")]
    {
        r.merge(crate::workspaces::configure_workspaces_routes(&app_state))
        .merge(crate::workspaces::configure_workspaces_ui_routes(&app_state))
    }
    #[cfg(not(feature = "workspaces"))]
    r
};

    let base_router = {
        let r = base_router;
        #[cfg(feature = "billing")]
        {
            r.merge(crate::products::configure_products_routes(&app_state))
                .merge(crate::products::configure_products_api_routes(&app_state))
        }
        #[cfg(not(feature = "billing"))]
        r
    };

    let base_router = {
        let r = base_router;
        #[cfg(feature = "tickets")]
        {
            r.merge(crate::tickets::configure_tickets_routes(&app_state))
                .merge(crate::tickets::configure_tickets_ui_routes(&app_state))
        }
        #[cfg(not(feature = "tickets"))]
        r
    };

    let base_router = {
        let r = base_router;
        #[cfg(feature = "people")]
        {
            r.merge(crate::people::configure_people_routes(&app_state))
                .merge(crate::people::configure_people_ui_routes(&app_state))
        }
        #[cfg(not(feature = "people"))]
        r
    };

    let base_router = {
        let r = base_router;
        #[cfg(feature = "attendant")]
        {
            r.merge(crate::attendant::configure_attendant_routes(&app_state))
                .merge(crate::attendant::configure_attendant_ui_routes(&app_state))
        }
        #[cfg(not(feature = "attendant"))]
        r
    };

    // Add UI routes based on availability
    let app_with_ui = if ui_path_exists {
        base_router
            .nest_service("/auth", ServeDir::new(format!("{}/auth", ui_path)))
            .nest_service("/suite", ServeDir::new(&ui_path))
            .nest_service("/themes", ServeDir::new(format!("{}/../themes", ui_path)))
            .fallback_service(
                tower_http::services::ServeFile::new(format!("{}/desktop.html", ui_path))
            )
    } else if use_embedded_ui {
        base_router.merge(crate::embedded_ui::embedded_ui_router())
    } else {
        base_router
    };

    // Clone rbac_manager for use in middleware
    let rbac_manager_for_middleware = Arc::clone(&rbac_manager);

    async fn csrf_cookie_injector(
        manager: Arc<CsrfManager>,
        request: axum::http::Request<axum::body::Body>,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        let mut response = next.run(request).await;
        let has_csrf = response
            .headers()
            .get_all(axum::http::header::SET_COOKIE)
            .iter()
            .any(|v| v.to_str().ok().map_or(false, |s| s.contains("csrf_token")));
        if !has_csrf {
            let token = manager.generate_signed_token();
            let cookie = manager.build_cookie(&token);
            if let Ok(cv) = cookie.parse::<axum::http::HeaderValue>() {
                response.headers_mut().insert(axum::http::header::SET_COOKIE, cv);
            }
        }
        response
    }

    let app =
        app_with_ui
            // W3C Distributed Tracing middleware — injects/parses traceparent headers
            .layer(axum::middleware::from_fn({
                let name = app_state.config.as_ref()
                    .map(|c| format!("botserver:{}", c.server.host))
                    .unwrap_or_else(|| "botserver".to_string());
                move |req, next| {
                    let name = name.clone();
                    async move { botcore::tracing::tracing_middleware_fn(name, req, next).await }
                }
            }))
            // CSRF cookie injector — outermost so it runs last (on response) to set cookies
            .layer(axum::middleware::from_fn({
                let csrf_manager = csrf_manager.clone();
                move |req, next| {
                    let mgr = csrf_manager.clone();
                    async move { csrf_cookie_injector(mgr, req, next).await }
                }
            }))
            // Sec middleware stack (order matters — last added is outermost/runs first)
            .layer(axum::middleware::from_fn(security_headers_middleware))
            .layer(security_headers_extension)
            .layer(rate_limit_extension)
            // Request ID tracking for all requests
            .layer(axum::middleware::from_fn(request_id_middleware))
            // CSRF validation middleware — runs before auth
            .layer(axum::middleware::from_fn({
                let csrf_manager = csrf_manager.clone();
                move |req, next| {
                    let mgr = csrf_manager.clone();
                    async move { csrf_middleware(mgr, req, next).await }
                }
            }))
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

    let stack = botcore::shared::utils::get_stack_path();
    let cert_dir = std::path::PathBuf::from(format!("{}/conf/system/certificates", stack));
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
            info!("Shutting down HTTPS server - draining active connections (10s timeout)...");
            handle_clone.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
            info!("HTTPS graceful shutdown initiated, waiting for connections to drain...");
        });

        axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(app.into_make_service())
            .await
            .map_err(|e| {
                error!("HTTPS server failed on {}: {}", addr, e);
                std::io::Error::other(e)
            })?;
    } else {
        if disable_tls {
            info!("TLS disabled via BOTSERVER_DISABLE_TLS environment variable");
        } else {
            info!("TLS certificates not found, using HTTP");
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
        info!("Server ready - shutdown via SIGINT (Ctrl+C) or SIGTERM (systemctl stop)");
        let result = axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await;
        match &result {
            Ok(()) => info!("HTTP server shut down gracefully"),
            Err(e) => error!("HTTP server shutdown with error: {}", e),
        }
        result.map_err(std::io::Error::other)?;
    }

    Ok(())
}

async fn handle_get_organization(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "No auth service"
        })))
    })?.lock().await;

    match auth_service.list_organizations().await {
        Ok(data) => {
            let orgs = data.as_array().cloned().unwrap_or_default();
            if let Some(org) = orgs.first() {
                Ok(Json(org.clone()))
            } else {
                Ok(Json(serde_json::json!({
                    "name": "Default Organization",
                    "id": "default",
                    "description": ""
                })))
            }
        }
        Err(e) => {
            log::warn!("Failed to list organizations: {}", e);
            Ok(Json(serde_json::json!({
                "name": "Default Organization",
                "id": "default",
                "description": ""
            })))
        }
    }
}

async fn handle_get_org_settings(
    axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let stack = botcore::shared::utils::get_stack_path();
    let config_path = format!("{}/conf/directory/org-settings.json", stack);

    let settings = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    Ok(Json(settings))
}

async fn handle_get_org_stats(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let mut users_total: i64 = 0;
    let mut bots_total: i64 = 0;

    if let Some(auth) = state.auth_service.as_ref() {
        let auth_service = auth.lock().await;
        if let Ok(data) = auth_service.list_users(1000, 0).await {
            users_total = data.as_array().map(|a| a.len() as i64).unwrap_or(0);
        }
    }

    if let Ok(mut conn) = state.conn.get() {
        use botcore::shared::models::schema::bots::dsl::*;
        if let Ok(count) = bots.count().get_result::<i64>(&mut conn) {
            bots_total = count;
        }
    }

    let mut kb_total: i64 = 0;
    let mut storage_bytes: i64 = 0;

    if let Ok(mut conn) = state.conn.get() {
        use botcore::shared::models::schema::drive_files::dsl::*;
        if let Ok(count) = drive_files
            .filter(file_type.eq("gbkb"))
            .count()
            .get_result::<i64>(&mut conn)
        {
            kb_total = count;
        }
        if let Ok(bytes) = drive_files
            .filter(file_size.is_not_null())
            .select(diesel::dsl::sql::<diesel::sql_types::BigInt>("COALESCE(SUM(file_size), 0)"))
            .first::<i64>(&mut conn)
        {
            storage_bytes = bytes;
        }
    }

    let storage_mb_val = storage_bytes / 1_048_576;

    Ok(Json(serde_json::json!({
        "users": { "used": users_total, "limit": 50 },
        "bots": { "used": bots_total, "limit": 20 },
        "kb_documents": { "used": kb_total, "limit": 500 },
        "storage_mb": { "used": storage_mb_val, "limit": 5120 }
    })))
}

async fn handle_delete_organization(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Organization deletion requested");

    let auth = state.auth_service.as_ref().ok_or_else(|| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "No auth service"
        })))
    })?;

    let auth_service = auth.lock().await;

    let orgs_data = auth_service.list_organizations().await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Failed to list organizations: {}", e)
        }))))?;

    let orgs = orgs_data.as_array().cloned().unwrap_or_default();
    let org_id = orgs.first()
        .and_then(|o| o.get("id").or_else(|| o.get("orgId")))
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    if org_id == "default" {
        return Err((axum::http::StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Cannot delete the default organization"
        }))));
    }

    let _ = auth_service.http_delete(format!("{}/v2/organizations/{}", auth_service.api_url(), org_id)).await;

    let stack = botcore::shared::utils::get_stack_path();
    let settings_path = std::path::PathBuf::from(format!("{}/conf/directory/org-settings.json", stack));
    let _ = std::fs::remove_file(settings_path);

    Ok(Json(serde_json::json!({"success": true, "message": format!("Organization {} deleted", org_id)})))
}

async fn handle_get_org_audit(
    axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let stack = botcore::shared::utils::get_stack_path();
    let log_path = std::path::PathBuf::from(format!("{}/conf/directory/audit-log.json", stack));

    let entries: Vec<serde_json::Value> = std::fs::read_to_string(&log_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let recent: Vec<serde_json::Value> = entries.into_iter().rev().take(50).collect();

    Ok(Json(serde_json::json!({
        "entries": recent,
        "total": recent.len()
    })))
}

async fn handle_export_org_data(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Organization data export requested");

    let mut export = serde_json::json!({
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "users": [],
        "bots": [],
        "settings": {}
    });

    if let Some(auth) = state.auth_service.as_ref() {
        let auth_service = auth.lock().await;
        if let Ok(data) = auth_service.list_users(1000, 0).await {
            export["users"] = data;
        }
    }

    if let Ok(mut conn) = state.conn.get() {
        use botcore::shared::models::schema::bots::dsl::*;
        if let Ok(bot_list) = bots.limit(100).load::<botcore::shared::models::core::Bot>(&mut conn) {
            let bot_names: Vec<serde_json::Value> = bot_list.iter().map(|b| {
                serde_json::json!({"id": b.id.to_string(), "name": b.name})
            }).collect();
            export["bots"] = serde_json::json!(bot_names);
        }
    }

    let stack = botcore::shared::utils::get_stack_path();
    let settings_path = format!("{}/conf/directory/org-settings.json", stack);
    if let Ok(settings_str) = std::fs::read_to_string(&settings_path) {
        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&settings_str) {
            export["settings"] = settings;
        }
    }

    let export_path = format!("{}/tmp/org-export-{}.json", stack, chrono::Utc::now().timestamp());
    let _ = std::fs::write(&export_path, serde_json::to_string_pretty(&export).unwrap_or_default());

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Export complete",
        "download_url": format!("/api/files/download?path={}", export_path)
    })))
}

async fn handle_update_organization(
    axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Organization settings update: {:?}", body);

    let stack = botcore::shared::utils::get_stack_path();
    let config_path = format!("{}/conf/directory/org-settings.json", stack);

    if let Some(parent) = std::path::Path::new(&config_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::error!("Failed to create config directory: {}", e);
        }
    }

    let existing = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let mut merged = existing;
    if let (Some(obj), Some(patch)) = (merged.as_object_mut(), body.as_object()) {
        for (k, v) in patch {
            obj.insert(k.clone(), v.clone());
        }
    }

    if let Err(e) = std::fs::write(&config_path, serde_json::to_string_pretty(&merged).unwrap_or_default()) {
        log::error!("Failed to save organization settings: {}", e);
        return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to save settings"
        }))));
    }

    append_audit_log("settings_updated", &body.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>().join(",")).unwrap_or_default());

    Ok(Json(serde_json::json!({"success": true})))
}

fn append_audit_log(action: &str, detail: &str) {
    let stack = botcore::shared::utils::get_stack_path();
    let log_path = format!("{}/conf/directory/audit-log.json", stack);

    if let Some(parent) = std::path::Path::new(&log_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut entries: Vec<serde_json::Value> = std::fs::read_to_string(&log_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    entries.push(serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "actor": "admin",
        "action": action,
        "detail": detail
    }));

    if entries.len() > 500 {
        entries = entries.split_off(entries.len() - 500);
    }

    let _ = std::fs::write(&log_path, serde_json::to_string_pretty(&entries).unwrap_or_default());
}

async fn handle_update_organization_contact(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Organization contact update: {:?}", body);
    let result = handle_update_organization(axum::extract::State(state), Json(body.clone())).await?;
    append_audit_log("contact_updated", &body.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>().join(",")).unwrap_or_default());
    Ok(result)
}

async fn handle_update_organization_branding(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Organization branding update: {:?}", body);
    let result = handle_update_organization(axum::extract::State(state), Json(body.clone())).await?;
    append_audit_log("branding_updated", &body.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>().join(",")).unwrap_or_default());
    Ok(result)
}

#[derive(serde::Deserialize)]
struct Office365MigrationRequest {
    tenant_id: String,
    client_id: String,
    client_secret: String,
    sync_mode: Option<String>,
}

async fn handle_office365_migration(
    axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<Office365MigrationRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Office 365 migration requested for tenant: {}", req.tenant_id);

    let mode = match req.sync_mode.as_deref() {
        Some("delta") => crate::directory::scim::sync::SyncMode::Delta,
        _ => crate::directory::scim::sync::SyncMode::Full,
    };

    let config = crate::directory::scim::sync::AzureAdConfig {
        tenant_id: req.tenant_id,
        client_id: req.client_id,
        client_secret: req.client_secret,
        sync_mode: mode,
    };

    let syncer = crate::directory::scim::sync::AzureAdSyncer::new(config);

    let auth_service = _state.auth_service.as_ref().ok_or_else(|| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "No auth service available"
        })))
    })?.lock().await;

    match syncer.sync(&*auth_service).await {
        Ok(result) => {
            info!("Office 365 migration complete: {:?}", result);
            Ok(Json(serde_json::json!({
                "success": true,
                "groups_created": result.groups_created,
                "groups_updated": result.groups_updated,
                "users_mapped": result.users_mapped,
                "users_created": result.users_created,
                "users_updated": result.users_updated,
                "errors": result.errors,
                "duration_ms": result.duration_ms
            })))
        }
        Err(e) => {
            log::error!("Office 365 migration failed: {}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Migration failed: {}", e)
                }))
            ))
        }
    }
}
