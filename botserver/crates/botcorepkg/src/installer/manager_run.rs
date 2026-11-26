use super::*;
use crate::db_utils::ZITADEL_PASSWORD_PLACEHOLDER;
use crate::generate_random_string;
use std::net::IpAddr;
use std::path::Path;

const QDRANT_LOOPBACK_HOST: &str = "127.0.0.1";

/// Never expose the Qdrant data plane on a wildcard interface. Only loopback
/// and RFC1918 / unique-local addresses are accepted; anything else (including
/// `0.0.0.0`, `::` and public addresses) falls back to loopback.
fn qdrant_bind_host(configured: Option<&str>) -> String {
    match configured {
        None => QDRANT_LOOPBACK_HOST.to_string(),
        Some(host) => {
            let host = host.trim();
            if host.eq_ignore_ascii_case("localhost") {
                return QDRANT_LOOPBACK_HOST.to_string();
            }
            match host.parse::<IpAddr>() {
                Ok(IpAddr::V4(addr)) if addr.is_loopback() || addr.is_private() => host.to_string(),
                Ok(IpAddr::V6(addr)) if addr.is_loopback() || addr.is_unique_local() => {
                    host.to_string()
                }
                _ => {
                    warn!(
                        "Qdrant bind host '{}' is not loopback/private; falling back to {}",
                        host, QDRANT_LOOPBACK_HOST
                    );
                    QDRANT_LOOPBACK_HOST.to_string()
                }
            }
        }
    }
}

fn usable_api_key(key: &str) -> bool {
    !key.is_empty() && key != "none" && !key.chars().any(char::is_control)
}

/// Resolve the key shared by the Qdrant config and the botserver client.
/// The boolean reports that a local key had to be generated because Vault was
/// not reachable yet — the caller then exports it into the running process.
fn qdrant_api_key(credentials: &HashMap<String, String>) -> (String, bool) {
    if let Some(key) = credentials
        .get("VECTORDB_API_KEY")
        .map(String::as_str)
        .filter(|key| usable_api_key(key))
    {
        return (key.to_string(), false);
    }
    for name in ["QDRANT_API_KEY", "VECTORDB_API_KEY"] {
        if let Some(key) = std::env::var(name)
            .ok()
            .filter(|key| usable_api_key(key))
        {
            return (key, false);
        }
    }
    (generate_random_string(48), true)
}

fn yaml_scalar(value: &str) -> String {
    format!(
        "\"{}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

fn config_api_key(text: &str) -> Option<&str> {
    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("api_key:")
            .map(|value| value.trim().trim_matches('"').trim_matches('\''))
    })
}

/// An existing config is insecure when it still binds a wildcard interface or
/// its credential is missing / differs from the one Vault now provides.
fn qdrant_config_is_insecure(text: &str, expected_key: &str) -> bool {
    text.contains("host: 0.0.0.0")
        || text.contains("host: ::")
        || config_api_key(text) != Some(expected_key)
}

fn render_qdrant_config(
    storage: &Path,
    snapshots: &Path,
    host: &str,
    api_key: &str,
) -> String {
    format!(
        "storage:\n storage_path: {}\n snapshots_path: {}\n\nservice:\n host: {}\n http_port: 6333\n grpc_port: 6334\n enable_tls: false\n api_key: {}\n\nlog_level: INFO\n",
        storage.display(),
        snapshots.display(),
        qdrant_bind_host(Some(host)),
        yaml_scalar(api_key)
    )
}

impl PackageManager {
    pub(super) fn register_components(&mut self) {
        self.components = Self::all_components();
    }

    pub fn start(&self, component: &str) -> Result<std::process::Child> {
        if let Some(component) = self.components.get(component) {
            let bin_path = self.base_path.join("bin").join(&component.name);
            let data_path = self.base_path.join("data").join(&component.name);
            let conf_path = self.base_path.join("conf");
            let logs_path = self.base_path.join("logs").join(&component.name);

            // #2817 — fetched before the "already running" early return so an
            // existing, insecure Qdrant config is upgraded on disk too, and
            // reused below when resolving component environment variables.
            let vault_credentials = installer_vault::fetch_vault_credentials();

            if component.name == "vector_db" {
                let (api_key, generated_locally) = qdrant_api_key(&vault_credentials);
                // Export to this process (botserver) so every QdrantClient
                // resolves the same credential the data plane was given.
                std::env::set_var("QDRANT_API_KEY", &api_key);
                if generated_locally {
                    std::env::set_var("VECTORDB_API_KEY", &api_key);
                    warn!(
                        "Generated a local Qdrant API key because Vault had none; persist it at secret/gbo/vectordb api_key"
                    );
                }

                let qdrant_conf = conf_path.join("vector_db/config.yaml");
                let needs_config = match std::fs::read_to_string(&qdrant_conf) {
                    Ok(text) => qdrant_config_is_insecure(&text, &api_key),
                    Err(_) => true,
                };
                if needs_config {
                    let storage = data_path.join("storage");
                    let snapshots = data_path.join("snapshots");
                    if let Err(e) = std::fs::create_dir_all(&storage) {
                        warn!("Failed to create qdrant storage dir: {}", e);
                    }
                    if let Err(e) = std::fs::create_dir_all(&snapshots) {
                        warn!("Failed to create qdrant snapshots dir: {}", e);
                    }
                    if let Some(parent) = qdrant_conf.parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            warn!("Failed to create qdrant config dir: {}", e);
                        }
                    }
                    let host = qdrant_bind_host(
                        vault_credentials
                            .get("VECTORDB_HOST")
                            .map(String::as_str),
                    );
                    let yaml = render_qdrant_config(&storage, &snapshots, &host, &api_key);
                    if let Err(e) = std::fs::write(&qdrant_conf, yaml) {
                        warn!("Failed to write qdrant config: {}", e);
                    } else {
                        info!(
                            "Generated authenticated, loopback-only qdrant config at {}",
                            qdrant_conf.display()
                        );
                    }
                }
            }

            let check_cmd = component
                .effective_check_cmd()
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy());

            let check_output = safe_shell_command(&check_cmd)
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

            // #2763 — the Zitadel service command/environment may embed the
            // DB role password placeholder. Resolve it from Vault before the
            // process is spawned; a Vault error must abort the start.
            let zitadel_password = if component
                .effective_exec_cmd()
                .contains(ZITADEL_PASSWORD_PLACEHOLDER)
                || component
                    .env_vars
                    .values()
                    .any(|value| value.contains(ZITADEL_PASSWORD_PLACEHOLDER))
            {
                crate::db_utils::zitadel_db_password().map_err(|error| {
                    anyhow::anyhow!(
                        "Cannot resolve Zitadel DB password for '{}' (Vault failure is fatal): {error}",
                        component.name
                    )
                })?
            } else {
                String::new()
            };
            let rendered_cmd = component
                .effective_exec_cmd()
                .replace(crate::secrets_engine::CLI_PLACEHOLDER, crate::secrets_engine::active_engine(&self.base_path).binary_name())
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy())
                .replace(ZITADEL_PASSWORD_PLACEHOLDER, &zitadel_password);

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

            let mut evaluated_envs = HashMap::new();
            for (k, v) in &component.env_vars {
                if let Some(var_name) = v.strip_prefix('$') {
                    let value = vault_credentials
                        .get(var_name)
                        .cloned()
                        .or_else(|| std::env::var(var_name).ok())
                        .unwrap_or_else(String::new);
                    evaluated_envs.insert(k.clone(), value);
                } else {
                    evaluated_envs.insert(
                        k.clone(),
                        v.replace(ZITADEL_PASSWORD_PLACEHOLDER, &zitadel_password),
                    );
                }
            }

            trace!(
                "About to spawn shell command for {}: {}",
                component.name,
                rendered_cmd
            );
            trace!("Working dir: {}", bin_path.display());
            #[cfg(target_os = "windows")]
            let child = SafeCommand::new("cmd")
                .and_then(|c| c.arg("/C"))
                .and_then(|c| c.trusted_shell_script_arg(&rendered_cmd))
                .and_then(|c| c.working_dir(&bin_path))
                .and_then(|cmd| cmd.spawn_with_envs(&evaluated_envs))
                .map_err(|e| anyhow::anyhow!("Failed to spawn process: {}", e));

            #[cfg(not(target_os = "windows"))]
            let child = SafeCommand::new("sh")
                .and_then(|c| c.arg("-c"))
                .and_then(|c| c.trusted_shell_script_arg(&rendered_cmd))
                .and_then(|c| c.working_dir(&bin_path))
                .and_then(|cmd| cmd.spawn_with_envs(&evaluated_envs))
                .map_err(|e| anyhow::anyhow!("Failed to spawn process: {}", e));

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
                    trace!("Component {} started successfully", component.name);

                    if component.name == "vault" && self.mode == InstallMode::Local {
                        if let Some(addr) = evaluated_envs.get("VAULT_ADDR") {
                            if !addr.is_empty() {
                                std::env::set_var("VAULT_ADDR", addr);
                            }
                        }
                        if let Err(e) = installer_vault::initialize_vault_local(&self.base_path) {
                            warn!("Failed to initialize Vault: {}", e);
                            warn!("Vault started but may need manual initialization");
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
                        trace!(
                            "Component {} may already be running, continuing anyway",
                            component.name
                        );

                        if component.name == "vault" && self.mode == InstallMode::Local {
                            if let Err(e) = installer_vault::ensure_env_file_exists(&self.base_path) {
                                log::warn!("Failed to ensure .env exists: {e}");
                            }
                        }

                        SafeCommand::noop_child()
                            .map_err(|e| anyhow::anyhow!("Failed to create noop process: {}", e))
                    } else {
                        Err(e)
                    }
                }
            }
        } else {
            Err(anyhow::anyhow!("Component not found: {}", component))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_and_public_bind_hosts_fall_back_to_loopback() {
        for host in ["0.0.0.0", "::", "[::]", "*", "", "8.8.8.8", "2001:db8::1"] {
            assert_eq!(qdrant_bind_host(Some(host)), "127.0.0.1", "host: {host}");
        }
        assert_eq!(qdrant_bind_host(None), "127.0.0.1");
    }

    #[test]
    fn loopback_and_private_bind_hosts_are_preserved() {
        for host in ["127.0.0.1", "127.0.0.2", "10.1.2.3", "192.168.1.10", "fd00::1"] {
            assert_eq!(qdrant_bind_host(Some(host)), host, "host: {host}");
        }
        // `localhost` is loopback, so it normalizes to the explicit loopback IP.
        assert_eq!(qdrant_bind_host(Some("localhost")), "127.0.0.1");
        assert_eq!(qdrant_bind_host(Some(" 127.0.0.1 ")), "127.0.0.1");
    }

    #[test]
    fn generated_config_binds_loopback_and_requires_a_key() {
        let config = render_qdrant_config(
            Path::new("/data/storage"),
            Path::new("/data/snapshots"),
            "0.0.0.0",
            "aBc123",
        );
        assert!(!config.contains("0.0.0.0"));
        assert!(config.contains("host: 127.0.0.1"));
        assert!(config.contains("api_key: \"aBc123\""));
        assert!(!qdrant_config_is_insecure(&config, "aBc123"));
        assert!(qdrant_config_is_insecure(&config, "rotated-key"));
    }

    #[test]
    fn legacy_config_is_detected_as_insecure() {
        let legacy = "service:\n host: 0.0.0.0\n http_port: 6333\n";
        assert!(qdrant_config_is_insecure(legacy, "aBc123"));
        assert_eq!(config_api_key(legacy), None);
    }

    #[test]
    fn vault_key_is_preferred_and_placeholder_is_rejected() {
        let mut credentials = HashMap::new();
        credentials.insert("VECTORDB_API_KEY".to_string(), "vault-key".to_string());
        assert_eq!(qdrant_api_key(&credentials), ("vault-key".to_string(), false));

        let mut placeholder = HashMap::new();
        placeholder.insert("VECTORDB_API_KEY".to_string(), "none".to_string());
        let env_has_key = ["QDRANT_API_KEY", "VECTORDB_API_KEY"].iter().any(|name| {
            std::env::var(name)
                .ok()
                .filter(|value| usable_api_key(value))
                .is_some()
        });
        let (key, generated) = qdrant_api_key(&placeholder);
        assert!(usable_api_key(&key));
        if !env_has_key {
            assert!(generated);
            assert_eq!(key.len(), 48);
            assert!(key.chars().all(|c| c.is_ascii_alphanumeric()));
        }
    }
}
