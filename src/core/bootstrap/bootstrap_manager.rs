// Bootstrap manager implementation
use crate::core::bootstrap::bootstrap_types::{BootstrapManager, BootstrapProgress};
use crate::core::bootstrap::bootstrap_utils::{alm_ci_health_check, alm_health_check, cache_health_check, drive_health_check, safe_pkill, tables_health_check, vault_health_check, vector_db_health_check, zitadel_health_check};
use crate::core::config::AppConfig;
use crate::core::package_manager::{InstallMode, PackageManager};
use crate::core::shared::utils::get_stack_path;
use crate::security::command_guard::SafeCommand;
use log::{info, warn};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

impl BootstrapManager {
    pub fn new(mode: InstallMode, tenant: Option<String>) -> Self {
        let stack_path = PathBuf::from(get_stack_path());

        Self {
            install_mode: mode,
            tenant,
            stack_path,
        }
    }

    pub fn stack_dir(&self, subpath: &str) -> PathBuf {
        self.stack_path.join(subpath)
    }

    pub fn vault_bin(&self) -> String {
        self.stack_dir("bin/vault/vault")
            .to_str()
            .unwrap_or(&get_stack_path())
            .to_string()
    }

    pub async fn kill_stack_processes(&self) -> anyhow::Result<()> {
        info!("Killing any existing stack processes...");

        let processes = crate::core::bootstrap::bootstrap_utils::get_processes_to_kill();
        for (name, args) in processes {
            safe_pkill(&[name.as_str()], &args);
        }

        // Give processes time to terminate
        sleep(Duration::from_millis(500)).await;

        info!("Stack processes terminated");
        Ok(())
    }

    pub async fn start_all(&mut self) -> anyhow::Result<()> {
        // If VAULT_ADDR points to a remote server, skip local service startup
        let vault_addr = std::env::var("VAULT_ADDR").unwrap_or_default();
        let is_remote_vault = !vault_addr.is_empty()
            && !vault_addr.contains("localhost")
            && !vault_addr.contains("127.0.0.1");

        if is_remote_vault {
            info!("Remote Vault detected ({}), skipping local service startup", vault_addr);
            info!("All services are assumed to be running in separate containers");
            return Ok(());
        }

        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;

        info!("Starting bootstrap process...");

        if pm.is_installed("vault") {
            let vault_already_running = vault_health_check();
            if vault_already_running {
                info!("Vault is already running");
            } else {
                info!("Starting Vault secrets service...");
                match pm.start("vault") {
                    Ok(_child) => {
                        info!("Vault process started, waiting for initialization...");
                        // Wait for vault to be ready
                        for _ in 0..10 {
                            sleep(Duration::from_secs(1)).await;
                            if vault_health_check() {
                                info!("Vault is responding");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Vault might already be running: {}", e);
                    }
                }
            }
        }

        if pm.is_installed("vector_db") {
            let vector_db_already_running = vector_db_health_check();
            if vector_db_already_running {
                info!("Vector database (Qdrant) is already running");
            } else {
                info!("Starting Vector database (Qdrant)...");
                match pm.start("vector_db") {
                    Ok(_child) => {
                        info!("Vector database process started, waiting for readiness...");
                        // Wait for vector_db to be ready (up to 45 seconds)
                        for i in 0..45 {
                            sleep(Duration::from_secs(1)).await;
                            if vector_db_health_check() {
                                info!("Vector database (Qdrant) is responding");
                                break;
                            }
                            if i == 44 {
                                warn!("Vector database did not respond after 45 seconds");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to start Vector database: {}", e);
                    }
                }
            }
        }

        if pm.is_installed("tables") {
            let tables_already_running = tables_health_check();
            if tables_already_running {
                info!("PostgreSQL is already running");
            } else {
                info!("Starting PostgreSQL...");
                match pm.start("tables") {
                    Ok(_child) => {
                        info!("PostgreSQL started");
                    }
                    Err(e) => {
                        warn!("Failed to start PostgreSQL: {}", e);
                    }
                }
            }
        }

        if pm.is_installed("cache") {
            let cache_already_running = cache_health_check();
            if cache_already_running {
                info!("Valkey cache is already running");
            } else {
                info!("Starting Valkey cache...");
                match pm.start("cache") {
                    Ok(_child) => {
                        info!("Valkey cache process started, waiting for readiness...");
                        for i in 0..30 {
                            sleep(Duration::from_secs(1)).await;
                            if cache_health_check() {
                                info!("Valkey cache is responding");
                                break;
                            }
                            if i == 29 {
                                warn!("Valkey cache did not respond after 30 seconds");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to start Valkey cache: {}", e);
                    }
                }
            }
        }

        if pm.is_installed("drive") {
            let drive_already_running = drive_health_check();
            if drive_already_running {
                info!("MinIO is already running");
            } else {
                info!("Starting MinIO...");
                match pm.start("drive") {
                    Ok(_child) => {
                        info!("MinIO started");
                    }
                    Err(e) => {
                        warn!("Failed to start MinIO: {}", e);
                    }
                }
            }
        }

        if pm.is_installed("directory") {
            let directory_already_running = zitadel_health_check();

            if directory_already_running {
                info!("Zitadel/Directory service is already running");

                let config_path = self.stack_dir("conf/system/directory_config.json");
                if !config_path.exists() {
                    info!("Creating OAuth client for Directory service...");
                    #[cfg(feature = "directory")]
                    match crate::core::package_manager::setup_directory().await {
                        Ok(_) => info!("OAuth client created successfully"),
                        Err(e) => warn!("Failed to create OAuth client: {}", e),
                    }
                    #[cfg(not(feature = "directory"))]
                    info!("Directory feature not enabled, skipping OAuth setup");
                } else {
                    info!("Directory config already exists, skipping OAuth setup");
                }
            } else {
                info!("Starting Zitadel/Directory service...");
                match pm.start("directory") {
                    Ok(_child) => {
                        info!("Directory service started, waiting for readiness...");
                        let mut zitadel_ready = false;
                        for i in 0..150 {
                            sleep(Duration::from_secs(2)).await;
                            if zitadel_health_check() {
                                info!("Zitadel/Directory service is responding after {}s", (i + 1) * 2);
                                zitadel_ready = true;
                                break;
                            }
                            if i % 15 == 14 {
                                info!("Zitadel health check: {}s elapsed, retrying...", (i + 1) * 2);
                            }
                        }
                        if !zitadel_ready {
                            warn!("Zitadel/Directory service did not respond after 300 seconds");
                        }

                        if zitadel_ready {
                            let config_path = self.stack_dir("conf/system/directory_config.json");
                            if !config_path.exists() {
                                info!("Creating OAuth client for Directory service...");
                                #[cfg(feature = "directory")]
                                match crate::core::package_manager::setup_directory().await {
                                    Ok(_) => info!("OAuth client created successfully"),
                                    Err(e) => warn!("Failed to create OAuth client: {}", e),
                                }
                                #[cfg(not(feature = "directory"))]
                                info!("Directory feature not enabled, skipping OAuth setup");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to start Directory service: {}", e);
                    }
                }
            }
        }

        if pm.is_installed("alm") {
            let alm_already_running = alm_health_check();
            if alm_already_running {
                info!("ALM (Forgejo) is already running");
            } else {
                info!("Starting ALM (Forgejo) service...");
                match pm.start("alm") {
                    Ok(_child) => {
                        info!("ALM service started");
                        tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
                        match crate::core::package_manager::setup_alm().await {
                            Ok(_) => info!("ALM setup and runner generation successful"),
                            Err(e) => warn!("ALM setup failed: {}", e),
                        }
                    }
                    Err(e) => {
                        warn!("Failed to start ALM service: {}", e);
                    }
                }
            }
        }

        if pm.is_installed("alm-ci") {
            let alm_ci_already_running = alm_ci_health_check();
            if alm_ci_already_running {
                info!("ALM CI (Forgejo Runner) is already running");
            } else {
                info!("Starting ALM CI (Forgejo Runner) service...");
                match pm.start("alm-ci") {
                    Ok(_child) => {
                        info!("ALM CI service started");
                    }
                    Err(e) => {
                        warn!("Failed to start ALM CI service: {}", e);
                    }
                }
            }
        }

        // Caddy is the web server
        let caddy_cmd = SafeCommand::new("caddy")
            .and_then(|c| c.arg("validate"))
            .and_then(|c| c.arg("--config"))
            .and_then(|c| c.arg("/etc/caddy/Caddyfile"));
        
        match caddy_cmd {
            Ok(cmd) => {
                match cmd.execute() {
                    Ok(_) => info!("Caddy configuration is valid"),
                    Err(e) => {
                        warn!("Caddy configuration error: {:?}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create caddy command: {:?}", e);
            }
        }

        info!("Bootstrap process completed!");
        Ok(())
    }

    /// Check system status
    pub fn system_status(&self) -> BootstrapProgress {
        BootstrapProgress::StartingComponent("System".to_string())
    }

    /// Run the bootstrap process
    pub async fn bootstrap(&mut self) -> anyhow::Result<()> {
        info!("Starting bootstrap process...");
        // Kill any existing processes
        self.kill_stack_processes().await?;

        // Install all required components
        self.install_all().await?;

        Ok(())
    }

    /// Install all required components
    pub async fn install_all(&mut self) -> anyhow::Result<()> {
        // If VAULT_ADDR is set and points to a remote server, skip local installation
        // All services are assumed to be running in separate containers
        let vault_addr = std::env::var("VAULT_ADDR").unwrap_or_default();
        let is_remote_vault = !vault_addr.is_empty() && !vault_addr.contains("localhost") && !vault_addr.contains("127.0.0.1");

        if is_remote_vault {
            info!("Remote Vault detected ({}), skipping local service installation", vault_addr);
            info!("All services are assumed to be running in separate containers");
            return Ok(());
        }

        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;

        // Install vault first (required for secrets management)
        if !pm.is_installed("vault") {
            info!("Installing Vault...");
            match pm.install("vault").await {
                Ok(Some(_)) => info!("Vault installed successfully"),
                Ok(None) => warn!("Vault installation returned no result"),
                Err(e) => warn!("Failed to install Vault: {}", e),
            }
        } else {
            info!("Vault already installed");
        }

        // Install other core components (names must match 3rdparty.toml)
		let core_components = ["tables", "cache", "drive", "directory", "llm", "vector_db", "alm", "alm-ci"];
        for component in core_components {
            if !pm.is_installed(component) {
                info!("Installing {}...", component);
                match pm.install(component).await {
                    Ok(Some(_)) => info!("{} installed successfully", component),
                    Ok(None) => warn!("{} installation returned no result", component),
                    Err(e) => warn!("Failed to install {}: {}", component, e),
                }
            }
        }

        Ok(())
    }

    /// Sync templates to database
    pub fn sync_templates_to_database(&self) -> anyhow::Result<()> {
        info!("Syncing templates to database...");
        // TODO: Implement actual template sync
        Ok(())
    }

    /// Upload templates to drive
    pub async fn upload_templates_to_drive(&self, _cfg: &AppConfig) -> anyhow::Result<()> {
        info!("Uploading templates to drive...");
        // TODO: Implement actual template upload
        Ok(())
    }
}

// Standalone functions for backward compatibility
pub use super::instance::{check_single_instance, release_instance_lock};
pub use super::vault::{has_installed_stack, reset_vault_only, get_db_password_from_vault};
