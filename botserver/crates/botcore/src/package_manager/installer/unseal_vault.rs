use super::*;
use botcorepkg::installer_vault::secret_store as secrets;

impl PackageManager {
    /// Read unseal keys from the private credential store, migrating a legacy
    /// plaintext file once. The stack copy is always removed after migration.
    fn unseal_keys(&self) -> Result<Vec<String>> {
        if let Ok(value) = secrets::read(secrets::UNSEAL_KEYS_SECRET) {
            let keys: Vec<String> = value
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(String::from)
                .collect();
            if !keys.is_empty() {
                return Ok(keys);
            }
        }
        let legacy = self.base_path.join("vault-unseal-keys");
        if !legacy.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&legacy)?;
        let keys: Vec<String> = content
            .lines()
            .filter_map(|line| {
                line.strip_prefix("VAULT_UNSEAL_KEY_")
                    .and_then(|rest| rest.split_once('='))
                    .map(|(_, key)| key.trim().to_string())
            })
            .filter(|key| !key.is_empty())
            .collect();
        if !keys.is_empty() {
            secrets::write(secrets::UNSEAL_KEYS_SECRET, &keys.join("\n"))?;
        }
        let _ = std::fs::remove_file(&legacy);
        info!("Migrated unseal keys out of the user-owned stack");
        Ok(keys)
    }

    fn unseal_key(&self, vault_bin: &std::path::Path, vault_addr: &str, key: &str) {
        let ca_cert = self.base_path.join("conf/system/certificates/ca/ca.crt");
        let vault_bin_str = vault_bin.to_string_lossy().to_string();
        let command = SafeCommand::new(&vault_bin_str)
            .and_then(|c| c.env("VAULT_ADDR", vault_addr))
            .and_then(|c| {
                if ca_cert.is_file() {
                    c.env("VAULT_CACERT", &ca_cert.to_string_lossy())
                } else {
                    Ok(c)
                }
            })
            .and_then(|c| c.args(&["operator", "unseal"]))
            .and_then(|c| c.trusted_arg(key))
            .and_then(|c| c.execute());
        match command {
            Ok(output) if output.status.success() => {}
            Ok(output) => warn!(
                "Unseal step failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(error) => warn!("Unseal command could not run: {error}"),
        }
    }

    pub(super) fn unseal_vault(&self, vault_bin: &std::path::Path, vault_addr: &str) -> Result<()> {
        info!("Unsealing Vault...");
        let keys = self.unseal_keys()?;
        for key in keys.iter().take(3) {
            self.unseal_key(vault_bin, vault_addr, key);
        }
        Ok(())
    }

    /// Ensure `.env` exists with a reference to the scoped token — never with
    /// the token value or the root token (#2498).
    pub(super) fn ensure_env_file_exists(&self) -> Result<()> {
        let vault_addr = match std::env::var("VAULT_ADDR") {
            Ok(addr) => addr,
            Err(_) => String::new(),
        };
        if vault_addr.is_empty() {
            return Ok(());
        }
        let ca_cert = self.base_path.join("conf/system/certificates/ca/ca.crt");
        let env_file = std::path::PathBuf::from(".env");
        secrets::write_env_file(&env_file, &vault_addr, Some(&ca_cert))?;
        info!("Created .env with a reference to the scoped Vault token");
        Ok(())
    }
}
