#![cfg_attr(feature = "desktop", windows_subsystem = "windows")]
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
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
    let (http_tx, http_rx) = tokio::sync::oneshot::channel();


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
            }
        }
    }


    if !no_ui {
        let progress_rx = Arc::new(tokio::sync::Mutex::new(progress_rx));
        let state_rx = Arc::new(tokio::sync::Mutex::new(state_rx));
        let handle = std::thread::Builder::new()
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
                        _ = http_rx => {}
                        _ = tokio::time::sleep(tokio::time::Duration::from_secs(300)) => {
                            eprintln!("UI initialization timeout");
                        }
                    }
                });
                if let Err(e) = ui.start_ui() {
                    eprintln!("UI error: {}", e);
                }
            })
            .expect("Failed to spawn UI thread");
        Some(handle)
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
    
    let http_handle = {
        let app_state = app_state.clone();
        let config = config.clone();
        let worker_count = worker_count;
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create HTTP runtime");
            rt.block_on(async {
                let server = HttpServer::new(move || {
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
                        .service(start_session)
                        .service(upload_file)
                        .service(voice_start)
                        .service(voice_stop)
                        .service(websocket_handler)
                        .service(crate::bot::create_bot_handler)
                        .service(crate::bot::mount_bot_handler)
                        .service(crate::bot::handle_user_input_handler)
                        .service(crate::bot::get_user_sessions_handler)
                        .service(crate::bot::get_conversation_history_handler)
                        .service(crate::bot::send_warning_handler);
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
                    app = app.configure(web_server::configure_app);

                    app
                })
                .workers(worker_count)
                .bind((config.server.host.clone(), config.server.port))?
                .run();
                let _ = http_tx.send(());
                server.await
            })
        })
    };

    let bot_orchestrator = BotOrchestrator::new(app_state.clone());
    tokio::spawn(async move {
        if let Err(e) = bot_orchestrator.mount_all_bots().await {
            error!("Failed to mount bots: {}", e);
        }
    });
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
    let app_state_for_llm = app_state.clone();
    tokio::spawn(async move {
        if let Err(e) = ensure_llama_servers_running(app_state_for_llm).await {
            error!("Failed to start LLM servers: {}", e);
        }
    });
    #[cfg(feature = "desktop")]
    if desktop_mode {
        // Tauri desktop mode
        let tauri_app = tauri::Builder::default()
            .setup(|app| {
                use tauri::WebviewWindowBuilder;

                    match WebviewWindowBuilder::new(
                        app,
                        "main",
                        tauri::WebviewUrl::App("index.html".into()),
                    )
                .build() {
                    Ok(_window) => Ok(()),
                    Err(e) if e.to_string().contains("WebviewLabelAlreadyExists") => {
                        log::warn!("Main window already exists, reusing existing window");
                        Ok(())
                    }
                    Err(e) => Err(e.into())
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

    http_handle.join().ok();
    Ok(())
}
