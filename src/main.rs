use axum::extract::{Extension, State};
use axum::http::StatusCode;
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
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use botserver::core;
use botserver::shared;

use botserver::core::automation;
use botserver::core::bootstrap;
use botserver::core::bot;
use botserver::core::package_manager;
use botserver::core::session;

#[cfg(feature = "attendance")]
use botserver::attendance;

#[cfg(feature = "calendar")]
use botserver::calendar;

#[cfg(feature = "directory")]
use botserver::directory;

#[cfg(feature = "email")]
use botserver::email;

#[cfg(feature = "llm")]
use botserver::llm;

#[cfg(feature = "meet")]
use botserver::meet;

#[cfg(feature = "whatsapp")]
use botserver::whatsapp;

use automation::AutomationService;
use bootstrap::BootstrapManager;
use botserver::core::bot::channels::{VoiceAdapter, WebChannelAdapter};
use botserver::core::bot::websocket_handler;
use botserver::core::bot::BotOrchestrator;
use botserver::core::config::AppConfig;

#[cfg(feature = "directory")]
use directory::auth_handler;
#[cfg(feature = "meet")]
use meet::{voice_start, voice_stop};
use package_manager::InstallMode;
use session::{create_session, get_session_history, get_sessions, start_session};
use shared::state::AppState;
use shared::utils::create_conn;
use shared::utils::create_s3_operator;

use botserver::BootstrapProgress;

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

fn print_shutdown_message() {
    println!();
    println!("\x1b[33m✨ Thank you for using General Bots!\x1b[0m");
    println!("\x1b[36m   pragmatismo.com.br\x1b[0m");
    println!();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
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
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .max_age(std::time::Duration::from_secs(3600));

    use crate::core::urls::ApiUrls;

    let mut api_router = Router::new()
        .route("/health", get(health_check_simple))
        .route(ApiUrls::HEALTH, get(health_check))
        .route(ApiUrls::SESSIONS, post(create_session))
        .route(ApiUrls::SESSIONS, get(get_sessions))
        .route(
            &ApiUrls::SESSION_HISTORY.replace(":id", "{session_id}"),
            get(get_session_history),
        )
        .route(
            &ApiUrls::SESSION_START.replace(":id", "{session_id}"),
            post(start_session),
        )
        .route(ApiUrls::WS, get(websocket_handler))
        .merge(botserver::drive::configure());

    #[cfg(feature = "directory")]
    {
        api_router = api_router
            .route(ApiUrls::AUTH, get(auth_handler))
            .merge(crate::core::directory::api::configure_user_routes())
            .merge(crate::directory::router::configure());
    }

    #[cfg(feature = "meet")]
    {
        api_router = api_router
            .route(ApiUrls::VOICE_START, post(voice_start))
            .route(ApiUrls::VOICE_STOP, post(voice_stop))
            .route(ApiUrls::WS_MEET, get(crate::meet::meeting_websocket))
            .merge(crate::meet::configure());
    }

    #[cfg(feature = "email")]
    {
        api_router = api_router.merge(crate::email::configure());
    }

    #[cfg(feature = "calendar")]
    {
        let calendar_engine =
            Arc::new(crate::calendar::CalendarEngine::new(app_state.conn.clone()));

        let reminder_engine = Arc::clone(&calendar_engine);
        tokio::spawn(async move {
            crate::calendar::start_reminder_job(reminder_engine).await;
        });

        api_router = api_router.merge(crate::calendar::caldav::create_caldav_router(
            calendar_engine,
        ));
    }

    api_router = api_router.merge(botserver::tasks::configure_task_routes());

    #[cfg(feature = "calendar")]
    {
        api_router = api_router.merge(crate::calendar::configure_calendar_routes());
    }

    api_router = api_router.merge(botserver::analytics::configure_analytics_routes());
    api_router = api_router.merge(botserver::paper::configure_paper_routes());
    api_router = api_router.merge(botserver::research::configure_research_routes());
    api_router = api_router.merge(botserver::sources::configure_sources_routes());
    api_router = api_router.merge(botserver::designer::configure_designer_routes());
    api_router = api_router.merge(botserver::basic::keywords::configure_db_routes());
    api_router = api_router.merge(botserver::basic::keywords::configure_app_server_routes());

    #[cfg(feature = "whatsapp")]
    {
        api_router = api_router.merge(crate::whatsapp::configure());
    }

    #[cfg(feature = "attendance")]
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

    let app = Router::new()
        .merge(api_router.with_state(app_state.clone()))
        // Static files fallback for legacy /apps/* paths
        .nest_service("/static", ServeDir::new(&site_path))
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
    } else {
        if disable_tls {
            info!("TLS disabled via BOTSERVER_DISABLE_TLS environment variable");
        } else {
            warn!("TLS certificates not found, using HTTP");
        }

        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("HTTP server listening on {}", addr);
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(std::io::Error::other)
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let no_ui = args.contains(&"--noui".to_string());
    let no_console = args.contains(&"--noconsole".to_string());

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

    let rust_log = {
        "info,botserver=info,\
         vaultrs=off,rustify=off,rustify_derive=off,\
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
         hickory_resolver=off,hickory_proto=off"
            .to_string()
    };

    std::env::set_var("RUST_LOG", &rust_log);

    use crate::llm::local::ensure_llama_servers_running;
    use botserver::config::ConfigManager;

    if no_console || no_ui {
        env_logger::Builder::from_env(env_logger::Env::default())
            .write_style(env_logger::WriteStyle::Always)
            .init();

        println!("Starting General Bots {}...", env!("CARGO_PKG_VERSION"));
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
                        let mut ui = botserver::console::XtreeUI::new();
                        ui.set_progress_channel(progress_rx);
                        ui.set_state_channel(state_rx);

                        if let Err(e) = ui.start_ui() {
                            eprintln!("UI error: {e}");
                        }
                    })
                    .expect("Failed to spawn UI thread"),
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

            if let Err(e) = bootstrap.ensure_services_running().await {
                warn!("Some services might not be running: {}", e);
            }

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
                    AppConfig::from_database(&pool)
                        .unwrap_or_else(|_| AppConfig::from_env().expect("Failed to load config"))
                }
                Err(e) => {
                    trace!(
                        "Database connection failed: {:?}, loading config from env",
                        e
                    );
                    AppConfig::from_env().expect("Failed to load config from env")
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
                Ok(pool) => AppConfig::from_database(&pool)
                    .unwrap_or_else(|_| AppConfig::from_env().expect("Failed to load config")),
                Err(_) => AppConfig::from_env().expect("Failed to load config from env"),
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
        AppConfig::from_env().expect("Failed to load config from env")
    });
    let config = std::sync::Arc::new(refreshed_cfg.clone());
    info!(
        "Server configured to listen on {}:{}",
        config.server.host, config.server.port
    );

    let cache_url = "redis://localhost:6379".to_string();
    let redis_client = match redis::Client::open(cache_url.as_str()) {
        Ok(client) => Some(Arc::new(client)),
        Err(e) => {
            log::warn!("Failed to connect to Redis: {}", e);
            None
        }
    };

    let web_adapter = Arc::new(WebChannelAdapter::new());
    let voice_adapter = Arc::new(VoiceAdapter::new());

    let drive = create_s3_operator(&config.drive)
        .await
        .expect("Failed to initialize Drive");

    let session_manager = Arc::new(tokio::sync::Mutex::new(session::SessionManager::new(
        pool.get().unwrap(),
        redis_client.clone(),
    )));

    #[cfg(feature = "directory")]
    let zitadel_config = botserver::directory::client::ZitadelConfig {
        issuer_url: "https://localhost:8080".to_string(),
        issuer: "https://localhost:8080".to_string(),
        client_id: "client_id".to_string(),
        client_secret: "client_secret".to_string(),
        redirect_uri: "https://localhost:8080/callback".to_string(),
        project_id: "default".to_string(),
        api_url: "https://localhost:8080".to_string(),
        service_account_key: None,
    };
    #[cfg(feature = "directory")]
    let auth_service = Arc::new(tokio::sync::Mutex::new(
        botserver::directory::AuthService::new(zitadel_config).unwrap(),
    ));
    let config_manager = ConfigManager::new(pool.clone());

    let mut bot_conn = pool.get().expect("Failed to get database connection");
    let (default_bot_id, default_bot_name) = crate::bot::get_default_bot(&mut bot_conn);
    info!(
        "Using default bot: {} (id: {})",
        default_bot_name, default_bot_id
    );

    let llm_url = config_manager
        .get_config(&default_bot_id, "llm-url", Some("http://localhost:8081"))
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    info!("LLM URL: {}", llm_url);

    let base_llm_provider = Arc::new(botserver::llm::OpenAIClient::new(
        "empty".to_string(),
        Some(llm_url.clone()),
    )) as Arc<dyn botserver::llm::LLMProvider>;

    let llm_provider: Arc<dyn botserver::llm::LLMProvider> = if let Some(ref cache) = redis_client {
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

        let embedding_service = Some(Arc::new(botserver::llm::cache::LocalEmbeddingService::new(
            embedding_url,
            embedding_model,
        ))
            as Arc<dyn botserver::llm::cache::EmbeddingService>);

        let cache_config = botserver::llm::cache::CacheConfig {
            ttl: 3600,
            semantic_matching: true,
            similarity_threshold: 0.85,
            max_similarity_checks: 100,
            key_prefix: "llm_cache".to_string(),
        };

        Arc::new(botserver::llm::cache::CachedLLMProvider::with_db_pool(
            base_llm_provider,
            cache.clone(),
            cache_config,
            embedding_service,
            pool.clone(),
        ))
    } else {
        base_llm_provider
    };

    let kb_manager = Arc::new(botserver::core::kb::KnowledgeBaseManager::new("work"));

    let task_engine = Arc::new(botserver::tasks::TaskEngine::new(pool.clone()));

    let metrics_collector = botserver::core::shared::analytics::MetricsCollector::new();

    let task_scheduler = None;

    let (attendant_tx, _attendant_rx) = tokio::sync::broadcast::channel::<
        botserver::core::shared::state::AttendantNotification,
    >(1000);

    let app_state = Arc::new(AppState {
        drive: Some(drive.clone()),
        s3_client: Some(drive),
        config: Some(cfg.clone()),
        conn: pool.clone(),
        database_url: crate::shared::utils::get_database_url_sync().unwrap_or_default(),
        bucket_name: "default.gbai".to_string(),
        cache: redis_client.clone(),
        session_manager: session_manager.clone(),
        metrics_collector,
        task_scheduler,
        llm_provider: llm_provider.clone(),
        #[cfg(feature = "directory")]
        auth_service: auth_service.clone(),
        channels: Arc::new(tokio::sync::Mutex::new({
            let mut map = HashMap::new();
            map.insert(
                "web".to_string(),
                web_adapter.clone() as Arc<dyn botserver::core::bot::channels::ChannelAdapter>,
            );
            map
        })),
        response_channels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        web_adapter: web_adapter.clone(),
        voice_adapter: voice_adapter.clone(),
        kb_manager: Some(kb_manager.clone()),
        task_engine,
        extensions: botserver::core::shared::state::Extensions::new(),
        attendant_broadcast: Some(attendant_tx),
    });

    let task_scheduler = Arc::new(botserver::tasks::scheduler::TaskScheduler::new(
        app_state.clone(),
    ));

    task_scheduler.start();

    if let Err(e) = botserver::core::kb::ensure_crawler_service_running(app_state.clone()).await {
        log::warn!("Failed to start website crawler service: {}", e);
    }

    let _ = state_tx.try_send(app_state.clone());
    progress_tx.send(BootstrapProgress::BootstrapComplete).ok();

    info!(
        "Starting HTTP server on {}:{}",
        config.server.host, config.server.port
    );

    let worker_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let _automation_service =
        botserver::core::automation::AutomationService::new(app_state.clone());
    info!("Automation service initialized with episodic memory scheduler");

    let bot_orchestrator = BotOrchestrator::new(app_state.clone());
    if let Err(e) = bot_orchestrator.mount_all_bots() {
        error!("Failed to mount bots: {}", e);
    }

    let automation_state = app_state.clone();
    tokio::spawn(async move {
        let automation = AutomationService::new(automation_state);
        automation.spawn().await.ok();
    });

    let app_state_for_llm = app_state.clone();
    tokio::spawn(async move {
        if let Err(e) = ensure_llama_servers_running(app_state_for_llm).await {
            error!("Failed to start LLM servers: {}", e);
        }
    });
    trace!("Initial data setup task spawned");

    trace!("Starting HTTP server on port {}...", config.server.port);
    run_axum_server(app_state, config.server.port, worker_count).await?;

    if let Some(handle) = ui_handle {
        handle.join().ok();
    }

    Ok(())
}
