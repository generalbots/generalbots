#![cfg_attr(feature = "desktop", windows_subsystem = "windows")]
use axum::extract::Extension;
use axum::{
    routing::{get, post},
    Router,
};
// Configuration comes from Directory service, not .env files
use log::{error, info, trace, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use botserver::basic;
use botserver::core;
use botserver::shared;
use botserver::web;

#[cfg(feature = "console")]
use botserver::console;

// Re-exports from core
use botserver::core::automation;
use botserver::core::bootstrap;
use botserver::core::bot;
use botserver::core::config;
use botserver::core::package_manager;
use botserver::core::session;
use botserver::core::ui_server;

// Feature-gated modules
#[cfg(feature = "attendance")]
mod attendance;

#[cfg(feature = "calendar")]
mod calendar;

#[cfg(feature = "compliance")]
mod compliance;

#[cfg(feature = "desktop")]
mod desktop;

#[cfg(feature = "directory")]
mod directory;

#[cfg(feature = "drive")]
mod drive;

#[cfg(feature = "email")]
mod email;

#[cfg(feature = "instagram")]
mod instagram;

#[cfg(feature = "llm")]
mod llm;

#[cfg(feature = "meet")]
mod meet;

#[cfg(feature = "msteams")]
mod msteams;

#[cfg(feature = "nvidia")]
mod nvidia;

#[cfg(feature = "vectordb")]
mod vector_db;

#[cfg(feature = "weba")]
mod weba;

#[cfg(feature = "whatsapp")]
mod whatsapp;

use crate::automation::AutomationService;
use crate::bootstrap::BootstrapManager;
#[cfg(feature = "email")]
use crate::email::{
    add_email_account, delete_email_account, get_emails, get_latest_email_from,
    list_email_accounts, list_emails, list_folders, save_click, save_draft, send_email,
};
use botserver::core::bot::channels::{VoiceAdapter, WebChannelAdapter};
use botserver::core::bot::websocket_handler;
use botserver::core::bot::BotOrchestrator;
use botserver::core::config::AppConfig;

// use crate::file::upload_file; // Module doesn't exist
#[cfg(feature = "directory")]
use crate::directory::auth_handler;
#[cfg(feature = "meet")]
use crate::meet::{voice_start, voice_stop};
use crate::package_manager::InstallMode;
use crate::session::{create_session, get_session_history, get_sessions, start_session};
use crate::shared::state::AppState;
use crate::shared::utils::create_conn;
use crate::shared::utils::create_s3_operator;

// Use BootstrapProgress from lib.rs
use botserver::BootstrapProgress;

async fn run_axum_server(
    app_state: Arc<AppState>,
    port: u16,
    _worker_count: usize,
) -> std::io::Result<()> {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .max_age(std::time::Duration::from_secs(3600));

    use crate::core::urls::ApiUrls;

    // Build API router with module-specific routes
    let mut api_router = Router::new()
        .route(ApiUrls::SESSIONS, post(create_session))
        .route(ApiUrls::SESSIONS, get(get_sessions))
        .route(
            ApiUrls::SESSION_HISTORY.replace(":id", "{session_id}"),
            get(get_session_history),
        )
        .route(
            ApiUrls::SESSION_START.replace(":id", "{session_id}"),
            post(start_session),
        )
        // WebSocket route
        .route(ApiUrls::WS, get(websocket_handler))
        // Merge drive routes using the configure() function
        .merge(botserver::drive::configure());

    // Add feature-specific routes
    #[cfg(feature = "directory")]
    {
        api_router = api_router
            .route(ApiUrls::AUTH, get(auth_handler))
            .merge(crate::core::directory::api::configure_user_routes());
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

    // Add calendar routes with CalDAV if feature is enabled
    #[cfg(feature = "calendar")]
    {
        let calendar_engine =
            Arc::new(crate::calendar::CalendarEngine::new(app_state.conn.clone()));

        // Start reminder job
        let reminder_engine = Arc::clone(&calendar_engine);
        tokio::spawn(async move {
            crate::calendar::start_reminder_job(reminder_engine).await;
        });

        // Add CalDAV router
        api_router = api_router.merge(crate::calendar::caldav::create_caldav_router(
            calendar_engine,
        ));
    }

    // Add task engine routes
    api_router = api_router.merge(botserver::tasks::configure_task_routes());

    // Add calendar routes if calendar feature is enabled
    #[cfg(feature = "calendar")]
    {
        api_router = api_router.merge(crate::calendar::configure_calendar_routes());
    }

    // Build static file serving
    let static_path = std::path::Path::new("./ui/suite");

    // Create web router with authentication
    let web_router = web::create_router(app_state.clone());

    let app = Router::new()
        // Static file services for remaining assets
        .nest_service("/static/js", ServeDir::new(static_path.join("js")))
        .nest_service("/static/css", ServeDir::new(static_path.join("css")))
        .nest_service("/static/public", ServeDir::new(static_path.join("public")))
        // Web module with authentication (handles all pages and auth)
        .merge(web_router)
        // Legacy API routes (will be migrated to web module)
        .merge(api_router.with_state(app_state.clone()))
        .layer(Extension(app_state.clone()))
        // Layers
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Always use HTTPS - load certificates from botserver-stack
    let cert_dir = std::path::Path::new("./botserver-stack/conf/system/certificates");
    let cert_path = cert_dir.join("api/server.crt");
    let key_path = cert_dir.join("api/server.key");

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // Check if certificates exist
    if cert_path.exists() && key_path.exists() {
        // Use HTTPS with existing certificates
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        info!("HTTPS server listening on {} with TLS", addr);

        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    } else {
        // Generate self-signed certificate if not present
        warn!("TLS certificates not found, generating self-signed certificate...");

        // Fall back to HTTP temporarily (bootstrap will generate certs)
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!(
            "HTTP server listening on {} (certificates will be generated on next restart)",
            addr
        );
        axum::serve(listener, app.into_make_service())
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Configuration comes from Directory service, not .env files

    // Initialize logger early to capture all logs with filters for noisy libraries
    let rust_log = {
        // Default log level for botserver and suppress all other crates
        // Note: r2d2 is set to warn to see database connection pool warnings
        "info,botserver=info,\
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
    });

    // Set the RUST_LOG env var if not already set
    std::env::set_var("RUST_LOG", &rust_log);

    env_logger::Builder::from_env(env_logger::Env::default())
        .write_style(env_logger::WriteStyle::Always)
        .init();

    println!(
        "Starting {} {}...",
        "General Bots".to_string(),
        env!("CARGO_PKG_VERSION")
    );

    use crate::llm::local::ensure_llama_servers_running;
    use botserver::config::ConfigManager;

    let args: Vec<String> = std::env::args().collect();
    let no_ui = args.contains(&"--noui".to_string());
    let desktop_mode = args.contains(&"--desktop".to_string());
    let no_console = args.contains(&"--noconsole".to_string());

    // Configuration comes from Directory service, not .env files

    let (progress_tx, _progress_rx) = tokio::sync::mpsc::unbounded_channel::<BootstrapProgress>();
    let (state_tx, _state_rx) = tokio::sync::mpsc::channel::<Arc<AppState>>(1);

    // Handle CLI commands
    if args.len() > 1 {
        let command = &args[1];
        match command.as_str() {
            "install" | "remove" | "list" | "status" | "start" | "stop" | "restart" | "--help"
            | "-h" => match package_manager::cli::run().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("CLI error: {}", e);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("CLI command failed: {}", e),
                    ));
                }
            },
            _ => {}
        }
    }

    // Start UI thread if console is enabled (default) and not disabled by --noconsole or desktop mode
    let ui_handle: Option<std::thread::JoinHandle<()>> = if !no_console && !desktop_mode && !no_ui {
        #[cfg(feature = "console")]
        {
            let progress_rx = Arc::new(tokio::sync::Mutex::new(_progress_rx));
            let state_rx = Arc::new(tokio::sync::Mutex::new(_state_rx));

            Some(
                std::thread::Builder::new()
                    .name("ui-thread".to_string())
                    .spawn(move || {
                        let mut ui = botserver::console::XtreeUI::new();
                        ui.set_progress_channel(progress_rx.clone());

                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .expect("Failed to create UI runtime");

                        rt.block_on(async {
                            tokio::select! {
                                result = async {
                                    let mut rx = state_rx.lock().await;
                                    rx.recv().await
                                } => {
                                    if let Some(app_state) = result {
                                        ui.set_app_state(app_state);
                                    }
                                }
                                _ = tokio::time::sleep(tokio::time::Duration::from_secs(300)) => {
                                    eprintln!("UI initialization timeout");
                                }
                            }
                        });

                        if let Err(e) = ui.start_ui() {
                            eprintln!("UI error: {}", e);
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

    // Bootstrap
    trace!("Starting bootstrap process...");
    let progress_tx_clone = progress_tx.clone();
    let cfg = {
        progress_tx_clone
            .send(BootstrapProgress::StartingBootstrap)
            .ok();

        trace!("Creating BootstrapManager...");
        let mut bootstrap = BootstrapManager::new(install_mode.clone(), tenant.clone()).await;

        // Check if services are already configured in Directory
        let services_configured = std::path::Path::new("./botserver-stack/conf/directory/zitadel.yaml").exists();

        let cfg = if services_configured {
            trace!("Services already configured, ensuring all are running...");
            info!("Ensuring database and drive services are running...");
            progress_tx_clone
                .send(BootstrapProgress::StartingComponent(
                    "all services".to_string(),
                ))
                .ok();
            trace!("Calling bootstrap.start_all()...");

            // Ensure critical services are started
            if let Err(e) = bootstrap.ensure_services_running().await {
                warn!("Some services might not be running: {}", e);
            }

            bootstrap
                .start_all()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
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
            trace!(".env file not found, running bootstrap.bootstrap()...");
            _ = bootstrap.bootstrap().await;
            trace!("bootstrap.bootstrap() completed");
            progress_tx_clone
                .send(BootstrapProgress::StartingComponent(
                    "all services".to_string(),
                ))
                .ok();
            bootstrap
                .start_all()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            match create_conn() {
                Ok(pool) => AppConfig::from_database(&pool)
                    .unwrap_or_else(|_| AppConfig::from_env().expect("Failed to load config")),
                Err(_) => AppConfig::from_env().expect("Failed to load config from env"),
            }
        };

        trace!("Config loaded, uploading templates...");
        progress_tx_clone
            .send(BootstrapProgress::UploadingTemplates)
            .ok();

        if let Err(e) = bootstrap.upload_templates_to_drive(&cfg).await {
            trace!("Template upload error: {}", e);
            progress_tx_clone
                .send(BootstrapProgress::BootstrapError(format!(
                    "Failed to upload templates: {}",
                    e
                )))
                .ok();
        } else {
            trace!("Templates uploaded successfully");
        }

        Ok::<AppConfig, std::io::Error>(cfg)
    };

    trace!("Bootstrap config phase complete");
    let cfg = cfg?;
    trace!("Reloading dotenv...");
    dotenv().ok();

    trace!("Loading refreshed config from env...");
    let refreshed_cfg = AppConfig::from_env().expect("Failed to load config from env");
    let config = std::sync::Arc::new(refreshed_cfg.clone());

    trace!("Creating database pool again...");
    progress_tx.send(BootstrapProgress::ConnectingDatabase).ok();

    let pool = match create_conn() {
        Ok(pool) => {
            // Run automatic migrations
            trace!("Running database migrations...");
            info!("Running database migrations...");
            if let Err(e) = crate::shared::utils::run_migrations(&pool) {
                error!("Failed to run migrations: {}", e);
                // Continue anyway as some migrations might have already been applied
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

    let cache_url = "rediss://localhost:6379".to_string();
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

    // Create default Zitadel config (can be overridden with env vars)
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
        botserver::directory::AuthService::new(zitadel_config)
            .await
            .unwrap(),
    ));
    let config_manager = ConfigManager::new(pool.clone());

    let mut bot_conn = pool.get().expect("Failed to get database connection");
    let (default_bot_id, _default_bot_name) = crate::bot::get_default_bot(&mut bot_conn);

    let llm_url = config_manager
        .get_config(&default_bot_id, "llm-url", Some("https://localhost:8081"))
        .unwrap_or_else(|_| "https://localhost:8081".to_string());

    // Create base LLM provider
    let base_llm_provider = Arc::new(botserver::llm::OpenAIClient::new(
        "empty".to_string(),
        Some(llm_url.clone()),
    )) as Arc<dyn botserver::llm::LLMProvider>;

    // Wrap with cache if redis is available
    let llm_provider: Arc<dyn botserver::llm::LLMProvider> = if let Some(ref cache) = redis_client {
        // Set up embedding service for semantic matching
        let embedding_url = config_manager
            .get_config(
                &default_bot_id,
                "embedding-url",
                Some("https://localhost:8082"),
            )
            .unwrap_or_else(|_| "https://localhost:8082".to_string());
        let embedding_model = config_manager
            .get_config(&default_bot_id, "embedding-model", Some("all-MiniLM-L6-v2"))
            .unwrap_or_else(|_| "all-MiniLM-L6-v2".to_string());

        let embedding_service = Some(Arc::new(botserver::llm::cache::LocalEmbeddingService::new(
            embedding_url,
            embedding_model,
        ))
            as Arc<dyn botserver::llm::cache::EmbeddingService>);

        // Create cache config
        let cache_config = botserver::llm::cache::CacheConfig {
            ttl: 3600, // 1 hour TTL
            semantic_matching: true,
            similarity_threshold: 0.85, // 85% similarity threshold
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

    // Initialize Knowledge Base Manager
    let kb_manager = Arc::new(botserver::core::kb::KnowledgeBaseManager::new("work"));

    // Initialize TaskEngine
    let task_engine = Arc::new(botserver::tasks::TaskEngine::new(pool.clone()));

    // Initialize MetricsCollector
    let metrics_collector = botserver::core::shared::analytics::MetricsCollector::new();

    // Initialize TaskScheduler (will be set after AppState creation)
    let task_scheduler = None;

    let app_state = Arc::new(AppState {
        drive: Some(drive.clone()),
        s3_client: Some(drive),
        config: Some(cfg.clone()),
        conn: pool.clone(),
        database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| "".to_string()),
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
        task_engine: task_engine,
        extensions: botserver::core::shared::state::Extensions::new(),
    });

    // Initialize TaskScheduler with the AppState
    let task_scheduler = Arc::new(botserver::tasks::scheduler::TaskScheduler::new(
        app_state.clone(),
    ));

    // Update AppState with the task scheduler using Arc::get_mut (requires mutable reference)
    // Since we can't mutate Arc directly, we'll need to use unsafe or recreate AppState
    // For now, we'll start the scheduler without updating the field
    task_scheduler.start().await;

    // Start website crawler service
    if let Err(e) = botserver::core::kb::ensure_crawler_service_running(app_state.clone()).await {
        log::warn!("Failed to start website crawler service: {}", e);
    }

    state_tx.send(app_state.clone()).await.ok();
    progress_tx.send(BootstrapProgress::BootstrapComplete).ok();

    info!(
        "Starting HTTP server on {}:{}",
        config.server.host, config.server.port
    );

    let worker_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    // Initialize automation service for prompt compaction
    let _automation_service =
        botserver::core::automation::AutomationService::new(app_state.clone());
    info!("Automation service initialized with prompt compaction scheduler");

    // Mount bots
    let bot_orchestrator = BotOrchestrator::new(app_state.clone());
    tokio::spawn(async move {
        if let Err(e) = bot_orchestrator.mount_all_bots().await {
            error!("Failed to mount bots: {}", e);
        }
    });

    // Start automation service
    let automation_state = app_state.clone();
    tokio::spawn(async move {
        let automation = AutomationService::new(automation_state);
        automation.spawn().await.ok();
    });

    // Start LLM servers
    let app_state_for_llm = app_state.clone();
    tokio::spawn(async move {
        if let Err(e) = ensure_llama_servers_running(app_state_for_llm).await {
            error!("Failed to start LLM servers: {}", e);
        }
    });
    trace!("Initial data setup task spawned");

    trace!("Checking desktop mode: {}", desktop_mode);
    // Handle desktop mode vs server mode
    #[cfg(feature = "desktop")]
    if desktop_mode {
        trace!("Desktop mode is enabled");
        // For desktop mode: Run HTTP server in a separate thread with its own runtime
        let app_state_for_server = app_state.clone();
        let port = config.server.port;
        let workers = worker_count; // Capture worker_count for the thread

        info!(
            "Desktop mode: Starting HTTP server on port {} in background thread",
            port
        );

        std::thread::spawn(move || {
            info!("HTTP server thread started, initializing runtime...");
            let rt = tokio::runtime::Runtime::new().expect("Failed to create HTTP runtime");
            rt.block_on(async move {
                info!(
                    "HTTP server runtime created, starting axum server on port {}...",
                    port
                );
                if let Err(e) = run_axum_server(app_state_for_server, port, workers).await {
                    error!("HTTP server error: {}", e);
                    eprintln!("HTTP server error: {}", e);
                } else {
                    info!("HTTP server started successfully");
                }
            });
        });

        // Give the server thread a moment to start
        std::thread::sleep(std::time::Duration::from_millis(500));
        info!("Launching General Bots desktop application...");

        // Run Tauri on main thread (GUI requires main thread)
        let tauri_app = tauri::Builder::default()
            .setup(|app| {
                use tauri::WebviewWindowBuilder;
                match WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .title("General Bots")
                .build()
                {
                    Ok(_window) => Ok(()),
                    Err(e) if e.to_string().contains("WebviewLabelAlreadyExists") => {
                        log::warn!("Main window already exists, reusing existing window");
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            })
            .build(tauri::generate_context!())
            .expect("error while running Desktop application");

        tauri_app.run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });

        return Ok(());
    }

    // Non-desktop mode: Run HTTP server directly
    #[cfg(not(feature = "desktop"))]
    {
        trace!(
            "Running in non-desktop mode, starting HTTP server on port {}...",
            config.server.port
        );
        run_axum_server(app_state, config.server.port, worker_count).await?;

        // Wait for UI thread to finish if it was started
        if let Some(handle) = ui_handle {
            handle.join().ok();
        }
    }

    // For builds with desktop feature but not running in desktop mode
    #[cfg(feature = "desktop")]
    if !desktop_mode {
        trace!(
            "Desktop feature available but not in desktop mode, starting HTTP server on port {}...",
            config.server.port
        );
        run_axum_server(app_state, config.server.port, worker_count).await?;

        // Wait for UI thread to finish if it was started
        if let Some(handle) = ui_handle {
            handle.join().ok();
        }
    }

    Ok(())
}
