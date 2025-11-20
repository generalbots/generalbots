#![cfg_attr(feature = "desktop", windows_subsystem = "windows")]
use axum::{
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use log::{error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

mod auth;
mod automation;
mod basic;
mod bootstrap;
mod bot;
mod channels;
mod config;
mod context;
mod drive_monitor;
#[cfg(feature = "email")]
mod email;
mod file;
mod llm;
mod llm_models;
mod meet;
mod nvidia;
mod package_manager;
mod session;
mod shared;
pub mod tests;
mod ui_tree;
mod web_server;

use crate::auth::auth_handler;
use crate::automation::AutomationService;
use crate::bootstrap::BootstrapManager;
use crate::bot::websocket_handler;
use crate::bot::BotOrchestrator;
use crate::channels::{VoiceAdapter, WebChannelAdapter};
use crate::config::AppConfig;
#[cfg(feature = "email")]
use crate::email::{
    get_emails, get_latest_email_from, list_emails, save_click, save_draft, send_email,
};
use crate::file::upload_file;
use crate::meet::{voice_start, voice_stop};
use crate::package_manager::InstallMode;
use crate::session::{create_session, get_session_history, get_sessions, start_session};
use crate::shared::state::AppState;
use crate::shared::utils::create_conn;
use crate::shared::utils::create_s3_operator;

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

    // Build API routes with State
    let api_router = Router::new()
        // Auth route
        .route("/api/auth", get(auth_handler))
        // Session routes
        .route("/api/sessions", post(create_session))
        .route("/api/sessions", get(get_sessions))
        .route(
            "/api/sessions/{session_id}/history",
            get(get_session_history),
        )
        .route("/api/sessions/{session_id}/start", post(start_session))
        // File routes
        .route("/api/files/upload/{folder_path}", post(upload_file))
        // Voice/Meet routes
        .route("/api/voice/start", post(voice_start))
        .route("/api/voice/stop", post(voice_stop))
        // WebSocket route
        .route("/ws", get(websocket_handler))
        // Bot routes
        .route("/api/bots", post(crate::bot::create_bot_handler))
        .route(
            "/api/bots/{bot_id}/mount",
            post(crate::bot::mount_bot_handler),
        )
        .route(
            "/api/bots/{bot_id}/input",
            post(crate::bot::handle_user_input_handler),
        )
        .route(
            "/api/bots/{bot_id}/sessions",
            get(crate::bot::get_user_sessions_handler),
        )
        .route(
            "/api/bots/{bot_id}/history",
            get(crate::bot::get_conversation_history_handler),
        )
        .route(
            "/api/bots/{bot_id}/warning",
            post(crate::bot::send_warning_handler),
        );

    // Add email routes if feature is enabled
    #[cfg(feature = "email")]
    let api_router = api_router
        .route("/api/email/latest", post(get_latest_email_from))
        .route("/api/email/get/{campaign_id}", get(get_emails))
        .route("/api/email/list", get(list_emails))
        .route("/api/email/send", post(send_email))
        .route("/api/email/draft", post(save_draft))
        .route("/api/email/click/{campaign_id}/{email}", get(save_click));

    // Build static file serving
    let static_path = std::path::Path::new("./web/desktop");

    let app = Router::new()
        .route("/", get(crate::web_server::index))
        .merge(api_router)
        .with_state(app_state.clone())
        .nest_service("/js", ServeDir::new(static_path.join("js")))
        .nest_service("/css", ServeDir::new(static_path.join("css")))
        .nest_service("/drive", ServeDir::new(static_path.join("drive")))
        .nest_service("/chat", ServeDir::new(static_path.join("chat")))
        .nest_service("/mail", ServeDir::new(static_path.join("mail")))
        .nest_service("/tasks", ServeDir::new(static_path.join("tasks")))
        .fallback_service(ServeDir::new(static_path))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("HTTP server listening on {}", addr);

    // Serve the app
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    println!(
        "Starting {} {}...",
        std::env::var("PLATFORM_NAME").unwrap_or("General Bots".to_string()),
        env!("CARGO_PKG_VERSION")
    );

    use crate::llm::local::ensure_llama_servers_running;
    use botserver::config::ConfigManager;

    let args: Vec<String> = std::env::args().collect();
    let no_ui = args.contains(&"--noui".to_string());
    let desktop_mode = args.contains(&"--desktop".to_string());

    dotenv().ok();

    let (progress_tx, progress_rx) = tokio::sync::mpsc::unbounded_channel::<BootstrapProgress>();
    let (state_tx, state_rx) = tokio::sync::mpsc::channel::<Arc<AppState>>(1);

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

    // Start UI thread if not in no-ui mode and not in desktop mode
    let ui_handle = if !no_ui && !desktop_mode {
        let progress_rx = Arc::new(tokio::sync::Mutex::new(progress_rx));
        let state_rx = Arc::new(tokio::sync::Mutex::new(state_rx));

        Some(
            std::thread::Builder::new()
                .name("ui-thread".to_string())
                .spawn(move || {
                    let mut ui = crate::ui_tree::XtreeUI::new();
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
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .write_style(env_logger::WriteStyle::Always)
            .init();
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
    let progress_tx_clone = progress_tx.clone();
    let cfg = {
        progress_tx_clone
            .send(BootstrapProgress::StartingBootstrap)
            .ok();

        let mut bootstrap = BootstrapManager::new(install_mode.clone(), tenant.clone()).await;
        let env_path = std::env::current_dir().unwrap().join(".env");

        let cfg = if env_path.exists() {
            progress_tx_clone
                .send(BootstrapProgress::StartingComponent(
                    "all services".to_string(),
                ))
                .ok();
            bootstrap
                .start_all()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            progress_tx_clone
                .send(BootstrapProgress::ConnectingDatabase)
                .ok();

            match create_conn() {
                Ok(pool) => AppConfig::from_database(&pool)
                    .unwrap_or_else(|_| AppConfig::from_env().expect("Failed to load config")),
                Err(_) => AppConfig::from_env().expect("Failed to load config from env"),
            }
        } else {
            _ = bootstrap.bootstrap().await;
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

        progress_tx_clone
            .send(BootstrapProgress::UploadingTemplates)
            .ok();

        if let Err(e) = bootstrap.upload_templates_to_drive(&cfg).await {
            progress_tx_clone
                .send(BootstrapProgress::BootstrapError(format!(
                    "Failed to upload templates: {}",
                    e
                )))
                .ok();
        }

        Ok::<AppConfig, std::io::Error>(cfg)
    };

    let cfg = cfg?;
    dotenv().ok();

    let refreshed_cfg = AppConfig::from_env().expect("Failed to load config from env");
    let config = std::sync::Arc::new(refreshed_cfg.clone());

    progress_tx.send(BootstrapProgress::ConnectingDatabase).ok();

    let pool = match create_conn() {
        Ok(pool) => pool,
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

    let cache_url =
        std::env::var("CACHE_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
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

    let auth_service = Arc::new(tokio::sync::Mutex::new(auth::AuthService::new()));
    let config_manager = ConfigManager::new(pool.clone());

    let mut bot_conn = pool.get().expect("Failed to get database connection");
    let (default_bot_id, _default_bot_name) = crate::bot::get_default_bot(&mut bot_conn);

    let llm_url = config_manager
        .get_config(&default_bot_id, "llm-url", Some("http://localhost:8081"))
        .unwrap_or_else(|_| "http://localhost:8081".to_string());

    let llm_provider = Arc::new(crate::llm::OpenAIClient::new(
        "empty".to_string(),
        Some(llm_url.clone()),
    ));

    let app_state = Arc::new(AppState {
        drive: Some(drive),
        config: Some(cfg.clone()),
        conn: pool.clone(),
        bucket_name: "default.gbai".to_string(),
        cache: redis_client.clone(),
        session_manager: session_manager.clone(),
        llm_provider: llm_provider.clone(),
        auth_service: auth_service.clone(),
        channels: Arc::new(tokio::sync::Mutex::new({
            let mut map = HashMap::new();
            map.insert(
                "web".to_string(),
                web_adapter.clone() as Arc<dyn crate::channels::ChannelAdapter>,
            );
            map
        })),
        response_channels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        web_adapter: web_adapter.clone(),
        voice_adapter: voice_adapter.clone(),
    });

    state_tx.send(app_state.clone()).await.ok();
    progress_tx.send(BootstrapProgress::BootstrapComplete).ok();

    info!(
        "Starting HTTP server on {}:{}",
        config.server.host, config.server.port
    );

    let worker_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

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

    // Handle desktop mode vs server mode
    #[cfg(feature = "desktop")]
    if desktop_mode {
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
        info!("Launching Tauri desktop application...");

        // Run Tauri on main thread (GUI requires main thread)
        let tauri_app = tauri::Builder::default()
            .setup(|app| {
                use tauri::WebviewWindowBuilder;
                match WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into()),
                )
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
            .expect("error while running tauri application");

        tauri_app.run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });

        return Ok(());
    }

    // Non-desktop mode: Run HTTP server directly
    run_axum_server(app_state, config.server.port, worker_count).await?;

    // Wait for UI thread to finish if it was started
    if let Some(handle) = ui_handle {
        handle.join().ok();
    }

    Ok(())
}
