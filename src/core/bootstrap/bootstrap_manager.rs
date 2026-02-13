// Bootstrap manager implementation
use crate::core::bootstrap::bootstrap_types::{BootstrapManager, BootstrapProgress};
use crate::core::bootstrap::bootstrap_utils::{safe_pkill, vault_health_check};
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
            info!("Starting Vector database...");
            match pm.start("vector_db") {
                Ok(_child) => {
                    info!("Vector database started");
                }
                Err(e) => {
                    warn!("Failed to start Vector database: {}", e);
                }
            }
        }

        if pm.is_installed("postgres") {
            info!("Starting PostgreSQL...");
            match pm.start("postgres") {
                Ok(_child) => {
                    info!("PostgreSQL started");
                }
                Err(e) => {
                    warn!("Failed to start PostgreSQL: {}", e);
                }
            }
        }

        if pm.is_installed("redis") {
            info!("Starting Redis...");
            match pm.start("redis") {
                Ok(_child) => {
                    info!("Redis started");
                }
                Err(e) => {
                    warn!("Failed to start Redis: {}", e);
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
