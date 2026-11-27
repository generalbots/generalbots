use super::*;

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

    pub(super) fn register_components(&mut self) {
        self.components = botcorepkg::installer::PackageManager::all_components();
    }

    pub fn start(&self, component: &str) -> Result<std::process::Child> {
        if let Some(component) = self.components.get(component) {
            let bin_path = self.base_path.join("bin").join(&component.name);
            let data_path = self.base_path.join("data").join(&component.name);
            let conf_path = self.base_path.join("conf");
            let logs_path = self.base_path.join("logs").join(&component.name);

            // Desktop stack: loopback-only configs (existing installs are
            // rewritten here), database auth and port checks before a start.
            #[cfg(windows)]
            {
                botcorepkg::desktop_hardening::harden_stack_configs(&self.base_path);
                if component.name == "tables" {
                    return Self::start_windows_tables(&bin_path, &data_path, &logs_path, &self.base_path);
                }
            }

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

            #[cfg(windows)]
            botcorepkg::port_guard::ensure_ports_free(&component.name, &component.ports, &self.base_path)
                .map_err(|message| anyhow::anyhow!(message))?;

            if component.name == "vector_db" {
                let qdrant_conf = conf_path.join("vector_db/config.yaml");
                if !qdrant_conf.exists() {
                    let storage = data_path.join("storage");
                    let snapshots = data_path.join("snapshots");
                    if let Err(e) = std::fs::create_dir_all(&storage) {
                        log::warn!("Could not create qdrant storage dir {}: {e}", storage.display());
                    }
                    if let Err(e) = std::fs::create_dir_all(&snapshots) {
                        log::warn!("Could not create qdrant snapshots dir {}: {e}", snapshots.display());
                    }
                    let yaml = format!(
                        "storage:\n  storage_path: {}\n  snapshots_path: {}\n\nservice:\n  host: {}\n  http_port: 6333\n  grpc_port: 6334\n  enable_tls: false\n\nlog_level: INFO\n",
                        storage.display(),
                        snapshots.display(),
                        if cfg!(windows) { botcorepkg::desktop_hardening::LOOPBACK } else { "0.0.0.0" }
                    );
                    if let Err(e) = std::fs::write(&qdrant_conf, yaml) {
                        warn!("Failed to write qdrant config: {}", e);
                    } else {
                        info!("Generated qdrant config at {}", qdrant_conf.display());
                    }
                }
            }

            // PostgreSQL is the one component whose data directory cannot be
            // recreated by its start command. If a previous install failed
            // after the binaries were extracted (or pgdata was removed), the
            // wizard used to wait the full 180 s only to read
            // "pg_ctl: directory ... does not exist". Initialise the cluster
            // here so the wizard either recovers or reports initdb's stderr.
            #[cfg(unix)]
            if self.mode == InstallMode::Local && component.name == "tables" {
                Self::ensure_tables_cluster(&bin_path, &data_path, &logs_path)?;
            }

            let rendered_cmd = component
                .effective_exec_cmd()
                .replace(botcorepkg::secrets_engine::CLI_PLACEHOLDER, botcorepkg::secrets_engine::active_engine(&self.base_path).binary_name())
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
                    let value = match vault_credentials
                        .get(var_name)
                        .cloned()
                        .or_else(|| std::env::var(var_name).ok())
                    {
                        Some(value) => value,
                        // A component may reference an env var that is not
                        // configured in this profile; pass an empty value.
                        None => String::new(),
                    };
                    evaluated_envs.insert(k.clone(), value);
                } else {
                    evaluated_envs.insert(k.clone(), v.clone());
                }
            }

            #[cfg(target_os = "windows")]
            if component.name == "llm" {
                let direct = bin_path.join("llama-server.exe");
                let executable = if direct.is_file() { direct } else { bin_path.join("bin/llama-server.exe") };
                let spawn_model = |port: &str, model: &str, embedding: bool| -> Result<std::process::Child> {
                    let log = std::fs::OpenOptions::new().create(true).append(true)
                        .open(logs_path.join(if embedding { "embedding.log" } else { "llm.log" }))?;
                    let mut command = SafeCommand::new(&executable.to_string_lossy())?
                        .arg("--port")?.arg(port)?.arg("--host")?.arg("127.0.0.1")?
                        .arg("-m")?.arg(&data_path.join(model).to_string_lossy())?
                        .arg("--ubatch-size")?.arg("512")?.working_dir(&bin_path)?;
                    if embedding {
                        command = command.arg("--embeddings")?.arg("--pooling")?.arg("mean")?
                            .arg("--n-gpu-layers")?.arg("0")?.arg("--ctx-size")?.arg("512")?;
                    }
                    command = command.stdout(std::process::Stdio::from(log.try_clone()?))
                        .stderr(std::process::Stdio::from(log));
                    command.spawn().map_err(Into::into)
                };
                let embedding_bound = std::net::TcpStream::connect_timeout(
                    &std::net::SocketAddr::from(([127, 0, 0, 1], 8082)),
                    std::time::Duration::from_millis(300),
                )
                .is_ok();
                let vector_db_off = crate::product::is_service_off("vector_db");
                if let Some((port, model, embedding)) = windows_llm_engine(embedding_bound, vector_db_off) {
                    return spawn_model(port, model, embedding);
                }
                info!("Embedding engine is already running; the chat engine starts only for a stored local model");
                return SafeCommand::noop_child()
                    .map_err(|e| anyhow::anyhow!("Failed to create noop process: {}", e));
            }

            #[cfg(target_os = "windows")]
            if component.name == "directory" {
                let log = std::fs::OpenOptions::new().create(true).append(true)
                    .open(logs_path.join("zitadel.log"))?;
                let initialized = std::fs::metadata(conf_path.join("directory/admin-pat.txt"))
                    .map(|metadata| metadata.len() > 0).unwrap_or(false);
                let mut command = SafeCommand::new(&bin_path.join("zitadel.exe").to_string_lossy())?
                    .arg(if initialized { "start" } else { "start-from-init" })?
                    .arg("--masterkey")?.arg("MasterkeyNeedsToHave32Characters")?
                    .arg("--tlsMode")?.arg("disabled")?
                    .working_dir(&bin_path)?;
                if !initialized {
                    command = command.arg("--steps")?
                        .arg(&conf_path.join("directory/zitadel-init-steps.yaml").to_string_lossy())?;
                }
                for (key, value) in &evaluated_envs {
                    if !value.is_empty() {
                        command = command.env(key, value)?;
                    }
                }
                // The database asks for passwords now: the first-run setup logs
                // in as the database owner with the password Vault holds.
                if let Some(owner) = vault_credentials.get("TABLES_PASSWORD").filter(|p| !p.is_empty()) {
                    if evaluated_envs.get("ZITADEL_DATABASE_POSTGRES_ADMIN_PASSWORD").is_none_or(|p| p.is_empty()) {
                        command = command.env("ZITADEL_DATABASE_POSTGRES_ADMIN_PASSWORD", owner)?;
                    }
                }
                command = command.stdout(std::process::Stdio::from(log.try_clone()?))
                    .stderr(std::process::Stdio::from(log));
                return command.spawn().map_err(Into::into);
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

            // No fixed wait here: the caller polls the service's health check
            // (and Vault's initialisation below polls Vault itself).
            trace!("Spawn result for {}: {:?}", component.name, child.is_ok());

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
                            if let Err(e) = self.ensure_env_file_exists() {
                                log::warn!("Could not recreate .env for an already-running Vault: {e}");
                            }
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

    /// Resolves the on-disk program for a component, if it can be determined
    /// from its binary name or exec command. Used by the first-run wizard to
    /// report a missing bundled binary immediately instead of waiting for a
    /// health timeout.
    pub fn executable_path(&self, component_name: &str) -> Option<std::path::PathBuf> {
        let component = self.components.get(component_name)?;
        let bin_root = self.base_path.join("bin").join(&component.name);
        // The exec command is what `start` actually runs, so verify that path
        // first; a stale install layout must be reported as missing rather than
        // waiting for a health timeout.
        let command = component.effective_exec_cmd();
        let mut tokens = command.split_whitespace();
        if let Some(first) = tokens.next() {
            let candidate = if first == "nohup" {
                tokens.next().unwrap_or(first)
            } else {
                first
            };
            let resolved = candidate
                .replace(botcorepkg::secrets_engine::CLI_PLACEHOLDER, botcorepkg::secrets_engine::active_engine(&self.base_path).binary_name())
                .replace("{{BIN_PATH}}", &bin_root.to_string_lossy());
            if resolved.contains('/') || resolved.contains('\\') {
                return Some(std::path::PathBuf::from(resolved));
            }
        }
        if let Some(name) = component.effective_binary_name() {
            let direct = bin_root.join(&name);
            if direct.exists() {
                return Some(direct);
            }
            let nested = bin_root.join("bin").join(&name);
            if nested.exists() {
                return Some(nested);
            }
        }
        None
    }

    /// Reports whether a component's own check command succeeds. The first-run
    /// wizard uses it to wait for each dependency in order without restarting
    /// an already-initialised component.
    pub fn is_running(&self, component_name: &str) -> bool {
        let Some(component) = self.components.get(component_name) else {
            return false;
        };
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
        if check_cmd.trim().is_empty() {
            return false;
        }
        let (shell, flag) = crate::os_abstraction::get_abstraction().shell_command();
        let command = match SafeCommand::new(shell).and_then(|command| command.arg(flag)) {
            Ok(command) => command,
            Err(e) => {
                log::debug!("is_running: could not build the {} check: {e}", component.name);
                return false;
            }
        };
        let command = match command.raw_shell_script_arg(&check_cmd) {
            Ok(command) => command,
            Err(e) => {
                log::debug!("is_running: invalid {} check: {e}", component.name);
                return false;
            }
        };
        // `SafeCommand` clears the environment; the vault check reads
        // `${VAULT_ADDR}`, so pass it explicitly (issue: first-run wizard).
        let command = match std::env::var("VAULT_ADDR") {
            Ok(address) if !address.trim().is_empty() => match command.env("VAULT_ADDR", &address) {
                Ok(command) => command,
                Err(_) => return false,
            },
            _ => command,
        };
        let check_ok = command
            .execute()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if check_ok {
            return true;
        }
        // A listening socket is not proof that a database or identity provider
        // can serve requests (orphaned Windows children can retain sockets).
        if matches!(component_name, "tables" | "directory" | "llm") {
            return false;
        }
        // A component bound to its declared port is running even when the
        // bundled check command assumes a different scheme (for example a
        // development OtterIO serving HTTP on the same port). Without this
        // fallback the first-run wizard would try to start a duplicate.
        component.ports.iter().any(|port| {
            std::net::TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], *port)),
                std::time::Duration::from_millis(300),
            )
            .is_ok()
        })
    }

}

/// The engine the `llm` component starts on Windows, as (port, model, embedding).
/// Only the embedding engine used by knowledge search, and only when its port
/// is free and the vector store is on (the embedding endpoint has no consumers
/// with vector_db off). The chat engine on 8081 is never started here: the boot
/// model check (`boot_runtime_ctl`) starts it once for a stored local `.gguf`
/// choice, with that model's context size, and a hosted choice leaves it off.
#[cfg(any(target_os = "windows", test))]
fn windows_llm_engine(embedding_bound: bool, vector_db_off: bool) -> Option<(&'static str, &'static str, bool)> {
    (!embedding_bound && !vector_db_off).then_some(("8082", "bge-small-en-v1.5-f32.gguf", true))
}

#[cfg(test)]
mod windows_llm_engine_tests {
    use super::windows_llm_engine;

    #[test]
    fn only_the_embedding_engine_starts_and_never_twice() {
        assert_eq!(windows_llm_engine(false, false), Some(("8082", "bge-small-en-v1.5-f32.gguf", true)));
        assert_eq!(windows_llm_engine(true, false), None, "already bound");
        for bound in [false, true] {
            assert!(windows_llm_engine(bound, false).is_none_or(|(port, ..)| port != "8081"));
        }
    }

    #[test]
    fn nothing_starts_when_the_vector_store_is_off() {
        assert_eq!(windows_llm_engine(false, true), None, "vector_db off => no embedding");
        assert_eq!(windows_llm_engine(true, true), None, "both off");
    }
}
