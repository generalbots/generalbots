use anyhow::Result;
use serde::Serialize;
use tracing::info;

use super::{caddy_hardener::CaddyHardener, fail2ban::Fail2banManager, firewall::FirewallManager};

#[derive(Debug, Serialize)]
pub struct SecurityFixReport {
    pub firewall: StepResult,
    pub fail2ban: StepResult,
    pub caddy: StepResult,
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

    info!("Starting security fix: fail2ban");
    let fail2ban = StepResult::from(Fail2banManager::apply().await);

    info!("Starting security fix: caddy hardening");
    let caddy = StepResult::from(CaddyHardener::apply().await);

    let success = firewall.ok && fail2ban.ok && caddy.ok;

    SecurityFixReport {
        firewall,
        fail2ban,
        caddy,
        success,
    }
}

/// Run status check across all security components.
pub async fn run_security_status() -> SecurityFixReport {
    let firewall = StepResult::from(FirewallManager::status().await);
    let fail2ban = StepResult::from(Fail2banManager::status().await);
    let caddy = StepResult::from(CaddyHardener::status().await);
    let success = firewall.ok && fail2ban.ok && caddy.ok;

    SecurityFixReport {
        firewall,
        fail2ban,
        caddy,
        success,
    }
}
