use crate::config::AppConfig;
use crate::package_manager::setup::{DirectorySetup, EmailSetup};
use crate::package_manager::{InstallMode, PackageManager};
use crate::shared::utils::{establish_pg_connection, init_secrets_manager};
use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use diesel::{Connection, RunQueryDsl};
use log::{debug, error, info, warn};
use rand::distr::Alphanumeric;
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, Issuer, KeyPair,
};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
#[derive(Debug)]
pub struct ComponentInfo {
    pub name: &'static str,
}
#[derive(Debug)]
pub struct BootstrapManager {
    pub install_mode: InstallMode,
    pub tenant: Option<String>,
}
impl BootstrapManager {
    pub async fn new(mode: InstallMode, tenant: Option<String>) -> Self {
        Self {
            install_mode: mode,
            tenant,
        }
    }

    /// Kill all processes running from the botserver-stack directory
    /// This ensures a clean startup when bootstrapping fresh
    pub fn kill_stack_processes() {
        info!("Killing any existing stack processes...");

        // Kill processes by pattern matching on botserver-stack path
        let patterns = vec![
            "botserver-stack/bin/vault",
            "botserver-stack/bin/tables",
            "botserver-stack/bin/drive",
            "botserver-stack/bin/cache",
            "botserver-stack/bin/directory",
            "botserver-stack/bin/llm",
            "botserver-stack/bin/email",
            "botserver-stack/bin/proxy",
            "botserver-stack/bin/dns",
            "botserver-stack/bin/meeting",
            "botserver-stack/bin/vector_db",
        ];

        for pattern in patterns {
            let _ = Command::new("pkill").args(["-9", "-f", pattern]).output();
        }

        // Also kill by specific process names (use -f for pattern match, not -x for exact)
        let process_names = vec![
            "vault server",
            "postgres",
            "minio",
            "redis-server",
            "zitadel",
            "llama-server",
            "stalwart",
            "caddy",
            "coredns",
            "livekit",
            "qdrant",
        ];

        for name in process_names {
            let _ = Command::new("pkill").args(["-9", "-f", name]).output();
        }

        // Kill processes by port - this catches any process using our ports
        // even if started from a different path
        let ports = vec![
            8200, // Vault
            5432, // PostgreSQL
            9000, // MinIO
            6379, // Redis
            8300, // Zitadel / Main API
            8081, // LLM server
            8082, // Embedding server
            25,   // Email SMTP
            443,  // HTTPS proxy
            53,   // DNS
        ];

        for port in ports {
            // Use fuser to kill processes on specific ports
            let _ = Command::new("fuser")
                .args(["-k", "-9", &format!("{}/tcp", port)])
                .output();
        }

        // Give processes time to die
        std::thread::sleep(std::time::Duration::from_millis(1000));
        info!("Stack processes terminated");
    }

    /// Check if another botserver process is already running on this stack
    pub fn check_single_instance() -> Result<bool> {
        let lock_file = PathBuf::from("./botserver-stack/.lock");
        if lock_file.exists() {
            // Check if the PID in the lock file is still running
            if let Ok(pid_str) = fs::read_to_string(&lock_file) {
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    let check = Command::new("kill").args(["-0", &pid.to_string()]).output();
                    if let Ok(output) = check {
                        if output.status.success() {
                            warn!("Another botserver process (PID {}) is already running on this stack", pid);
                            return Ok(false);
                        }
                    }
                }
            }
        }
        // Write our PID to the lock file
        let pid = std::process::id();
        if let Some(parent) = lock_file.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&lock_file, pid.to_string()).ok();
        Ok(true)
    }

    /// Release the instance lock on shutdown
    pub fn release_instance_lock() {
        let lock_file = PathBuf::from("./botserver-stack/.lock");
        if lock_file.exists() {
            fs::remove_file(&lock_file).ok();
        }
    }

    /// Check if botserver-stack has installed components (indicating a working installation)
    /// This is used to prevent accidental re-initialization of existing installations
    fn has_installed_stack() -> bool {
        let stack_dir = PathBuf::from("./botserver-stack");
        if !stack_dir.exists() {
            return false;
        }
        
        // Check for key indicators of an installed stack
        let indicators = vec![
            "./botserver-stack/bin/vault/vault",
            "./botserver-stack/data/vault",
            "./botserver-stack/conf/vault/config.hcl",
        ];
        
        indicators.iter().any(|path| PathBuf::from(path).exists())
    }

    /// Reset only Vault credentials (when re-initialization is needed)
    /// CRITICAL: This should NEVER be called if botserver-stack exists with installed components!
    /// NEVER deletes user data in botserver-stack
    fn reset_vault_only() -> Result<()> {
        // SAFETY CHECK: NEVER reset if stack is installed
        if Self::has_installed_stack() {
            error!("REFUSING to reset Vault credentials - botserver-stack is installed!");
            error!("If you need to re-initialize, manually delete botserver-stack directory first");
            return Err(anyhow::anyhow!(
                "Cannot reset Vault - existing installation detected. Manual intervention required."
            ));
        }

        let vault_init = PathBuf::from("./botserver-stack/conf/vault/init.json");
        let env_file = PathBuf::from("./.env");

        // Only remove vault init.json and .env - NEVER touch data/
        if vault_init.exists() {
            info!("Removing vault init.json for re-initialization...");
            fs::remove_file(&vault_init)?;
        }

        if env_file.exists() {
            info!("Removing .env file for re-initialization...");
            fs::remove_file(&env_file)?;
        }

        Ok(())
    }
    pub async fn start_all(&mut self) -> Result<()> {
        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;

        // VAULT MUST START FIRST - all other services depend on it for secrets
        if pm.is_installed("vault") {
            // Check if Vault is already running before trying to start
            let vault_already_running = Command::new("sh")
                .arg("-c")
                .arg("curl -f -s http://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200 >/dev/null 2>&1")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if vault_already_running {
                info!("Vault is already running");
            } else {
                info!("Starting Vault secrets service...");
                match pm.start("vault") {
                    Ok(_child) => {
                        info!("Vault process started, waiting for initialization...");
                    }
                    Err(e) => {
                        warn!("Vault might already be running: {}", e);
                    }
                }

                // Wait for Vault to be ready (up to 10 seconds)
                for i in 0..10 {
                    let vault_ready = Command::new("sh")
                        .arg("-c")
                        .arg("curl -f -s http://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200 >/dev/null 2>&1")
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);

                    if vault_ready {
                        info!("Vault is responding");
                        break;
                    }
                    if i < 9 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }

            // Try to unseal Vault - if this fails, we need to handle carefully
            if let Err(e) = self.ensure_vault_unsealed().await {
                warn!("Vault unseal failed: {}", e);

                // CRITICAL: If stack is installed, NEVER try to re-initialize
                // Just try restarting Vault a few more times
                if Self::has_installed_stack() {
                    error!("Vault failed to unseal but stack is installed - NOT re-initializing");
                    error!("Try manually restarting Vault or check ./botserver-stack/logs/vault/vault.log");
                    
                    // Kill only Vault process and try to restart
                    let _ = Command::new("pkill")
                        .args(["-9", "-f", "botserver-stack/bin/vault"])
                        .output();
                    
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    
                    // Try to restart Vault
                    if let Err(e) = pm.start("vault") {
                        warn!("Failed to restart Vault: {}", e);
                    }
                    
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    
                    // Final attempt to unseal
                    if let Err(e) = self.ensure_vault_unsealed().await {
                        return Err(anyhow::anyhow!(
                            "Vault failed to start/unseal after restart: {}. Manual intervention required.", e
                        ));
                    }
                } else {
                    // No installed stack, safe to re-initialize
                    warn!("No installed stack detected - proceeding with re-initialization");

                    // Kill only Vault process, reset only Vault credentials
                    let _ = Command::new("pkill")
                        .args(["-9", "-f", "botserver-stack/bin/vault"])
                        .output();

                    if let Err(e) = Self::reset_vault_only() {
                        error!("Failed to reset Vault: {}", e);
                        return Err(e);
                    }

                    // Run bootstrap to re-initialize Vault
                    self.bootstrap().await?;

                    // After bootstrap, services are already running
                    info!("Vault re-initialization complete");
                    return Ok(());
                }
            }

            // Initialize SecretsManager so other code can use Vault
            info!("Initializing SecretsManager...");
            match init_secrets_manager().await {
                Ok(_) => info!("SecretsManager initialized successfully"),
                Err(e) => {
                    error!("Failed to initialize SecretsManager: {}", e);
                    return Err(anyhow::anyhow!(
                        "SecretsManager initialization failed: {}",
                        e
                    ));
                }
            }
        }

        // Start tables (PostgreSQL) - needed for database operations
        if pm.is_installed("tables") {
            info!("Starting PostgreSQL database...");
            match pm.start("tables") {
                Ok(_child) => {
                    // Give PostgreSQL time to initialize
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    info!("PostgreSQL started");
                }
                Err(e) => {
                    warn!("PostgreSQL might already be running: {}", e);
                }
            }
        }

        // Start other components (order matters less for these)
        let other_components = vec![
            ComponentInfo { name: "cache" },
            ComponentInfo { name: "drive" },
            ComponentInfo { name: "llm" },
            ComponentInfo { name: "email" },
            ComponentInfo { name: "proxy" },
            ComponentInfo { name: "directory" },
            ComponentInfo { name: "alm" },
            ComponentInfo { name: "alm_ci" },
            ComponentInfo { name: "dns" },
            ComponentInfo { name: "meeting" },
            ComponentInfo {
                name: "remote_terminal",
            },
            ComponentInfo { name: "vector_db" },
            ComponentInfo { name: "host" },
        ];

        for component in other_components {
            if pm.is_installed(component.name) {
                match pm.start(component.name) {
                    Ok(_child) => {
                        info!("Started component: {}", component.name);
                    }
                    Err(e) => {
                        debug!(
                            "Component {} might already be running: {}",
                            component.name,
                            e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn generate_secure_password(&self, length: usize) -> String {
        let mut rng = rand::rng();
        let base: String = (0..length.saturating_sub(4))
            .map(|_| {
                let byte = rand::Rng::sample(&mut rng, Alphanumeric);
                char::from(byte)
            })
            .collect();
        // Add required symbols/complexity for Zitadel password policy
        // Use ! instead of @ to avoid breaking database connection strings
        format!("{}!1Aa", base)
    }

    /// Ensure critical services are running - Vault MUST be first
    /// Order: vault -> tables -> drive
    /// If fresh_start is true, kills existing processes first
    pub async fn ensure_services_running(&mut self) -> Result<()> {
        info!("Ensuring critical services are running...");

        let installer = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;

        // Check if we need to bootstrap first
        let vault_installed = installer.is_installed("vault");
        let vault_initialized = PathBuf::from("./botserver-stack/conf/vault/init.json").exists();

        if !vault_installed || !vault_initialized {
            info!("Stack not fully bootstrapped, running bootstrap first...");
            // Kill any leftover processes
            Self::kill_stack_processes();

            // Run bootstrap - this will start all services
            self.bootstrap().await?;

            // After bootstrap, services are already running, just ensure Vault is unsealed and env vars set
            info!("Bootstrap complete, verifying Vault is ready...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            if let Err(e) = self.ensure_vault_unsealed().await {
                warn!("Failed to unseal Vault after bootstrap: {}", e);
            }

            // Services were started by bootstrap, no need to restart them
            return Ok(());
        }

        // If we get here, bootstrap was already done previously - just start services
        // VAULT MUST BE FIRST - it provides all secrets
        if installer.is_installed("vault") {
            // Check if Vault is already running
            let vault_running = Command::new("sh")
                .arg("-c")
                .arg("curl -f -s http://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200 >/dev/null 2>&1")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if !vault_running {
                info!("Starting Vault secrets service...");
                match installer.start("vault") {
                    Ok(_child) => {
                        info!("Vault started successfully");
                        // Give Vault time to initialize
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                    Err(e) => {
                        warn!("Vault might already be running or failed to start: {}", e);
                    }
                }
            } else {
                info!("Vault is already running");
            }

            // Always try to unseal Vault (it may have restarted)
            // If unseal fails, try to restart Vault process only - NEVER delete other services
            if let Err(e) = self.ensure_vault_unsealed().await {
                warn!("Vault unseal failed: {} - attempting Vault restart only", e);

                // Kill ONLY Vault process - preserve all other services
                let _ = Command::new("pkill")
                    .args(["-9", "-f", "botserver-stack/bin/vault"])
                    .output();

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                // Try to restart Vault without full bootstrap
                let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;
                if let Err(e) = pm.start("vault") {
                    warn!("Failed to restart Vault: {}", e);
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                // Try unseal again
                if let Err(e) = self.ensure_vault_unsealed().await {
                    warn!("Vault still not responding after restart: {}", e);

                    // CRITICAL: If stack is installed, NEVER try to re-initialize
                    // This protects existing installations from being destroyed
                    if Self::has_installed_stack() {
                        error!("CRITICAL: Vault failed but botserver-stack is installed!");
                        error!("REFUSING to delete init.json or .env - this would destroy your installation");
                        error!("Please check ./botserver-stack/logs/vault/vault.log for errors");
                        error!("You may need to manually restart Vault or check its configuration");
                        return Err(anyhow::anyhow!(
                            "Vault failed to start. Manual intervention required. Check logs at ./botserver-stack/logs/vault/vault.log"
                        ));
                    }

                    // Only reset if NO installed stack (fresh/broken install)
                    warn!("No installed stack detected - attempting Vault re-initialization");
                    if let Err(reset_err) = Self::reset_vault_only() {
                        error!("Failed to reset Vault: {}", reset_err);
                        return Err(reset_err);
                    }

                    // Install/configure ONLY Vault - NOT full bootstrap
                    info!("Re-initializing Vault only (preserving other services)...");
                    if let Err(e) = pm.install("vault").await {
                        return Err(anyhow::anyhow!("Failed to re-initialize Vault: {}", e));
                    }

                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                    if let Err(e) = self.ensure_vault_unsealed().await {
                        return Err(anyhow::anyhow!(
                            "Failed to configure Vault after re-initialization: {}",
                            e
                        ));
                    }
                }

                info!("Vault recovery complete");
            }

            // Initialize SecretsManager so other code can use Vault
            info!("Initializing SecretsManager...");
            match init_secrets_manager().await {
                Ok(_) => info!("SecretsManager initialized successfully"),
                Err(e) => {
                    error!("Failed to initialize SecretsManager: {}", e);
                    return Err(anyhow::anyhow!(
                        "SecretsManager initialization failed: {}",
                        e
                    ));
                }
            }
        } else {
            // Vault not installed - cannot proceed, need to run bootstrap
            warn!("Vault (secrets) component not installed - run bootstrap first");
            return Err(anyhow::anyhow!(
                "Vault not installed. Run bootstrap command first."
            ));
        }

        // Check and start PostgreSQL (after Vault is running)
        if installer.is_installed("tables") {
            info!("Starting PostgreSQL database service...");
            match installer.start("tables") {
                Ok(_child) => {
                    info!("PostgreSQL started successfully");
                    // Give PostgreSQL time to initialize
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }
                Err(e) => {
                    warn!(
                        "PostgreSQL might already be running or failed to start: {}",
                        e
                    );
                }
            }
        } else {
            warn!("PostgreSQL (tables) component not installed");
        }

        // Check and start MinIO
        if installer.is_installed("drive") {
            info!("Starting MinIO drive service...");
            match installer.start("drive") {
                Ok(_child) => {
                    info!("MinIO started successfully");
                    // Give MinIO time to initialize
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
                Err(e) => {
                    warn!("MinIO might already be running or failed to start: {}", e);
                }
            }
        } else {
            warn!("MinIO (drive) component not installed");
        }

        Ok(())
    }

    /// Ensure Vault is unsealed (required after restart)
    /// Returns Ok(()) if Vault is ready, Err if it needs re-initialization
    async fn ensure_vault_unsealed(&self) -> Result<()> {
        let vault_init_path = PathBuf::from("./botserver-stack/conf/vault/init.json");
        let vault_addr = "http://localhost:8200";

        if !vault_init_path.exists() {
            return Err(anyhow::anyhow!(
                "Vault init.json not found - needs re-initialization"
            ));
        }

        // Read unseal key from init.json
        let init_json = fs::read_to_string(&vault_init_path)?;
        let init_data: serde_json::Value = serde_json::from_str(&init_json)?;

        let unseal_key = init_data["unseal_keys_b64"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let root_token = init_data["root_token"].as_str().unwrap_or("").to_string();

        if unseal_key.is_empty() || root_token.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid Vault init.json - needs re-initialization"
            ));
        }

        // First check if Vault is initialized (not just running)
        let status_output = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "VAULT_ADDR={} ./botserver-stack/bin/vault/vault status -format=json 2>/dev/null",
                vault_addr
            ))
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()?;

        let status_str = String::from_utf8_lossy(&status_output.stdout);

        // Parse status - handle both success and error cases
        if let Ok(status) = serde_json::from_str::<serde_json::Value>(&status_str) {
            let initialized = status["initialized"].as_bool().unwrap_or(false);
            let sealed = status["sealed"].as_bool().unwrap_or(true);

            if !initialized {
                // Vault is running but not initialized - this means data was deleted
                // We need to re-run bootstrap
                warn!("Vault is running but not initialized - data may have been deleted");
                return Err(anyhow::anyhow!(
                    "Vault not initialized - needs re-bootstrap"
                ));
            }

            if sealed {
                info!("Unsealing Vault...");
                let unseal_output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!(
                        "VAULT_ADDR={} ./botserver-stack/bin/vault/vault operator unseal {} >/dev/null 2>&1",
                        vault_addr, unseal_key
                    ))
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .output()?;

                if !unseal_output.status.success() {
                    let stderr = String::from_utf8_lossy(&unseal_output.stderr);
                    warn!("Vault unseal may have failed: {}", stderr);
                }

                // Verify unseal succeeded
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let verify_output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!(
                        "VAULT_ADDR={} ./botserver-stack/bin/vault/vault status -format=json 2>/dev/null",
                        vault_addr
                    ))
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .output()?;

                let verify_str = String::from_utf8_lossy(&verify_output.stdout);
                if let Ok(verify_status) = serde_json::from_str::<serde_json::Value>(&verify_str) {
                    if verify_status["sealed"].as_bool().unwrap_or(true) {
                        return Err(anyhow::anyhow!(
                            "Failed to unseal Vault - may need re-initialization"
                        ));
                    }
                }
                info!("Vault unsealed successfully");
            }
        } else {
            // Could not parse status - Vault might not be responding properly
            warn!("Could not get Vault status: {}", status_str);
            return Err(anyhow::anyhow!("Vault not responding properly"));
        }

        // Set environment variables for other components
        std::env::set_var("VAULT_ADDR", vault_addr);
        std::env::set_var("VAULT_TOKEN", &root_token);
        std::env::set_var("VAULT_SKIP_VERIFY", "true");

        // Also set mTLS cert paths
        std::env::set_var(
            "VAULT_CACERT",
            "./botserver-stack/conf/system/certificates/ca/ca.crt",
        );
        std::env::set_var(
            "VAULT_CLIENT_CERT",
            "./botserver-stack/conf/system/certificates/botserver/client.crt",
        );
        std::env::set_var(
            "VAULT_CLIENT_KEY",
            "./botserver-stack/conf/system/certificates/botserver/client.key",
        );

        info!("Vault environment configured");
        Ok(())
    }

    pub async fn bootstrap(&mut self) -> Result<()> {
        info!("=== BOOTSTRAP STARTING ===");

        // Kill any existing stack processes first - critical for dev machines
        // where old processes may be running from a deleted/recreated stack
        info!("Cleaning up any existing stack processes...");
        Self::kill_stack_processes();

        // Generate certificates first (including for Vault)
        info!("Generating TLS certificates...");
        if let Err(e) = self.generate_certificates().await {
            error!("Failed to generate certificates: {}", e);
        }

        // Create Vault configuration with mTLS
        info!("Creating Vault configuration...");
        if let Err(e) = self.create_vault_config().await {
            error!("Failed to create Vault config: {}", e);
        }

        // Generate secure passwords for all services - these are ONLY used during bootstrap
        // and immediately stored in Vault. NO LEGACY ENV VARS.
        let db_password = self.generate_secure_password(24);
        let drive_accesskey = self.generate_secure_password(20);
        let drive_secret = self.generate_secure_password(40);
        let cache_password = self.generate_secure_password(24);

        // Configuration is stored in Vault, not .env files
        info!("Configuring services through Vault...");

        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone()).unwrap();

        // Vault MUST be installed first - it stores all secrets
        // Order: vault -> tables -> directory -> drive -> cache -> llm
        let required_components = vec![
            "vault",     // Secrets management - MUST BE FIRST
            "tables",    // Database - required by Directory
            "directory", // Identity service - manages users
            "drive",     // S3 storage - credentials in Vault
            "cache",     // Redis cache
            "llm",       // LLM service
        ];

        // Special check: Vault needs setup even if binary exists but not initialized
        let vault_needs_setup = !PathBuf::from("./botserver-stack/conf/vault/init.json").exists();

        for component in required_components {
            // For vault, also check if it needs initialization
            let is_installed = pm.is_installed(component);
            let needs_install = if component == "vault" {
                !is_installed || vault_needs_setup
            } else {
                !is_installed
            };

            info!(
                "Component {}: installed={}, needs_install={}, vault_needs_setup={}",
                component, is_installed, needs_install, vault_needs_setup
            );

            if needs_install {
                info!("Installing/configuring component: {}", component);
                // Quick check if component might be running - don't hang on this
                let bin_path = pm.base_path.join("bin").join(component);
                let binary_name = pm
                    .components
                    .get(component)
                    .and_then(|cfg| cfg.binary_name.clone())
                    .unwrap_or_else(|| component.to_string());

                // Only terminate for services that are known to conflict
                // Use simple, fast commands with timeout
                if component == "vault" || component == "tables" || component == "directory" {
                    let _ = Command::new("sh")
                        .arg("-c")
                        .arg(format!(
                            "pkill -9 -f '{}/{}' 2>/dev/null; true",
                            bin_path.display(),
                            binary_name
                        ))
                        .status();
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }

                info!("Installing component: {}", component);
                let install_result = pm.install(component).await;
                if let Err(e) = install_result {
                    error!("Failed to install component {}: {}", component, e);
                    if component == "vault" {
                        return Err(anyhow::anyhow!("Failed to install Vault: {}", e));
                    }
                }
                info!("Component {} installed successfully", component);

                // After tables is installed, START PostgreSQL and create Zitadel config files before installing directory
                if component == "tables" {
                    info!("Starting PostgreSQL database...");
                    match pm.start("tables") {
                        Ok(_) => {
                            info!("PostgreSQL started successfully");
                            // Give PostgreSQL time to initialize
                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        }
                        Err(e) => {
                            warn!("Failed to start PostgreSQL: {}", e);
                        }
                    }

                    // Run migrations using direct connection (Vault not set up yet)
                    info!("Running database migrations...");
                    let database_url =
                        format!("postgres://gbuser:{}@localhost:5432/botserver", db_password);
                    match diesel::PgConnection::establish(&database_url) {
                        Ok(mut conn) => {
                            if let Err(e) = self.apply_migrations(&mut conn) {
                                error!("Failed to apply migrations: {}", e);
                            } else {
                                info!("Database migrations applied");
                            }
                        }
                        Err(e) => {
                            error!("Failed to connect to database for migrations: {}", e);
                        }
                    }

                    info!("Creating Directory configuration files...");
                    if let Err(e) = self.configure_services_in_directory(&db_password).await {
                        error!("Failed to create Directory config files: {}", e);
                    }
                }

                // Directory configuration - setup happens after install starts Zitadel
                if component == "directory" {
                    info!("Waiting for Directory to be ready...");
                    if let Err(e) = self.setup_directory().await {
                        // Don't fail completely - Zitadel may still be usable with first instance setup
                        warn!("Directory additional setup had issues: {}", e);
                    }
                }

                // After Vault is installed, START the server then initialize it
                if component == "vault" {
                    info!("Setting up Vault secrets service...");

                    // Verify vault binary exists and is executable
                    let vault_bin = PathBuf::from("./botserver-stack/bin/vault/vault");
                    if !vault_bin.exists() {
                        error!("Vault binary not found at {:?}", vault_bin);
                        return Err(anyhow::anyhow!("Vault binary not found after installation"));
                    }
                    info!("Vault binary verified at {:?}", vault_bin);

                    // Ensure logs directory exists
                    let vault_log_path = PathBuf::from("./botserver-stack/logs/vault/vault.log");
                    if let Some(parent) = vault_log_path.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            error!("Failed to create vault logs directory: {}", e);
                        }
                    }

                    // Ensure data directory exists
                    let vault_data_path = PathBuf::from("./botserver-stack/data/vault");
                    if let Err(e) = fs::create_dir_all(&vault_data_path) {
                        error!("Failed to create vault data directory: {}", e);
                    }

                    info!("Starting Vault server...");

                    // Try starting vault directly first
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg("cd ./botserver-stack/bin/vault && nohup ./vault server -config=../../conf/vault/config.hcl > ../../logs/vault/vault.log 2>&1 &")
                        .status();
                    std::thread::sleep(std::time::Duration::from_secs(2));

                    // Check if it's running now
                    let check = std::process::Command::new("pgrep")
                        .args(["-f", "vault server"])
                        .output();
                    if let Ok(output) = &check {
                        let pids = String::from_utf8_lossy(&output.stdout);
                        if !pids.trim().is_empty() {
                            info!("Vault server started");
                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        } else {
                            debug!("Direct start failed, trying pm.start...");
                            match pm.start("vault") {
                                Ok(_) => {
                                    info!("Vault server started");
                                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                }
                                Err(e) => {
                                    error!("Failed to start Vault server: {}", e);
                                    return Err(anyhow::anyhow!(
                                        "Failed to start Vault server: {}",
                                        e
                                    ));
                                }
                            }
                        }
                    }

                    // Verify vault is running
                    let final_check = std::process::Command::new("pgrep")
                        .args(["-f", "vault server"])
                        .output();
                    if let Ok(output) = final_check {
                        let pids = String::from_utf8_lossy(&output.stdout);
                        if pids.trim().is_empty() {
                            error!("Vault is not running after all start attempts");
                            return Err(anyhow::anyhow!("Failed to start Vault server"));
                        }
                    }

                    info!("Initializing Vault with secrets...");
                    if let Err(e) = self
                        .setup_vault(
                            &db_password,
                            &drive_accesskey,
                            &drive_secret,
                            &cache_password,
                        )
                        .await
                    {
                        error!("Failed to setup Vault: {}", e);
                        // Check vault.log for more details
                        if vault_log_path.exists() {
                            if let Ok(log_content) = fs::read_to_string(&vault_log_path) {
                                let last_lines: Vec<&str> =
                                    log_content.lines().rev().take(20).collect();
                                error!("Vault log (last 20 lines):");
                                for line in last_lines.iter().rev() {
                                    error!("  {}", line);
                                }
                            }
                        }
                        // Vault is critical - fail the bootstrap
                        return Err(anyhow::anyhow!("Vault setup failed: {}. Check ./botserver-stack/logs/vault/vault.log for details.", e));
                    }

                    // Initialize the global SecretsManager so other components can use Vault
                    info!("Initializing SecretsManager...");
                    debug!(
                        "VAULT_ADDR={:?}, VAULT_TOKEN set={}",
                        std::env::var("VAULT_ADDR").ok(),
                        std::env::var("VAULT_TOKEN").is_ok()
                    );
                    match init_secrets_manager().await {
                        Ok(_) => info!("SecretsManager initialized successfully"),
                        Err(e) => {
                            error!("Failed to initialize SecretsManager: {}", e);
                            // Don't continue if SecretsManager fails - it's required for DB connection
                            return Err(anyhow::anyhow!(
                                "SecretsManager initialization failed: {}",
                                e
                            ));
                        }
                    }
                }

                if component == "email" {
                    info!("Auto-configuring Email (Stalwart)...");
                    if let Err(e) = self.setup_email().await {
                        error!("Failed to setup Email: {}", e);
                    }
                }

                if component == "proxy" {
                    info!("Configuring Caddy reverse proxy...");
                    if let Err(e) = self.setup_caddy_proxy().await {
                        error!("Failed to setup Caddy: {}", e);
                    }
                }

                if component == "dns" {
                    info!("Configuring CoreDNS for dynamic DNS...");
                    if let Err(e) = self.setup_coredns().await {
                        error!("Failed to setup CoreDNS: {}", e);
                    }
                }
            }
        }
        info!("=== BOOTSTRAP COMPLETED SUCCESSFULLY ===");
        Ok(())
    }

    /// Configure database and drive credentials in Directory
    /// This creates the Zitadel config files BEFORE Zitadel is installed
    /// db_password is passed directly from bootstrap - NO ENV VARS
    async fn configure_services_in_directory(&self, db_password: &str) -> Result<()> {
        info!("Creating Zitadel configuration files...");

        let zitadel_config_path = PathBuf::from("./botserver-stack/conf/directory/zitadel.yaml");
        let steps_config_path = PathBuf::from("./botserver-stack/conf/directory/steps.yaml");
        // Use absolute path for PAT file since zitadel runs from bin/directory/
        let pat_path =
            std::env::current_dir()?.join("botserver-stack/conf/directory/admin-pat.txt");

        fs::create_dir_all(zitadel_config_path.parent().unwrap())?;

        // Generate Zitadel database password
        let zitadel_db_password = self.generate_secure_password(24);

        // Create zitadel.yaml - main configuration
        // Note: Zitadel uses lowercase 'postgres' and nested User/Admin with Username field
        let zitadel_config = format!(
            r#"Log:
  Level: info
  Formatter:
    Format: text

Database:
  postgres:
    Host: localhost
    Port: 5432
    Database: zitadel
    User:
      Username: zitadel
      Password: "{}"
      SSL:
        Mode: disable
    Admin:
      Username: gbuser
      Password: "{}"
      SSL:
        Mode: disable

Machine:
  Identification:
    Hostname:
      Enabled: true

ExternalSecure: false
ExternalDomain: localhost
ExternalPort: 8300

DefaultInstance:
  OIDCSettings:
    AccessTokenLifetime: 12h
    IdTokenLifetime: 12h
    RefreshTokenIdleExpiration: 720h
    RefreshTokenExpiration: 2160h
"#,
            zitadel_db_password,
            db_password, // Use the password passed directly from bootstrap
        );

        fs::write(&zitadel_config_path, zitadel_config)?;
        info!("Created zitadel.yaml configuration");

        // Create steps.yaml - first instance setup that generates admin PAT
        // Use Machine user with PAT for API access (Human users don't generate PAT files)
        let steps_config = format!(
            r#"FirstInstance:
  InstanceName: "BotServer"
  DefaultLanguage: "en"
  PatPath: "{}"
  Org:
    Name: "BotServer"
    Machine:
      Machine:
        Username: "admin-sa"
        Name: "Admin Service Account"
      Pat:
        ExpirationDate: "2099-12-31T23:59:59Z"
    Human:
      UserName: "admin"
      FirstName: "Admin"
      LastName: "User"
      Email:
        Address: "admin@localhost"
        Verified: true
      Password: "{}"
      PasswordChangeRequired: false
"#,
            pat_path.to_string_lossy(),
            self.generate_secure_password(16),
        );

        fs::write(&steps_config_path, steps_config)?;
        info!("Created steps.yaml for first instance setup");

        // Create zitadel database in PostgreSQL
        info!("Creating zitadel database...");
        let create_db_result = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "PGPASSWORD='{}' psql -h localhost -p 5432 -U gbuser -d postgres -c \"CREATE DATABASE zitadel\" 2>&1 || true",
                db_password
            ))
            .output();

        if let Ok(output) = create_db_result {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.contains("already exists") {
                info!("Created zitadel database");
            }
        }

        // Create zitadel user
        let create_user_result = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "PGPASSWORD='{}' psql -h localhost -p 5432 -U gbuser -d postgres -c \"CREATE USER zitadel WITH PASSWORD '{}' SUPERUSER\" 2>&1 || true",
                db_password,
                zitadel_db_password
            ))
            .output();

        if let Ok(output) = create_user_result {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.contains("already exists") {
                info!("Created zitadel database user");
            }
        }

        info!("Zitadel configuration files created");
        Ok(())
    }

    /// Setup Caddy as reverse proxy for all services
    async fn setup_caddy_proxy(&self) -> Result<()> {
        let caddy_config = PathBuf::from("./botserver-stack/conf/proxy/Caddyfile");
        fs::create_dir_all(caddy_config.parent().unwrap())?;

        let config = format!(
            r#"{{
    admin off
    auto_https disable_redirects
}}

# Main API
api.botserver.local {{
    tls /botserver-stack/conf/system/certificates/caddy/server.crt /botserver-stack/conf/system/certificates/caddy/server.key
    reverse_proxy {}
}}

# Directory/Auth service
auth.botserver.local {{
    tls /botserver-stack/conf/system/certificates/caddy/server.crt /botserver-stack/conf/system/certificates/caddy/server.key
    reverse_proxy {}
}}

# LLM service
llm.botserver.local {{
    tls /botserver-stack/conf/system/certificates/caddy/server.crt /botserver-stack/conf/system/certificates/caddy/server.key
    reverse_proxy {}
}}

# Mail service
mail.botserver.local {{
    tls /botserver-stack/conf/system/certificates/caddy/server.crt /botserver-stack/conf/system/certificates/caddy/server.key
    reverse_proxy {}
}}

# Meet service
meet.botserver.local {{
    tls /botserver-stack/conf/system/certificates/caddy/server.crt /botserver-stack/conf/system/certificates/caddy/server.key
    reverse_proxy {}
}}
"#,
            crate::core::urls::InternalUrls::DIRECTORY_BASE.replace("https://", ""),
            crate::core::urls::InternalUrls::DIRECTORY_BASE.replace("https://", ""),
            crate::core::urls::InternalUrls::LLM.replace("https://", ""),
            crate::core::urls::InternalUrls::EMAIL.replace("https://", ""),
            crate::core::urls::InternalUrls::LIVEKIT.replace("https://", "")
        );

        fs::write(caddy_config, config)?;
        info!("Caddy proxy configured");
        Ok(())
    }

    /// Setup CoreDNS for dynamic DNS service
    async fn setup_coredns(&self) -> Result<()> {
        let dns_config = PathBuf::from("./botserver-stack/conf/dns/Corefile");
        fs::create_dir_all(dns_config.parent().unwrap())?;

        let zone_file = PathBuf::from("./botserver-stack/conf/dns/botserver.local.zone");

        // Create Corefile
        let corefile = r#"botserver.local:53 {
    file /botserver-stack/conf/dns/botserver.local.zone
    reload 10s
    log
}

.:53 {
    forward . 8.8.8.8 8.8.4.4
    cache 30
    log
}
"#;

        fs::write(dns_config, corefile)?;

        // Create initial zone file with component names
        let zone = r#"$ORIGIN botserver.local.
$TTL 60
@       IN      SOA     ns1.botserver.local. admin.botserver.local. (
                        2024010101      ; Serial
                        3600            ; Refresh
                        1800            ; Retry
                        604800          ; Expire
                        60              ; Minimum TTL
)
        IN      NS      ns1.botserver.local.
ns1     IN      A       127.0.0.1

; Core services
api         IN      A       127.0.0.1
tables      IN      A       127.0.0.1
drive       IN      A       127.0.0.1
cache       IN      A       127.0.0.1
vectordb    IN      A       127.0.0.1
vault       IN      A       127.0.0.1

; Application services
llm         IN      A       127.0.0.1
embedding   IN      A       127.0.0.1
directory   IN      A       127.0.0.1
auth        IN      A       127.0.0.1
email       IN      A       127.0.0.1
meet        IN      A       127.0.0.1

; Dynamic entries will be added below
"#;

        fs::write(zone_file, zone)?;
        info!("CoreDNS configured for dynamic DNS");
        Ok(())
    }

    /// Setup Directory (Zitadel) with default organization and user
    async fn setup_directory(&self) -> Result<()> {
        let config_path = PathBuf::from("./config/directory_config.json");
        let pat_path = PathBuf::from("./botserver-stack/conf/directory/admin-pat.txt");

        // Ensure config directory exists
        tokio::fs::create_dir_all("./config").await?;

        // Wait for Directory to be ready and check for PAT file
        info!("Waiting for Zitadel to be ready...");
        let mut attempts = 0;
        let max_attempts = 60; // 60 seconds max wait

        while attempts < max_attempts {
            // Check if Zitadel is healthy
            let health_check = std::process::Command::new("curl")
                .args(["-f", "-s", "http://localhost:8300/healthz"])
                .output();

            if let Ok(output) = health_check {
                if output.status.success() {
                    info!("Zitadel is healthy");
                    break;
                }
            }

            attempts += 1;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        if attempts >= max_attempts {
            warn!("Zitadel health check timed out, continuing anyway...");
        }

        // Wait a bit more for PAT file to be generated
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // Read the admin PAT generated by Zitadel first instance setup
        let admin_token = if pat_path.exists() {
            let token = fs::read_to_string(&pat_path)?;
            let token = token.trim().to_string();
            info!("Loaded admin PAT from {}", pat_path.display());
            Some(token)
        } else {
            warn!("Admin PAT file not found at {}", pat_path.display());
            warn!("Zitadel first instance setup may not have completed");
            None
        };

        let mut setup = DirectorySetup::new(
            "http://localhost:8300".to_string(), // Use HTTP since TLS is disabled
            config_path,
        );

        // Set the admin token if we have it
        if let Some(token) = admin_token {
            setup.set_admin_token(token);
        } else {
            // If no PAT, we can't proceed with API calls
            info!("Directory setup skipped - no admin token available");
            info!("First instance setup created initial admin user via steps.yaml");
            return Ok(());
        }

        // Wait a bit more for Zitadel to be fully ready
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Try to create additional organization for bot users
        let org_name = "default";
        match setup
            .create_organization(org_name, "Default Organization")
            .await
        {
            Ok(org_id) => {
                info!("Created default organization: {}", org_name);

                // Generate secure passwords
                let user_password = self.generate_secure_password(16);

                // Create user@default account for regular bot usage
                match setup
                    .create_user(
                        &org_id,
                        "user",
                        "user@default",
                        &user_password,
                        "User",
                        "Default",
                        false,
                    )
                    .await
                {
                    Ok(regular_user) => {
                        info!("Created regular user: user@default");
                        info!("   Regular user ID: {}", regular_user.id);
                    }
                    Err(e) => {
                        warn!("Failed to create regular user: {}", e);
                    }
                }

                // Create OAuth2 application for BotServer
                match setup.create_oauth_application(&org_id).await {
                    Ok((project_id, client_id, client_secret)) => {
                        info!("Created OAuth2 application in project: {}", project_id);

                        // Save configuration
                        let admin_user = crate::package_manager::setup::DefaultUser {
                            id: "admin".to_string(),
                            username: "admin".to_string(),
                            email: "admin@localhost".to_string(),
                            password: "".to_string(), // Don't store password
                            first_name: "Admin".to_string(),
                            last_name: "User".to_string(),
                        };

                        if let Ok(config) = setup
                            .save_config(
                                org_id.clone(),
                                org_name.to_string(),
                                admin_user,
                                client_id.clone(),
                                client_secret,
                            )
                            .await
                        {
                            info!("Directory initialized successfully!");
                            info!("   Organization: default");
                            info!("   Client ID: {}", client_id);
                            info!("   Login URL: {}", config.base_url);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to create OAuth2 application: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create organization: {}", e);
                info!("Using Zitadel's default organization from first instance setup");
            }
        }

        info!("Directory setup complete");
        Ok(())
    }

    /// Setup Vault with all service secrets and write .env file with VAULT_* variables
    async fn setup_vault(
        &self,
        db_password: &str,
        drive_accesskey: &str,
        drive_secret: &str,
        cache_password: &str,
    ) -> Result<()> {
        let vault_conf_path = PathBuf::from("./botserver-stack/conf/vault");
        let vault_init_path = vault_conf_path.join("init.json");
        let env_file_path = PathBuf::from("./.env");

        // Wait for Vault to be ready
        info!("Waiting for Vault to be ready...");
        let mut attempts = 0;
        let max_attempts = 30;

        while attempts < max_attempts {
            // First check if Vault process is running
            let ps_check = std::process::Command::new("sh")
                .arg("-c")
                .arg("pgrep -f 'vault server' || echo 'NOT_RUNNING'")
                .output();

            if let Ok(ps_output) = ps_check {
                let ps_result = String::from_utf8_lossy(&ps_output.stdout);
                if ps_result.contains("NOT_RUNNING") {
                    warn!("Vault process is not running (attempt {})", attempts + 1);
                    // Check vault.log for crash info
                    let vault_log_path = PathBuf::from("./botserver-stack/logs/vault/vault.log");
                    if vault_log_path.exists() {
                        if let Ok(log_content) = fs::read_to_string(&vault_log_path) {
                            let last_lines: Vec<&str> =
                                log_content.lines().rev().take(10).collect();
                            warn!("Vault log (last 10 lines):");
                            for line in last_lines.iter().rev() {
                                warn!("  {}", line);
                            }
                        }
                    }
                }
            }

            let health_check = std::process::Command::new("curl")
                .args(["-f", "-s", "http://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200"])
                .output();

            if let Ok(output) = health_check {
                if output.status.success() {
                    info!("Vault is responding");
                    break;
                } else {
                    // Log the HTTP response for debugging
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.is_empty() && attempts % 5 == 0 {
                        debug!("Vault health check attempt {}: {}", attempts + 1, stderr);
                    }
                }
            } else if attempts % 5 == 0 {
                warn!("Vault health check curl failed (attempt {})", attempts + 1);
            }

            attempts += 1;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        if attempts >= max_attempts {
            warn!(
                "Vault health check timed out after {} attempts",
                max_attempts
            );
            // Final check of vault.log
            let vault_log_path = PathBuf::from("./botserver-stack/logs/vault/vault.log");
            if vault_log_path.exists() {
                if let Ok(log_content) = fs::read_to_string(&vault_log_path) {
                    let last_lines: Vec<&str> = log_content.lines().rev().take(20).collect();
                    error!("Vault log (last 20 lines):");
                    for line in last_lines.iter().rev() {
                        error!("  {}", line);
                    }
                }
            } else {
                error!("Vault log file does not exist at {:?}", vault_log_path);
            }
            return Err(anyhow::anyhow!(
                "Vault not ready after {} seconds. Check ./botserver-stack/logs/vault/vault.log for details.",
                max_attempts
            ));
        }

        // Check if Vault is already initialized
        let vault_addr = "http://localhost:8200";
        std::env::set_var("VAULT_ADDR", vault_addr);
        std::env::set_var("VAULT_SKIP_VERIFY", "true");

        // Read init.json if it exists (from post_install_cmds)
        let (unseal_key, root_token) = if vault_init_path.exists() {
            info!("Reading Vault initialization from init.json...");
            let init_json = fs::read_to_string(&vault_init_path)?;
            let init_data: serde_json::Value = serde_json::from_str(&init_json)?;

            let unseal_key = init_data["unseal_keys_b64"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let root_token = init_data["root_token"].as_str().unwrap_or("").to_string();

            (unseal_key, root_token)
        } else {
            // Check if .env exists with VAULT_TOKEN - try to recover from that
            let env_token = if env_file_path.exists() {
                if let Ok(env_content) = fs::read_to_string(&env_file_path) {
                    env_content.lines()
                        .find(|line| line.starts_with("VAULT_TOKEN="))
                        .map(|line| line.trim_start_matches("VAULT_TOKEN=").to_string())
                } else {
                    None
                }
            } else {
                None
            };

            // Initialize Vault if not already done
            info!("Initializing Vault...");
            // Clear any mTLS env vars that might interfere with CLI
            let init_output = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} ./botserver-stack/bin/vault/vault operator init -key-shares=1 -key-threshold=1 -format=json",
                    vault_addr
                ))
                .output()?;

            if !init_output.status.success() {
                let stderr = String::from_utf8_lossy(&init_output.stderr);
                if stderr.contains("already initialized") {
                    warn!("Vault already initialized but init.json not found");
                    
                    // If we have a token from .env, check if Vault is already unsealed
                    // and we can continue (maybe it was manually unsealed)
                    if let Some(_token) = env_token {
                        info!("Found VAULT_TOKEN in .env, checking if Vault is unsealed...");
                        
                        // Check Vault status
                        let status_check = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(format!(
                                "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} ./botserver-stack/bin/vault/vault status -format=json 2>/dev/null",
                                vault_addr
                            ))
                            .output();
                        
                        if let Ok(status_output) = status_check {
                            let status_str = String::from_utf8_lossy(&status_output.stdout);
                            if let Ok(status) = serde_json::from_str::<serde_json::Value>(&status_str) {
                                let sealed = status["sealed"].as_bool().unwrap_or(true);
                                if !sealed {
                                    // Vault is unsealed! We can continue with the token from .env
                                    warn!("Vault is already unsealed - continuing with existing token");
                                    warn!("NOTE: Unseal key is lost - Vault will need manual unseal after restart");
                                    return Ok(());  // Skip rest of setup, Vault is already working
                                }
                            }
                        }
                        
                        // Vault is sealed but we don't have unseal key
                        error!("Vault is sealed and unseal key is lost (init.json missing)");
                        error!("Options:");
                        error!("  1. If you have a backup of init.json, restore it to ./botserver-stack/conf/vault/init.json");
                        error!("  2. To start fresh, delete ./botserver-stack/data/vault/ and restart");
                        return Err(anyhow::anyhow!(
                            "Vault is sealed but unseal key is lost. See error messages above for recovery options."
                        ));
                    }
                    
                    // No token in .env either
                    error!("Vault already initialized but credentials are lost");
                    error!("Options:");
                    error!("  1. If you have a backup of init.json, restore it to ./botserver-stack/conf/vault/init.json");  
                    error!("  2. To start fresh, delete ./botserver-stack/data/vault/ and ./botserver-stack/conf/vault/init.json and restart");
                    return Err(anyhow::anyhow!(
                        "Vault initialized but credentials lost. See error messages above for recovery options."
                    ));
                }
                return Err(anyhow::anyhow!("Vault init failed: {}", stderr));
            }

            let init_json = String::from_utf8_lossy(&init_output.stdout);
            fs::write(&vault_init_path, init_json.as_ref())?;
            fs::set_permissions(&vault_init_path, std::fs::Permissions::from_mode(0o600))?;

            let init_data: serde_json::Value = serde_json::from_str(&init_json)?;
            let unseal_key = init_data["unseal_keys_b64"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let root_token = init_data["root_token"].as_str().unwrap_or("").to_string();

            (unseal_key, root_token)
        };

        if root_token.is_empty() {
            return Err(anyhow::anyhow!("Failed to get Vault root token"));
        }

        // Unseal Vault
        info!("Unsealing Vault...");
        // Clear any mTLS env vars that might interfere with CLI
        let unseal_output = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} ./botserver-stack/bin/vault/vault operator unseal {}",
                vault_addr, unseal_key
            ))
            .output()?;

        if !unseal_output.status.success() {
            let stderr = String::from_utf8_lossy(&unseal_output.stderr);
            if !stderr.contains("already unsealed") {
                warn!("Vault unseal warning: {}", stderr);
            }
        }

        // Set VAULT_TOKEN for subsequent commands
        std::env::set_var("VAULT_TOKEN", &root_token);

        // WRITE .env IMMEDIATELY so SecretsManager can work
        info!("Writing .env file with Vault configuration...");
        let env_content = format!(
            r#"# BotServer Environment Configuration
# Generated by bootstrap - DO NOT ADD OTHER SECRETS HERE
# All secrets are stored in Vault at the paths below:
#   - gbo/tables     - PostgreSQL credentials
#   - gbo/drive      - MinIO/S3 credentials
#   - gbo/cache      - Redis credentials
#   - gbo/directory  - Zitadel credentials
#   - gbo/email      - Email credentials
#   - gbo/llm        - LLM API keys
#   - gbo/encryption - Encryption keys

# Vault Configuration - THESE ARE THE ONLY ALLOWED ENV VARS
VAULT_ADDR={}
VAULT_TOKEN={}

# Vault uses HTTP for local development (TLS disabled in config.hcl)
# In production, enable TLS and set VAULT_CACERT, VAULT_CLIENT_CERT, VAULT_CLIENT_KEY

# Cache TTL for secrets (seconds)
VAULT_CACHE_TTL=300
"#,
            vault_addr, root_token
        );
        fs::write(&env_file_path, &env_content)?;
        info!("  * Created .env file with Vault configuration");

        // Re-initialize SecretsManager now that .env exists
        info!("Re-initializing SecretsManager with Vault credentials...");
        match init_secrets_manager().await {
            Ok(_) => info!("  * SecretsManager now connected to Vault"),
            Err(e) => warn!("SecretsManager re-init warning: {}", e),
        }

        // Enable KV secrets engine at gbo/ path
        info!("Enabling KV secrets engine...");
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault secrets enable -path=secret kv-v2 2>&1 || true",
                vault_addr, root_token
            ))
            .output();

        // Store secrets in Vault - ONLY if they don't already exist
        // This protects existing customer data in distributed environments
        info!("Storing secrets in Vault (only if not existing)...");

        // Helper to check if a secret path exists
        let secret_exists = |path: &str| -> bool {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault kv get {} 2>/dev/null",
                    vault_addr, root_token, path
                ))
                .output();
            output.map(|o| o.status.success()).unwrap_or(false)
        };

        // Database credentials - only create if not existing
        if !secret_exists("secret/gbo/tables") {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault kv put secret/gbo/tables host=localhost port=5432 database=botserver username=gbuser password='{}'",
                    vault_addr, root_token, db_password
                ))
                .output()?;
            info!("  Stored database credentials");
        } else {
            info!("  Database credentials already exist - preserving");
        }

        // Drive credentials - only create if not existing
        if !secret_exists("secret/gbo/drive") {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault kv put secret/gbo/drive accesskey='{}' secret='{}'",
                    vault_addr, root_token, drive_accesskey, drive_secret
                ))
                .output()?;
            info!("  Stored drive credentials");
        } else {
            info!("  Drive credentials already exist - preserving");
        }

        // Cache credentials - only create if not existing
        if !secret_exists("secret/gbo/cache") {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault kv put secret/gbo/cache password='{}'",
                    vault_addr, root_token, cache_password
                ))
                .output()?;
            info!("  Stored cache credentials");
        } else {
            info!("  Cache credentials already exist - preserving");
        }

        // Directory placeholder - only create if not existing
        if !secret_exists("secret/gbo/directory") {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault kv put secret/gbo/directory url=https://localhost:8300 project_id= client_id= client_secret=",
                    vault_addr, root_token
                ))
                .output()?;
            info!("  Created directory placeholder");
        } else {
            info!("  Directory credentials already exist - preserving");
        }

        // LLM placeholder - only create if not existing
        if !secret_exists("secret/gbo/llm") {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault kv put secret/gbo/llm openai_key= anthropic_key= groq_key=",
                    vault_addr, root_token
                ))
                .output()?;
            info!("  Created LLM placeholder");
        } else {
            info!("  LLM credentials already exist - preserving");
        }

        // Email placeholder - only create if not existing
        if !secret_exists("secret/gbo/email") {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault kv put secret/gbo/email username= password=",
                    vault_addr, root_token
                ))
                .output()?;
            info!("  Created email placeholder");
        } else {
            info!("  Email credentials already exist - preserving");
        }

        // Encryption key - only create if not existing (CRITICAL - never overwrite!)
        if !secret_exists("secret/gbo/encryption") {
            let encryption_key = self.generate_secure_password(32);
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault kv put secret/gbo/encryption master_key='{}'",
                    vault_addr, root_token, encryption_key
                ))
                .output()?;
            info!("  Generated and stored encryption key");
        } else {
            info!("  Encryption key already exists - preserving (CRITICAL)");
        }

        info!("Vault setup complete!");
        info!("   Vault UI: {}/ui", vault_addr);
        info!("   Root token saved to: {}", vault_init_path.display());

        Ok(())
    }

    /// Setup Email (Stalwart) with Directory integration
    pub async fn setup_email(&self) -> Result<()> {
        let config_path = PathBuf::from("./config/email_config.json");
        let directory_config_path = PathBuf::from("./config/directory_config.json");

        let mut setup = EmailSetup::new(
            crate::core::urls::InternalUrls::DIRECTORY_BASE.to_string(),
            config_path,
        );

        // Try to integrate with Directory if it exists
        let directory_config = if directory_config_path.exists() {
            Some(directory_config_path)
        } else {
            None
        };

        let config = setup.initialize(directory_config).await?;

        info!("Email server initialized successfully!");
        info!("   SMTP: {}:{}", config.smtp_host, config.smtp_port);
        info!("   IMAP: {}:{}", config.imap_host, config.imap_port);
        info!("   Admin: {} / {}", config.admin_user, config.admin_pass);
        if config.directory_integration {
            info!("    Integrated with Directory for authentication");
        }

        Ok(())
    }

    async fn get_drive_client(config: &AppConfig) -> Client {
        let endpoint = if config.drive.server.ends_with('/') {
            config.drive.server.clone()
        } else {
            format!("{}/", config.drive.server)
        };

        // Get credentials from config, or fetch from Vault if empty
        let (access_key, secret_key) =
            if config.drive.access_key.is_empty() || config.drive.secret_key.is_empty() {
                // Try to get from Vault using the global SecretsManager
                match crate::shared::utils::get_secrets_manager().await {
                    Some(manager) if manager.is_enabled() => {
                        match manager.get_drive_credentials().await {
                            Ok((ak, sk)) => (ak, sk),
                            Err(e) => {
                                warn!("Failed to get drive credentials from Vault: {}", e);
                                (
                                    config.drive.access_key.clone(),
                                    config.drive.secret_key.clone(),
                                )
                            }
                        }
                    }
                    _ => (
                        config.drive.access_key.clone(),
                        config.drive.secret_key.clone(),
                    ),
                }
            } else {
                (
                    config.drive.access_key.clone(),
                    config.drive.secret_key.clone(),
                )
            };

        let base_config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region("auto")
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                access_key, secret_key, None, None, "static",
            ))
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&base_config)
            .force_path_style(true)
            .build();
        aws_sdk_s3::Client::from_conf(s3_config)
    }

    /// Sync bot configurations from template config.csv files to database
    /// This is separate from drive upload and does not require S3 connection
    pub fn sync_templates_to_database(&self) -> Result<()> {
        let mut conn = establish_pg_connection()?;
        self.create_bots_from_templates(&mut conn)?;
        Ok(())
    }

    pub async fn upload_templates_to_drive(&self, _config: &AppConfig) -> Result<()> {
        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            return Ok(());
        }
        let client = Self::get_drive_client(_config).await;
        let mut read_dir = tokio::fs::read_dir(templates_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with(".gbai")
            {
                let bot_name = path.file_name().unwrap().to_string_lossy().to_string();
                let bucket = bot_name.trim_start_matches('/').to_string();
                if client.head_bucket().bucket(&bucket).send().await.is_err() {
                    match client.create_bucket().bucket(&bucket).send().await {
                        Ok(_) => {
                            self.upload_directory_recursive(&client, &path, &bucket, "/")
                                .await?;
                        }
                        Err(e) => {
                            error!("Failed to create bucket {}: {:?}", bucket, e);
                            return Err(anyhow::anyhow!("Failed to create bucket {}: {}. Check S3 credentials and endpoint configuration", bucket, e));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn create_bots_from_templates(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        use crate::shared::models::schema::bots;
        use diesel::prelude::*;

        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            warn!("Templates directory does not exist");
            return Ok(());
        }

        // Get the default bot (created by migrations) - we'll sync all template configs to it
        let default_bot: Option<(uuid::Uuid, String)> = bots::table
            .filter(bots::is_active.eq(true))
            .select((bots::id, bots::name))
            .first(conn)
            .optional()?;

        let (default_bot_id, default_bot_name) = match default_bot {
            Some((id, name)) => (id, name),
            None => {
                error!("No active bot found in database - cannot sync template configs");
                return Ok(());
            }
        };

        info!(
            "Syncing template configs to bot '{}' ({})",
            default_bot_name, default_bot_id
        );

        // Only sync the default.gbai template config (main config for the system)
        let default_template = templates_dir.join("default.gbai");
        if default_template.exists() {
            let config_path = default_template.join("default.gbot").join("config.csv");

            if config_path.exists() {
                match std::fs::read_to_string(&config_path) {
                    Ok(csv_content) => {
                        debug!("Syncing config.csv from {:?}", config_path);
                        if let Err(e) =
                            self.sync_config_csv_to_db(conn, &default_bot_id, &csv_content)
                        {
                            error!("Failed to sync config.csv: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Could not read config.csv: {}", e);
                    }
                }
            } else {
                debug!("No config.csv found at {:?}", config_path);
            }
        } else {
            debug!("default.gbai template not found");
        }

        Ok(())
    }

    /// Sync config.csv content to the bot_configuration table
    /// This is critical for loading LLM settings on fresh starts
    fn sync_config_csv_to_db(
        &self,
        conn: &mut diesel::PgConnection,
        bot_id: &uuid::Uuid,
        content: &str,
    ) -> Result<()> {
        let mut synced = 0;
        let mut skipped = 0;
        let lines: Vec<&str> = content.lines().collect();

        debug!(
            "Parsing config.csv with {} lines for bot {}",
            lines.len(),
            bot_id
        );

        for (line_num, line) in lines.iter().enumerate().skip(1) {
            // Skip header line (name,value)
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ',').collect();
            if parts.len() >= 2 {
                let key = parts[0].trim();
                let value = parts[1].trim();

                if key.is_empty() {
                    skipped += 1;
                    continue;
                }

                // Use UUID type since migration 6.1.1 converted column to UUID
                let new_id = uuid::Uuid::new_v4();

                match diesel::sql_query(
                    "INSERT INTO bot_configuration (id, bot_id, config_key, config_value, config_type, created_at, updated_at) \
                     VALUES ($1, $2, $3, $4, 'string', NOW(), NOW()) \
                     ON CONFLICT (bot_id, config_key) DO UPDATE SET config_value = EXCLUDED.config_value, updated_at = NOW()"
                )
                .bind::<diesel::sql_types::Uuid, _>(new_id)
                .bind::<diesel::sql_types::Uuid, _>(bot_id)
                .bind::<diesel::sql_types::Text, _>(key)
                .bind::<diesel::sql_types::Text, _>(value)
                .execute(conn) {
                    Ok(_) => {
                        synced += 1;
                    }
                    Err(e) => {
                        error!("Failed to sync config key '{}' at line {}: {}", key, line_num + 1, e);
                        // Continue with other keys instead of failing completely
                    }
                }
            }
        }

        if synced > 0 {
            info!(
                "Synced {} config values for bot {} (skipped {} empty lines)",
                synced, bot_id, skipped
            );
        } else {
            warn!(
                "No config values synced for bot {} - check config.csv format",
                bot_id
            );
        }
        Ok(())
    }
    fn upload_directory_recursive<'a>(
        &'a self,
        client: &'a Client,
        local_path: &'a Path,
        bucket: &'a str,
        prefix: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let _normalized_path = if local_path.to_string_lossy().ends_with('/') {
                local_path.to_string_lossy().to_string()
            } else {
                format!("{}/", local_path.display())
            };
            let mut read_dir = tokio::fs::read_dir(local_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let path = entry.path();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let mut key = prefix.trim_matches('/').to_string();
                if !key.is_empty() {
                    key.push('/');
                }
                key.push_str(&file_name);
                if path.is_file() {
                    let content = tokio::fs::read(&path).await?;
                    client
                        .put_object()
                        .bucket(bucket)
                        .key(&key)
                        .body(content.into())
                        .send()
                        .await?;
                } else if path.is_dir() {
                    self.upload_directory_recursive(client, &path, bucket, &key)
                        .await?;
                }
            }
            Ok(())
        })
    }
    pub fn apply_migrations(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

        // Run migrations silently - don't output to console
        if let Err(e) = conn.run_pending_migrations(MIGRATIONS) {
            error!("Failed to apply migrations: {}", e);
            return Err(anyhow::anyhow!("Migration error: {}", e));
        }

        Ok(())
    }

    /// Create Vault configuration with mTLS settings
    async fn create_vault_config(&self) -> Result<()> {
        let vault_conf_dir = PathBuf::from("./botserver-stack/conf/vault");
        let config_path = vault_conf_dir.join("config.hcl");

        fs::create_dir_all(&vault_conf_dir)?;

        // Vault is started from botserver-stack/bin/vault/, so paths must be relative to that
        // From bin/vault/ to conf/ is ../../conf/
        // From bin/vault/ to data/ is ../../data/
        let config = r#"# Vault Configuration
# Generated by BotServer bootstrap
# Note: Paths are relative to botserver-stack/bin/vault/ (Vault's working directory)

# Storage backend - file-based for single instance
storage "file" {
  path = "../../data/vault"
}

# Listener with TLS DISABLED for local development
# In production, enable TLS with proper certificates
listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = true
}

# API settings - use HTTP for local dev
api_addr = "http://localhost:8200"
cluster_addr = "http://localhost:8201"

# UI enabled for administration
ui = true

# Disable memory locking (for development - enable in production)
disable_mlock = true

# Telemetry
telemetry {
  disable_hostname = true
}

# Log level
log_level = "info"
"#;

        fs::write(&config_path, config)?;

        // Create data directory for Vault storage
        fs::create_dir_all("./botserver-stack/data/vault")?;

        info!(
            "Created Vault config with mTLS at {}",
            config_path.display()
        );
        Ok(())
    }

    /// Generate TLS certificates for all services
    async fn generate_certificates(&self) -> Result<()> {
        let cert_dir = PathBuf::from("./botserver-stack/conf/system/certificates");

        // Create certificate directory structure
        fs::create_dir_all(&cert_dir)?;
        fs::create_dir_all(cert_dir.join("ca"))?;

        // Check if CA already exists
        let ca_cert_path = cert_dir.join("ca/ca.crt");
        let ca_key_path = cert_dir.join("ca/ca.key");

        // CA params for issuer creation
        let mut ca_params = CertificateParams::default();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CountryName, "BR");
        dn.push(DnType::OrganizationName, "BotServer");
        dn.push(DnType::CommonName, "BotServer CA");
        ca_params.distinguished_name = dn;

        ca_params.not_before = time::OffsetDateTime::now_utc();
        ca_params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(3650);

        let ca_key_pair: KeyPair = if ca_cert_path.exists() && ca_key_path.exists() {
            info!("Using existing CA certificate");
            // Load existing CA key
            let key_pem = fs::read_to_string(&ca_key_path)?;
            KeyPair::from_pem(&key_pem)?
        } else {
            info!("Generating new CA certificate");
            let key_pair = KeyPair::generate()?;
            let cert = ca_params.self_signed(&key_pair)?;

            // Save CA certificate and key
            fs::write(&ca_cert_path, cert.pem())?;
            fs::write(&ca_key_path, key_pair.serialize_pem())?;

            key_pair
        };

        // Create issuer from CA params and key
        let ca_issuer = Issuer::from_params(&ca_params, &ca_key_pair);

        // Generate client certificate for botserver (for mTLS to all services)
        let botserver_dir = cert_dir.join("botserver");
        fs::create_dir_all(&botserver_dir)?;

        let client_cert_path = botserver_dir.join("client.crt");
        let client_key_path = botserver_dir.join("client.key");

        if !client_cert_path.exists() || !client_key_path.exists() {
            info!("Generating mTLS client certificate for botserver");

            let mut client_params = CertificateParams::default();
            client_params.not_before = time::OffsetDateTime::now_utc();
            client_params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(365);

            let mut client_dn = DistinguishedName::new();
            client_dn.push(DnType::CountryName, "BR");
            client_dn.push(DnType::OrganizationName, "BotServer");
            client_dn.push(DnType::CommonName, "botserver-client");
            client_params.distinguished_name = client_dn;

            // Add client auth extended key usage
            client_params
                .subject_alt_names
                .push(rcgen::SanType::DnsName("botserver".to_string().try_into()?));

            let client_key = KeyPair::generate()?;
            let client_cert = client_params.signed_by(&client_key, &ca_issuer)?;

            fs::write(&client_cert_path, client_cert.pem())?;
            fs::write(&client_key_path, client_key.serialize_pem())?;
            fs::copy(&ca_cert_path, botserver_dir.join("ca.crt"))?;

            info!(
                "Generated mTLS client certificate at {}",
                client_cert_path.display()
            );
        }

        // Services that need certificates - Vault FIRST
        // Using component names: tables (postgres), drive (minio), cache (redis), vectordb (qdrant)
        let services = vec![
            (
                "vault",
                vec!["localhost", "127.0.0.1", "vault.botserver.local"],
            ),
            ("api", vec!["localhost", "127.0.0.1", "api.botserver.local"]),
            ("llm", vec!["localhost", "127.0.0.1", "llm.botserver.local"]),
            (
                "embedding",
                vec!["localhost", "127.0.0.1", "embedding.botserver.local"],
            ),
            (
                "vectordb",
                vec!["localhost", "127.0.0.1", "vectordb.botserver.local"],
            ),
            (
                "tables",
                vec!["localhost", "127.0.0.1", "tables.botserver.local"],
            ),
            (
                "cache",
                vec!["localhost", "127.0.0.1", "cache.botserver.local"],
            ),
            (
                "drive",
                vec!["localhost", "127.0.0.1", "drive.botserver.local"],
            ),
            (
                "directory",
                vec![
                    "localhost",
                    "127.0.0.1",
                    "directory.botserver.local",
                    "auth.botserver.local",
                ],
            ),
            (
                "email",
                vec![
                    "localhost",
                    "127.0.0.1",
                    "email.botserver.local",
                    "smtp.botserver.local",
                    "imap.botserver.local",
                ],
            ),
            (
                "meet",
                vec![
                    "localhost",
                    "127.0.0.1",
                    "meet.botserver.local",
                    "turn.botserver.local",
                ],
            ),
            (
                "caddy",
                vec![
                    "localhost",
                    "127.0.0.1",
                    "*.botserver.local",
                    "botserver.local",
                ],
            ),
        ];

        for (service, sans) in services {
            let service_dir = cert_dir.join(service);
            fs::create_dir_all(&service_dir)?;

            let cert_path = service_dir.join("server.crt");
            let key_path = service_dir.join("server.key");

            // Skip if certificate already exists
            if cert_path.exists() && key_path.exists() {
                continue;
            }

            info!("Generating certificate for {}", service);

            // Generate service certificate
            let mut params = CertificateParams::default();
            params.not_before = time::OffsetDateTime::now_utc();
            params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(365);

            let mut dn = DistinguishedName::new();
            dn.push(DnType::CountryName, "BR");
            dn.push(DnType::OrganizationName, "BotServer");
            dn.push(DnType::CommonName, &format!("{}.botserver.local", service));
            params.distinguished_name = dn;

            // Add SANs
            for san in sans {
                params
                    .subject_alt_names
                    .push(rcgen::SanType::DnsName(san.to_string().try_into()?));
            }

            let key_pair = KeyPair::generate()?;
            let cert = params.signed_by(&key_pair, &ca_issuer)?;

            // Save certificate and key
            fs::write(cert_path, cert.pem())?;
            fs::write(key_path, key_pair.serialize_pem())?;

            // Copy CA cert to service directory for easy access
            fs::copy(&ca_cert_path, service_dir.join("ca.crt"))?;
        }

        info!("TLS certificates generated successfully");
        Ok(())
    }
}
