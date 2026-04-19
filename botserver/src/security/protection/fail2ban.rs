use anyhow::{Context, Result};
use tracing::info;

use crate::security::command_guard::SafeCommand;

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

    /// Install and configure fail2ban in the proxy (Caddy) LXC container.
    pub async fn apply_proxy() -> Result<String> {
        let mut log = String::new();

        // Install fail2ban inside the proxy container
        Self::lxc_exec(PROXY_CONTAINER, &["apt-get", "install", "-y", "--fix-missing", "fail2ban"])
            .await
            .context("failed to install fail2ban in proxy container")?;

        // Write caddy filter
        std::fs::write("/tmp/gb-caddy-filter.conf", CADDY_FILTER)
            .context("failed to write caddy filter to /tmp")?;
        SafeCommand::new("lxc")?
            .arg("file")?
            .arg("push")?
            .arg("/tmp/gb-caddy-filter.conf")?
            .arg(&format!("{PROXY_CONTAINER}/etc/fail2ban/filter.d/caddy.conf"))?
            .execute()
            .context("lxc file push caddy filter failed")?;

        // Disable default sshd jail (no sshd in proxy)
        std::fs::write("/tmp/gb-proxy-defaults.conf", PROXY_DEFAULTS_DEBIAN)
            .context("failed to write proxy defaults to /tmp")?;
        SafeCommand::new("lxc")?
            .arg("file")?
            .arg("push")?
            .arg("/tmp/gb-proxy-defaults.conf")?
            .arg(&format!("{PROXY_CONTAINER}/etc/fail2ban/jail.d/defaults-debian.conf"))?
            .execute()
            .context("lxc file push proxy defaults failed")?;

        // Write proxy jail.local
        std::fs::write("/tmp/gb-proxy-jail.local", PROXY_JAIL_LOCAL)
            .context("failed to write proxy jail.local to /tmp")?;
        SafeCommand::new("lxc")?
            .arg("file")?
            .arg("push")?
            .arg("/tmp/gb-proxy-jail.local")?
            .arg(&format!("{PROXY_CONTAINER}/etc/fail2ban/jail.local"))?
            .execute()
            .context("lxc file push proxy jail.local failed")?;

        // Enable and restart
        Self::lxc_exec(PROXY_CONTAINER, &["systemctl", "enable", "--now", "fail2ban"]).await?;
        Self::lxc_exec(PROXY_CONTAINER, &["systemctl", "restart", "fail2ban"]).await?;

        log.push_str("fail2ban configured in proxy container (caddy-http-flood jail)\n");
        info!("fail2ban proxy jail applied");
        Ok(log)
    }

    async fn lxc_exec(container: &str, cmd: &[&str]) -> Result<std::process::Output> {
        let mut c = SafeCommand::new("lxc")?
            .arg("exec")?
            .arg(container)?
            .arg("--")?;
        for arg in cmd {
            c = c.arg(arg)?;
        }
        c.execute().context("lxc exec failed")
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
        std::fs::write("/tmp/gb-jail.local", JAIL_LOCAL)
            .context("failed to write jail config to /tmp")?;

        SafeCommand::new("sudo")?
            .arg("cp")?
            .arg("/tmp/gb-jail.local")?
            .arg("/etc/fail2ban/jail.local")?
            .execute()
            .context("failed to copy jail.local")?;

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
