use super::*;
use std::net::SocketAddr;

pub async fn run_axum_server(
    app_state: Arc<AppState>,
    port: u16,
    _worker_count: usize,
) -> std::io::Result<()> {
    let sec = security_setup::setup_security(&app_state).await;

    use crate::core::product::PRODUCT_CONFIG;
    {
        let config = PRODUCT_CONFIG.read().unwrap_or_else(|e| {
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

    // External drives (OneDrive / Google Drive): create the two tables once at
    // boot, then let the scheduler keep the mirrored listings fresh. Both are
    // best-effort — the tab degrades to "not connected" if they fail, which is
    // better than refusing to serve the rest of the API.
    match app_state.conn.get() {
        Ok(mut conn) => {
            if let Err(e) = botdrive::external::ensure_schema(&mut conn) {
                error!("External drive schema could not be created: {e}");
            }
        }
        Err(e) => error!("External drive schema check skipped, pool unavailable: {e}"),
    }
    botdrive::external::engine::spawn_scheduler(app_state.conn.clone());

    // Operational view (#2116): fold request counters into
    // `ops_request_metrics` every 30 s and record this boot as a deploy
    // marker when the running build differs from the last one.
    #[cfg(feature = "analytics")]
    {
        botanalytics::ops_metrics::spawn_flusher(std::sync::Arc::new(app_state.conn.clone()));
        botanalytics::ops_metrics::record_deploy_marker(std::sync::Arc::new(
            app_state.conn.clone(),
        ));
    }

    let sub_router = sub_router::build_sub_router(&app_state, port, &mut api_router).await;

    #[cfg(feature = "deployment")]
    {
        tokio::spawn(async {
            let gateway_state = std::sync::Arc::new(crate::deployment::GatewayState::default());
            let gateway_router = crate::deployment::configure_gateway_routes(gateway_state);
            // IMP-09: this listener is a separate `axum::serve`, so it needs
            // the layer explicitly — the main router's layer never reaches it.
            let gateway_router = with_security_headers(
                gateway_router,
                SecurityHeadersConfig::from_env(),
            );
            let gateway_addr = SocketAddr::from(([0, 0, 0, 0], 5860));
            match tokio::net::TcpListener::bind(gateway_addr).await {
                Ok(listener) => {
                    log::info!("Deploy Gateway Server listening on http://0.0.0.0:5860");
                    // IMP-07 — the gateway's rate limiter keys its per-client
                    // budget on `ConnectInfo<SocketAddr>`, which axum only
                    // inserts for this make-service flavour.
                    let service = gateway_router
                        .into_make_service_with_connect_info::<SocketAddr>();
                    if let Err(e) = axum::serve(listener, service).await
                    {
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

    // IMP-09 header set, shared with botui: HSTS is on unless
    // `GB_SECURITY_HSTS=off` (plain-HTTP local dev only — see
    // `botsecurity_core::headers::HSTS_ENV`).
    let security_headers_config = SecurityHeadersConfig::from_env();

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
        // Single-user desktop (`no_security=true`): there is no account
        // session or auth cookie for CSRF to protect, and the double-submit
        // cookie cannot work because the token cookie is HttpOnly while the
        // desktop sends no bearer token to skip the check. Leaving CSRF on
        // rejected every local POST/DELETE — file uploads (#1840), downloads
        // (#1834) and folder operations — with 403. Normal authenticated
        // deployments keep CSRF enabled.
        enabled: !sec.no_security,
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
            // Peer registration is machine-to-machine and authenticates with
            // the optional GB_PEER_TOKEN bearer header, not a CSRF cookie.
            "/api/peer/*".into(),
            "/api/cloud/auth*".into(),
            "/api/sheet*".into(),
            "/suite/sheet*".into(),
            "/api/auth/cloud-sso".into(),
            "/api/auth/suite-sso".into(),
            "/api/auth/unified-login".into(),
            "/api/catalog".into(),
            "/ws".into(),
            "/ws/".into(),
            "/webhook/whatsapp".into(),
            "/api/whatsapp/webhook".into(),
            "/api/facebook/webhook".into(),
            "/webhook".into(),
            // Inbound channel webhooks arrive from the provider, which cannot
            // hold a CSRF token; each handler proves the call itself (#1327).
            "/webhook/telegram*".into(),
            "/api/instagram/webhook".into(),
            "/api/msteams/messages".into(),
            // CalDAV clients (Thunderbird, Apple Calendar, DAVx5) authenticate
            // with HTTP Basic and their write verbs — PROPFIND, REPORT, PUT,
            // DELETE — are not in the exempt method list, so a CSRF check
            // rejected every discovery request and no client could sync. The
            // DAV router authenticates and authorizes each request itself
            // (#1335).
            "/caldav*".into(),
            "/.well-known/caldav".into(),
        ],
        ..Default::default()
    };
    let csrf_manager = {
        let fallback_secret = "dev-csrf-fallback-32byte-minimum!!";
        let secret = if csrf_secret.len() >= 32 {
            csrf_secret.as_bytes()
        } else {
            log::warn!(
                "CSRF secret too short ({} bytes), using fallback",
                csrf_secret.len()
            );
            fallback_secret.as_bytes()
        };
        match CsrfManager::new(csrf_config, secret) {
            Ok(manager) => Arc::new(manager),
            Err(e) => {
                log::error!("Failed to create CSRF manager: {e}");
                return Err(std::io::Error::other(format!("CSRF init failed: {e}")));
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

    let mut app_state_with_auth = (*app_state).clone();
    if let Some(jwt_mgr) = sec.jwt_manager {
        app_state_with_auth.jwt_manager = Some(jwt_mgr as Arc<dyn botlib::traits::JwtService>);
    }
    app_state_with_auth.auth_provider_registry =
        Some(sec.auth_provider_registry as Arc<dyn std::any::Any + Send + Sync>);
    {
        let rbac: Arc<dyn botlib::traits::RbacService> = sec.rbac_manager.clone();
        app_state_with_auth.rbac_manager = Some(rbac);
    }
    let app_state = Arc::new(app_state_with_auth);

    let oauth_state = Arc::new(botcoreoauth::routes::OAuthState_ {
        conn: app_state.conn.clone(),
        base_url: format!("http://localhost:{}", port),
    });

    let base_router = build_base_router(
        app_state.clone(),
        api_router,
        sub_router,
        oauth_state,
        &site_path,
    );

    let app = apply_middleware(
        base_router,
        app_state.clone(),
        ServerLayers {
            cors: sec.cors,
            csrf_manager,
            panic_config,
            security_headers: security_headers_config,
            rate_limit: rate_limit_extension,
            rbac_manager: sec.rbac_manager,
            auth_middleware_state: sec.auth_middleware_state,
        },
    );

    listen(app, port).await
}
pub(super) fn build_base_router(
    app_state: Arc<AppState>,
    api_router: Router<Arc<AppState>>,
    sub_router: Router<()>,
    oauth_state: Arc<botcoreoauth::routes::OAuthState_>,
    site_path: &str,
) -> Router {
    #[cfg(feature = "vibe")]
    let _ = site_path;
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
    #[cfg(feature = "slides")]
    let base_router = add_slides_routes(base_router, &app_state);
    #[cfg(feature = "plan")]
    let base_router = add_plan_routes(base_router, &app_state);
    #[cfg(feature = "video")]
    let base_router = add_video_routes(base_router, &app_state);
    #[cfg(feature = "workspaces")]
    let base_router = add_workspaces_routes(base_router, &app_state);
    #[cfg(feature = "products")]
    let base_router = add_products_routes(base_router, &app_state);
    #[cfg(feature = "tickets")]
    let base_router = add_tickets_routes(base_router, &app_state);
    #[cfg(feature = "people")]
    let base_router = add_people_routes(base_router, &app_state);
    #[cfg(feature = "attendant")]
    let base_router = add_attendant_routes(base_router, &app_state);
    #[cfg(not(feature = "vibe"))]
    let base_router = base_router.nest_service("/static", ServeDir::new(site_path));

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
        // Unmatched paths serve the desktop shell, because the suite routes its
        // applications on the client. A path belonging to a machine-facing
        // surface is not an application route though: answering the shell meant
        // `200 text/html` was returned to callers that expected JSON, DAV XML or
        // an acknowledgement, which is how a whole family of unreachable
        // endpoints stayed invisible (see the axum 0.7 `:param` spelling).
        let shell_path = format!("{}/desktop.html", ui_path);
        base_router
            .nest_service("/auth", ServeDir::new(format!("{}/auth", ui_path)))
            .nest_service("/suite", ServeDir::new(&ui_path))
            .nest_service("/themes", ServeDir::new(format!("{}/../themes", ui_path)))
            .fallback(
                move |uri: axum::http::Uri, headers: axum::http::HeaderMap| {
                    let shell_path = shell_path.clone();
                    async move { fallback_response(&uri, &headers, &shell_path).await }
                },
            )
    } else if use_embedded_ui {
        base_router.merge(crate::embedded_ui::embedded_ui_router())
    } else {
        base_router
    }
}
/// A path that belongs to a machine-facing surface — the API, the DAV router,
/// an inbound webhook or well-known discovery — rather than an application
/// route. Such a caller expects JSON, DAV XML or an acknowledgement, never the
/// desktop shell.
fn is_machine_facing(path: &str) -> bool {
    ["/api/", "/caldav", "/webhook", "/.well-known/"]
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

/// Answers a request no route matched.
///
/// An application route receives the desktop shell. A path that belongs to the
/// API, the DAV router or an inbound webhook receives a `404` naming the path,
/// so a route that is missing — or declared with the wrong parameter syntax — is
/// reported instead of being masked by a successful HTML response.
pub(super) async fn fallback_response(
    uri: &axum::http::Uri,
    headers: &axum::http::HeaderMap,
    shell_path: &str,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    let path = uri.path();
    if is_machine_facing(path) {
        let locale = botcore::i18n::locale_from_headers(headers);
        return (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({
                "error": "not_found",
                "message": botcore::i18n::t(&locale, "error-http-no-route"),
                "path": path,
            })),
        )
            .into_response();
    }

    match tokio::fs::read(shell_path).await {
        Ok(bytes) => (
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            bytes,
        )
            .into_response(),
        Err(error) => {
            log::error!("Failed to read the desktop shell at {shell_path}: {error}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[cfg(test)]
#[path = "run_axum_server_tests.rs"]
mod tests;
