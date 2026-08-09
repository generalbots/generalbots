#![recursion_limit = "512"]

// Module declarations
pub mod main_module; // ci-timing
pub mod session_pool;
pub mod soon_features;

// Re-export commonly used items from main_module
pub use main_module::{BootstrapProgress, health_check, health_check_simple, receive_client_errors};

// Use mimalloc as the global allocator when the feature is enabled (replaced tikv-jemalloc due to RUSTSEC-2024-0436)
#[cfg(feature = "jemalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Module declarations for feature-gated modules
#[cfg(feature = "analytics")]
pub mod analytics;
#[cfg(feature = "attendant")]
pub mod attendant;
#[cfg(feature = "automation")]
pub use botautotask as auto_task;
#[cfg(feature = "vibe")]
pub mod vibe;
#[cfg(feature = "scripting")]
pub mod basic;
#[cfg(feature = "billing")]
pub mod billing;
#[cfg(feature = "saas")]
pub mod management;
pub mod botmodels;
#[cfg(feature = "canvas")]
pub mod canvas;
#[cfg(feature = "social")]
pub mod channels;
#[cfg(feature = "people")]
pub mod contacts;
#[cfg(feature = "people")]
pub mod crm;
pub mod core;
#[cfg(feature = "designer")]
pub mod designer;
#[cfg(feature = "dashboards")]
pub mod dashboards;
#[cfg(feature = "deployment")]
pub mod deployment;
pub mod api;
pub mod browser;
#[cfg(feature = "docs")]
pub mod docs;
pub mod embedded_ui;
#[cfg(feature = "learn")]
pub mod learn;
#[cfg(feature = "legal")]
pub mod legal;
#[cfg(feature = "maintenance")]
pub mod maintenance;
#[cfg(feature = "monitoring")]
pub mod monitoring;
#[cfg(feature = "multimodal")]
pub mod multimodal;
#[cfg(feature = "marketing")]
pub mod marketing;
#[cfg(feature = "paper")]
pub mod paper;
#[cfg(feature = "people")]
pub mod people;
#[cfg(feature = "player")]
pub mod player;
#[cfg(feature = "products")]
pub mod products;
#[cfg(feature = "project")]
pub mod project;
#[cfg(feature = "research")]
pub mod research;
#[cfg(feature = "search")]
pub mod search;
#[cfg(feature = "security")]
pub mod security;
pub mod settings;
#[cfg(feature = "sheet")]
pub mod sheet;
#[cfg(feature = "slides")]
pub mod slides;
#[cfg(feature = "plan")]
pub mod plan;
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
pub mod apps;


#[cfg(feature = "desktop")]
pub mod desktop;

#[cfg(feature = "fraud")]
pub mod fraud;

#[cfg(feature = "inventory")]
pub mod inventory;

#[cfg(feature = "gl")]
pub mod gl;

#[cfg(feature = "retail")]
pub mod retail;

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

#[cfg(feature = "timeseries")]
pub mod timeseries;

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
#[cfg(feature = "llm")]
pub use llm::cache::{CacheConfig, CachedLLMProvider, CachedResponse, LocalEmbeddingService};
#[cfg(feature = "llm")]
pub use llm::DynamicLLMProvider;

#[cfg(feature = "tasks")]
pub use tasks::TaskEngine;

use dotenvy::dotenv;
use log::{error, info, trace, warn};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    use main_module::{
        init_database, init_logging_and_i18n, load_config, parse_cli_args, run_axum_server,
        run_bootstrap, start_background_services, BootstrapProgress,
    };
    #[cfg(not(feature = "security"))]
    fn set_global_panic_hook() {}

    set_global_panic_hook();

    let args: Vec<String> = std::env::args().collect();
    let no_ui = args.contains(&"--noui".to_string());

    crate::main_module::handle_security_command(&args).await;

    #[cfg(feature = "console")]
    let no_console = args.contains(&"--noconsole".to_string());

    #[cfg(not(feature = "console"))]
    let no_console = true;

    let _ = rustls::crypto::ring::default_provider().install_default();

    if std::env::var("GBO_NO_DOTENV").as_deref() != Ok("1") {
        dotenvy::dotenv().ok();
    }

    let env_path_early = std::path::Path::new("./.env");
    let stack = botcore::shared::utils::get_stack_path();
    let vault_init_path_early = std::path::PathBuf::from(format!("{}/conf/vault/init.json", stack));
    let vault_addr = std::env::var("VAULT_ADDR").unwrap_or_default();
    let is_remote_vault = !vault_addr.is_empty()
        && !vault_addr.contains("localhost")
        && !vault_addr.contains("127.0.0.1");

    let bootstrap_ready = is_remote_vault || (env_path_early.exists() && vault_init_path_early.exists() && {
        std::fs::read_to_string(env_path_early)
            .map(|content| content.contains("VAULT_TOKEN="))
            .unwrap_or(false)
    });

    if bootstrap_ready {
        if let Err(e) = botcore::shared::utils::init_secrets_manager().await {
            warn!(
                "Failed to initialize SecretsManager: {}. Falling back to env vars.",
                e
            );
        } else {
            info!("Secrets loaded from Vault");
        }
    } else {
        trace!("Bootstrap not complete - skipping early SecretsManager init");
    }

    let noise_filters = crate::main_module::get_noise_filters();

    let rust_log = match std::env::var("RUST_LOG") {
        Ok(existing) if !existing.is_empty() => format!("{},{}", existing, noise_filters),
        _ => format!("info,{}", noise_filters),
    };
// Test mold+incremental build

    std::env::set_var("RUST_LOG", &rust_log);

    init_logging_and_i18n(no_console, no_ui);

    let (progress_tx, _progress_rx) = tokio::sync::mpsc::unbounded_channel::<BootstrapProgress>();
    let (state_tx, _state_rx) = tokio::sync::mpsc::channel::<Arc<botcore::shared::state::AppState>>(1);

    if args.len() > 1 {
        let command = &args[1];
        match command.as_str() {
            "install" | "remove" | "list" | "status" | "start" | "stop" | "restart"
            | "rotate-secret" | "rotate-secrets" | "vault"
            | "--version" | "-v" | "--help" | "-h" => match crate::core::package_manager::cli::run().await {
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
            std::env::set_var("GBO_STACK_PATH", path);
            info!("Using custom stack path: {}", path);
        }
    }

    let cfg = run_bootstrap(install_mode, tenant, &progress_tx).await?;

    trace!("Bootstrap config phase complete");
    trace!("Reloading dotenv...");
    if std::env::var("GBO_NO_DOTENV").as_deref() != Ok("1") {
        dotenv().ok();
    }

    let pool = init_database(&progress_tx).await?;
    info!("Database initialized - PostgreSQL connected");
    let refreshed_cfg = load_config(&pool).await?;
    let config = std::sync::Arc::new(refreshed_cfg.clone());

    #[cfg(feature = "cache")]
    let redis_client = main_module::init_redis().await;

    #[cfg(not(feature = "cache"))]
    let redis_client: Option<Arc<redis::Client>> = None;

    let app_state = main_module::create_app_state(cfg, pool, &redis_client).await?;

    // Seed the fiscal Drive objects (invoice folder + cash-flow spreadsheets,
    // issues #722/#723/#724) once the drive client is available.
    #[cfg(feature = "sampledata")]
    if let Some(drive) = app_state.drive.clone() {
        let seed_pool = app_state.conn.clone();
        tokio::spawn(async move {
            botsampledata::seed_drive_fiscal(&seed_pool, drive.as_ref()).await;
        });
    }

    // #750 — seed the Pragmatismo reference bot payload (start.bas with VIBE
    // bridge keywords, PROMPT.md, config.csv and MCP tool definitions) into
    // the pragmatismo.gbai bucket.
    #[cfg(feature = "sampledata")]
    if let Some(drive) = app_state.drive.clone() {
        tokio::spawn(async move {
            botsampledata::seed_pragmatismo_payload(drive.as_ref()).await;
        });
    }

    // Wire SESSION_CACHE into auth middleware so gb_xxx tokens resolve
    // to their stored roles (e.g. "admin") instead of falling back to Role::User.
    // When the in-memory cache misses (e.g. after a restart), sessions are
    // rehydrated from the login_sessions table so users keep their login.
    #[cfg(all(feature = "security", feature = "directory"))]
    {
        // Wire the DB pool so suite sessions persist across restarts.
        botcoredirectory::auth_routes::set_session_pool(app_state.conn.clone());
        let session_pool = app_state.conn.clone();
        botsecurity::set_session_cache_lookup(Box::new(move |token: &str| {
            let cache = botcoredirectory::auth_routes::SESSION_CACHE.try_read().ok()?;
            match cache.get(token) {
                Some(u) => Some(botsecurity::SessionCacheEntry {
                    user_id: u.user_id.clone(),
                    email: u.email.clone(),
                    roles: u.roles.clone(),
                }),
                None => {
                    // Rehydrate from the persisted login_sessions table.
                    use diesel::RunQueryDsl;
                    let mut conn = session_pool.get().ok()?;
                    #[derive(diesel::QueryableByName)]
                    struct Row {
                        #[diesel(sql_type = diesel::sql_types::Text)]
                        user_data: String,
                    }
                    let row: Row = match diesel::sql_query(
                        "SELECT user_data FROM login_sessions WHERE token = $1 LIMIT 1",
                    )
                    .bind::<diesel::sql_types::Text, _>(token)
                    .get_result(&mut conn)
                    {
                        Ok(r) => r,
                        Err(_) => return None,
                    };
                    let user: botcoredirectory::auth_routes::SessionUserData =
                        serde_json::from_str(&row.user_data).ok()?;
                    Some(botsecurity::SessionCacheEntry {
                        user_id: user.user_id.clone(),
                        email: user.email.clone(),
                        roles: user.roles.clone(),
                    })
                }
            }
        }));
    }

    // Resume workflows after server restart
    if let Err(e) =
        crate::basic::keywords::orchestration::resume_workflows_on_startup(Arc::new(crate::basic::AppStateBasicRuntime(app_state.clone()))).await
    {
        log::warn!("Failed to resume workflows on startup: {}", e);
    }

    crate::main_module::init_task_scheduler(&app_state);

    #[cfg(any(feature = "research", feature = "llm"))]
    if let Err(e) = crate::core::kb::ensure_crawler_service_running(app_state.clone()).await {
        log::warn!("Failed to start website crawler service: {}", e);
    }

    // Start memory monitoring - check every 30 seconds, warn if growth > 50MB
    use botcore::shared::memory_monitor::{log_process_memory, start_memory_monitor};
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

    crate::main_module::start_automation_service(app_state.clone());

    trace!("Initial data setup task spawned");
    trace!("All system threads started, starting HTTP server...");

    // Watchdog thread: logs diagnostics every 10s, independent of tokio runtime.
    // If the tokio runtime freezes, this thread continues writing logs.
    {
        use std::time::Duration;
        let pool = app_state.conn.clone();
        let server_port = config.server.port;
        std::thread::Builder::new()
            .name("watchdog".to_string())
            .spawn(move || {
                info!("[watchdog] started on std::thread, monitoring tokio runtime");
                loop {
                    std::thread::sleep(Duration::from_secs(10));
                    let state = pool.state();
                    let http_ok = match (format!("127.0.0.1:{}", server_port)).parse::<std::net::SocketAddr>() {
                        Ok(addr) => std::net::TcpStream::connect_timeout(
                            &addr,
                            Duration::from_secs(1),
                        ).is_ok(),
                        Err(_) => false,
                    };
                    info!(
                        "[watchdog] alive | pool: {} conns, {} idle, {} in_use | http: {}",
                        state.connections,
                        state.idle_connections,
                        state.connections.saturating_sub(state.idle_connections),
                        if http_ok { "OK" } else { "DOWN" },
                    );
                }
            })
            .ok();
    }

    info!("Server started on port {}", config.server.port);
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

use std::sync::Arc;
#[cfg(feature = "security")]
use crate::security::set_global_panic_hook;