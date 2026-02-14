#![recursion_limit = "512"]

// Module declarations
pub mod main_module;

// Re-export commonly used items from main_module
pub use main_module::{BootstrapProgress, health_check, health_check_simple, receive_client_errors};

// Use jemalloc as the global allocator when the feature is enabled
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// Module declarations for feature-gated modules
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

// Re-export commonly used types
#[cfg(feature = "drive")]
pub use drive::drive_monitor::DriveMonitor;

#[cfg(feature = "llm")]
pub use llm::cache::{CacheConfig, CachedLLMProvider, CachedResponse, LocalEmbeddingService};
#[cfg(feature = "llm")]
pub use llm::DynamicLLMProvider;

#[cfg(feature = "tasks")]
pub use tasks::TaskEngine;

use dotenvy::dotenv;
use log::{error, info, trace, warn};
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    use main_module::{
        init_database, init_logging_and_i18n, load_config, parse_cli_args, run_axum_server,
        run_bootstrap, start_background_services, BootstrapProgress,
    };
    use crate::core::shared::memory_monitor::MemoryStats;
    use crate::core::shared::memory_monitor::register_thread;
    use crate::security::set_global_panic_hook;

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
        if let Err(e) = crate::core::shared::utils::init_secrets_manager().await {
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

    init_logging_and_i18n(no_console, no_ui);

    let (progress_tx, _progress_rx) = tokio::sync::mpsc::unbounded_channel::<BootstrapProgress>();
    let (state_tx, _state_rx) = tokio::sync::mpsc::channel::<Arc<crate::core::shared::state::AppState>>(1);

    if args.len() > 1 {
        let command = &args[1];
        match command.as_str() {
            "install" | "remove" | "list" | "status" | "start" | "stop" | "restart" | "--help"
            | "-h" => match crate::core::package_manager::cli::run().await {
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

    let (install_mode, tenant) = parse_cli_args(&args);

    if let Some(idx) = args.iter().position(|a| a == "--stack-path") {
        if let Some(path) = args.get(idx + 1) {
            std::env::set_var("BOTSERVER_STACK_PATH", path);
            info!("Using custom stack path: {}", path);
        }
    }

    let cfg = run_bootstrap(install_mode, tenant, &progress_tx).await?;

    trace!("Bootstrap config phase complete");
    trace!("Reloading dotenv...");
    dotenv().ok();

    let pool = init_database(&progress_tx).await?;
    let refreshed_cfg = load_config(&pool).await?;
    let config = std::sync::Arc::new(refreshed_cfg.clone());

    #[cfg(feature = "cache")]
    let redis_client = main_module::init_redis().await;

    #[cfg(not(feature = "cache"))]
    let redis_client: Option<Arc<redis::Client>> = None;

    let app_state = main_module::create_app_state(cfg, pool, &redis_client).await?;

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
    use crate::core::shared::memory_monitor::{log_process_memory, start_memory_monitor};
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

    let bot_orchestrator = crate::core::bot::BotOrchestrator::new(app_state.clone());
    if let Err(e) = bot_orchestrator.mount_all_bots() {
        error!("Failed to mount bots: {}", e);
    }

    #[cfg(feature = "llm")]
    {
        let app_state_for_llm = app_state.clone();
        trace!("ensure_llama_servers_running starting...");
        if let Err(e) = crate::llm::local::ensure_llama_servers_running(app_state_for_llm).await {
            error!("Failed to start LLM servers: {}", e);
        }
        trace!("ensure_llama_servers_running completed");
    }

    start_background_services(app_state.clone(), &app_state.conn).await;

    #[cfg(feature = "automation")]
    {
        let automation_state = app_state.clone();
        tokio::spawn(async move {
            register_thread("automation-service", "automation");
            let automation = crate::core::automation::AutomationService::new(automation_state);
            trace!(
                "[TASK] AutomationService starting, RSS={}",
                MemoryStats::format_bytes(MemoryStats::current().rss_bytes)
            );
            loop {
                crate::core::shared::memory_monitor::record_thread_activity("automation-service");
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


