use anyhow::{Context, Result};
use tracing::info;

use crate::security::command_guard::SafeCommand;

/// UFW rules to apply on the host.
/// Mail ports are included for the email container passthrough.
const ALLOWED_TCP: &[u16] = &[22, 80, 443, 25, 465, 587, 993, 995, 143, 110, 4190];

pub struct FirewallManager;

impl FirewallManager {
    /// Install ufw if missing, apply deny-all + allow rules, enable.
    pub async fn apply() -> Result<String> {
        let mut log = String::new();

        Self::install_ufw(&mut log).await?;
        Self::configure_rules(&mut log).await?;
        Self::enable_ufw(&mut log).await?;

        Ok(log)
    }

    pub async fn status() -> Result<String> {
        let out = SafeCommand::new("sudo")?
            .arg("ufw")?
            .arg("status")?
            .arg("verbose")?
            .execute()
            .context("ufw status failed")?;
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    async fn install_ufw(log: &mut String) -> Result<()> {
        // Check if already installed
        let which = SafeCommand::new("which")
            .and_then(|c| c.arg("ufw"))
            .and_then(|c| c.execute());

        if which.map(|o| o.status.success()).unwrap_or(false) {
            log.push_str("ufw already installed\n");
            return Ok(());
        }

        info!("Installing ufw");
        SafeCommand::new("sudo")?
            .arg("apt-get")?
            .arg("install")?
            .arg("-y")?
            .arg("ufw")?
            .execute()
            .context("apt-get install ufw failed")?;

        log.push_str("ufw installed\n");
        Ok(())
    }

    async fn configure_rules(log: &mut String) -> Result<()> {
        // Reset to clean state (non-interactive)
        SafeCommand::new("sudo")?
            .arg("ufw")?
            .arg("--force")?
            .arg("reset")?
            .execute()
            .context("ufw reset failed")?;

        // Default deny incoming, allow outgoing
        SafeCommand::new("sudo")?
            .arg("ufw")?
            .arg("default")?
            .arg("deny")?
            .arg("incoming")?
            .execute()?;

        SafeCommand::new("sudo")?
            .arg("ufw")?
            .arg("default")?
            .arg("allow")?
            .arg("outgoing")?
            .execute()?;

        // Allow LXC bridge traffic (containers talk to each other)
        SafeCommand::new("sudo")?
            .arg("ufw")?
            .arg("allow")?
            .arg("in")?
            .arg("on")?
            .arg("lxdbr0")?
            .execute()
            .ok(); // non-fatal if bridge name differs

        for port in ALLOWED_TCP {
            SafeCommand::new("sudo")?
                .arg("ufw")?
                .arg("allow")?
                .arg(format!("{port}/tcp").as_str())?
                .execute()
                .with_context(|| format!("ufw allow {port}/tcp failed"))?;
            log.push_str(&format!("allowed {port}/tcp\n"));
        }

        Ok(())
    }

    async fn enable_ufw(log: &mut String) -> Result<()> {
        SafeCommand::new("sudo")?
            .arg("ufw")?
            .arg("--force")?
            .arg("enable")?
            .execute()
            .context("ufw enable failed")?;

        log.push_str("ufw enabled\n");
        info!("Firewall enabled");
        Ok(())
    }
}
