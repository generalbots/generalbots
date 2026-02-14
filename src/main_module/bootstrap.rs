//! Bootstrap and application initialization logic

use log::{error, info, trace, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::core::bot::channels::{VoiceAdapter, WebChannelAdapter};
use crate::core::bot::BotOrchestrator;
use crate::core::bot_database::BotDatabaseManager;
use crate::core::config::AppConfig;
use crate::core::config::ConfigManager;
use crate::core::package_manager::InstallMode;
use crate::core::session::SessionManager;
use crate::core::shared::state::AppState;
use crate::core::shared::utils::create_conn;
use crate::core::shared::utils::create_s3_operator;

use super::BootstrapProgress;

#[cfg(feature = "llm")]
use crate::llm::local::ensure_llama_servers_running;

/// Initialize logging and i18n
pub fn init_logging_and_i18n(no_console: bool, no_ui: bool) {
    use crate::core::i18n;

    if no_console || no_ui {
        botlib::logging::init_compact_logger_with_style("info");
        println!("Starting General Bots {}...", env!("CARGO_PKG_VERSION"));
    }

    let locales_path = if std::path::Path::new("./locales").exists() {
        "./locales"
    } else if std::path::Path::new("../botlib/locales").exists() {
        "../botlib/locales"
    } else if std::path::Path::new("../locales").exists() {
        "../locales"
    } else {
        "./locales"
    };
    if let Err(e) = i18n::init_i18n(locales_path) {
        warn!(
            "Failed to initialize i18n from {}: {}. Translations will show keys.",
            locales_path, e
        );
    } else {
        info!(
            "i18n initialized from {} with locales: {:?}",
            locales_path,
            i18n::available_locales()
        );
    }
}

/// Parse command line arguments for install mode and tenant
pub fn parse_cli_args(args: &[String]) -> (InstallMode, Option<String>) {
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

    (install_mode, tenant)
}

/// Run the bootstrap process
pub async fn run_bootstrap(
    install_mode: InstallMode,
    tenant: Option<String>,
    progress_tx: &tokio::sync::mpsc::UnboundedSender<BootstrapProgress>,
) -> Result<AppConfig, std::io::Error> {
    use crate::core::bootstrap::BootstrapManager;

    trace!("Starting bootstrap process...");
    let progress_tx_clone = progress_tx.clone();
    let cfg = {
        progress_tx_clone
            .send(BootstrapProgress::StartingBootstrap)
            .ok();

        trace!("Creating BootstrapManager...");
        let mut bootstrap = BootstrapManager::new(install_mode, tenant);

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
                    AppConfig::from_database(&pool).unwrap_or_else(|e| {
                        warn!("Failed to load config from database: {}, trying env", e);
                        AppConfig::from_env().unwrap_or_else(|env_e| {
                            error!("Failed to load config from env: {}", env_e);
                            AppConfig::default()
                        })
                    })
                }
                Err(e) => {
                    trace!(
                        "Database connection failed: {:?}, loading config from env",
                        e
                    );
                    AppConfig::from_env().unwrap_or_else(|e| {
                        error!("Failed to load config from env: {}", e);
                        AppConfig::default()
                    })
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
                Ok(pool) => AppConfig::from_database(&pool).unwrap_or_else(|e| {
                    warn!("Failed to load config from database: {}, trying env", e);
                    AppConfig::from_env().unwrap_or_else(|env_e| {
                        error!("Failed to load config from env: {}", env_e);
                        AppConfig::default()
                    })
                }),
                Err(_) => AppConfig::from_env().unwrap_or_else(|e| {
                    error!("Failed to load config from env: {}", e);
                    AppConfig::default()
                }),
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
    cfg
}

/// Initialize database pool and run migrations
pub async fn init_database(
    progress_tx: &tokio::sync::mpsc::UnboundedSender<BootstrapProgress>,
) -> Result<crate::core::shared::utils::DbPool, std::io::Error> {
    use crate::core::shared::utils;

    trace!("Creating database pool again...");
    progress_tx.send(BootstrapProgress::ConnectingDatabase).ok();

    // Ensure secrets manager is initialized before creating database connection
    crate::core::shared::utils::init_secrets_manager().await;

    let pool = match create_conn() {
        Ok(pool) => {
            trace!("Running database migrations...");
            info!("Running database migrations...");
            if let Err(e) = utils::run_migrations(&pool) {
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

    Ok(pool)
}

/// Load configuration from database
pub async fn load_config(
    pool: &crate::core::shared::utils::DbPool,
) -> Result<AppConfig, std::io::Error> {
    info!("Loading config from database after template sync...");
    let refreshed_cfg = AppConfig::from_database(pool).unwrap_or_else(|e| {
        warn!(
            "Failed to load config from database: {}, falling back to env",
            e
        );
        AppConfig::from_env().unwrap_or_else(|e| {
            error!("Failed to load config from env: {}", e);
            AppConfig::default()
        })
    });
    let config = std::sync::Arc::new(refreshed_cfg.clone());
    info!(
        "Server configured to listen on {}:{}",
        config.server.host, config.server.port
    );

    Ok(refreshed_cfg)
}

/// Initialize Redis cache
#[cfg(feature = "cache")]
pub async fn init_redis() -> Option<Arc<redis::Client>> {
    let cache_url = "redis://localhost:6379".to_string();
    match redis::Client::open(cache_url.as_str()) {
        Ok(client) => Some(Arc::new(client)),
        Err(e) => {
            log::warn!("Failed to connect to Redis: {}", e);
            None
        }
    }
}

/// Create the AppState
pub async fn create_app_state(
    cfg: AppConfig,
    pool: crate::core::shared::utils::DbPool,
    #[cfg(feature = "cache")] redis_client: &Option<Arc<redis::Client>>,
) -> Result<Arc<AppState>, std::io::Error> {
    use std::collections::HashMap;

    let config = std::sync::Arc::new(cfg.clone());

    #[cfg(feature = "cache")]
    let redis_client = redis_client.clone();
    #[cfg(not(feature = "cache"))]
    let redis_client: Option<Arc<redis::Client>> = None;

    let web_adapter = Arc::new(WebChannelAdapter::new());
    let voice_adapter = Arc::new(VoiceAdapter::new());

    #[cfg(feature = "drive")]
    let drive = match create_s3_operator(&config.drive).await {
        Ok(client) => client,
        Err(e) => {
            return Err(std::io::Error::other(format!("Failed to initialize Drive: {}", e)));
        }
    };

    #[cfg(feature = "drive")]
    super::ensure_vendor_files_in_minio(&drive).await;

    let session_manager = Arc::new(Mutex::new(SessionManager::new(
        pool.get().map_err(|e| {
            std::io::Error::other(format!("Failed to get database connection: {}", e))
        })?,
        #[cfg(feature = "cache")]
        redis_client.clone(),
    )));

    #[cfg(feature = "directory")]
    let (auth_service, zitadel_config) = init_directory_service()?;

    #[cfg(feature = "directory")]
    bootstrap_directory_admin(&zitadel_config).await;

    let config_manager = ConfigManager::new(pool.clone());

    let mut bot_conn = pool
        .get()
        .map_err(|e| std::io::Error::other(format!("Failed to get database connection: {}", e)))?;
    let (default_bot_id, default_bot_name) = crate::core::bot::get_default_bot(&mut bot_conn);
    info!(
        "Using default bot: {} (id: {})",
        default_bot_name, default_bot_id
    );

    let llm_url = config_manager
        .get_config(&default_bot_id, "llm-url", Some("http://localhost:8081"))
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    info!("LLM URL: {}", llm_url);

    let llm_model = config_manager
        .get_config(&default_bot_id, "llm-model", Some(""))
        .unwrap_or_default();
    if !llm_model.is_empty() {
        info!("LLM Model: {}", llm_model);
    }

    let _llm_key = config_manager
        .get_config(&default_bot_id, "llm-key", Some(""))
        .unwrap_or_default();

    // LLM endpoint path configuration
    let llm_endpoint_path = config_manager
        .get_config(
            &default_bot_id,
            "llm-endpoint-path",
            Some("/v1/chat/completions"),
        )
        .unwrap_or_else(|_| "/v1/chat/completions".to_string());

    #[cfg(feature = "llm")]
    let base_llm_provider = crate::llm::create_llm_provider_from_url(
        &llm_url,
        if llm_model.is_empty() {
            None
        } else {
            Some(llm_model.clone())
        },
        Some(llm_endpoint_path.clone()),
    );

    #[cfg(feature = "llm")]
    let dynamic_llm_provider = Arc::new(crate::llm::DynamicLLMProvider::new(base_llm_provider));

    #[cfg(feature = "llm")]
    {
        // Ensure the DynamicLLMProvider is initialized with the correct config from database
        // This makes the system robust: even if the URL was set before server startup,
        // the provider will use the correct configuration
        info!("Initializing DynamicLLMProvider with config: URL={}, Model={}, Endpoint={}",
              llm_url,
              if llm_model.is_empty() { "(default)" } else { &llm_model },
              llm_endpoint_path.clone());
        dynamic_llm_provider.update_from_config(
            &llm_url,
            if llm_model.is_empty() { None } else { Some(llm_model.clone()) },
            Some(llm_endpoint_path),
        ).await;
        info!("DynamicLLMProvider initialized successfully");
    }

    #[cfg(feature = "llm")]
    let llm_provider = init_llm_provider(
        &config_manager,
        default_bot_id.to_string().as_str(),
        dynamic_llm_provider.clone(),
        &pool,
        redis_client.clone(),
    );

    #[cfg(any(feature = "research", feature = "llm"))]
    let kb_manager = Arc::new(crate::core::kb::KnowledgeBaseManager::new("work"));

    #[cfg(feature = "tasks")]
    let task_engine = Arc::new(crate::tasks::TaskEngine::new(pool.clone()));

    let metrics_collector = crate::core::shared::analytics::MetricsCollector::new();

    #[cfg(feature = "tasks")]
    let task_scheduler = None;

    let (attendant_tx, _attendant_rx) =
        tokio::sync::broadcast::channel::<crate::core::shared::state::AttendantNotification>(1000);

    let (task_progress_tx, _task_progress_rx) =
        tokio::sync::broadcast::channel::<crate::core::shared::state::TaskProgressEvent>(1000);

    // Initialize BotDatabaseManager for per-bot database support
    let database_url = crate::core::shared::utils::get_database_url_sync().unwrap_or_default();
    let bot_database_manager = Arc::new(BotDatabaseManager::new(pool.clone(), &database_url));

    // Sync all bot databases on startup - ensures each bot has its own database
    info!("Syncing bot databases on startup...");
    match bot_database_manager.sync_all_bot_databases() {
        Ok(sync_result) => {
            info!(
                "Bot database sync complete: {} created, {} verified, {} errors",
                sync_result.databases_created,
                sync_result.databases_verified,
                sync_result.errors.len()
            );
            for err in &sync_result.errors {
                warn!("Bot database sync error: {}", err);
            }
        }
        Err(e) => {
            error!("Failed to sync bot databases: {}", e);
        }
    }

    let app_state = Arc::new(AppState {
        #[cfg(feature = "drive")]
        drive: Some(drive),
        config: Some(cfg.clone()),
        conn: pool.clone(),
        database_url: database_url.clone(),
        bot_database_manager: bot_database_manager.clone(),
        bucket_name: "default.gbai".to_string(),
        #[cfg(feature = "cache")]
        cache: redis_client.clone(),
        session_manager: session_manager.clone(),
        metrics_collector,
        #[cfg(feature = "tasks")]
        task_scheduler,
        #[cfg(feature = "llm")]
        llm_provider: llm_provider.clone(),
        #[cfg(feature = "llm")]
        dynamic_llm_provider: Some(dynamic_llm_provider.clone()),
        #[cfg(feature = "directory")]
        auth_service: auth_service.clone(),
        channels: Arc::new(tokio::sync::Mutex::new({
            let mut map = HashMap::new();
            map.insert(
                "web".to_string(),
                web_adapter.clone() as Arc<dyn crate::core::bot::channels::ChannelAdapter>,
            );
            map
        })),
        response_channels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        web_adapter: web_adapter.clone(),
        voice_adapter: voice_adapter.clone(),
        #[cfg(any(feature = "research", feature = "llm"))]
        kb_manager: Some(kb_manager.clone()),
        #[cfg(feature = "tasks")]
        task_engine,
        extensions: {
            let ext = crate::core::shared::state::Extensions::new();
            #[cfg(feature = "llm")]
            ext.insert_blocking(Arc::clone(&dynamic_llm_provider));
            ext
        },
        attendant_broadcast: Some(attendant_tx),
        task_progress_broadcast: Some(task_progress_tx),
        billing_alert_broadcast: None,
        task_manifests: Arc::new(std::sync::RwLock::new(HashMap::new())),
        #[cfg(feature = "project")]
        project_service: Arc::new(tokio::sync::RwLock::new(
            crate::project::ProjectService::new(),
        )),
        #[cfg(feature = "compliance")]
        legal_service: Arc::new(tokio::sync::RwLock::new(crate::legal::LegalService::new())),
        jwt_manager: None,
        auth_provider_registry: None,
        rbac_manager: None,
    });

    Ok(app_state)
}

#[cfg(feature = "directory")]
fn init_directory_service() -> Result<(Arc<Mutex<crate::directory::AuthService>>, crate::directory::ZitadelConfig), std::io::Error> {
    let zitadel_config = {
        // Try to load from directory_config.json first
        let config_path = "./config/directory_config.json";
        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let base_url = json
                    .get("base_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("http://localhost:8300");
                let client_id = json.get("client_id").and_then(|v| v.as_str()).unwrap_or("");
                let client_secret = json
                    .get("client_secret")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                info!(
                    "Loaded Zitadel config from {}: url={}",
                    config_path, base_url
                );

                crate::directory::ZitadelConfig {
                    issuer_url: base_url.to_string(),
                    issuer: base_url.to_string(),
                    client_id: client_id.to_string(),
                    client_secret: client_secret.to_string(),
                    redirect_uri: format!("{}/callback", base_url),
                    project_id: "default".to_string(),
                    api_url: base_url.to_string(),
                    service_account_key: None,
                }
            } else {
                warn!("Failed to parse directory_config.json, using defaults");
                default_zitadel_config()
            }
        } else {
            warn!("directory_config.json not found, using default Zitadel config");
            default_zitadel_config()
        }
    };

    let auth_service = Arc::new(tokio::sync::Mutex::new(
        crate::directory::AuthService::new(zitadel_config.clone())
            .map_err(|e| std::io::Error::other(format!("Failed to create auth service: {}", e)))?,
    ));

    Ok((auth_service, zitadel_config))
}

#[cfg(feature = "directory")]
fn default_zitadel_config() -> crate::directory::ZitadelConfig {
    crate::directory::ZitadelConfig {
        issuer_url: "http://localhost:8300".to_string(),
        issuer: "http://localhost:8300".to_string(),
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uri: "http://localhost:8300/callback".to_string(),
        project_id: "default".to_string(),
        api_url: "http://localhost:8300".to_string(),
        service_account_key: None,
    }
}

#[cfg(feature = "directory")]
async fn bootstrap_directory_admin(zitadel_config: &crate::directory::ZitadelConfig) {
    use crate::directory::{bootstrap, ZitadelClient};

    let pat_path = std::path::Path::new("./botserver-stack/conf/directory/admin-pat.txt");
    let bootstrap_client = if pat_path.exists() {
        match std::fs::read_to_string(pat_path) {
            Ok(pat_token) => {
                let pat_token = pat_token.trim().to_string();
                info!("Using admin PAT token for bootstrap authentication");
                ZitadelClient::with_pat_token(zitadel_config.clone(), pat_token)
                    .map_err(|e| {
                        std::io::Error::other(format!(
                            "Failed to create bootstrap client with PAT: {}",
                            e
                        ))
                    })
            }
            Err(e) => {
                warn!(
                    "Failed to read admin PAT token: {}, falling back to OAuth2",
                    e
                );
                ZitadelClient::new(zitadel_config.clone()).map_err(|e| {
                    std::io::Error::other(format!("Failed to create bootstrap client: {}", e))
                })
            }
        }
    } else {
        info!("Admin PAT not found, using OAuth2 client credentials for bootstrap");
        ZitadelClient::new(zitadel_config.clone()).map_err(|e| {
            std::io::Error::other(format!("Failed to create bootstrap client: {}", e))
        })
    };

    let bootstrap_client = match bootstrap_client {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create bootstrap client: {}", e);
            return;
        }
    };

    match bootstrap::check_and_bootstrap_admin(&bootstrap_client).await {
        Ok(Some(_)) => {
            info!("Bootstrap completed - admin credentials displayed in console");
        }
        Ok(None) => {
            info!("Admin user exists, bootstrap skipped");
        }
        Err(e) => {
            warn!("Bootstrap check failed (Zitadel may not be ready): {}", e);
        }
    }
}

#[cfg(feature = "llm")]
fn init_llm_provider(
    config_manager: &ConfigManager,
    default_bot_id: &str,
    dynamic_llm_provider: Arc<crate::llm::DynamicLLMProvider>,
    pool: &crate::core::shared::utils::DbPool,
    redis_client: Option<Arc<redis::Client>>,
) -> Arc<dyn crate::llm::LLMProvider> {
    use crate::llm::cache::{CacheConfig, CachedLLMProvider, EmbeddingService, LocalEmbeddingService};

    if let Some(ref cache) = redis_client {
        let bot_id = Uuid::parse_str(default_bot_id).unwrap_or_default();
        let embedding_url = config_manager
            .get_config(
                &bot_id,
                "embedding-url",
                Some("http://localhost:8082"),
            )
            .unwrap_or_else(|_| "http://localhost:8082".to_string());
        let embedding_model = config_manager
            .get_config(&bot_id, "embedding-model", Some("all-MiniLM-L6-v2"))
            .unwrap_or_else(|_| "all-MiniLM-L6-v2".to_string());
        info!("Embedding URL: {}", embedding_url);
        info!("Embedding Model: {}", embedding_model);

        let embedding_service = Some(Arc::new(LocalEmbeddingService::new(
            embedding_url,
            embedding_model,
        )) as Arc<dyn EmbeddingService>);

        let cache_config = CacheConfig {
            ttl: 3600,
            semantic_matching: true,
            similarity_threshold: 0.85,
            max_similarity_checks: 100,
            key_prefix: "llm_cache".to_string(),
        };

        Arc::new(CachedLLMProvider::with_db_pool(
            dynamic_llm_provider.clone() as Arc<dyn crate::llm::LLMProvider>,
            cache.clone(),
            cache_config,
            embedding_service,
            pool.clone(),
        ))
    } else {
        dynamic_llm_provider.clone() as Arc<dyn crate::llm::LLMProvider>
    }
}

/// Start background services and monitors
pub async fn start_background_services(
    app_state: Arc<AppState>,
    pool: &crate::core::shared::utils::DbPool,
) {
    #[cfg(feature = "drive")]
    use crate::core::shared::memory_monitor::{log_process_memory, start_memory_monitor};

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
    start_memory_monitor(30, 50);
    info!("Memory monitor started");
    log_process_memory();

    let bot_orchestrator = BotOrchestrator::new(app_state.clone());
    if let Err(e) = bot_orchestrator.mount_all_bots() {
        error!("Failed to mount bots: {}", e);
    }

    #[cfg(feature = "llm")]
    {
        let app_state_for_llm = app_state.clone();
        trace!("ensure_llama_servers_running starting...");
        if let Err(e) = ensure_llama_servers_running(app_state_for_llm).await {
            error!("Failed to start LLM servers: {}", e);
        }
        trace!("ensure_llama_servers_running completed");
    }

    #[cfg(feature = "drive")]
    start_drive_monitors(app_state.clone(), pool).await;
}

#[cfg(feature = "drive")]
async fn start_drive_monitors(
    app_state: Arc<AppState>,
    pool: &crate::core::shared::utils::DbPool,
) {
    use crate::core::shared::memory_monitor::register_thread;
    use crate::core::shared::models::schema::bots;
    use diesel::prelude::*;

    // Start DriveMonitor for each active bot
    let drive_monitor_state = app_state.clone();
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        register_thread("drive-monitor", "drive");

        let bots_to_monitor = tokio::task::spawn_blocking(move || {
            use uuid::Uuid;
            let mut conn = match pool_clone.get() {
                Ok(conn) => conn,
                Err(_) => return Vec::new(),
            };
            bots::dsl::bots.filter(bots::dsl::is_active.eq(true))
                .select((bots::dsl::id, bots::dsl::name))
                .load::<(Uuid, String)>(&mut conn)
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default();

        info!("Found {} active bots to monitor", bots_to_monitor.len());

        for (bot_id, bot_name) in bots_to_monitor {
            // Skip default bot - it's managed locally via ConfigWatcher
            if bot_name == "default" {
                info!("Skipping DriveMonitor for 'default' bot - managed via ConfigWatcher");
                continue;
            }

            let bucket_name = format!("{}.gbai", bot_name);
            let monitor_state = drive_monitor_state.clone();
            let bot_id_clone = bot_id;
            let bucket_name_clone = bucket_name.clone();

            tokio::spawn(async move {
                use crate::DriveMonitor;
                register_thread(&format!("drive-monitor-{}", bot_name), "drive");
                trace!("DriveMonitor::new starting for bot: {}", bot_name);
                let monitor =
                    DriveMonitor::new(monitor_state, bucket_name_clone, bot_id_clone);
                trace!(
                    "DriveMonitor::new done for bot: {}, calling start_monitoring...",
                    bot_name
                );
                info!(
                    "Starting DriveMonitor for bot: {} (bucket: {})",
                    bot_name, bucket_name
                );
                if let Err(e) = monitor.start_monitoring().await {
                    error!("DriveMonitor failed for bot {}: {}", bot_name, e);
                }
                trace!(
                    "DriveMonitor start_monitoring returned for bot: {}",
                    bot_name
                );
            });
        }
    });

    // Start local file monitor for ~/data/*.gbai directories
    let local_monitor_state = app_state.clone();
    tokio::spawn(async move {
        register_thread("local-file-monitor", "drive");
        trace!("Starting LocalFileMonitor for ~/data/*.gbai directories");
        let monitor = crate::drive::local_file_monitor::LocalFileMonitor::new(local_monitor_state);
        if let Err(e) = monitor.start_monitoring().await {
            error!("LocalFileMonitor failed: {}", e);
        } else {
            info!("LocalFileMonitor started - watching ~/data/*.gbai/.gbdialog/*.bas");
        }
    });

    // Start config file watcher for ~/data/*.gbai/*.gbot/config.csv
    let config_watcher_state = app_state.clone();
    tokio::spawn(async move {
        register_thread("config-file-watcher", "drive");
        trace!("Starting ConfigWatcher for ~/data/*.gbai/*.gbot/config.csv");

        // Determine data directory
        let data_dir = std::env::var("DATA_DIR")
            .or_else(|_| std::env::var("HOME").map(|h| format!("{}/data", h)))
            .unwrap_or_else(|_| "./botserver-stack/data".to_string());
        let data_dir = std::path::PathBuf::from(data_dir);

        let watcher = crate::core::config::watcher::ConfigWatcher::new(
            data_dir,
            config_watcher_state,
        );
        Arc::new(watcher).spawn();

        info!("ConfigWatcher started - watching ~/data/*.gbai/*.gbot/config.csv");
    });
}
