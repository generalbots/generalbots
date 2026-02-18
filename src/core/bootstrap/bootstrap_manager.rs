// Bootstrap manager implementation
use crate::core::bootstrap::bootstrap_types::{BootstrapManager, BootstrapProgress};
use crate::core::bootstrap::bootstrap_utils::{cache_health_check, safe_pkill, vault_health_check, vector_db_health_check};
use crate::core::config::AppConfig;
use crate::core::package_manager::{InstallMode, PackageManager};
use log::{info, warn};
use std::path::PathBuf;
use std::process::Command;
use tokio::time::{sleep, Duration};

impl BootstrapManager {
    pub fn new(mode: InstallMode, tenant: Option<String>) -> Self {
        let stack_path = std::env::var("BOTSERVER_STACK_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./botserver-stack"));

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
            .unwrap_or("./botserver-stack/bin/vault/vault")
            .to_string()
    }

    pub async fn kill_stack_processes(&self) -> anyhow::Result<()> {
        info!("Killing any existing stack processes...");

        let processes = crate::core::bootstrap::bootstrap_utils::get_processes_to_kill();
        for (name, args) in processes {
            // safe_pkill expects &[&str] for pattern, so convert the name
            safe_pkill(&[name], &args);
        }

        // Give processes time to terminate
        sleep(Duration::from_millis(500)).await;

        info!("Stack processes terminated");
        Ok(())
    }

    pub async fn start_all(&mut self) -> anyhow::Result<()> {
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
                        // Wait for vector_db to be ready
                        for i in 0..15 {
                            sleep(Duration::from_secs(1)).await;
                            if vector_db_health_check() {
                                info!("Vector database (Qdrant) is responding");
                                break;
                            }
                            if i == 14 {
                                warn!("Vector database did not respond after 15 seconds");
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

        if pm.is_installed("cache") {
            let cache_already_running = cache_health_check();
            if cache_already_running {
                info!("Valkey cache is already running");
            } else {
                info!("Starting Valkey cache...");
                match pm.start("cache") {
                    Ok(_child) => {
                        info!("Valkey cache process started, waiting for readiness...");
                        // Wait for cache to be ready
                        for i in 0..12 {
                            sleep(Duration::from_secs(1)).await;
                            if cache_health_check() {
                                info!("Valkey cache is responding");
                                break;
                            }
                            if i == 11 {
                                warn!("Valkey cache did not respond after 12 seconds");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to start Valkey cache: {}", e);
                    }
                }
            }
        }

        if pm.is_installed("minio") {
            info!("Starting MinIO...");
            match pm.start("minio") {
                Ok(_child) => {
                    info!("MinIO started");
                }
                Err(e) => {
                    warn!("Failed to start MinIO: {}", e);
                }
            }
        }

        // Caddy is the web server
        match Command::new("caddy")
            .arg("validate")
            .arg("--config")
            .arg("/etc/caddy/Caddyfile")
            .output()
        {
            Ok(_) => info!("Caddy configuration is valid"),
            Err(e) => {
                warn!("Caddy configuration error: {:?}", e);
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
		let core_components = ["tables", "cache", "drive", "llm"];
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
