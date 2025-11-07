#![cfg_attr(feature = "desktop", windows_subsystem = "windows")]
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use log::error;
use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
#[cfg(feature = "desktop")]
mod ui;
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
use crate::file::{init_drive, upload_file};
use crate::meet::{voice_start, voice_stop};
use crate::package_manager::InstallMode;
use crate::session::{create_session, get_session_history, get_sessions, start_session};
use crate::shared::state::AppState;
use crate::web_server::{bot_index, index, static_files};

#[cfg(not(feature = "desktop"))]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    use botserver::config::ConfigManager;

    use crate::llm::local::ensure_llama_servers_running;

    let args: Vec<String> = std::env::args().collect();
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
            _ => {
                eprintln!("Unknown command: {}", command);
                eprintln!("Run 'botserver --help' for usage information");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unknown command: {}", command),
                ));
            }
        }
    }

    // Rest of the original main function remains unchanged...
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .write_style(env_logger::WriteStyle::Always)
        .init();

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

    let mut bootstrap = BootstrapManager::new(install_mode.clone(), tenant.clone()).await;

    // Prevent double bootstrap: skip if environment already initialized
    let env_path = std::env::current_dir()?
        .join("botserver-stack")
        .join(".env");
    let cfg = if env_path.exists() {
        info!("Environment already initialized, skipping bootstrap");

        match diesel::Connection::establish(&std::env::var("DATABASE_URL").unwrap()) {
            Ok(mut conn) => {
                AppConfig::from_database(&mut conn).expect("Failed to load config from DB")
            }
            Err(_) => AppConfig::from_env().expect("Failed to load config from env"),
        }
    } else {
        match bootstrap.bootstrap().await {
            Ok(config) => {
                info!("Bootstrap completed successfully");
                config
            }
            Err(e) => {
                log::error!("Bootstrap failed: {}", e);
                match diesel::Connection::establish(
                    &std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                        "postgres://gbuser:@localhost:5432/botserver".to_string()
                    }),
                ) {
                    Ok(mut conn) => {
                        AppConfig::from_database(&mut conn).expect("Failed to load config from DB")
                    }
                    Err(_) => AppConfig::from_env().expect("Failed to load config from env"),
                }
            }
        }
    };

    // Start all services (synchronous)
    if let Err(e) = bootstrap.start_all() {
        log::warn!("Failed to start all services: {}", e);
    }

    // Upload templates (asynchronous)
    if let Err(e) = futures::executor::block_on(bootstrap.upload_templates_to_drive(&cfg)) {
        log::warn!("Failed to upload templates to MinIO: {}", e);
    }

    // Refresh configuration from environment to ensure latest DATABASE_URL and credentials
    dotenv().ok();
    let refreshed_cfg = AppConfig::from_env().expect("Failed to load config from env");
    let config = std::sync::Arc::new(refreshed_cfg.clone());
    let db_pool = match diesel::Connection::establish(&refreshed_cfg.database_url()) {
        Ok(conn) => Arc::new(Mutex::new(conn)),
        Err(e) => {
            log::error!("Failed to connect to main database: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Database connection failed: {}", e),
            ));
        }
    };

    let cache_url = std::env::var("CACHE_URL")
        .or_else(|_| std::env::var("REDIS_URL"))
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = match redis::Client::open(cache_url.as_str()) {
        Ok(client) => Some(Arc::new(client)),
        Err(e) => {
            log::warn!("Failed to connect to Redis: Redis URL did not parse- {}", e);
            None
        }
    };
    let web_adapter = Arc::new(WebChannelAdapter::new());
    let voice_adapter = Arc::new(VoiceAdapter::new());

    let drive = init_drive(&config.drive)
        .await
        .expect("Failed to initialize Drive");

    let session_manager = Arc::new(tokio::sync::Mutex::new(session::SessionManager::new(
        diesel::Connection::establish(&cfg.database_url()).unwrap(),
        redis_client.clone(),
    )));

    let auth_service = Arc::new(tokio::sync::Mutex::new(auth::AuthService::new()));

    let conn = diesel::Connection::establish(&cfg.database_url()).unwrap();
    let config_manager = ConfigManager::new(Arc::new(Mutex::new(conn)));
    let mut bot_conn = diesel::Connection::establish(&cfg.database_url()).unwrap();
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
        conn: db_pool.clone(),
        bucket_name: "default.gbai".to_string(), // Default bucket name
        cache: redis_client.clone(),
        session_manager: session_manager.clone(),
        llm_provider: llm_provider.clone(),
        auth_service: auth_service.clone(),
        channels: Arc::new(Mutex::new({
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

    info!(
        "Starting HTTP server on {}:{}",
        config.server.host, config.server.port
    );
    let worker_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    // Initialize bot orchestrator and mount all bots
    let bot_orchestrator = BotOrchestrator::new(app_state.clone());

    // Mount all active bots from database
    if let Err(e) = bot_orchestrator.mount_all_bots().await {
        log::error!("Failed to mount bots: {}", e);
        // Use BotOrchestrator::send_warning to notify system admins
        let msg = format!("Bot mount failure: {}", e);
        let _ = bot_orchestrator
            .send_warning("System", "AdminBot", msg.as_str())
            .await;
    } else {
        let _sessions = get_sessions;
        log::info!("Session handler registered successfully");
    }

    let automation_state = app_state.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime for automation");
        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, async move {
            let automation = AutomationService::new(automation_state);
            automation.spawn().await.ok();
        });
    });

    if let Err(e) = ensure_llama_servers_running(&app_state).await {
        error!("Failed to stat LLM servers: {}", e);
    }

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        let app_state_clone = app_state.clone();
        let mut app = App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(Logger::new("HTTP REQUEST: %a %{User-Agent}i"))
            .app_data(web::Data::from(app_state_clone))
            .service(auth_handler)
            .service(create_session)
            .service(get_session_history)
            .service(get_sessions)
            .service(index)
            .service(start_session)
            .service(upload_file)
            .service(voice_start)
            .service(voice_stop)
            .service(websocket_handler)
            .service(crate::bot::create_bot_handler)
            .service(crate::bot::mount_bot_handler)
            .service(crate::bot::handle_user_input_handler)
            .service(crate::bot::get_user_sessions_handler)
            .service(crate::bot::get_conversation_history_handler);

        #[cfg(feature = "email")]
        {
            app = app
                .service(get_latest_email_from)
                .service(get_emails)
                .service(list_emails)
                .service(send_email)
                .service(save_draft)
                .service(save_click);
        }
        app = app.service(static_files);
        app = app.service(bot_index);
        app
    })
    .workers(worker_count)
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
