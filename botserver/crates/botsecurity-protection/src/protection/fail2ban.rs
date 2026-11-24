use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use tracing::info;

use botsecurity_core::command_guard::SafeCommand;

use super::secure_staging::SecureStaging;

/// Host jail.local installed by the root-owned fail2ban writer. Staged in a
/// private, unpredictable directory before the privileged copy (issue #2608).
const JAIL_LOCAL: &str = r#"[DEFAULT]
bantime  = 1h
findtime = 10m
maxretry = 5
backend  = systemd

[sshd]
enabled  = true
port     = ssh
logpath  = %(sshd_log)s
maxretry = 3

[postfix]
enabled  = true
port     = smtp,465,587
logpath  = %(postfix_log)s

[dovecot]
enabled  = true
port     = pop3,pop3s,imap,imaps,submission,465,sieve
logpath  = %(dovecot_log)s
"#;

/// jail.local for the proxy container (Caddy) — no sshd, only HTTP flood
const PROXY_JAIL_LOCAL: &str = r#"[DEFAULT]
bantime  = 1h
findtime = 10m
maxretry = 5

[caddy-http-flood]
enabled   = true
filter    = caddy
logpath   = /opt/gbo/logs/access.log
maxretry  = 100
findtime  = 60s
bantime   = 1h
action    = iptables-multiport[name=caddy, port="80,443", protocol=tcp]
"#;

/// Disable the debian default sshd jail (proxy has no sshd)
const PROXY_DEFAULTS_DEBIAN: &str = r#"[sshd]
enabled = false
"#;

/// fail2ban filter for Caddy JSON access log
const CADDY_FILTER: &str = r#"[Definition]
failregex = ^.*"remote_ip":"<HOST>".*"status":4[0-9][0-9].*$
            ^.*"client_ip":"<HOST>".*"status":4[0-9][0-9].*$
ignoreregex =
datepattern = {"ts":\s*%%s
"#;

const PROXY_CONTAINER: &str = "pragmatismo-proxy";

/// Pure predicate: is `path` a safe source for the privileged jail copy?
///
/// The file must live directly inside a fresh `SecureStaging` directory
/// (`gb-secure-stage-<uuid>`, mode `0700`) and itself be a `payload-<uuid>`
/// file with mode `0600`. Random, collision-resistant suffixes make the path
/// unguessable; the mode checks prevent a broad/world-readable source from
/// ever being handed to the root-owned writer. No filesystem access here, so
/// the rule can be unit-tested directly (issue #2608).
fn is_safe_privileged_source(path: &Path, dir_mode: u32, file_mode: u32) -> bool {
    if !path.is_absolute() || dir_mode & 0o777 != 0o700 || file_mode & 0o777 != 0o600 {
        return false;
    }
    let (Some(dir_name), Some(file_name)) = (
        path.parent()
            .and_then(Path::file_name)
            .and_then(|n| n.to_str()),
        path.file_name().and_then(|n| n.to_str()),
    ) else {
        return false;
    };
    let (Some(dir_suffix), Some(file_suffix)) = (
        dir_name.strip_prefix("gb-secure-stage-"),
        file_name.strip_prefix("payload-"),
    ) else {
        return false;
    };
    let is_uuid_suffix =
        |s: &str| s.len() >= 16 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    is_uuid_suffix(dir_suffix) && is_uuid_suffix(file_suffix)
}

#[cfg(unix)]
fn staged_permissions(dir: &Path, file: &Path) -> Result<(u32, u32)> {
    use std::os::unix::fs::PermissionsExt;
    let dir_mode = fs::metadata(dir)
        .context("reading staging directory metadata")?
        .permissions()
        .mode();
    let file_mode = fs::metadata(file)
        .context("reading staged file metadata")?
        .permissions()
        .mode();
    Ok((dir_mode, file_mode))
}

#[cfg(not(unix))]
fn staged_permissions(dir: &Path, file: &Path) -> Result<(u32, u32)> {
    Err(anyhow!(
        "privileged jail installation requires POSIX permissions ({} -> {})",
        dir.display(),
        file.display()
    ))
}

pub struct Fail2banManager;

impl Fail2banManager {
    /// Install fail2ban if missing, write jail config, restart service.
    pub async fn apply() -> Result<String> {
        let mut log = String::new();

        Self::install(&mut log).await?;
        Self::write_jail_config(&mut log).await?;
        Self::restart_service(&mut log).await?;

        Ok(log)
    }

    /// Install and configure fail2ban in the proxy (Caddy) Incus container.
    pub async fn apply_proxy() -> Result<String> {
        let mut log = String::new();

        // Install fail2ban inside the proxy container
        Self::incus_exec(
            PROXY_CONTAINER,
            &["apt-get", "install", "-y", "--fix-missing", "fail2ban"],
        )
        .await
        .context("failed to install fail2ban in proxy container")?;

        // Write caddy filter through private staging (#2790)
        let filter = super::private_staging::StagedConfig::create("gb-caddy-filter.conf", CADDY_FILTER.as_bytes())
            .context("failed to stage caddy filter privately")?;
        SafeCommand::new("incus")?
            .arg("file")?
            .arg("push")?
            .arg("/tmp/gb-caddy-filter.conf")?
            .arg(format!(
                "{PROXY_CONTAINER}/etc/fail2ban/filter.d/caddy.conf"
            ))?
            .execute()
            .context("incus file push caddy filter failed")?;

        // Disable default sshd jail (no sshd in proxy)
        let defaults = super::private_staging::StagedConfig::create("gb-proxy-defaults.conf", PROXY_DEFAULTS_DEBIAN.as_bytes())
            .context("failed to stage proxy defaults privately")?;
        SafeCommand::new("incus")?
            .arg("file")?
            .arg("push")?
            .arg("/tmp/gb-proxy-defaults.conf")?
            .arg(format!(
                "{PROXY_CONTAINER}/etc/fail2ban/jail.d/defaults-debian.conf"
            ))?
            .execute()
            .context("incus file push proxy defaults failed")?;

        // Write proxy jail.local
        let jail = super::private_staging::StagedConfig::create("gb-proxy-jail.local", PROXY_JAIL_LOCAL.as_bytes())
            .context("failed to stage proxy jail.local privately")?;
        SafeCommand::new("incus")?
            .arg("file")?
            .arg("push")?
            .arg(jail.as_path().to_string_lossy().as_ref())?
            .arg(format!("{PROXY_CONTAINER}/etc/fail2ban/jail.local"))?
            .execute()
            .context("incus file push proxy jail.local failed")?;

        // Enable and restart
        Self::incus_exec(
            PROXY_CONTAINER,
            &["systemctl", "enable", "--now", "fail2ban"],
        )
        .await?;
        Self::incus_exec(PROXY_CONTAINER, &["systemctl", "restart", "fail2ban"]).await?;

        log.push_str("fail2ban configured in proxy container (caddy-http-flood jail)\n");
        info!("fail2ban proxy jail applied");
        Ok(log)
    }

    async fn incus_exec(container: &str, cmd: &[&str]) -> Result<std::process::Output> {
        let mut c = SafeCommand::new("incus")?
            .arg("exec")?
            .arg(container)?
            .arg("--")?;
        for arg in cmd {
            c = c.arg(arg)?;
        }
        c.execute().context("incus exec failed")
    }

    pub async fn status() -> Result<String> {
        let out = SafeCommand::new("sudo")?
            .arg("fail2ban-client")?
            .arg("status")?
            .execute()
            .context("fail2ban-client status failed")?;
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    async fn install(log: &mut String) -> Result<()> {
        let which = SafeCommand::new("which")
            .and_then(|c| c.arg("fail2ban-client"))
            .and_then(|c| c.execute());

        if which.map(|o| o.status.success()).unwrap_or(false) {
            log.push_str("fail2ban already installed\n");
            return Ok(());
        }

        info!("Installing fail2ban");
        SafeCommand::new("sudo")?
            .arg("apt-get")?
            .arg("install")?
            .arg("-y")?
            .arg("fail2ban")?
            .execute()
            .context("apt-get install fail2ban failed")?;

        log.push_str("fail2ban installed\n");
        Ok(())
    }

    async fn write_jail_config(log: &mut String) -> Result<()> {
        // Stage in a fresh, unpredictable 0700 directory with a 0600 file
        // (O_EXCL via create_new). SecureStaging removes the whole directory on
        // every exit path, so no predictable /tmp path is ever handed to a
        // privileged command (issues #2608, #2790).
        let staging = SecureStaging::new()?;
        let staged = staging.stage(JAIL_LOCAL.as_bytes())?;

        // Verify the source before the privileged copy. A local user cannot
        // traverse the 0700 directory, so the file cannot be pre-created,
        // symlinked or swapped between this check and the root-owned writer.
        let (dir_mode, file_mode) = staged_permissions(staging.path(), &staged)?;
        if !is_safe_privileged_source(&staged, dir_mode, file_mode) {
            return Err(anyhow!("refusing privileged copy from unsafe staging path"));
        }

        let staged_arg = staged.to_string_lossy().into_owned();
        SafeCommand::new("sudo")?
            .arg(super::sudoers::JAIL_WRITER_PATH)?
            .arg(&staged_arg)?
            .execute()
            .context("failed to install jail.local via root-owned helper")?;

        log.push_str("jail.local written (ssh + postfix + dovecot jails)\n");
        Ok(())
    }

    async fn restart_service(log: &mut String) -> Result<()> {
        SafeCommand::new("sudo")?
            .arg("systemctl")?
            .arg("enable")?
            .arg("--now")?
            .arg("fail2ban")?
            .execute()
            .context("failed to enable fail2ban")?;

        SafeCommand::new("sudo")?
            .arg("systemctl")?
            .arg("restart")?
            .arg("fail2ban")?
            .execute()
            .context("failed to restart fail2ban")?;

        log.push_str("fail2ban enabled and running\n");
        info!("fail2ban restarted");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAFE_DIR: &str = "gb-secure-stage-550e8400e29b41d4a716446655440000";
    const SAFE_FILE: &str = "payload-550e8400e29b41d4a716446655440000";

    fn staged_path(dir: &str, file: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(dir).join(file)
    }

    #[test]
    fn safe_staged_source_is_accepted() {
        let path = staged_path(SAFE_DIR, SAFE_FILE);
        assert!(is_safe_privileged_source(&path, 0o700, 0o600));
    }

    #[test]
    fn broad_permissions_are_rejected() {
        let path = staged_path(SAFE_DIR, SAFE_FILE);
        assert!(
            !is_safe_privileged_source(&path, 0o755, 0o600),
            "dir must be 0700"
        );
        assert!(
            !is_safe_privileged_source(&path, 0o700, 0o644),
            "file must be 0600"
        );
        assert!(!is_safe_privileged_source(&path, 0o777, 0o666));
    }

    #[test]
    fn predictable_and_non_staged_paths_are_rejected() {
        // The vulnerable pre-fix path must never be accepted as a source.
        assert!(!is_safe_privileged_source(
            &std::env::temp_dir().join("gb-jail.local"),
            0o700,
            0o600
        ));
        assert!(
            !is_safe_privileged_source(
                &staged_path("gb-secure-stage-short", SAFE_FILE),
                0o700,
                0o600
            ),
            "suffix must be unpredictable"
        );
        assert!(
            !is_safe_privileged_source(&staged_path(SAFE_DIR, "jail.local"), 0o700, 0o600),
            "file must be a random payload name"
        );
        assert!(!is_safe_privileged_source(
            Path::new("relative/gb-secure-stage-550e8400e29b41d4a716446655440000/payload-550e8400e29b41d4a716446655440000"),
            0o700,
            0o600
        ), "path must be absolute");
    }
}
