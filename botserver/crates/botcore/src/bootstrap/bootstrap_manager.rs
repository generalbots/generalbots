// Bootstrap manager implementation
use crate::bootstrap::bootstrap_types::{BootstrapManager, BootstrapProgress};
use crate::bootstrap::bootstrap_utils::{
    alm_health_check, cache_health_check, drive_health_check, safe_pkill,
    tables_health_check, vault_health_check, vector_db_health_check, zitadel_health_check,
};
use crate::package_manager::{InstallMode, PackageManager};
use anyhow::Context;
use diesel::RunQueryDsl;
use log::{info, warn};
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

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

        let processes = crate::bootstrap::bootstrap_utils::get_processes_to_kill();
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
            info!(
                "Remote Vault detected ({}), skipping local service startup",
                vault_addr
            );
            info!("All services are assumed to be running in separate containers");
            return Ok(());
        }

        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;

        info!("Starting bootstrap process (parallel services)...");

        // Phase 1: Start vault first (blocking, ensures credentials available for other services).
        // Vault init includes unseal + seeding credentials - must finish before services that
        // need those credentials (drive, cache) can start.
        if pm.is_installed("vault") {
            if vault_health_check() {
                info!("vault is already running");
            } else {
                info!("Starting vault...");
                let vault_pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;
                tokio::task::spawn_blocking(move || {
                    match vault_pm.start("vault") {
                        Ok(_) => info!("vault started and initialized"),
                        Err(e) => warn!("Failed to start vault: {}", e),
                    }
                })
                .await
                .ok();
            }
        }

        // Phase 2: Start all other services IN PARALLEL (vault credentials now available).
        // Start them all first (fast nohup launches), then wait for readiness concurrently.
        type ServiceConfig = (&'static str, fn() -> bool, u32, bool);
        let other_services: [ServiceConfig; 6] = [
            ("vector_db", vector_db_health_check, 45, true),
            ("tables", tables_health_check, 0, false),
            ("cache", cache_health_check, 30, true),
            ("drive", drive_health_check, 0, false),
            ("directory", zitadel_health_check, 60, true),
            ("alm", alm_health_check, 0, false),
        ];

        type WaitService = (&'static str, fn() -> bool, u32);
        let mut wait_services: Vec<WaitService> = Vec::new();

        for (name, check_fn, max_wait, need_wait) in &other_services {
            if !pm.is_installed(name) {
                continue;
            }
            let already_running = check_fn();
            if already_running {
                info!("{} is already running", name);
            } else {
                info!("Starting {}...", name);
                match pm.start(name) {
                    Ok(_) => {
                        info!("{} started", name);
                        if *need_wait {
                            wait_services.push((*name, *check_fn, *max_wait));
                        }
                    }
                    Err(e) => {
                        warn!("Failed to start {}: {}", name, e);
                    }
                }
            }
        }

        // Phase 2: Wait for all started services CONCURRENTLY using spawn_blocking
        // Total wait = max of all waits, not sum (since they run in parallel)
        if !wait_services.is_empty() {
            info!(
                "Waiting for {} services to become ready (parallel)...",
                wait_services.len()
            );
            let mut handles = Vec::new();
            for (name, check_fn, max_wait) in &wait_services {
                let name = *name;
                let check_fn = *check_fn;
                let max_wait = *max_wait;
                handles.push(tokio::task::spawn_blocking(move || {
                    let start = std::time::Instant::now();
                    for _ in 0..max_wait {
                        if check_fn() {
                            let elapsed = start.elapsed().as_secs();
                            info!("{} is responding after {}s", name, elapsed);
                            return true;
                        }
                        std::thread::sleep(Duration::from_secs(1));
                    }
                    warn!(
                        "{} did not respond after {} seconds, continuing anyway",
                        name, max_wait
                    );
                    false
                }));
            }
            for h in handles {
                h.await.ok();
            }
        }

        // Phase 3: Post-startup setup tasks (depends on services being ready)

        // Directory OAuth setup (needs Zitadel running)
        if pm.is_installed("directory") {
            let config_path = self.stack_dir("conf/system/directory_config.json");
            if !config_path.exists() {
                info!("Creating OAuth client for Directory service...");
                #[cfg(feature = "directory")]
                match crate::package_manager::setup_directory().await {
                    Ok(_) => info!("OAuth client created successfully"),
                    Err(e) => warn!("Failed to create OAuth client in Directory service (Zitadel): {}. Make sure Zitadel is running on the configured port.", e),
                }
                #[cfg(not(feature = "directory"))]
                info!("Directory feature not enabled, skipping OAuth setup");
            } else {
                info!("Directory config already exists, skipping OAuth setup");
            }
        }

        // ALM setup (needs Forgejo running)
        if pm.is_installed("alm") {
            let already_running = alm_health_check();
            if !already_running {
                info!("Waiting for ALM (Forgejo) to be ready...");
                let mut alm_ready = false;
                for _ in 0..30 {
                    sleep(Duration::from_secs(1)).await;
                    if alm_health_check() {
                        alm_ready = true;
                        break;
                    }
                }
                if alm_ready {
                    match crate::package_manager::setup_alm().await {
                        Ok(_) => info!("ALM setup and runner generation successful"),
                        Err(e) => warn!("ALM setup failed: {}", e),
                    }
                }
            } else {
                match crate::package_manager::setup_alm().await {
                    Ok(_) => info!("ALM setup and runner generation successful"),
                    Err(e) => warn!("ALM setup failed: {}", e),
                }
            }
        }

        // Caddy configuration validation
        let caddy_cmd = SafeCommand::new("caddy")
            .and_then(|c| c.arg("validate"))
            .and_then(|c| c.arg("--config"))
            .and_then(|c| c.arg("/etc/caddy/Caddyfile"));

        match caddy_cmd {
            Ok(cmd) => match cmd.execute() {
                Ok(_) => info!("Caddy configuration is valid"),
                Err(e) => {
                    info!("Caddy configuration not validated: {:?}", e);
                }
            },
            Err(e) => {
                info!("Caddy command unavailable: {:?}", e);
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
                Ok(None) => info!("Vault installation returned no result"),
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
                    Ok(None) => info!("{} installation returned no result", component),
                    Err(e) => warn!("Failed to install {}: {}", component, e),
                }
            }
        }

        Ok(())
    }

    /// Find the directory holding template bots: the cloned templates repo
    /// (`work/templates`) takes precedence, falling back to the bundled
    /// `bottemplates/bots` directory shipped with the workspace.
    fn templates_source_dir(&self) -> PathBuf {
        let work = PathBuf::from("work/templates");
        if work.is_dir() {
            work
        } else {
            PathBuf::from("bottemplates/bots")
        }
    }

    /// Sync the on-disk template bots into the `app_templates` table so the
    /// templates API lists real, persisted entries. Each `*.gbai` directory
    /// becomes one row; existing names are refreshed (version/description),
    /// never duplicated.
    pub fn sync_templates_to_database(&self) -> anyhow::Result<()> {
        info!("Syncing templates to database...");
        let source = self.templates_source_dir();
        if !source.is_dir() {
            warn!("No templates directory found at {}; skipping sync", source.display());
            return Ok(());
        }

        let mut conn = crate::shared::utils::establish_pg_connection()?;
        let branch_id = crate::shared::utils::current_org_id();

        let mut synced = 0usize;
        let entries = std::fs::read_dir(&source)
            .with_context(|| format!("Failed to read templates dir {}", source.display()))?;
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Skipping unreadable template entry: {e}");
                    continue;
                }
            };
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = match entry.file_name().into_string() {
                Ok(name) => name,
                Err(_) => continue,
            };
            let Some(bot_name) = dir_name.strip_suffix(".gbai") else {
                continue;
            };
            if bot_name.is_empty() {
                continue;
            }

            // Idempotent by (name, branch): refresh an existing row, insert a
            // new one otherwise — never duplicate templates across restarts.
            let updated = diesel::sql_query(
                "UPDATE app_templates SET version = '1.0' \
                 WHERE name = $1 AND branch_id = $2",
            )
            .bind::<diesel::sql_types::Text, _>(bot_name.to_string())
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .execute(&mut conn)
            .with_context(|| format!("Failed to update template {bot_name}"))?;
            if updated == 0 {
                let id = Uuid::new_v4();
                diesel::sql_query(
                    "INSERT INTO app_templates (id, name, description, kind, version, author, branch_id) \
                     VALUES ($1, $2, '', 'bot', '1.0', '', $3)",
                )
                .bind::<diesel::sql_types::Uuid, _>(id)
                .bind::<diesel::sql_types::Text, _>(bot_name.to_string())
                .bind::<diesel::sql_types::Uuid, _>(branch_id)
                .execute(&mut conn)
                .with_context(|| format!("Failed to insert template {bot_name}"))?;
            }
            synced += 1;
        }
        info!("Synced {synced} templates to database");
        Ok(())
    }

    /// Copy template bots into the org work path (`{work}/{org}.gborg/`)
    /// so the drive monitor discovers and loads them into MinIO. Existing
    /// copies are replaced recursively; the whole pass is best-effort and
    /// never fails startup when a template is malformed.
    pub async fn upload_templates_to_drive(&self, _cfg: &AppConfig) -> anyhow::Result<()> {
        info!("Uploading templates to drive...");
        let source = self.templates_source_dir();
        if !source.is_dir() {
            warn!("No templates directory found at {}; skipping upload", source.display());
            return Ok(());
        }

        let org_id = crate::shared::utils::current_org_id();
        let org_work = crate::shared::utils::get_org_work_path(org_id);
        let target_root = PathBuf::from(org_work);
        if !target_root.exists() {
            std::fs::create_dir_all(&target_root)
                .with_context(|| format!("Failed to create org work dir {}", target_root.display()))?;
        }

        let mut uploaded = 0usize;
        let entries = std::fs::read_dir(&source)
            .with_context(|| format!("Failed to read templates dir {}", source.display()))?;
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Skipping unreadable template entry: {e}");
                    continue;
                }
            };
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = match entry.file_name().into_string() {
                Ok(name) => name,
                Err(_) => continue,
            };
            let target = target_root.join(&dir_name);
            if let Err(e) = copy_dir_recursive(&path, &target) {
                warn!("Failed to copy template {dir_name}: {e}");
                continue;
            }
            uploaded += 1;
        }
        info!("Uploaded {uploaded} templates to drive work path");
        Ok(())
    }
}

/// Recursively copy a directory tree, replacing the destination. Returns an
/// error with context when any file cannot be copied, leaving already-copied
/// files in place (the caller treats each template independently).
fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create {}", dst.display()))?;
    for entry in std::fs::read_dir(src)
        .with_context(|| format!("Failed to read {}", src.display()))?
    {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", src.display()))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).with_context(|| {
                format!("Failed to copy {} to {}", from.display(), to.display())
            })?;
        }
    }
    Ok(())
}

// Standalone functions for backward compatibility
pub use super::instance::{check_single_instance, release_instance_lock};
pub use super::vault::{has_installed_stack, reset_vault_only, get_db_password_from_vault};

use crate::config::AppConfig;
use crate::shared::utils::get_stack_path;
use botlib::security::command_guard::SafeCommand;
use std::path::PathBuf;
