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
                    command = command.env(key, value)?;
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
            let resolved = candidate.replace("{{BIN_PATH}}", &bin_root.to_string_lossy());
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
        if matches!(component_name, "tables" | "directory") {
            return false;
        }
        // A component bound to its declared port is running even when the
        // bundled check command assumes a different scheme (for example a
        // development MinIO serving HTTP on the same port). Without this
        // fallback the first-run wizard would try to start a duplicate.
        component.ports.iter().any(|port| {
            std::net::TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], *port)),
                std::time::Duration::from_millis(300),
            )
            .is_ok()
        })
    }

    /// Creates the PostgreSQL cluster when `initdb` never ran (the first
    /// install failed after the binaries were extracted, or `pgdata` was
    /// removed). Only used for local installs: container installs create the
    /// cluster inside the container through the install commands.
    #[cfg(unix)]
    fn ensure_tables_cluster(
        bin_path: &std::path::Path,
        data_path: &std::path::Path,
        logs_path: &std::path::Path,
    ) -> Result<()> {
        let pgdata = data_path.join("pgdata");
        if pgdata.exists() {
            return Ok(());
        }
        let initdb = bin_path.join("bin").join("initdb");
        if !initdb.exists() {
            return Err(anyhow::anyhow!(
                "the database binaries are missing ({}); reinstall the database component",
                initdb.display()
            ));
        }
        std::fs::create_dir_all(logs_path)?;

        // theseus-rs PostgreSQL 18.x bundles libstdc++ from Debian 12, which
        // needs glibc 2.36 (`arc4random`). Ubuntu 22.04 ships glibc 2.35, so
        // the dynamic loader aborts before initdb runs. Drop the bundled copy
        // only when it cannot load, so the host libstdc++ is used instead.
        let bundled_stdcpp = bin_path.join("lib").join("libstdc++.so.6");
        if bundled_stdcpp.exists() {
            let probe = SafeCommand::new("bash")
                .and_then(|c| c.arg("-c"))
                .and_then(|c| c.trusted_shell_script_arg("./bin/initdb --version"))
                .and_then(|c| c.working_dir(bin_path))
                .map_err(|e| anyhow::anyhow!("could not probe initdb: {e}"))?;
            if let Ok(output) = probe.execute() {
                if !output.status.success() {
                    std::fs::remove_file(&bundled_stdcpp).map_err(|e| {
                        anyhow::anyhow!(
                            "could not replace the bundled libstdc++ ({}): {e}",
                            bundled_stdcpp.display()
                        )
                    })?;
                }
            }
        }

        let password = database_password();
        let pwfile = logs_path.join("pwfile");
        std::fs::write(&pwfile, format!("{password}\n"))
            .map_err(|e| anyhow::anyhow!("could not write {}: {e}", pwfile.display()))?;

        // Reuse the root-aware wrapper written by the install step: initdb
        // refuses to run as root, and server installs start as root.
        let pgwrap = bin_path.join("pgwrap.sh");
        let init = format!(
            "cd '{}' && ./bin/initdb -D '{}' -U gbuser --pwfile='{}'",
            bin_path.display(),
            pgdata.display(),
            pwfile.display()
        );
        let script = if pgwrap.exists() {
            format!("'{}' \"{init}\"", pgwrap.display())
        } else {
            init
        };
        let output = SafeCommand::new("bash")
            .and_then(|c| c.arg("-c"))
            .and_then(|c| c.trusted_shell_script_arg(&script))
            .and_then(|c| c.working_dir(bin_path))
            .map_err(|e| anyhow::anyhow!("could not run initdb: {e}"))?
            .execute()
            .map_err(|e| anyhow::anyhow!("could not run initdb: {e}"))?;
        let _ = std::fs::remove_file(&pwfile);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr: String = stderr.trim().chars().take(600).collect();
            return Err(anyhow::anyhow!(
                "the database cluster could not be initialised (initdb exit {:?}): {}",
                output.status.code(),
                if stderr.is_empty() { "<no stderr>" } else { stderr.as_str() }
            ));
        }
        Ok(())
    }
}

/// Resolves the database password for a fresh cluster exactly like the install
/// commands resolve `{{DB_PASSWORD}}`.
#[cfg(unix)]
fn database_password() -> String {
    if let Ok(password) = std::env::var("BOOTSTRAP_DB_PASSWORD") {
        if !password.is_empty() {
            return password;
        }
    }
    match crate::shared::utils::get_database_url_sync() {
        Ok(url) => crate::shared::utils::parse_database_url(&url).1,
        Err(_) => String::new(),
    }
}
