//! Secret material handling for first-run provisioning (#2498).
//!
//! Root tokens and Vault unseal keys must never be written into the stack the
//! kiosk user owns. They live in a 0700 credential directory owned by the
//! `gbsvc` system account (`/etc/generalbots/credentials` on the appliance) or,
//! for development installs without that account, in a private 0700 directory
//! under the OS temp dir. The root token is used in memory during `vault init`
//! only; the token persisted for the sidecar is a scoped `gb-sidecar` token.
//!
//! When the credential directory is present but not writable by the
//! provisioner, a root-owned helper (`/usr/local/libexec/gb-secrets`, installed
//! by the OS image) performs the store/read and writes an audit event for every
//! open that is not made by the sidecar UID.

use anyhow::{Context, Result};
use botlib::security::SafeCommand;
use log::{info, warn};
use std::path::{Path, PathBuf};

pub use super::token_recovery::recover_scoped_token;
pub use super::vault_policy::{create_scoped_token, SIDECAR_POLICY, SIDECAR_POLICY_TEXT};

/// Credential directory created by `desktop/os` (root:gbsvc 0750).
pub const CREDENTIAL_DIR: &str = "/etc/generalbots/credentials";
/// Root helper used when the credential directory is not writable directly.
pub const HELPER: &str = "/usr/local/libexec/gb-secrets";
/// Secret file holding the scoped sidecar token.
pub const VAULT_TOKEN_SECRET: &str = "vault-token";
/// Secret file holding the unseal keys (one per line).
pub const UNSEAL_KEYS_SECRET: &str = "vault-unseal-keys";

fn valid_name(name: &str) -> Result<()> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        || name.starts_with('.')
    {
        return Err(anyhow::anyhow!("Invalid secret name: {name}"));
    }
    Ok(())
}

#[cfg(unix)]
fn mode_of(path: &Path) -> Result<u32> {
    use std::os::unix::fs::PermissionsExt;
    let metadata =
        std::fs::metadata(path).with_context(|| format!("Cannot stat {}", path.display()))?;
    Ok(metadata.permissions().mode() & 0o777)
}

/// A credential directory is secure when "other" has no access. The appliance
/// uses root:gbsvc 0750; a development fallback uses 0700.
#[cfg(unix)]
fn secure_dir(path: &Path) -> Result<()> {
    let found = mode_of(path)?;
    if found & 0o007 != 0 {
        return Err(anyhow::anyhow!(
            "Insecure credential directory {}: mode {:o} grants world access",
            path.display(),
            found
        ));
    }
    Ok(())
}

/// A secret file is secure when "other" has no access and "group" cannot write.
/// The appliance uses root:gbsvc 0640; a development fallback uses 0600.
#[cfg(unix)]
fn secure_file(path: &Path) -> Result<()> {
    let found = mode_of(path)?;
    if found & 0o007 != 0 || found & 0o020 != 0 {
        return Err(anyhow::anyhow!(
            "Insecure secret file {}: mode {:o}",
            path.display(),
            found
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn secure_dir(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn secure_file(_path: &Path) -> Result<()> {
    Ok(())
}

pub(crate) fn write_private(path: &Path, value: &str) -> Result<()> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        if parent.exists() {
            // Never widen an appliance directory that is already root:gbsvc.
            secure_dir(parent)?;
        } else {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create {}", parent.display()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
            }
        }
    }
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        use std::os::unix::fs::PermissionsExt;
        // The botcore installer marks generated files owner-read-only; make
        // sure a later regeneration can still open them for writing.
        if path.exists() {
            if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
                log::warn!("Failed to set permissions on {}: {e}", path.display());
            }
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .with_context(|| format!("Cannot write {}", path.display()))?;
        file.write_all(value.as_bytes())?;
    }
    #[cfg(not(unix))]
    std::fs::write(path, value)?;
    secure_file(path)?;
    Ok(())
}

/// Resolve the secret directory. The appliance directory wins when provisioned;
/// otherwise see [`super::secret_location::fallback_directory`].
pub fn directory() -> Result<PathBuf> {
    let configured = std::env::var("GB_SECRET_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from);
    let path = match configured {
        Some(path) => path,
        None if Path::new(CREDENTIAL_DIR).is_dir() => PathBuf::from(CREDENTIAL_DIR),
        None => super::secret_location::fallback_directory(),
    };
    let created = !path.exists();
    std::fs::create_dir_all(&path)
        .with_context(|| format!("Cannot create secret directory {}", path.display()))?;
    if created && path.as_path() != Path::new(CREDENTIAL_DIR) {
        // A directory we create ourselves must be private from the start;
        // never chmod the appliance directory (root:gbsvc 0750).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    secure_dir(&path)?;
    Ok(path)
}

fn helper_call(args: &[&str]) -> Result<std::process::Output> {
    let mut command = SafeCommand::new("sudo").context("sudo required for secret helper")?;
    command = command.args(args)?;
    command
        .execute()
        .context("Cannot execute secret store helper")
}

/// Write a secret. Falls back to the root helper when the credential directory
/// belongs to the service account and cannot be written by the provisioner.
pub fn write(name: &str, value: &str) -> Result<PathBuf> {
    valid_name(name)?;
    let dir = directory()?;
    let path = dir.join(name);
    if dir.as_path() == Path::new(CREDENTIAL_DIR) {
        // Appliance: always go through the root-owned helper so the file ends
        // up root:gbsvc 0640 no matter which account provisions it.
        let source = std::env::temp_dir().join(format!(
            "gb-secret-input-{}-{name}",
            std::process::id()
        ));
        write_private(&source, value)?;
        let source_arg = source.to_string_lossy().to_string();
        let result = helper_call(&[HELPER, "store", name, &source_arg]);
        if let Err(e) = std::fs::remove_file(&source) {
            log::warn!("Failed to remove temporary secret input {}: {e}", source.display());
        }
        result?;
        info!("Stored secret {name} through the root helper");
        return Ok(path);
    }
    // Windows has no file modes to keep other accounts out: seal the secret
    // with DPAPI for the current user instead.
    #[cfg(windows)]
    crate::protected_file::write_secret(&path, value)
        .with_context(|| format!("Cannot write secret {name}"))?;
    #[cfg(not(windows))]
    write_private(&path, value)?;
    Ok(path)
}

fn read_direct(name: &str) -> Result<String> {
    let path = directory()?.join(name);
    secure_file(&path)?;
    let content = crate::protected_file::read_secret(&path)
        .with_context(|| format!("Cannot read secret {name}"))?;
    audit_open(name);
    Ok(content.trim_end().to_string())
}

fn read_via_helper(name: &str) -> Result<String> {
    let dest = std::env::temp_dir().join(format!("gb-secret-read-{}-{name}", std::process::id()));
    let dest_arg = dest.to_string_lossy().to_string();
    let result = helper_call(&[HELPER, "read", name, &dest_arg]);
    if let Err(error) = result {
        if let Err(e) = std::fs::remove_file(&dest) {
            log::warn!("Failed to remove temporary secret output {}: {e}", dest.display());
        }
        return Err(error);
    }
    audit_open(name);
    let content = std::fs::read_to_string(&dest)
        .with_context(|| format!("Cannot read helper output for secret {name}"))?;
    if let Err(e) = std::fs::remove_file(&dest) {
        log::warn!("Failed to remove temporary secret output {}: {e}", dest.display());
    }
    Ok(content.trim_end().to_string())
}

/// Emit an audit record when a non-sidecar process opens the store. The root
/// helper performs the same check against its own caller UID.
fn audit_open(name: &str) {
    #[cfg(unix)]
    {
        let expected = match std::env::var("GB_SIDECAR_UID") {
            Ok(v) => Some(v),
            Err(e) => { log::debug!("GB_SIDECAR_UID not set: {e}"); None }
        };
        let current = nix_current_uid();
        if let (Some(expected), Some(current)) = (expected, current) {
            if expected != current.to_string() {
                warn!("SECURITY AUDIT: secret {name} opened by uid {current}, sidecar uid {expected}");
            }
        }
    }
}

#[cfg(unix)]
fn nix_current_uid() -> Option<u32> {
    // `std` has no getuid; read the effective owner of /proc/self (Linux) and
    // fall back to none on platforms without procfs.
    std::fs::metadata("/proc/self").ok().map(|m| {
        use std::os::unix::fs::MetadataExt;
        m.uid()
    })
}

/// Read a secret from the private store (or through the root helper).
pub fn read(name: &str) -> Result<String> {
    valid_name(name)?;
    match read_direct(name) {
        Ok(value) => Ok(value),
        Err(error) if directory()?.as_path() == Path::new(CREDENTIAL_DIR) => {
            read_via_helper(name).with_context(|| error.to_string())
        }
        Err(error) => Err(error),
    }
}

/// Delete a secret after it has been used (root tokens never survive init).
pub fn remove(name: &str) -> Result<()> {
    valid_name(name)?;
    let dir = directory()?;
    let path = dir.join(name);
    if !path.exists() {
        return Ok(());
    }
    if dir.as_path() == Path::new(CREDENTIAL_DIR) {
        helper_call(&[HELPER, "remove", name])?;
        return Ok(());
    }
    std::fs::remove_file(&path)
        .with_context(|| format!("Cannot remove {}", path.display()))
}

/// Return a copy of the init payload with all secret material removed, so the
/// persisted `init.json` can never be used to recover the root token.
pub fn sanitized_init_json(value: &serde_json::Value) -> serde_json::Value {
    let mut safe = value.clone();
    if let Some(object) = safe.as_object_mut() {
        object.remove("root_token");
        object.remove("unseal_keys_b64");
        object.remove("recovery_keys_b64");
    }
    safe
}

/// `.env` entries for the sidecar: address and a *path* to the scoped token,
/// never the token value itself and never the root token. Paths are quoted so
/// a Windows path (`C:\Users\…`) parses.
pub fn env_lines(vault_addr: &str, ca_cert: Option<&Path>) -> Result<String> {
    use super::secret_location::env_path;
    let path = directory()?.join(VAULT_TOKEN_SECRET);
    let mut lines = format!(
        "# Vault Configuration (auto-generated, #2498)\nVAULT_ADDR={vault_addr}\nVAULT_TOKEN_FILE={}\n",
        env_path(&path)
    );
    if let Some(ca_cert) = ca_cert {
        lines.push_str(&format!("VAULT_CACERT={}\n", env_path(ca_cert)));
    }
    Ok(lines)
}

/// Put the scoped token into `VAULT_TOKEN` for in-process consumers that still
/// read the environment directly. No token is ever written to `.env`.
pub fn load_token_into_env() -> Result<()> {
    if std::env::var("VAULT_TOKEN")
        .ok()
        .is_some_and(|value| !value.is_empty())
    {
        return Ok(());
    }
    if let Ok(token) = read(VAULT_TOKEN_SECRET) {
        std::env::set_var("VAULT_TOKEN", token);
    }
    Ok(())
}

/// Replace all persisted Vault credential lines, keeping unrelated settings.
pub fn write_env_file(env_file: &Path, vault_addr: &str, ca_cert: Option<&Path>) -> Result<()> {
    let lines = env_lines(vault_addr, ca_cert)?;
    let existing = std::fs::read_to_string(env_file).unwrap_or_else(|e| {
        log::warn!("Failed to read {}: {e}", env_file.display());
        String::new()
    });
    let kept: Vec<&str> = existing
        .lines()
        .filter(|line| {
            !line.starts_with("# Vault Configuration")
                && !line.starts_with("VAULT_ADDR=")
                && !line.starts_with("VAULT_TOKEN=")
                && !line.starts_with("VAULT_TOKEN_FILE=")
                && !line.starts_with("VAULT_CACERT=")
        })
        .collect();
    let mut content = kept.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    content.push_str(&lines);
    write_private(env_file, &content)
}
