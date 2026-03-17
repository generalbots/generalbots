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
