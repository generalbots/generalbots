//! HTTP server initialization and routing

use axum::Router;
use std::sync::Arc;
use std::net::SocketAddr;
use log::{error, info, warn};
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;
use crate::security::{
    create_rate_limit_layer, create_security_headers_layer, request_id_middleware,
    security_headers_middleware, csrf_middleware,
    CsrfConfig, CsrfManager, CombinedRateLimiter,
    HttpRateLimitConfig, PanicHandlerConfig, SecurityHeadersConfig,
};
use botcore::shared::state::AppState;
use botlib::SystemLimits;

use super::routes::{security_setup, api_setup, sub_router};

pub async fn run_axum_server(
    app_state: Arc<AppState>,
    port: u16,
    _worker_count: usize,
) -> std::io::Result<()> {
    let sec = security_setup::setup_security(&app_state).await;

    use crate::core::product::PRODUCT_CONFIG;
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

    let mut api_router = api_setup::setup_api_routes();

    let sub_router = sub_router::build_sub_router(&app_state, port, &mut api_router);

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

    api_router = api_setup::add_base_api_routes(api_router);

    let site_path = app_state
        .config
        .as_ref()
        .map(|c| c.site_path.clone())
        .unwrap_or_else(|| format!("{}/sites", botcore::shared::utils::get_stack_path()));

    #[cfg(not(feature = "vibe"))]
    info!("Serving apps from: {}", site_path);

    let http_rate_config = HttpRateLimitConfig::api();
    let system_limits = SystemLimits::default();
    let (rate_limit_extension, _rate_limiter) =
        create_rate_limit_layer(http_rate_config, system_limits);

    let security_headers_config = SecurityHeadersConfig::default();
    let security_headers_extension = create_security_headers_layer(security_headers_config.clone());

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
            "/api/health".into(), "/api/auth".into(), "/api/auth/login".into(),
            "/api/auth/refresh".into(), "/api/auth/bootstrap".into(), "/api/setup/status".into(),
            "/api/product".into(), "/api/manifest".into(), "/api/i18n".into(),
            "/api/client-errors".into(), "/api/cloud/auth*".into(), "/api/catalog".into(), "/ws".into(),
            "/ws/".into(), "/webhook/whatsapp".into(), "/webhook".into(),
        ],
        ..Default::default()
    };
    let csrf_manager = {
        let fallback_secret = "dev-csrf-fallback-32byte-minimum!!";
        let secret = if csrf_secret.len() >= 32 {
            csrf_secret.as_bytes()
        } else {
            log::warn!("CSRF secret too short ({} bytes), using fallback", csrf_secret.len());
            fallback_secret.as_bytes()
        };
        match CsrfManager::new(csrf_config, secret) {
            Ok(manager) => Arc::new(manager),
            Err(e) => {
                log::error!("Failed to create CSRF manager: {e}");
                return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("CSRF init failed: {e}")));
            }
        }
    };

    info!("Security middleware enabled: rate limiting, CSRF, security headers, panic handler, request ID tracking, authentication");

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
        info!("External UI folder not found at '{}', using embedded UI", ui_path);
        let file_count = crate::embedded_ui::list_embedded_files().len();
        info!("Embedded UI contains {} files", file_count);
    } else {
        warn!("No UI available: folder '{}' not found and no embedded UI", ui_path);
    }

    let mut app_state_with_auth = (*app_state).clone();
    if let Some(jwt_mgr) = sec.jwt_manager {
        app_state_with_auth.jwt_manager = Some(jwt_mgr as Arc<dyn botlib::traits::JwtService>);
    }
    app_state_with_auth.auth_provider_registry = Some(sec.auth_provider_registry as Arc<dyn std::any::Any + Send + Sync>);
    {
        let rbac: Arc<dyn botlib::traits::RbacService> = sec.rbac_manager.clone();
        app_state_with_auth.rbac_manager = Some(rbac);
    }
    let app_state = Arc::new(app_state_with_auth);

    let oauth_state = Arc::new(botcoreoauth::routes::OAuthState_ {
        conn: app_state.conn.clone(),
        base_url: format!("http://localhost:{}", port),
    });

    let base_router = build_base_router(app_state.clone(), api_router, sub_router, oauth_state, &site_path);

    let app = apply_middleware(base_router, app_state.clone(), sec.cors, csrf_manager, panic_config, security_headers_extension, rate_limit_extension, sec.rbac_manager, sec.auth_middleware_state);

    listen(app, port).await
}

fn build_base_router(
    app_state: Arc<AppState>,
    api_router: Router<Arc<AppState>>,
    sub_router: Router<()>,
    oauth_state: Arc<botcoreoauth::routes::OAuthState_>,
    site_path: &str,
) -> Router {
    #[cfg(feature = "deployment")]
    let base_router = {
        Router::new()
            .merge(api_router.with_state(app_state.clone()))
            .merge(sub_router)
            .nest("/", botcoreoauth::routes::configure(oauth_state))
    };
    #[cfg(not(feature = "deployment"))]
    let base_router = Router::new()
        .merge(api_router.with_state(app_state.clone()))
        .merge(sub_router)
        .nest("/", botcoreoauth::routes::configure(oauth_state));

    #[cfg(feature = "calendar")]
    let base_router = add_calendar_routes(base_router, &app_state);
    #[cfg(feature = "mail")]
    let base_router = add_mail_routes(base_router, &app_state);
    #[cfg(feature = "sheet")]
    let base_router = add_sheet_routes(base_router);
    #[cfg(feature = "slides")]
    let base_router = add_slides_routes(base_router, &app_state);
    #[cfg(feature = "plan")]
    let base_router = add_plan_routes(base_router);
    #[cfg(feature = "video")]
    let base_router = add_video_routes(base_router, &app_state);
    #[cfg(feature = "workspaces")]
    let base_router = add_workspaces_routes(base_router, &app_state);
    #[cfg(feature = "billing")]
    let base_router = add_products_routes(base_router, &app_state);
    #[cfg(feature = "tickets")]
    let base_router = add_tickets_routes(base_router, &app_state);
    #[cfg(feature = "people")]
    let base_router = add_people_routes(base_router, &app_state);
    #[cfg(feature = "attendant")]
    let base_router = add_attendant_routes(base_router, &app_state);
    #[cfg(not(feature = "vibe"))]
    let base_router = base_router
        .nest_service("/static", ServeDir::new(site_path));

    let ui_path = std::env::var("BOTUI_PATH").unwrap_or_else(|_| {
        if std::path::Path::new("./botui/ui/suite").exists() { "./botui/ui/suite".to_string() }
        else if std::path::Path::new("../botui/ui/suite").exists() { "../botui/ui/suite".to_string() }
        else { "./botui/ui/suite".to_string() }
    });
    let ui_path_exists = std::path::Path::new(&ui_path).exists();
    let use_embedded_ui = !ui_path_exists && crate::embedded_ui::has_embedded_ui();

    if ui_path_exists {
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
    }
}

fn apply_middleware(
    app: Router,
    app_state: Arc<AppState>,
    cors: tower_http::cors::CorsLayer,
    csrf_manager: Arc<CsrfManager>,
    panic_config: PanicHandlerConfig,
    security_headers_extension: axum::Extension<SecurityHeadersConfig>,
    rate_limit_extension: axum::Extension<Arc<CombinedRateLimiter>>,
    rbac_manager: Arc<crate::security::RbacManager>,
    auth_middleware_state: crate::security::AuthMiddlewareState,
) -> Router {
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
            .any(|v| v.to_str().ok().is_some_and(|s| s.contains("csrf_token")));
        if !has_csrf {
            let token = manager.generate_signed_token();
            let cookie = manager.build_cookie(&token);
            if let Ok(cv) = cookie.parse::<axum::http::HeaderValue>() {
                response.headers_mut().insert(axum::http::header::SET_COOKIE, cv);
            }
        }
        response
    }

    app
        .layer(axum::middleware::from_fn({
            let name = app_state.config.as_ref()
                .map(|c| format!("botserver:{}", c.server.host))
                .unwrap_or_else(|| "botserver".to_string());
            move |req, next| {
                let name = name.clone();
                async move { botcore::tracing::tracing_middleware_fn(name, req, next).await }
            }
        }))
        .layer(axum::middleware::from_fn({
            let csrf_manager = csrf_manager.clone();
            move |req, next| {
                let mgr = csrf_manager.clone();
                async move { csrf_cookie_injector(mgr, req, next).await }
            }
        }))
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(security_headers_extension)
        .layer(rate_limit_extension)
        .layer(axum::middleware::from_fn(request_id_middleware))
        .layer(axum::middleware::from_fn({
            let csrf_manager = csrf_manager.clone();
            move |req, next| {
                let mgr = csrf_manager.clone();
                async move { csrf_middleware(mgr, req, next).await }
            }
        }))
        .layer(axum::middleware::from_fn(
            move |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                let rbac = rbac_manager.clone();
                async move { crate::security::rbac_middleware_fn(req, next, rbac).await }
            },
        ))
        .layer(axum::middleware::from_fn(
            move |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                let state = auth_middleware_state.clone();
                async move {
                    crate::security::auth_middleware_with_providers(req, next, state).await
                }
            },
        ))
        .layer(axum::middleware::from_fn(move |req, next| {
            let config = panic_config.clone();
            async move {
                crate::security::panic_handler_middleware_with_config(req, next, &config).await
            }
        }))
        .layer(axum::Extension(app_state))
        .layer(cors)
        .layer(axum::middleware::from_fn(crate::security::strip_proxy_cors_middleware))
        .layer(TraceLayer::new_for_http())
}

#[cfg(feature = "calendar")]
fn add_calendar_routes(r: Router, s: &Arc<AppState>) -> Router {
    r.merge(crate::calendar::configure_calendar_routes().with_state(Arc::new(s.conn.clone())))
        .merge(crate::calendar::configure_calendar_ui_routes().with_state(Arc::new(s.conn.clone())))
        .merge(crate::calendar::create_caldav_router().with_state(Arc::new(s.conn.clone())))
}

#[cfg(feature = "mail")]
fn add_mail_routes(r: Router, s: &Arc<AppState>) -> Router {
    let email_state = crate::email::models::AppState {
        pool: Arc::new(s.conn.clone()),
        get_default_bot: Arc::new(|_conn: &mut diesel::PgConnection| (uuid::Uuid::nil(), "default".to_string())),
        secrets_provider: Arc::new(|_key: &str| Err("secrets not available".to_string())),
    };
    r.merge(crate::email::routes::configure(Arc::new(email_state)))
}

#[cfg(feature = "sheet")]
fn add_sheet_routes(r: Router) -> Router {
    r.merge(crate::sheet::routes::configure_sheet_routes().with_state(Arc::new(crate::sheet::state::SheetState { drive: None })))
}

#[cfg(feature = "slides")]
fn add_slides_routes(r: Router, s: &Arc<AppState>) -> Router {
    r.merge(crate::slides::configure_slides_routes(s.clone()))
}

#[cfg(feature = "plan")]
fn add_plan_routes(r: Router) -> Router {
    r.merge(crate::plan::configure_plan_routes())
}

#[cfg(feature = "video")]
fn add_video_routes(r: Router, s: &Arc<AppState>) -> Router {
    r.merge(crate::video::configure_video_routes().with_state(Arc::new(botvideo::routes::AppState { conn: s.conn.clone(), cache: None })))
        .merge(crate::video::configure_video_ui_routes().with_state(Arc::new(botvideo::routes::AppState { conn: s.conn.clone(), cache: None })))
}

#[cfg(feature = "workspaces")]
fn add_workspaces_routes(r: Router, s: &Arc<AppState>) -> Router {
    let make_state = || Arc::new(botworkspaces::WorkspacesState {
        pool: Arc::new(s.conn.clone()),
        get_default_bot: (|_conn: &mut diesel::PgConnection| uuid::Uuid::nil()) as fn(&mut diesel::PgConnection) -> uuid::Uuid,
    });
    r.merge(crate::workspaces::configure_workspaces_routes().with_state(make_state()))
        .merge(crate::workspaces::configure_workspaces_ui_routes().with_state(make_state()))
}

#[cfg(feature = "billing")]
fn add_products_routes(r: Router, s: &Arc<AppState>) -> Router {
    let make_state = || Arc::new(botproducts::ProductsState {
        pool: Arc::new(s.conn.clone()),
        get_default_bot: Some((|_conn: &mut diesel::PgConnection| uuid::Uuid::nil()) as fn(&mut diesel::PgConnection) -> uuid::Uuid),
    });
    r.merge(crate::products::configure_products_routes().with_state(make_state()))
        .merge(crate::products::configure_products_api_routes().with_state(make_state()))
}

#[cfg(feature = "tickets")]
fn add_tickets_routes(r: Router, s: &Arc<AppState>) -> Router {
    let make_state = || Arc::new(bottickets::TicketsState {
        pool: Arc::new(s.conn.clone()),
        get_default_bot: (|_conn: &mut diesel::PgConnection| uuid::Uuid::nil()) as fn(&mut diesel::PgConnection) -> uuid::Uuid,
    });
    r.merge(crate::tickets::configure_tickets_routes().with_state(make_state()))
        .merge(crate::tickets::ui::configure_tickets_ui_routes().with_state(make_state()))
}

#[cfg(feature = "people")]
fn add_people_routes(r: Router, s: &Arc<AppState>) -> Router {
    let people_state = Arc::new(crate::people::PeopleState {
        pool: Arc::new(s.conn.clone()),
        get_default_bot: Arc::new(|_conn: &mut diesel::PgConnection| uuid::Uuid::nil()),
    });
    r.merge(crate::people::configure_people_routes().with_state(people_state.clone()))
        .merge(crate::people::ui::configure_people_ui_routes().with_state(people_state))
}

#[cfg(feature = "attendant")]
fn add_attendant_routes(r: Router, s: &Arc<AppState>) -> Router {
    r.merge(crate::attendant::configure_attendant_routes().with_state(Arc::new(botattendant::AttendantConfig {
        pool: Arc::new(s.conn.clone()),
        get_default_bot: (|_conn: &mut diesel::PgConnection| uuid::Uuid::nil()) as fn(&mut diesel::PgConnection) -> uuid::Uuid,
    })))
    .merge(crate::attendant::configure_attendant_ui_routes().with_state(Arc::new(botattendant::AttendantConfig {
        pool: Arc::new(s.conn.clone()),
        get_default_bot: (|_conn: &mut diesel::PgConnection| uuid::Uuid::nil()) as fn(&mut diesel::PgConnection) -> uuid::Uuid,
    })))
}

pub(crate) async fn listen(app: Router, port: u16) -> std::io::Result<()> {
    crate::main_module::listen_impl::listen(app, port).await
}
