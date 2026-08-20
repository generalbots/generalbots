use crate::package_manager::component::ComponentConfig;
use crate::package_manager::os::detect_os;
use crate::package_manager::{InstallMode, OsType};
use anyhow::{Context, Result};
use log::{info, trace, warn};
use std::io::Write;


fn safe_sh_command(script: &str) -> Option<std::process::Output> {
    let abs = crate::os_abstraction::get_abstraction();
    let (shell, flag) = abs.shell_command();
    #[cfg(target_os = "windows")]
    let script = &format!("\"{script}\"");
    SafeCommand::new(shell)
        .and_then(|c| c.arg(flag))
        .and_then(|c| c.raw_shell_script_arg(script))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

fn safe_pgrep(args: &[&str]) -> Option<std::process::Output> {
    let abs = crate::os_abstraction::get_abstraction();
    let cmd_name = abs.process_grep_command();
    SafeCommand::new(cmd_name)
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}


#[derive(Debug)]
pub struct PackageManager {
    pub mode: InstallMode,
    pub os_type: OsType,
    pub base_path: PathBuf,
    pub tenant: String,
    pub components: HashMap<String, ComponentConfig>,
}

impl PackageManager {
    pub fn new(mode: InstallMode, tenant: Option<String>) -> Result<Self> {
        let os_type = detect_os();
        let base_path = if mode == InstallMode::Container {
            PathBuf::from("/opt/gbo")
        } else if let Ok(custom_path) = std::env::var("BOTSERVER_STACK_PATH") {
            PathBuf::from(custom_path)
        } else {
            std::env::current_dir()?.join("botserver-stack")
        };
        let tenant = tenant.unwrap_or_else(|| "default".to_string());

        let mut pm = Self {
            mode,
            os_type,
            base_path,
            tenant,
            components: HashMap::new(),
        };
        pm.register_components();
        Ok(pm)
    }

    pub fn with_base_path(
        mode: InstallMode,
        tenant: Option<String>,
        base_path: PathBuf,
    ) -> Result<Self> {
        let os_type = detect_os();
        let tenant = tenant.unwrap_or_else(|| "default".to_string());

        let mut pm = Self {
            mode,
            os_type,
            base_path,
            tenant,
            components: HashMap::new(),
        };
        pm.register_components();
        Ok(pm)
    }

    fn register_components(&mut self) {
        self.components = botcorepkg::installer::PackageManager::all_components();
    }

    pub fn start(&self, component: &str) -> Result<std::process::Child> {
        if let Some(component) = self.components.get(component) {
            let bin_path = self.base_path.join("bin").join(&component.name);
            let data_path = self.base_path.join("data").join(&component.name);
            let conf_path = self.base_path.join("conf");
            let logs_path = self.base_path.join("logs").join(&component.name);

            let check_cmd = component
                .effective_check_cmd()
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy());

            let check_output = safe_sh_command(&check_cmd)
                .map(|o| o.status.success())
                .unwrap_or(false);

            if check_output {
                info!(
                    "Component {} is already running, skipping start",
                    component.name
                );
                return SafeCommand::noop_child()
                    .map_err(|e| anyhow::anyhow!("Failed to create noop process: {}", e));
            }

            // Generate qdrant config.yaml if missing
            if component.name == "vector_db" {
                let qdrant_conf = conf_path.join("vector_db/config.yaml");
                if !qdrant_conf.exists() {
                    let storage = data_path.join("storage");
                    let snapshots = data_path.join("snapshots");
                    let _ = std::fs::create_dir_all(&storage);
                    let _ = std::fs::create_dir_all(&snapshots);
                    let yaml = format!(
                        "storage:\n  storage_path: {}\n  snapshots_path: {}\n\nservice:\n  host: 0.0.0.0\n  http_port: 6333\n  grpc_port: 6334\n  enable_tls: false\n\nlog_level: INFO\n",
                        storage.display(),
                        snapshots.display()
                    );
                    if let Err(e) = std::fs::write(&qdrant_conf, yaml) {
                        warn!("Failed to write qdrant config: {}", e);
                    }
 else {
                        info!("Generated qdrant config at {}", qdrant_conf.display());
                    }
                }
            }

            let rendered_cmd = component
                .effective_exec_cmd()
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy());

        if let Err(e) = std::fs::create_dir_all(&logs_path) {
            warn!("Failed to create log directory {}: {}", logs_path.display(), e);
        }

            trace!(
                "Starting component {} with command: {}",
                component.name,
                rendered_cmd
            );
            trace!(
                "Working directory: {}, logs_path: {}",
                bin_path.display(),
                logs_path.display()
            );

            let vault_credentials = Self::fetch_vault_credentials();

            let mut evaluated_envs = HashMap::new();
            for (k, v) in &component.env_vars {
                if let Some(var_name) = v.strip_prefix('$') {
                    let value = vault_credentials
                        .get(var_name)
                        .cloned()
                        .or_else(|| std::env::var(var_name).ok())
                        .unwrap_or_default();
                    evaluated_envs.insert(k.clone(), value);
                } else {
                    evaluated_envs.insert(k.clone(), v.clone());
                }
            }

            trace!(
                "About to spawn shell command for {}: {}",
                component.name,
                rendered_cmd
            );
            trace!("Working dir: {}", bin_path.display());
            let abs = crate::os_abstraction::get_abstraction();
            let (shell, flag) = abs.shell_command();
            #[cfg(target_os = "windows")]
            let wrapped_cmd = format!("\"{rendered_cmd}\"");
            #[cfg(target_os = "windows")]
            let script_arg: &str = &wrapped_cmd;
            #[cfg(not(target_os = "windows"))]
            let script_arg: &str = &rendered_cmd;
            let child = SafeCommand::new(shell)
                .and_then(|c| c.arg(flag))
                .and_then(|c| c.raw_shell_script_arg(script_arg))
                .and_then(|c| c.working_dir(&bin_path))
                .and_then(|cmd| cmd.spawn_with_envs(&evaluated_envs));

            trace!("Spawn result for {}: {:?}", component.name, child.is_ok());
            std::thread::sleep(std::time::Duration::from_secs(2));

            trace!(
                "Checking if {} process exists after 2s sleep...",
                component.name
            );
            let check_proc = safe_pgrep(&["-f", &component.name]);
            if let Some(output) = check_proc {
                let pids = String::from_utf8_lossy(&output.stdout);
                trace!("pgrep '{}' result: '{}'", component.name, pids.trim());
            }

            match child {
                Ok(c) => {
                    trace!("Component {} spawned successfully", component.name);

                    if component.name == "vault" && self.mode == InstallMode::Local {
                        if let Some(addr) = evaluated_envs.get("VAULT_ADDR") {
                            if !addr.is_empty() {
                                std::env::set_var("VAULT_ADDR", addr);
                            }
                        }
                        if let Err(e) = self.initialize_vault_local() {
                            warn!("Failed to initialize Vault: {}", e);
                        }
                    }

                    Ok(c)
                }
                Err(e) => {
                    log::error!("Spawn failed for {}: {}", component.name, e);
                    let err_msg = e.to_string();
                    if err_msg.contains("already running")
                        || err_msg.contains("be running")
                        || component.name == "tables"
                    {
                        trace!("Component {} may already be running", component.name);
                        if component.name == "vault" && self.mode == InstallMode::Local {
                            let _ = self.ensure_env_file_exists();
                        }
                        
                        Err(anyhow::anyhow!("Already running"))
                    } else {
                        Err(anyhow::anyhow!("Failed to start component {}: {}", component.name, e))
                    }
                }
            }
        } else {
            Err(anyhow::anyhow!("Component not found: {}", component))
        }
    }

    fn fetch_vault_credentials() -> HashMap<String, String> {
        let mut credentials = HashMap::new();

        dotenvy::dotenv().ok();

        let base_path = std::env::var("BOTSERVER_STACK_PATH")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join("botserver-stack")
            });

        let vault_addr = std::env::var("VAULT_ADDR").unwrap_or_default();
        let vault_token = std::env::var("VAULT_TOKEN").unwrap_or_default();

        if vault_token.is_empty() || vault_addr.is_empty() {
            info!("VAULT_ADDR or VAULT_TOKEN not set yet, using bootstrap defaults");
            return credentials;
        }

        let client_cert = base_path.join("conf/system/certificates/botserver/client.crt");
        let client_key = base_path.join("conf/system/certificates/botserver/client.key");
        let vault_check = SafeCommand::new("curl")
            .and_then(|c| {
                c.args(&[
                    "-sfk",
                    "--cert",
                    &client_cert.to_string_lossy(),
                    "--key",
                    &client_key.to_string_lossy(),
                    &format!("{}/v1/sys/health", vault_addr),
                ])
            })
            .and_then(|c| c.execute())
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !vault_check {
            trace!(
                "Vault not reachable at {}, skipping credential fetch",
                vault_addr
            );
            return credentials;
        }

        #[cfg(target_os = "windows")]
        let vault_bin = base_path.join("bin/vault/vault.exe");
        #[cfg(not(target_os = "windows"))]
        let vault_bin = base_path.join("bin/vault/vault");
        let ca_cert_path = std::env::var("VAULT_CACERT").unwrap_or_else(|_| {
            base_path
                .join("conf/system/certificates/ca/ca.crt")
                .to_string_lossy()
                .to_string()
        });

        let services = [
            ("drive", "secret/gbo/drive"),
            ("cache", "secret/gbo/cache"),
            ("tables", "secret/gbo/tables"),
            ("vectordb", "secret/gbo/vectordb"),
            ("directory", "secret/gbo/directory"),
            ("llm", "secret/gbo/llm"),
            ("meet", "secret/gbo/meet"),
            ("alm", "secret/gbo/alm"),
            ("encryption", "secret/gbo/encryption"),
        ];

        for (service_name, vault_path) in &services {
            let result = SafeCommand::new(vault_bin.to_str().unwrap_or("vault"))
                .and_then(|c| {
                    let mut command = c
                        .env("VAULT_ADDR", &vault_addr)?
                        .env("VAULT_TOKEN", &vault_token)?;
                    if std::path::Path::new(&ca_cert_path).is_file() {
                        command = command.env("VAULT_CACERT", &ca_cert_path)?;
                    }
                    Ok(command)
                })
                .and_then(|c| {
                    c.args(&["kv", "get", "-format=json", "-tls-skip-verify", vault_path])
                })
                .and_then(|c| c.execute());

            if let Ok(output) = result {
                if output.status.success() {
                    let json_str = String::from_utf8_lossy(&output.stdout);
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        if let Some(data) = json.get("data").and_then(|d| d.get("data")) {
                            if let Some(obj) = data.as_object() {
                                let prefix = service_name.to_uppercase();
                                for (key, value) in obj {
                                    if let Some(v) = value.as_str() {
                                        let env_key = format!("{}_{}", prefix, key.to_uppercase());
                                        credentials.insert(env_key, v.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        trace!("Fetched {} credentials from Vault", credentials.len());
        credentials
    }

    /// Initialize Vault locally (non-LXC mode) and create .env file
    ///
    /// This function:
    /// 1. Checks if Vault is already initialized (via health endpoint or data dir)
    /// 2. If initialized but sealed, unseals with existing keys from vault-unseal-keys
    /// 3. If not initialized, runs `vault operator init` to get root token and unseal keys
    /// 4. Creates .env file with VAULT_ADDR and VAULT_TOKEN
    /// 5. Creates vault-unseal-keys file with proper permissions
    /// 6. Unseals Vault with 3 keys
    fn initialize_vault_local(&self) -> Result<()> {
        use std::io::Write;

        info!("Initializing Vault locally (non-LXC mode)...");

        let bin_path = self.base_path.join("bin/vault");
        let conf_path = self.base_path.join("conf");
        #[cfg(target_os = "windows")]
        let vault_bin = bin_path.join("vault.exe");
        #[cfg(not(target_os = "windows"))]
        let vault_bin = bin_path.join("vault");
        let vault_data = self.base_path.join("data/vault");

        // Check if Vault data directory exists (real indicator of initialized state)
        let vault_data_exists = vault_data.exists();

        if !vault_data_exists {
            info!("Vault data directory not found, will initialize fresh");
        } else {
            info!("Vault data directory found, checking health...");
        }

        // Wait for Vault to be ready
        info!("Waiting for Vault to start...");
        std::thread::sleep(std::time::Duration::from_secs(3));

        let vault_addr = std::env::var("VAULT_ADDR")
            .ok()
            .filter(|a| !a.is_empty())
            .ok_or_else(|| anyhow::anyhow!("VAULT_ADDR must be set in .env or environment"))?;
        let ca_cert = conf_path.join("system/certificates/ca/ca.crt");

        // Only attempt recovery if data directory exists
        if vault_data_exists {
            // Check if Vault is already initialized via health endpoint
            let health_cmd = format!(
                "curl -f -s --connect-timeout 2 -k {}/v1/sys/health",
                vault_addr
            );
            let health_output = safe_sh_command(&health_cmd);

            let already_initialized = if let Some(ref output) = health_output {
                if output.status.success() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(
                        &String::from_utf8_lossy(&output.stdout),
                    ) {
                        json.get("initialized")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                    } else {
                        false
                    }
                } else {
                    // Health endpoint returns 503 when sealed but initialized
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.contains("\"initialized\":true")
                        || stderr.contains("\"initialized\":true")
                }
            } else {
                false
            };

            if already_initialized {
                info!("Vault already initialized (detected via health/data), skipping init");
                return self.recover_existing_vault();
            }
        }

        // Initialize Vault: wait for it to be ready, then run init
        let init_cmd = format!(
            "{} operator init -tls-skip-verify -key-shares=5 -key-threshold=3 -format=json -address={}",
            vault_bin.display(),
            vault_addr
        );

        // Wait for Vault to be ready (up to 120s, polling every 2s). Windows
        // first-launch of the ~112MB binary can exceed 30s under AV scanning.
        info!("Waiting for Vault to be ready on {}...", vault_addr);
        let vault_ready = 'wait: {
            for i in 1..=60 {
                // Use curl without -f (health returns 503 when sealed, which is fine)
                let check_cmd = format!(
                    "curl -sk --connect-timeout 2 --max-time 4 -o /dev/null -w '%{{http_code}}' {}/v1/sys/health",
                    vault_addr
                );
                if let Some(output) = safe_sh_command(&check_cmd) {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // 501 = uninitialized (still ready for `operator init`); 503 = sealed
                    if output.status.success()
                        || stdout.contains("501")
                        || stdout.contains("503")
                        || stdout.contains("200")
                    {
                        break 'wait true;
                    }
                }
                info!("Waiting for Vault... (attempt {}/15)", i);
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            false
        };

        if !vault_ready {
            return Err(anyhow::anyhow!("Vault did not become ready after 30s"));
        }

        // Check if already initialized
        let check_json = format!("curl -sk --connect-timeout 2 --max-time 4 {}/v1/sys/health", vault_addr);
        if let Some(output) = safe_sh_command(&check_json) {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("\"initialized\":true") {
                info!("Vault already initialized, recovering existing data");
                return self.recover_existing_vault();
            }
        }

        info!("Running vault operator init...");
        let output = safe_sh_command(&init_cmd)
            .ok_or_else(|| anyhow::anyhow!("Failed to execute vault init command"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("already initialized") {
                info!("Vault already initialized, recovering existing data");
                return self.recover_existing_vault();
            }
            return Err(anyhow::anyhow!("Failed to initialize Vault: {}", stderr));
        }

        let init_output = String::from_utf8_lossy(&output.stdout);
        let init_json_val: serde_json::Value =
            serde_json::from_str(&init_output).context("Failed to parse Vault init output")?;

        let root_token = init_json_val["root_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No root_token in vault init output"))?
            .to_string();
        let unseal_keys: Vec<String> = init_json_val["unseal_keys_b64"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No unseal_keys_b64 in vault init output"))?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        // Set VAULT_TOKEN early so it's available for all subsequent operations
        std::env::set_var("VAULT_TOKEN", &root_token);

        // Save init.json
        let init_json = self.base_path.join("conf/vault/init.json");
        std::fs::create_dir_all(
            init_json.parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid init.json path: no parent directory"))?
        )?;
        std::fs::write(&init_json, serde_json::to_string_pretty(&init_json_val)?)?;
        info!("Created {}", init_json.display());

        // Create .env file with Vault credentials
        let env_file = std::path::PathBuf::from(".env");
        let env_content = format!(
            r#"
# Vault Configuration (auto-generated)
VAULT_ADDR={}
VAULT_TOKEN={}
VAULT_CACERT={}
"#,
            vault_addr,
            root_token,
            ca_cert.display()
        );

        if env_file.exists() {
            let existing = std::fs::read_to_string(&env_file)?;
            if existing.contains("VAULT_ADDR=") {
                warn!(".env already contains VAULT_ADDR, not overwriting");
            } else {
                let mut file = std::fs::OpenOptions::new().append(true).open(&env_file)?;
                file.write_all(env_content.as_bytes())?;
                info!("Appended Vault config to .env");
            }
        } else {
            std::fs::write(&env_file, env_content.trim_start())?;
            info!("Created .env with Vault config");
        }

        // Create vault-unseal-keys file in botserver directory (next to .env)
        let unseal_keys_file = self.base_path.join("vault-unseal-keys");
        let keys_content: String = unseal_keys
            .iter()
            .enumerate()
            .map(|(i, key)| {
                format!("VAULT_UNSEAL_KEY_{}={key}\n", i + 1)
            })
            .collect();

        std::fs::write(&unseal_keys_file, keys_content)?;

        {
            let abs = crate::os_abstraction::get_abstraction();
            let _ = abs.set_readonly_owner(&unseal_keys_file)
                .map_err(|e| warn!("Failed to set permissions on {}: {}", unseal_keys_file.display(), e));
        }
        info!("Created {} (chmod 600)", unseal_keys_file.display());

        // Unseal Vault (need 3 keys)
        self.unseal_vault(&vault_bin, &vault_addr)?;

        info!("Vault initialized and unsealed successfully");
        info!("✓ Created .env with VAULT_ADDR, VAULT_TOKEN");
        info!("✓ Created /opt/gbo/secrets/vault-unseal-keys (chmod 600)");

        // Enable KV2 secrets engine at 'secret/' path
        info!("Enabling KV2 secrets engine at 'secret/'...");
        let enable_result = SafeCommand::new(vault_bin.to_str().unwrap_or("vault"))
            .and_then(|c| c.env("VAULT_ADDR", vault_addr.as_str()))
            .and_then(|c| c.env("VAULT_TOKEN", root_token.as_str()))
            .and_then(|c| c.env("VAULT_CACERT", ca_cert.to_str().unwrap_or("")))
            .and_then(|c| c.args(&["secrets", "enable", "-path=secret", "kv-v2"]))
            .and_then(|c| c.execute());
        match enable_result {
            Ok(output) => {
                if output.status.success() {
                    info!("KV2 secrets engine enabled at 'secret/'");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("path is already in use") {
                        info!("KV2 secrets engine already enabled");
                    } else {
                        warn!("Failed to enable KV2 secrets engine: {}", stderr);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to execute KV2 enable command: {e}");
            }
        }

        // Write default credentials to Vault for all components
        self.seed_vault_defaults(&vault_addr, &root_token, &ca_cert, &vault_bin)?;

        Ok(())
    }

    /// Check if Vault already has seeded credentials (to avoid overwriting on recovery)
    fn vault_seeds_exist(
        &self,
        vault_addr: &str,
        root_token: &str,
        ca_cert: &std::path::Path,
        vault_bin: &std::path::Path,
    ) -> Result<bool> {
        let args = vec![
            "kv".to_string(),
            "get".to_string(),
            "-tls-skip-verify".to_string(),
            format!("-address={}", vault_addr),
            "-field=accesskey".to_string(),
            "secret/gbo/drive".to_string(),
        ];

        let result = SafeCommand::new(vault_bin.to_str().unwrap_or("vault"))
            .and_then(|c| {
                let mut cmd = c;
                for arg in &args {
                    cmd = cmd.trusted_arg(arg)?;
                }
                Ok(cmd)
            })
            .and_then(|c| {
                let mut command = c
                    .env("VAULT_ADDR", vault_addr)?
                    .env("VAULT_TOKEN", root_token)?;
                if ca_cert.is_file() {
                    command = command.env("VAULT_CACERT", ca_cert.to_str().unwrap_or(""))?;
                }
                Ok(command)
            })
            .and_then(|c| c.execute());

        match result {
            Ok(output) => {
                if output.status.success() {
                    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    Ok(!value.is_empty())
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// Seed default credentials into Vault KV2 after initialization
    fn seed_vault_defaults(
        &self,
        vault_addr: &str,
        root_token: &str,
        ca_cert: &std::path::Path,
        vault_bin: &std::path::Path,
    ) -> Result<()> {
        info!("Seeding default credentials into Vault...");

        let drive_user = super::generate_random_string(16);
        let drive_pass = super::generate_random_string(32);
        let cache_pass = super::generate_random_string(32);
        let db_pass = super::generate_random_string(32);
        let master_key = super::generate_random_string(64);
        let meet_app_id = super::generate_random_string(24);
        let meet_app_secret = super::generate_random_string(48);
        let alm_token = super::generate_random_string(40);

        info!(
            "Generated strong random credentials for: drive, cache, tables, encryption, meet, alm"
        );

        let defaults: Vec<(&str, Vec<(String, String)>)> = vec![
            (
                "secret/gbo/drive",
                vec![
                    ("accesskey".to_string(), drive_user),
                    ("secret".to_string(), drive_pass),
                    ("host".to_string(), "127.0.0.1".to_string()),
                    ("port".to_string(), "9100".to_string()),
                    ("bucket".to_string(), "default.gborg".to_string()),
                    ("url".to_string(), "".to_string()),
                ],
            ),
            (
                "secret/gbo/cache",
                vec![
                    ("password".to_string(), cache_pass),
                    ("host".to_string(), "localhost".to_string()),
                    ("port".to_string(), "6379".to_string()),
                    ("url".to_string(), "redis://localhost:6379".to_string()),
                ],
            ),
            (
                "secret/gbo/tables",
                vec![
                    ("password".to_string(), db_pass),
                    ("host".to_string(), "localhost".to_string()),
                    ("port".to_string(), "5432".to_string()),
                    ("database".to_string(), "botserver".to_string()),
                    ("username".to_string(), "gbuser".to_string()),
                    ("url".to_string(), "postgres://localhost:5432".to_string()),
                ],
            ),
            (
                "secret/gbo/directory",
                vec![
                    ("url".to_string(), "".to_string()),
                    ("host".to_string(), "localhost".to_string()),
                    ("port".to_string(), "8300".to_string()),
                    ("project_id".to_string(), "none".to_string()),
                    ("client_id".to_string(), "none".to_string()),
                    ("client_secret".to_string(), "none".to_string()),
                ],
            ),
            (
                "secret/gbo/email",
                vec![
                    ("smtp_host".to_string(), "none".to_string()),
                    ("smtp_port".to_string(), "587".to_string()),
                    ("smtp_user".to_string(), "none".to_string()),
                    ("smtp_password".to_string(), "none".to_string()),
                    ("smtp_from".to_string(), "none".to_string()),
                ],
            ),
        (
            "secret/gbo/llm",
            vec![
                ("url".to_string(), "".to_string()),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "8081".to_string()),
                ("model".to_string(), "gpt-4".to_string()),
                ("openai_key".to_string(), "none".to_string()),
                ("anthropic_key".to_string(), "none".to_string()),
                ("ollama_url".to_string(), "".to_string()),
 ("embedding_url".to_string(), "http://localhost:8082/v1/embeddings".to_string()),
 ("embedding_model".to_string(), "bge-small-en-v1.5-f32.gguf".to_string()),
 ("embedding_port".to_string(), "8082".to_string()),
 ("embedding_dimensions".to_string(), "384".to_string()),
            ],
        ),
            (
                "secret/gbo/encryption",
                vec![("master_key".to_string(), master_key)],
            ),
            (
                "secret/gbo/meet",
                vec![
                    ("url".to_string(), "".to_string()),
                    ("host".to_string(), "localhost".to_string()),
                    ("port".to_string(), "7880".to_string()),
                    ("app_id".to_string(), meet_app_id),
                    ("app_secret".to_string(), meet_app_secret),
                ],
            ),
            (
                "secret/gbo/vectordb",
                vec![
                    ("url".to_string(), "http://127.0.0.1:6333".to_string()),
                    ("host".to_string(), "127.0.0.1".to_string()),
                    ("port".to_string(), "6333".to_string()),
                    ("grpc_port".to_string(), "6334".to_string()),
                    ("api_key".to_string(), "none".to_string()),
                ],
            ),
            (
                "secret/gbo/alm",
                vec![
                    ("url".to_string(), "".to_string()),
                    ("host".to_string(), "localhost".to_string()),
                    ("port".to_string(), "3000".to_string()),
                    ("token".to_string(), alm_token),
                    ("default_org".to_string(), "none".to_string()),
                ],
            ),
        ];

        for (path, kv_pairs) in &defaults {
            let mut args = vec![
                "kv".to_string(),
                "put".to_string(),
                "-tls-skip-verify".to_string(),
                format!("-address={}", vault_addr),
                path.to_string(),
            ];
            for (k, v) in kv_pairs.iter() {
                args.push(format!("{}={}", k, v));
            }

            let result = SafeCommand::new(vault_bin.to_str().unwrap_or("vault"))
                .and_then(|c| {
                    let mut cmd = c;
                    for arg in &args {
                        cmd = cmd.trusted_arg(arg)?;
                    }
                    Ok(cmd)
                })
                .and_then(|c| {
                    let mut command = c
                        .env("VAULT_ADDR", vault_addr)?
                        .env("VAULT_TOKEN", root_token)?;
                    if ca_cert.is_file() {
                        command = command.env("VAULT_CACERT", ca_cert.to_str().unwrap_or(""))?;
                    }
                    Ok(command)
                })
                .and_then(|c| c.execute());

            match result {
                Ok(output) => {
                    if output.status.success() {
                        info!("Seeded Vault defaults at {}", path);
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        warn!("Failed to seed {} in Vault: {}", path, stderr);
                    }
                }
                Err(e) => {
                    warn!("Failed to execute vault put for {}: {}", path, e);
                }
            }
        }

        info!("Vault defaults seeded successfully");
        Ok(())
    }

    /// Recover existing Vault installation (already initialized but may be sealed)
    fn recover_existing_vault(&self) -> Result<()> {


        info!("Recovering existing Vault installation...");

        let vault_addr = std::env::var("VAULT_ADDR")
            .ok()
            .filter(|a| !a.is_empty())
            .ok_or_else(|| anyhow::anyhow!("VAULT_ADDR must be set in .env or environment"))?;
        let ca_cert = self.base_path.join("conf/system/certificates/ca/ca.crt");
        #[cfg(target_os = "windows")]
        let vault_bin = self.base_path.join("bin/vault/vault.exe");
        #[cfg(not(target_os = "windows"))]
        let vault_bin = self.base_path.join("bin/vault/vault");

        // Try to read existing unseal keys
        let unseal_keys_file = self.base_path.join("vault-unseal-keys");
        let unseal_keys = if unseal_keys_file.exists() {
            info!("Found existing vault-unseal-keys file");
            let content = std::fs::read_to_string(&unseal_keys_file)?;
            content
                .lines()
                .filter_map(|line| {
                    line.strip_prefix("VAULT_UNSEAL_KEY_")
                        .and_then(|rest| rest.split_once('='))
                        .map(|(_, key)| key.to_string())
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // Prefer a token supplied by the launcher (for example a token kept
        // outside the repository), then fall back to the original init data.
        let init_json = self.base_path.join("conf/vault/init.json");
        let root_token = std::env::var("VAULT_TOKEN")
            .ok()
            .filter(|token| !token.trim().is_empty())
            .or_else(|| {
                if init_json.exists() {
                    let content = std::fs::read_to_string(&init_json).ok()?;
                    let json = serde_json::from_str::<serde_json::Value>(&content).ok()?;
                    json.get("root_token")
                        .and_then(|value| value.as_str())
                        .map(String::from)
                } else {
                    None
                }
            });

        // Unseal if we have keys
        if !unseal_keys.is_empty() {
            info!("Unsealing Vault with existing keys...");
            for (i, key) in unseal_keys.iter().take(3).enumerate() {
                let unseal_cmd = format!(
                    "{} operator unseal -tls-skip-verify -address={} {}",
                    vault_bin.display(),
                    vault_addr,
                    key
                );
                let unseal_output = safe_sh_command(&unseal_cmd);
                if let Some(ref output) = unseal_output {
                    if !output.status.success() {
                        warn!("Unseal step {} may have failed", i + 1);
                    }
                }
            }
        }

        // Create .env if we have root token
        if let Some(ref token) = root_token {
            let env_file = std::path::PathBuf::from(".env");
            let env_content = format!(
                r#"
# Vault Configuration (auto-generated)
VAULT_ADDR={}
VAULT_TOKEN={}
VAULT_CACERT={}
"#,
                vault_addr,
                token,
                ca_cert.display()
            );

            if env_file.exists() {
                let existing = std::fs::read_to_string(&env_file)?;
                if !existing.contains("VAULT_ADDR=") {
                    let mut file = std::fs::OpenOptions::new().append(true).open(&env_file)?;
                    file.write_all(env_content.as_bytes())?;
                    info!("Appended Vault config to .env");
                }
            } else {
                std::fs::write(&env_file, env_content.trim_start())?;
                info!("Created .env with Vault config");
            }
            std::env::set_var("VAULT_TOKEN", token);
        } else {
            warn!("No root token found - Vault may need manual recovery");
        }

        // Seed defaults ONLY if not already present (skip during recovery to preserve credentials)
        if let Some(ref token) = root_token {
            if self.vault_seeds_exist(&vault_addr, token, &ca_cert, &vault_bin)? {
                info!("Vault credentials already exist, skipping seed on recovery");
            } else {
                let _ = self.seed_vault_defaults(&vault_addr, token, &ca_cert, &vault_bin);
            }
        }

        info!("Vault recovery complete");
        Ok(())
    }

    /// Unseal Vault with 3 keys
    fn unseal_vault(&self, vault_bin: &std::path::Path, vault_addr: &str) -> Result<()> {
        info!("Unsealing Vault...");
        let unseal_keys_file = self.base_path.join("vault-unseal-keys");
        if unseal_keys_file.exists() {
            let content = std::fs::read_to_string(&unseal_keys_file)?;
            let keys: Vec<String> = content
                .lines()
                .filter_map(|line| {
                    line.strip_prefix("VAULT_UNSEAL_KEY_")
                        .and_then(|rest| rest.split_once('='))
                        .map(|(_, key)| key.to_string())
                })
                .collect();

            for (i, key) in keys.iter().take(3).enumerate() {
                let unseal_cmd = format!(
                    "{} operator unseal -tls-skip-verify -address={} {}",
                    vault_bin.display(),
                    vault_addr,
                    key
                );
                let unseal_output = safe_sh_command(&unseal_cmd);
                if let Some(ref output) = unseal_output {
                    if !output.status.success() {
                        warn!("Unseal step {} may have failed", i + 1);
                    }
                }
            }
        }
        Ok(())
    }

    /// Ensure .env file exists with Vault credentials
    fn ensure_env_file_exists(&self) -> Result<()> {
        let init_json = self.base_path.join("conf/vault/init.json");
        let env_file = std::path::PathBuf::from(".env");

        if !init_json.exists() {
            return Ok(()); // No init, no .env needed yet
        }

        let init_content = std::fs::read_to_string(&init_json)?;
        let init_json_val: serde_json::Value = serde_json::from_str(&init_content)?;

        let root_token = init_json_val["root_token"]
            .as_str()
            .context("No root_token in init.json")?;

        let conf_path = self.base_path.join("conf");
        let ca_cert = conf_path.join("system/certificates/ca/ca.crt");
        let vault_addr = std::env::var("VAULT_ADDR").unwrap_or_default();

        let env_content = format!(
            r#"
# Vault Configuration (auto-generated)
VAULT_ADDR={}
VAULT_TOKEN={}
VAULT_CACERT={}
"#,
            vault_addr,
            root_token,
            ca_cert.display()
        );

        if env_file.exists() {
            let existing = std::fs::read_to_string(&env_file)?;
            if existing.contains("VAULT_ADDR=") {
                return Ok(());
            }
            let mut file = std::fs::OpenOptions::new().append(true).open(&env_file)?;
            use std::io::Write;
            file.write_all(env_content.as_bytes())?;
        } else {
            std::fs::write(&env_file, env_content.trim_start())?;
        }

        info!("Created .env with Vault credentials");
        Ok(())
    }
}

use botlib::security::command_guard::SafeCommand;
use std::collections::HashMap;
use std::path::PathBuf;
