//! Vault seed detection used before re-seeding during recovery.

use anyhow::Result;
use botlib::security::SafeCommand;

pub fn vault_seeds_exist(
    vault_addr: &str,
    root_token: &str,
    ca_cert: &std::path::Path,
    vault_bin: &std::path::Path,
) -> Result<bool> {
    let mut args = vec!["kv".to_string(), "get".to_string()];
    if ca_cert.is_file() {
        args.push("-ca-cert".to_string());
        args.push(ca_cert.to_string_lossy().to_string());
    } else {
        args.push("-tls-skip-verify".to_string());
    }
    args.push("-field=accesskey".to_string());
    args.push("secret/gbo/drive".to_string());

    let result = SafeCommand::new(vault_bin.to_str().unwrap_or("vault"))
        .and_then(|c| {
            let mut cmd = c.env("VAULT_ADDR", vault_addr)?.env("VAULT_TOKEN", root_token)?;
            for arg in &args {
                cmd = cmd.trusted_arg(arg)?;
            }
            Ok(cmd)
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
