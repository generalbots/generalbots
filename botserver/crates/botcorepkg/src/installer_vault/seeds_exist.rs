//! Vault seed detection used before re-seeding during recovery.

use anyhow::Result;
use botlib::security::SafeCommand;

pub fn vault_seeds_exist(
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
            c.env("VAULT_ADDR", vault_addr)
                .and_then(|c| c.env("VAULT_TOKEN", root_token))
                .and_then(|c| c.env("VAULT_CACERT", ca_cert.to_str().unwrap_or("")))
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
