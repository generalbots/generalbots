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
    pub stack_path: PathBuf,
}
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

    fn stack_dir(&self, subpath: &str) -> PathBuf {
        self.stack_path.join(subpath)
    }

    fn vault_bin(&self) -> String {
        self.stack_dir("bin/vault/vault")
            .to_str()
            .unwrap_or("./botserver-stack/bin/vault/vault")
            .to_string()
    }

    pub fn kill_stack_processes() {
        info!("Killing any existing stack processes...");

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

        let ports = vec![8200, 5432, 9000, 6379, 8300, 8081, 8082, 25, 443, 53];

        for port in ports {
            let _ = Command::new("fuser")
                .args(["-k", "-9", &format!("{}/tcp", port)])
                .output();
        }

        std::thread::sleep(std::time::Duration::from_millis(1000));
        info!("Stack processes terminated");
    }

    pub fn check_single_instance() -> Result<bool> {
        let stack_path = std::env::var("BOTSERVER_STACK_PATH")
            .unwrap_or_else(|_| "./botserver-stack".to_string());
        let lock_file = PathBuf::from(&stack_path).join(".lock");
        if lock_file.exists() {
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

        let pid = std::process::id();
        if let Some(parent) = lock_file.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&lock_file, pid.to_string()).ok();
        Ok(true)
    }

    pub fn release_instance_lock() {
        let stack_path = std::env::var("BOTSERVER_STACK_PATH")
            .unwrap_or_else(|_| "./botserver-stack".to_string());
        let lock_file = PathBuf::from(&stack_path).join(".lock");
        if lock_file.exists() {
            fs::remove_file(&lock_file).ok();
        }
    }

    fn has_installed_stack() -> bool {
        let stack_path = std::env::var("BOTSERVER_STACK_PATH")
            .unwrap_or_else(|_| "./botserver-stack".to_string());
        let stack_dir = PathBuf::from(&stack_path);
        if !stack_dir.exists() {
            return false;
        }

        let indicators = [
            stack_dir.join("bin/vault/vault"),
            stack_dir.join("data/vault"),
            stack_dir.join("conf/vault/config.hcl"),
        ];

        indicators.iter().any(|path| path.exists())
    }

    fn reset_vault_only() -> Result<()> {
        if Self::has_installed_stack() {
            error!("REFUSING to reset Vault credentials - botserver-stack is installed!");
            error!("If you need to re-initialize, manually delete botserver-stack directory first");
            return Err(anyhow::anyhow!(
                "Cannot reset Vault - existing installation detected. Manual intervention required."
            ));
        }

        let stack_path = std::env::var("BOTSERVER_STACK_PATH")
            .unwrap_or_else(|_| "./botserver-stack".to_string());
        let vault_init = PathBuf::from(&stack_path).join("conf/vault/init.json");
        let env_file = PathBuf::from("./.env");

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

        if pm.is_installed("vault") {
            let vault_already_running = Command::new("sh")
                .arg("-c")
                .arg("curl -f -s 'http://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200' >/dev/null 2>&1")
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

                for i in 0..10 {
                    let vault_ready = Command::new("sh")
                        .arg("-c")
                        .arg("curl -f -s 'http://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200' >/dev/null 2>&1")
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

            if let Err(e) = self.ensure_vault_unsealed().await {
                warn!("Vault unseal failed: {}", e);

                if Self::has_installed_stack() {
                    error!("Vault failed to unseal but stack is installed - NOT re-initializing");
                    error!("Try manually restarting Vault or check ./botserver-stack/logs/vault/vault.log");

                    let _ = Command::new("pkill")
                        .args(["-9", "-f", "botserver-stack/bin/vault"])
                        .output();

                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                    if let Err(e) = pm.start("vault") {
                        warn!("Failed to restart Vault: {}", e);
                    }

                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                    if let Err(e) = self.ensure_vault_unsealed().await {
                        return Err(anyhow::anyhow!(
                            "Vault failed to start/unseal after restart: {}. Manual intervention required.", e
                        ));
                    }
                } else {
                    warn!("No installed stack detected - proceeding with re-initialization");

                    let _ = Command::new("pkill")
                        .args(["-9", "-f", "botserver-stack/bin/vault"])
                        .output();

                    if let Err(e) = Self::reset_vault_only() {
                        error!("Failed to reset Vault: {}", e);
                        return Err(e);
                    }

                    self.bootstrap().await?;

                    info!("Vault re-initialization complete");
                    return Ok(());
                }
            }

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

        if pm.is_installed("tables") {
            info!("Starting PostgreSQL database...");
            match pm.start("tables") {
                Ok(_child) => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    info!("PostgreSQL started");
                }
                Err(e) => {
                    warn!("PostgreSQL might already be running: {}", e);
                }
            }
        }

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
                        if component.name == "drive" {
                            for i in 0..15 {
                                let drive_ready = Command::new("sh")
                                    .arg("-c")
                                    .arg("curl -f -s 'http://127.0.0.1:9000/minio/health/live' >/dev/null 2>&1")
                                    .stdout(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::null())
                                    .status()
                                    .map(|s| s.success())
                                    .unwrap_or(false);

                                if drive_ready {
                                    info!("MinIO drive is ready and responding");
                                    break;
                                }
                                if i < 14 {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                } else {
                                    warn!("MinIO drive health check timed out after 15s");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!(
                            "Component {} might already be running: {}",
                            component.name, e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn generate_secure_password(length: usize) -> String {
        let mut rng = rand::rng();
        let base: String = (0..length.saturating_sub(4))
            .map(|_| {
                let byte = rand::Rng::sample(&mut rng, Alphanumeric);
                char::from(byte)
            })
            .collect();

        format!("{}!1Aa", base)
    }

    pub async fn ensure_services_running(&mut self) -> Result<()> {
        info!("Ensuring critical services are running...");

        let installer = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;

        let vault_installed = installer.is_installed("vault");
        let vault_initialized = self.stack_dir("conf/vault/init.json").exists();

        if !vault_installed || !vault_initialized {
            info!("Stack not fully bootstrapped, running bootstrap first...");

            Self::kill_stack_processes();

            self.bootstrap().await?;

            info!("Bootstrap complete, verifying Vault is ready...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            if let Err(e) = self.ensure_vault_unsealed().await {
                warn!("Failed to unseal Vault after bootstrap: {}", e);
            }

            return Ok(());
        }

        if installer.is_installed("vault") {
            let vault_running = Command::new("sh")
                .arg("-c")
                .arg("curl -f -s 'http://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200' >/dev/null 2>&1")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if vault_running {
                info!("Vault is already running");
            } else {
                info!("Starting Vault secrets service...");
                match installer.start("vault") {
                    Ok(_child) => {
                        info!("Vault started successfully");

                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                    Err(e) => {
                        warn!("Vault might already be running or failed to start: {}", e);
                    }
                }
            }

            if let Err(e) = self.ensure_vault_unsealed().await {
                let err_msg = e.to_string();

                if err_msg.contains("not running") || err_msg.contains("connection refused") {
                    info!("Vault not running - starting it now...");
                    let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;
                    if let Err(e) = pm.start("vault") {
                        warn!("Failed to start Vault: {}", e);
                    }
                } else {
                    warn!("Vault unseal failed: {} - attempting Vault restart only", e);

                    let _ = Command::new("pkill")
                        .args(["-9", "-f", "botserver-stack/bin/vault"])
                        .output();

                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                    let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;
                    if let Err(e) = pm.start("vault") {
                        warn!("Failed to restart Vault: {}", e);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                if let Err(e) = self.ensure_vault_unsealed().await {
                    warn!("Vault still not responding after restart: {}", e);

                    if Self::has_installed_stack() {
                        error!("CRITICAL: Vault failed but botserver-stack is installed!");
                        error!("REFUSING to delete init.json or .env - this would destroy your installation");
                        error!("Please check ./botserver-stack/logs/vault/vault.log for errors");
                        error!("You may need to manually restart Vault or check its configuration");
                        return Err(anyhow::anyhow!(
                            "Vault failed to start. Manual intervention required. Check logs at ./botserver-stack/logs/vault/vault.log"
                        ));
                    }

                    warn!("No installed stack detected - attempting Vault re-initialization");
                    if let Err(reset_err) = Self::reset_vault_only() {
                        error!("Failed to reset Vault: {}", reset_err);
                        return Err(reset_err);
                    }

                    info!("Re-initializing Vault only (preserving other services)...");
                    let pm_reinit =
                        PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;
                    if let Err(e) = pm_reinit.install("vault").await {
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
            warn!("Vault (secrets) component not installed - run bootstrap first");
            return Err(anyhow::anyhow!(
                "Vault not installed. Run bootstrap command first."
            ));
        }

        if installer.is_installed("tables") {
            info!("Starting PostgreSQL database service...");
            match installer.start("tables") {
                Ok(_child) => {
                    info!("PostgreSQL started successfully");

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

        if installer.is_installed("drive") {
            info!("Starting MinIO drive service...");
            match installer.start("drive") {
                Ok(_child) => {
                    info!("MinIO started successfully");

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

    async fn ensure_vault_unsealed(&self) -> Result<()> {
        let vault_init_path = self.stack_dir("conf/vault/init.json");
        let vault_addr = "http://localhost:8200";

        if !vault_init_path.exists() {
            return Err(anyhow::anyhow!(
                "Vault init.json not found - needs re-initialization"
            ));
        }

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

        let vault_bin = self.vault_bin();
        let mut status_str = String::new();
        let mut parsed_status: Option<serde_json::Value> = None;

        let mut connection_refused = false;
        for attempt in 0..10 {
            if attempt > 0 {
                info!(
                    "Waiting for Vault to be ready (attempt {}/10)...",
                    attempt + 1
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }

            let status_output = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "VAULT_ADDR={} {} status -format=json 2>&1",
                    vault_addr, vault_bin
                ))
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()?;

            status_str = String::from_utf8_lossy(&status_output.stdout).to_string();
            let stderr_str = String::from_utf8_lossy(&status_output.stderr).to_string();

            if status_str.contains("connection refused")
                || stderr_str.contains("connection refused")
            {
                connection_refused = true;
            } else {
                connection_refused = false;
                if let Ok(status) = serde_json::from_str::<serde_json::Value>(&status_str) {
                    parsed_status = Some(status);
                    break;
                }
            }
        }

        if connection_refused {
            warn!("Vault is not running after retries (connection refused)");
            return Err(anyhow::anyhow!("Vault not running - needs to be started"));
        }

        if let Some(status) = parsed_status {
            let initialized = status["initialized"].as_bool().unwrap_or(false);
            let sealed = status["sealed"].as_bool().unwrap_or(true);

            if !initialized {
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
                        "VAULT_ADDR={} {} operator unseal {} >/dev/null 2>&1",
                        vault_addr, vault_bin, unseal_key
                    ))
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .output()?;

                if !unseal_output.status.success() {
                    let stderr = String::from_utf8_lossy(&unseal_output.stderr);
                    warn!("Vault unseal may have failed: {}", stderr);
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let verify_output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!(
                        "VAULT_ADDR={} {} status -format=json 2>/dev/null",
                        vault_addr, vault_bin
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
            let vault_pid = std::process::Command::new("pgrep")
                .args(["-f", "vault server"])
                .output()
                .ok()
                .and_then(|o| {
                    String::from_utf8_lossy(&o.stdout)
                        .trim()
                        .parse::<i32>()
                        .ok()
                });

            if vault_pid.is_some() {
                warn!("Vault process exists but not responding - killing and will restart");
                let _ = std::process::Command::new("pkill")
                    .args(["-9", "-f", "vault server"])
                    .status();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }

            warn!("Could not get Vault status after retries: {}", status_str);
            return Err(anyhow::anyhow!("Vault not responding properly"));
        }

        std::env::set_var("VAULT_ADDR", vault_addr);
        std::env::set_var("VAULT_TOKEN", &root_token);
        std::env::set_var("VAULT_SKIP_VERIFY", "true");

        std::env::set_var(
            "VAULT_CACERT",
            self.stack_dir("conf/system/certificates/ca/ca.crt")
                .to_str()
                .unwrap_or(""),
        );
        std::env::set_var(
            "VAULT_CLIENT_CERT",
            self.stack_dir("conf/system/certificates/botserver/client.crt")
                .to_str()
                .unwrap_or(""),
        );
        std::env::set_var(
            "VAULT_CLIENT_KEY",
            self.stack_dir("conf/system/certificates/botserver/client.key")
                .to_str()
                .unwrap_or(""),
        );

        info!("Vault environment configured");
        Ok(())
    }

    pub async fn bootstrap(&mut self) -> Result<()> {
        info!("=== BOOTSTRAP STARTING ===");

        info!("Cleaning up any existing stack processes...");
        Self::kill_stack_processes();

        info!("Generating TLS certificates...");
        if let Err(e) = self.generate_certificates() {
            error!("Failed to generate certificates: {}", e);
        }

        info!("Creating Vault configuration...");
        if let Err(e) = self.create_vault_config() {
            error!("Failed to create Vault config: {}", e);
        }

        let db_password = Self::generate_secure_password(24);
        let drive_accesskey = Self::generate_secure_password(20);
        let drive_secret = Self::generate_secure_password(40);
        let cache_password = Self::generate_secure_password(24);

        info!("Configuring services through Vault...");

        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;

        let required_components = vec!["vault", "tables", "directory", "drive", "cache", "llm", "vector_db"];

        let vault_needs_setup = !self.stack_dir("conf/vault/init.json").exists();

        for component in required_components {
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

                let bin_path = pm.base_path.join("bin").join(component);
                let binary_name = pm
                    .components
                    .get(component)
                    .and_then(|cfg| cfg.binary_name.clone())
                    .unwrap_or_else(|| component.to_string());

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

                if component == "tables" {
                    info!("Starting PostgreSQL database...");
                    match pm.start("tables") {
                        Ok(_) => {
                            info!("PostgreSQL started successfully");

                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        }
                        Err(e) => {
                            warn!("Failed to start PostgreSQL: {}", e);
                        }
                    }

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
                    if let Err(e) = self.configure_services_in_directory(&db_password) {
                        error!("Failed to create Directory config files: {}", e);
                    }
                }

                if component == "directory" {
                    info!("Starting Directory (Zitadel) service...");
                    match pm.start("directory") {
                        Ok(_) => {
                            info!("Directory service started successfully");

                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                        Err(e) => {
                            warn!("Failed to start Directory service: {}", e);
                        }
                    }

                    info!("Waiting for Directory to be ready...");
                    if let Err(e) = self.setup_directory().await {
                        warn!("Directory additional setup had issues: {}", e);
                    }
                }

                if component == "vault" {
                    info!("Setting up Vault secrets service...");

                    let vault_bin = self.stack_dir("bin/vault/vault");
                    if !vault_bin.exists() {
                        error!("Vault binary not found at {}", vault_bin.display());
                        return Err(anyhow::anyhow!("Vault binary not found after installation"));
                    }
                    info!("Vault binary verified at {}", vault_bin.display());

                    let vault_log_path = self.stack_dir("logs/vault/vault.log");
                    if let Some(parent) = vault_log_path.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            error!("Failed to create vault logs directory: {}", e);
                        }
                    }

                    let vault_data_path = self.stack_dir("data/vault");
                    if let Err(e) = fs::create_dir_all(&vault_data_path) {
                        error!("Failed to create vault data directory: {}", e);
                    }

                    info!("Starting Vault server...");

                    let vault_bin_dir = self.stack_dir("bin/vault");
                    let vault_start_cmd = format!(
                        "cd {} && nohup ./vault server -config=../../conf/vault/config.hcl > ../../logs/vault/vault.log 2>&1 &",
                        vault_bin_dir.display()
                    );
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&vault_start_cmd)
                        .status();
                    std::thread::sleep(std::time::Duration::from_secs(2));

                    let check = std::process::Command::new("pgrep")
                        .args(["-f", "vault server"])
                        .output();
                    if let Ok(output) = &check {
                        let pids = String::from_utf8_lossy(&output.stdout);
                        if pids.trim().is_empty() {
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
                        } else {
                            info!("Vault server started");
                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        }
                    }

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

                        return Err(anyhow::anyhow!("Vault setup failed: {}. Check ./botserver-stack/logs/vault/vault.log for details.", e));
                    }

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
                    if let Err(e) = self.setup_caddy_proxy() {
                        error!("Failed to setup Caddy: {}", e);
                    }
                }

                if component == "dns" {
                    info!("Configuring CoreDNS for dynamic DNS...");
                    if let Err(e) = self.setup_coredns() {
                        error!("Failed to setup CoreDNS: {}", e);
                    }
                }
            }
        }
        info!("=== BOOTSTRAP COMPLETED SUCCESSFULLY ===");
        Ok(())
    }

    fn configure_services_in_directory(&self, db_password: &str) -> Result<()> {
        info!("Creating Zitadel configuration files...");

        let zitadel_config_path = self.stack_dir("conf/directory/zitadel.yaml");
        let steps_config_path = self.stack_dir("conf/directory/steps.yaml");

        let pat_path = if self.stack_path.is_absolute() {
            self.stack_dir("conf/directory/admin-pat.txt")
        } else {
            std::env::current_dir()?.join(self.stack_dir("conf/directory/admin-pat.txt"))
        };

        fs::create_dir_all(zitadel_config_path.parent().ok_or_else(|| anyhow::anyhow!("Invalid zitadel config path"))?)?;

        let zitadel_db_password = Self::generate_secure_password(24);

        let zitadel_config = format!(
            r#"Log:
  Level: info
  Formatter:
    Format: text

Port: 8300

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
            zitadel_db_password, db_password,
        );

        fs::write(&zitadel_config_path, zitadel_config)?;
        info!("Created zitadel.yaml configuration");

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
            Self::generate_secure_password(16),
        );

        fs::write(&steps_config_path, steps_config)?;
        info!("Created steps.yaml for first instance setup");

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

    fn setup_caddy_proxy(&self) -> Result<()> {
        let caddy_config = self.stack_dir("conf/proxy/Caddyfile");
        fs::create_dir_all(caddy_config.parent().ok_or_else(|| anyhow::anyhow!("Invalid caddy config path"))?)?;

        let config = format!(
            r"{{
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
",
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

    fn setup_coredns(&self) -> Result<()> {
        let dns_config = self.stack_dir("conf/dns/Corefile");
        fs::create_dir_all(dns_config.parent().ok_or_else(|| anyhow::anyhow!("Invalid dns config path"))?)?;

        let zone_file = self.stack_dir("conf/dns/botserver.local.zone");

        let corefile = r"botserver.local:53 {
    file /botserver-stack/conf/dns/botserver.local.zone
    reload 10s
    log
}

.:53 {
    forward . 8.8.8.8 8.8.4.4
    cache 30
    log
}
";

        fs::write(dns_config, corefile)?;

        let zone = r"$ORIGIN botserver.local.
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
";

        fs::write(zone_file, zone)?;
        info!("CoreDNS configured for dynamic DNS");
        Ok(())
    }

    async fn setup_directory(&self) -> Result<()> {
        let config_path = PathBuf::from("./config/directory_config.json");
        let pat_path = self.stack_dir("conf/directory/admin-pat.txt");

        tokio::fs::create_dir_all("./config").await?;

        info!("Waiting for Zitadel to be ready...");
        let mut attempts = 0;
        let max_attempts = 60;

        while attempts < max_attempts {
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

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

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

        let mut setup = DirectorySetup::new("http://localhost:8300".to_string(), config_path);

        if let Some(token) = admin_token {
            setup.set_admin_token(token);
        } else {
            info!("Directory setup skipped - no admin token available");
            info!("First instance setup created initial admin user via steps.yaml");
            return Ok(());
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let org_name = "default";
        match setup
            .create_organization(org_name, "Default Organization")
            .await
        {
            Ok(org_id) => {
                info!("Created default organization: {}", org_name);

                let user_password = Self::generate_secure_password(16);

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

                match setup.create_oauth_application(&org_id).await {
                    Ok((project_id, client_id, client_secret)) => {
                        info!("Created OAuth2 application in project: {}", project_id);

                        let admin_user = crate::package_manager::setup::DefaultUser {
                            id: "admin".to_string(),
                            username: "admin".to_string(),
                            email: "admin@localhost".to_string(),
                            password: "".to_string(),
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

    async fn setup_vault(
        &self,
        db_password: &str,
        drive_accesskey: &str,
        drive_secret: &str,
        cache_password: &str,
    ) -> Result<()> {
        let vault_conf_path = self.stack_dir("conf/vault");
        let vault_init_path = vault_conf_path.join("init.json");
        let env_file_path = PathBuf::from("./.env");

        info!("Waiting for Vault to be ready...");
        let mut attempts = 0;
        let max_attempts = 30;

        while attempts < max_attempts {
            let ps_check = std::process::Command::new("sh")
                .arg("-c")
                .arg("pgrep -f 'vault server' || echo 'NOT_RUNNING'")
                .output();

            if let Ok(ps_output) = ps_check {
                let ps_result = String::from_utf8_lossy(&ps_output.stdout);
                if ps_result.contains("NOT_RUNNING") {
                    warn!("Vault process is not running (attempt {})", attempts + 1);

                    let vault_log_path = self.stack_dir("logs/vault/vault.log");
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
                }

                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() && attempts % 5 == 0 {
                    debug!("Vault health check attempt {}: {}", attempts + 1, stderr);
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

            let vault_log_path = self.stack_dir("logs/vault/vault.log");
            if vault_log_path.exists() {
                if let Ok(log_content) = fs::read_to_string(&vault_log_path) {
                    let last_lines: Vec<&str> = log_content.lines().rev().take(20).collect();
                    error!("Vault log (last 20 lines):");
                    for line in last_lines.iter().rev() {
                        error!("  {}", line);
                    }
                }
            } else {
                error!(
                    "Vault log file does not exist at {}",
                    vault_log_path.display()
                );
            }
            return Err(anyhow::anyhow!(
                "Vault not ready after {} seconds. Check ./botserver-stack/logs/vault/vault.log for details.",
                max_attempts
            ));
        }

        let vault_addr = "http://localhost:8200";
        std::env::set_var("VAULT_ADDR", vault_addr);
        std::env::set_var("VAULT_SKIP_VERIFY", "true");

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
            let env_token = if env_file_path.exists() {
                if let Ok(env_content) = fs::read_to_string(&env_file_path) {
                    env_content
                        .lines()
                        .find(|line| line.starts_with("VAULT_TOKEN="))
                        .map(|line| line.trim_start_matches("VAULT_TOKEN=").to_string())
                } else {
                    None
                }
            } else {
                None
            };

            info!("Initializing Vault...");
            let vault_bin = self.vault_bin();

            let init_output = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} {} operator init -key-shares=1 -key-threshold=1 -format=json",
                    vault_addr, vault_bin
                ))
                .output()?;

            if !init_output.status.success() {
                let stderr = String::from_utf8_lossy(&init_output.stderr);
                if stderr.contains("already initialized") {
                    warn!("Vault already initialized but init.json not found");

                    if let Some(_token) = env_token {
                        info!("Found VAULT_TOKEN in .env, checking if Vault is unsealed...");

                        let status_check = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(format!(
                                "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} {} status -format=json 2>/dev/null",
                                vault_addr, vault_bin
                            ))
                            .output();

                        if let Ok(status_output) = status_check {
                            let status_str = String::from_utf8_lossy(&status_output.stdout);
                            if let Ok(status) =
                                serde_json::from_str::<serde_json::Value>(&status_str)
                            {
                                let sealed = status["sealed"].as_bool().unwrap_or(true);
                                if !sealed {
                                    warn!("Vault is already unsealed - continuing with existing token");
                                    warn!("NOTE: Unseal key is lost - Vault will need manual unseal after restart");
                                    return Ok(());
                                }
                            }
                        }

                        error!("Vault is sealed and unseal key is lost (init.json missing)");
                        error!("Options:");
                        error!("  1. If you have a backup of init.json, restore it to ./botserver-stack/conf/vault/init.json");
                        error!(
                            "  2. To start fresh, delete ./botserver-stack/data/vault/ and restart"
                        );
                        return Err(anyhow::anyhow!(
                            "Vault is sealed but unseal key is lost. See error messages above for recovery options."
                        ));
                    }

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

        info!("Unsealing Vault...");
        let vault_bin = self.vault_bin();

        let unseal_output = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} {} operator unseal {}",
                vault_addr, vault_bin, unseal_key
            ))
            .output()?;

        if !unseal_output.status.success() {
            let stderr = String::from_utf8_lossy(&unseal_output.stderr);
            if !stderr.contains("already unsealed") {
                warn!("Vault unseal warning: {}", stderr);
            }
        }

        std::env::set_var("VAULT_TOKEN", &root_token);

        info!("Writing .env file with Vault configuration...");
        let env_content = format!(
            r"# BotServer Environment Configuration
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
",
            vault_addr, root_token
        );
        fs::write(&env_file_path, &env_content)?;
        info!("  * Created .env file with Vault configuration");

        info!("Re-initializing SecretsManager with Vault credentials...");
        match init_secrets_manager().await {
            Ok(_) => info!("  * SecretsManager now connected to Vault"),
            Err(e) => warn!("SecretsManager re-init warning: {}", e),
        }

        info!("Enabling KV secrets engine...");
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} {} secrets enable -path=secret kv-v2 2>&1 || true",
                vault_addr, root_token, vault_bin
            ))
            .output();

        info!("Storing secrets in Vault (only if not existing)...");

        let vault_bin_clone = vault_bin.clone();
        let secret_exists = |path: &str| -> bool {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} {} kv get {} 2>/dev/null",
                    vault_addr, root_token, vault_bin_clone, path
                ))
                .output();
            output.map(|o| o.status.success()).unwrap_or(false)
        };

        if secret_exists("secret/gbo/tables") {
            info!("  Database credentials already exist - preserving");
        } else {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} {} kv put secret/gbo/tables host=localhost port=5432 database=botserver username=gbuser password='{}'",
                    vault_addr, root_token, vault_bin, db_password
                ))
                .output()?;
            info!("  Stored database credentials");
        }

        if secret_exists("secret/gbo/drive") {
            info!("  Drive credentials already exist - preserving");
        } else {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} {} kv put secret/gbo/drive accesskey='{}' secret='{}'",
                    vault_addr, root_token, vault_bin, drive_accesskey, drive_secret
                ))
                .output()?;
            info!("  Stored drive credentials");
        }

        if secret_exists("secret/gbo/cache") {
            info!("  Cache credentials already exist - preserving");
        } else {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} {} kv put secret/gbo/cache password='{}'",
                    vault_addr, root_token, vault_bin, cache_password
                ))
                .output()?;
            info!("  Stored cache credentials");
        }

        if secret_exists("secret/gbo/directory") {
            info!("  Directory credentials already exist - preserving");
        } else {
            use rand::Rng;
            let masterkey: String = rand::rng()
                .sample_iter(&rand::distr::Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} {} kv put secret/gbo/directory url=https://localhost:8300 project_id= client_id= client_secret= masterkey={}",
                    vault_addr, root_token, vault_bin, masterkey
                ))
                .output()?;
            info!("  Created directory placeholder with masterkey");
        }

        if secret_exists("secret/gbo/llm") {
            info!("  LLM credentials already exist - preserving");
        } else {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} {} kv put secret/gbo/llm openai_key= anthropic_key= groq_key=",
                    vault_addr, root_token, vault_bin
                ))
                .output()?;
            info!("  Created LLM placeholder");
        }

        if secret_exists("secret/gbo/email") {
            info!("  Email credentials already exist - preserving");
        } else {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} {} kv put secret/gbo/email username= password=",
                    vault_addr, root_token, vault_bin
                ))
                .output()?;
            info!("  Created email placeholder");
        }

        if secret_exists("secret/gbo/encryption") {
            info!("  Encryption key already exists - preserving (CRITICAL)");
        } else {
            let encryption_key = Self::generate_secure_password(32);
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} {} kv put secret/gbo/encryption master_key='{}'",
                    vault_addr, root_token, vault_bin, encryption_key
                ))
                .output()?;
            info!("  Generated and stored encryption key");
        }

        info!("Vault setup complete!");
        info!("   Vault UI: {}/ui", vault_addr);
        info!("   Root token saved to: {}", vault_init_path.display());

        Ok(())
    }

    pub async fn setup_email(&self) -> Result<()> {
        let config_path = PathBuf::from("./config/email_config.json");
        let directory_config_path = PathBuf::from("./config/directory_config.json");

        let mut setup = EmailSetup::new(
            crate::core::urls::InternalUrls::DIRECTORY_BASE.to_string(),
            config_path,
        );

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

        let (access_key, secret_key) =
            if config.drive.access_key.is_empty() || config.drive.secret_key.is_empty() {
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

    pub fn sync_templates_to_database(&self) -> Result<()> {
        let mut conn = establish_pg_connection()?;
        Self::create_bots_from_templates(&mut conn)?;
        Ok(())
    }

    pub async fn upload_templates_to_drive(&self, _config: &AppConfig) -> Result<()> {
        let possible_paths = [
            "../bottemplates",
            "bottemplates",
            "botserver-templates",
            "templates",
        ];

        let templates_dir = possible_paths.iter().map(Path::new).find(|p| p.exists());

        let templates_dir = match templates_dir {
            Some(dir) => {
                info!("Using templates from: {}", dir.display());
                dir
            }
            None => {
                info!("No templates directory found, skipping template upload");
                return Ok(());
            }
        };
        let client = Self::get_drive_client(_config).await;
        let mut read_dir = tokio::fs::read_dir(templates_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .ends_with(".gbai")
            {
                let bot_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                let bucket = bot_name.trim_start_matches('/').to_string();
                if client.head_bucket().bucket(&bucket).send().await.is_err() {
                    match client.create_bucket().bucket(&bucket).send().await {
                        Ok(_) => {
                            Self::upload_directory_recursive(&client, &path, &bucket, "/").await?;
                        }
                        Err(e) => {
                            warn!("S3/MinIO not available, skipping bucket {}: {}", bucket, e);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn create_bots_from_templates(conn: &mut diesel::PgConnection) -> Result<()> {
        use crate::shared::models::schema::bots;
        use diesel::prelude::*;

        let possible_paths = [
            "../bottemplates",
            "bottemplates",
            "botserver-templates",
            "templates",
        ];

        let templates_dir = possible_paths
            .iter()
            .map(PathBuf::from)
            .find(|p| p.exists());

        let templates_dir = match templates_dir {
            Some(dir) => {
                info!("Loading templates from: {}", dir.display());
                dir
            }
            None => {
                warn!(
                    "Templates directory does not exist (checked: {:?})",
                    possible_paths
                );
                return Ok(());
            }
        };

        let default_bot: Option<(uuid::Uuid, String)> = bots::table
            .filter(bots::is_active.eq(true))
            .select((bots::id, bots::name))
            .first(conn)
            .optional()?;

        let Some((default_bot_id, default_bot_name)) = default_bot else {
            error!("No active bot found in database - cannot sync template configs");
            return Ok(());
        };

        info!(
            "Syncing template configs to bot '{}' ({})",
            default_bot_name, default_bot_id
        );

        let default_template = templates_dir.join("default.gbai");
        info!(
            "Looking for default template at: {}",
            default_template.display()
        );
        if default_template.exists() {
            let config_path = default_template.join("default.gbot").join("config.csv");

            if config_path.exists() {
                match std::fs::read_to_string(&config_path) {
                    Ok(csv_content) => {
                        debug!("Syncing config.csv from {}", config_path.display());
                        if let Err(e) =
                            Self::sync_config_csv_to_db(conn, &default_bot_id, &csv_content)
                        {
                            error!("Failed to sync config.csv: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Could not read config.csv: {}", e);
                    }
                }
            } else {
                debug!("No config.csv found at {}", config_path.display());
            }
        } else {
            debug!("default.gbai template not found");
        }

        Ok(())
    }

    fn sync_config_csv_to_db(
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
                let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
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
                    Self::upload_directory_recursive(client, &path, bucket, &key).await?;
                }
            }
            Ok(())
        })
    }
    pub fn apply_migrations(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

        if let Err(e) = conn.run_pending_migrations(MIGRATIONS) {
            error!("Failed to apply migrations: {}", e);
            return Err(anyhow::anyhow!("Migration error: {}", e));
        }

        Ok(())
    }

    fn create_vault_config(&self) -> Result<()> {
        let vault_conf_dir = self.stack_dir("conf/vault");
        let config_path = vault_conf_dir.join("config.hcl");

        fs::create_dir_all(&vault_conf_dir)?;

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

        fs::create_dir_all(self.stack_dir("data/vault"))?;

        info!(
            "Created Vault config with mTLS at {}",
            config_path.display()
        );
        Ok(())
    }

    fn generate_certificates(&self) -> Result<()> {
        let cert_dir = self.stack_dir("conf/system/certificates");

        fs::create_dir_all(&cert_dir)?;
        fs::create_dir_all(cert_dir.join("ca"))?;

        let ca_cert_path = cert_dir.join("ca/ca.crt");
        let ca_key_path = cert_dir.join("ca/ca.key");

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

            let key_pem = fs::read_to_string(&ca_key_path)?;
            KeyPair::from_pem(&key_pem)?
        } else {
            info!("Generating new CA certificate");
            let key_pair = KeyPair::generate()?;
            let cert = ca_params.self_signed(&key_pair)?;

            fs::write(&ca_cert_path, cert.pem())?;
            fs::write(&ca_key_path, key_pair.serialize_pem())?;

            key_pair
        };

        let ca_issuer = Issuer::from_params(&ca_params, &ca_key_pair);

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

            if cert_path.exists() && key_path.exists() {
                continue;
            }

            info!("Generating certificate for {}", service);

            let mut params = CertificateParams::default();
            params.not_before = time::OffsetDateTime::now_utc();
            params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(365);

            let mut dn = DistinguishedName::new();
            dn.push(DnType::CountryName, "BR");
            dn.push(DnType::OrganizationName, "BotServer");
            dn.push(DnType::CommonName, format!("{service}.botserver.local"));
            params.distinguished_name = dn;

            for san in sans {
                params
                    .subject_alt_names
                    .push(rcgen::SanType::DnsName(san.to_string().try_into()?));
            }

            let key_pair = KeyPair::generate()?;
            let cert = params.signed_by(&key_pair, &ca_issuer)?;

            fs::write(cert_path, cert.pem())?;
            fs::write(key_path, key_pair.serialize_pem())?;

            fs::copy(&ca_cert_path, service_dir.join("ca.crt"))?;
        }

        info!("TLS certificates generated successfully");
        Ok(())
    }
}
