use super::*;

impl PackageManager {
    pub(super) fn initialize_vault_local(&self) -> Result<()> {

        info!("Initializing Vault locally (non-LXC mode)...");

        let conf_path = self.base_path.join("conf");
        let vault_bin = botcorepkg::secrets_engine::cli_path(&self.base_path);
        let vault_data = self.base_path.join("data/vault");

        // Check if Vault data directory exists (real indicator of initialized state)
        let vault_data_exists = vault_data.exists();

        if !vault_data_exists {
            info!("Vault data directory not found, will initialize fresh");
        } else {
            info!("Vault data directory found, checking health...");
        }

        // Vault's readiness is polled below; no fixed wait first.
        info!("Waiting for Vault to start...");

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

        // #2498 — the root token stays in memory for init-time CLI calls only.
        // Persist a scoped token in the private credential store instead.
        let scoped = botcorepkg::installer_vault::secret_store::create_scoped_token(
            &vault_addr, &root_token, &ca_cert, &vault_bin,
        )?;
        botcorepkg::installer_vault::secret_store::write(
            botcorepkg::installer_vault::secret_store::VAULT_TOKEN_SECRET, &scoped,
        )?;
        botcorepkg::installer_vault::secret_store::write(
            botcorepkg::installer_vault::secret_store::UNSEAL_KEYS_SECRET,
            &unseal_keys.join("\n"),
        )?;
        botcorepkg::installer_vault::secret_store::load_token_into_env()?;

        // Save a sanitized init.json: no root token and no unseal keys.
        let init_json = self.base_path.join("conf/vault/init.json");
        std::fs::create_dir_all(
            init_json.parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid init.json path: no parent directory"))?
        )?;
        let safe_json = serde_json::to_string_pretty(
            &botcorepkg::installer_vault::secret_store::sanitized_init_json(&init_json_val),
        )?;
        std::fs::write(&init_json, safe_json)?;
        {
            let abs = crate::os_abstraction::get_abstraction();
            if let Err(e) = abs.set_readonly_owner(&init_json) {
                warn!("Failed to set permissions on {}: {e}", init_json.display());
            }
        }
        info!("Created {} (no secret material)", init_json.display());

        // `.env` references the scoped token; it never contains a token value.
        let env_file = std::path::PathBuf::from(".env");
        botcorepkg::installer_vault::secret_store::write_env_file(
            &env_file, &vault_addr, Some(&ca_cert),
        )?;
        {
            let abs = crate::os_abstraction::get_abstraction();
            if let Err(e) = abs.set_readonly_owner(&env_file) {
                warn!("Failed to set permissions on {}: {e}", env_file.display());
            }
        }

        // Migration: remove the legacy plaintext unseal-key file.
        let legacy_keys = self.base_path.join("vault-unseal-keys");
        if legacy_keys.exists() {
            if let Err(e) = std::fs::remove_file(&legacy_keys) {
                log::warn!("Could not remove legacy unseal-key file {}: {e}", legacy_keys.display());
            }
        }

        // Unseal Vault (need 3 keys from the private store)
        self.unseal_vault(&vault_bin, &vault_addr)?;

        info!("Vault initialized and unsealed; scoped token stored outside the stack");

        // Enable the KV2 secrets engine at 'secret/' (idempotent).
        self.ensure_kv_mount(&vault_addr, &root_token, &ca_cert, &vault_bin);

        // Write default credentials to Vault for all components
        self.seed_vault_defaults(&vault_addr, &root_token, &ca_cert, &vault_bin)?;

        Ok(())
    }

    /// Idempotently ensure the KV v2 engine is mounted at `secret/`.
    ///
    /// Both the fresh-init and the recovery path must call this. A stack whose
    /// Vault data survived (reset that kept `data/vault`, an enable step that
    /// failed once) would otherwise never mount `secret/`, so every
    /// `secret/gbo/*` put/get fails and the assistant silently falls back to
    /// defaults on every boot.
    pub(super) fn ensure_kv_mount(
        &self,
        vault_addr: &str,
        root_token: &str,
        ca_cert: &std::path::Path,
        vault_bin: &std::path::Path,
    ) {
        info!("Ensuring KV2 secrets engine at 'secret/'...");
        let enable_result = SafeCommand::new(vault_bin.to_str().unwrap_or("vault"))
            .and_then(|c| c.env("VAULT_ADDR", vault_addr))
            .and_then(|c| c.env("VAULT_TOKEN", root_token))
            .and_then(|c| {
                // Vault runs over plain HTTP on Windows and no CA file exists;
                // pointing the CLI at a missing file makes it fail before it
                // contacts the server.
                if ca_cert.is_file() {
                    c.env("VAULT_CACERT", ca_cert.to_str().unwrap_or(""))
                } else {
                    Ok(c)
                }
            })
            .and_then(|c| c.args(&["secrets", "enable", "-path=secret", "kv-v2"]))
            .and_then(|c| c.execute());
        match enable_result {
            Ok(output) if output.status.success() => {
                info!("KV2 secrets engine enabled at 'secret/'");
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("path is already in use") {
                    info!("KV2 secrets engine already enabled at 'secret/'");
                } else if stderr.contains("permission denied") {
                    // The server's limited token (desktop) may not manage
                    // mounts; it only exists once Vault was set up, and that
                    // setup enabled the engine with the root token.
                    info!("KV2 secrets engine is managed at Vault setup; the server token does not change mounts");
                } else {
                    warn!("Failed to enable KV2 secrets engine: {}", stderr);
                }
            }
            Err(e) => {
                warn!("Failed to execute KV2 enable command: {e}");
            }
        }
    }

    /// Check if Vault already has seeded credentials (to avoid overwriting on recovery)
    pub(super) fn vault_seeds_exist(
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
}
