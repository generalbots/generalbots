use super::*;

impl PackageManager {
    pub(super) fn seed_vault_defaults(
        &self,
        vault_addr: &str,
        root_token: &str,
        ca_cert: &std::path::Path,
        vault_bin: &std::path::Path,
    ) -> Result<()> {
        info!("Seeding default credentials into Vault...");

        let drive_user = crate::package_manager::generate_random_string(16);
        let drive_pass = crate::package_manager::generate_random_string(32);
        let cache_pass = crate::package_manager::generate_random_string(32);
        let db_pass = crate::package_manager::generate_random_string(32);
        let master_key = crate::package_manager::generate_random_string(64);
        let meet_app_id = crate::package_manager::generate_random_string(24);
        let meet_app_secret = crate::package_manager::generate_random_string(48);
        let alm_token = crate::package_manager::generate_random_string(40);

        info!(
            "Generated strong random credentials for: drive, cache, tables, encryption, meet, alm"
        );

        let defaults: Vec<(&str, Vec<(String, String)>)> = vec![
            (
                // #1968 - canonical peer-node settings. The node stays off until an
                // operator runs `vault kv put secret/gbo/peer enabled=1`.
                "secret/gbo/peer",
                vec![
                    ("enabled".to_string(), "0".to_string()),
                    ("node-id".to_string(), "".to_string()),
                    ("node-name".to_string(), "".to_string()),
                    ("advertise-addr".to_string(), "peer.generalbots.org".to_string()),
                    ("data-dir".to_string(), "".to_string()),
                    ("heartbeat-ttl-secs".to_string(), "90".to_string()),
                    ("token".to_string(), "".to_string()),
                    ("signing-key".to_string(), "".to_string()),
                ],
            ),
            (
                "secret/gbo/drive",
                vec![
                    ("accesskey".to_string(), drive_user),
                    ("secret".to_string(), drive_pass),
                    ("host".to_string(), "127.0.0.1".to_string()),
                    ("port".to_string(), "9100".to_string()),
                    ("bucket".to_string(), "default.gborg".to_string()),
                    ("url".to_string(), "".to_string()),
                    // Backend selection (#1468). MinIO stays the runtime default;
                    // `endpoint` overrides host/port when pointing at B2 or R2.
                    ("backend".to_string(), "minio".to_string()),
                    ("region".to_string(), "auto".to_string()),
                    ("fallback_endpoint".to_string(), "".to_string()),
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
            // `-cas=0` writes only when the path has no version yet. Seeding
            // runs when the probe path is missing, which is also the state of
            // a stack that was only partly seeded: without check-and-set the
            // defaults below replaced secrets that already existed there, such
            // as the LLM endpoint chosen in the first-run wizard.
            let mut args = vec![
                "kv".to_string(),
                "put".to_string(),
                "-cas=0".to_string(),
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
                        if stderr.contains("check-and-set") {
                            info!("Vault already holds {path}; existing values kept");
                        } else {
                            warn!("Failed to seed {path} in Vault: {stderr}");
                        }
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
    pub(super) fn recover_existing_vault(&self) -> Result<()> {
        use botcorepkg::installer_vault::secret_store as secrets;

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

        // #2498 — unseal keys come from the private credential store. Migrate a
        // legacy stack file once, then delete it so the shell cannot read it.
        let unseal_keys = if let Ok(value) = secrets::read(secrets::UNSEAL_KEYS_SECRET) {
            value
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(String::from)
                .collect::<Vec<_>>()
        } else {
            let legacy = self.base_path.join("vault-unseal-keys");
            let keys = if legacy.exists() {
                let content = std::fs::read_to_string(&legacy)?;
                content
                    .lines()
                    .filter_map(|line| {
                        line.strip_prefix("VAULT_UNSEAL_KEY_")
                            .and_then(|rest| rest.split_once('='))
                            .map(|(_, key)| key.trim().to_string())
                    })
                    .filter(|key| !key.is_empty())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            if !keys.is_empty() {
                secrets::write(secrets::UNSEAL_KEYS_SECRET, &keys.join("\n"))?;
                let _ = std::fs::remove_file(&legacy);
                info!("Migrated unseal keys out of the user-owned stack");
            }
            keys
        };

        // Unseal if we have keys.
        if !unseal_keys.is_empty() {
            info!("Unsealing Vault with keys from the private credential store...");
            // `safe_sh_command` runs through SafeCommand, which clears the
            // environment — the CLI never sees VAULT_CACERT/VAULT_SKIP_VERIFY.
            // Without an explicit TLS flag every unseal died on `x509`, the
            // stack stayed sealed and Drive came up disabled on every boot.
            let tls_flag = if ca_cert.exists() {
                format!("-ca-cert={}", ca_cert.display())
            } else {
                "-tls-skip-verify".to_string()
            };
            for key in unseal_keys.iter().take(3) {
                let unseal_cmd = format!(
                    "{} operator unseal -address={} {} {}",
                    vault_bin.display(),
                    vault_addr,
                    tls_flag,
                    key
                );
                let unseal_output = safe_sh_command(&unseal_cmd);
                if let Some(ref output) = unseal_output {
                    if !output.status.success() {
                        warn!("Unseal step may have failed: {}", String::from_utf8_lossy(&output.stderr).trim());
                    }
                }
            }
        }

        // #2498 — only the scoped sidecar token is persisted, never the root
        // token from init.json. `.env` is rewritten with a VAULT_TOKEN_FILE
        // reference; no token value is written to the user-owned stack.
        secrets::load_token_into_env()?;
        let scoped_token = secrets::read(secrets::VAULT_TOKEN_SECRET);
        secrets::write_env_file(&std::path::PathBuf::from(".env"), &vault_addr, Some(&ca_cert))?;

        match scoped_token {
            Ok(token) => {
                self.ensure_kv_mount(&vault_addr, &token, &ca_cert, &vault_bin);
                if self.vault_seeds_exist(&vault_addr, &token, &ca_cert, &vault_bin)? {
                    info!("Vault credentials already exist, skipping seed on recovery");
                } else {
                    let _ = self.seed_vault_defaults(&vault_addr, &token, &ca_cert, &vault_bin);
                }
            }
            Err(_) => warn!("No scoped Vault token in the credential store — re-run first-run provisioning"),
        }

        info!("Vault recovery complete");
        Ok(())
    }
}
