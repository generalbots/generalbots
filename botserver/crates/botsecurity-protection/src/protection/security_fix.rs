use anyhow::Result;
use serde::Serialize;
use tracing::info;

use super::{
    caddy_hardener::CaddyHardener,
    dns_hardener::DnsHardener,
    fail2ban::Fail2banManager,
    firewall::FirewallManager,
};

#[derive(Debug, Serialize)]
pub struct SecurityFixReport {
    pub firewall: StepResult,
    pub fail2ban: StepResult,
    pub fail2ban_proxy: StepResult,
    pub caddy: StepResult,
    pub dns: StepResult,
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct StepResult {
    pub ok: bool,
    pub output: String,
}

impl StepResult {
    fn from(result: Result<String>) -> Self {
        match result {
            Ok(output) => Self { ok: true, output },
            Err(e) => Self {
                ok: false,
                output: e.to_string(),
            },
        }
    }
}

/// Run all security hardening steps in sequence.
/// Each step is independent — a failure in one does not abort the others.
pub async fn run_security_fix() -> SecurityFixReport {
    info!("Starting security fix: firewall");
    let firewall = StepResult::from(FirewallManager::apply().await);

    info!("Starting security fix: fail2ban (host/email)");
    let fail2ban = StepResult::from(Fail2banManager::apply().await);

    info!("Starting security fix: fail2ban proxy (caddy-http-flood)");
    let fail2ban_proxy = StepResult::from(Fail2banManager::apply_proxy().await);

    info!("Starting security fix: caddy hardening");
    let caddy = StepResult::from(CaddyHardener::apply().await);

    info!("Starting security fix: CoreDNS hardening (ACL + errors)");
    let dns = StepResult::from(
        DnsHardener::apply(&["pragmatismo.com.br", "ddsites.com.br"], "82.29.59.188").await,
    );

    let success = firewall.ok && fail2ban.ok && fail2ban_proxy.ok && caddy.ok && dns.ok;

    SecurityFixReport {
        firewall,
        fail2ban,
        fail2ban_proxy,
        caddy,
        dns,
        success,
    }
}

/// Run status check across all security components.
pub async fn run_security_status() -> SecurityFixReport {
    let firewall = StepResult::from(FirewallManager::status().await);
    let fail2ban = StepResult::from(Fail2banManager::status().await);
    let fail2ban_proxy = StepResult::from(Fail2banManager::status().await);
    let caddy = StepResult::from(CaddyHardener::status().await);
    let dns = StepResult::from(DnsHardener::status().await);
    let success = firewall.ok && fail2ban.ok && caddy.ok && dns.ok;

    SecurityFixReport {
        firewall,
        fail2ban,
        fail2ban_proxy,
        caddy,
        dns,
        success,
    }
}
