//! Status snapshot and bootstrap hook for the resolved mTLS configuration.

use super::config::{MtlsConfig, RolloutStage, TransportMode};
use anyhow::{anyhow, Result};
use log::info;
use std::sync::OnceLock;

/// Status snapshot for startup diagnostics and admin reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MtlsStatus {
    /// This build carries the mTLS configuration plumbing (#2590).
    pub supported: bool,
    /// True only after explicit opt-in; never the default or desktop-local.
    pub enabled: bool,
    pub mode: TransportMode,
    pub stage: RolloutStage,
}

impl Default for MtlsStatus {
    fn default() -> Self {
        Self {
            supported: true,
            enabled: false,
            mode: TransportMode::Plaintext,
            stage: RolloutStage::CertProvisioning,
        }
    }
}

static CONFIG: OnceLock<Result<MtlsConfig, String>> = OnceLock::new();

/// Bootstrap hook: resolve and cache the configuration once. Subsequent calls
/// return the same snapshot, so a late environment change cannot half-enable a
/// transport after startup.
pub fn init() -> Result<&'static MtlsConfig> {
    let cached = CONFIG.get_or_init(|| match MtlsConfig::load() {
        Ok(config) => {
            info!(
                "[mtls] configuration loaded: mode={} stage={}",
                config.mode.as_str(),
                config.stage.as_str()
            );
            Ok(config)
        }
        Err(error) => {
            log::error!("[mtls] configuration could not be loaded: {error}");
            Err(error.to_string())
        }
    });
    match cached {
        Ok(config) => Ok(config),
        Err(message) => Err(anyhow!("mTLS configuration unavailable: {message}")),
    }
}

/// Capability flag for admin/status reporting. The configuration plumbing is
/// present in this build; it does not imply any data client is hardened yet.
#[must_use]
pub fn is_supported() -> bool {
    true
}

/// Effective `mtls_enabled` value for reporting: true only after explicit
/// opt-in and a successful bootstrap resolution.
#[must_use]
pub fn is_enabled() -> bool {
    match init() {
        Ok(config) => config.mode == TransportMode::MutualTls,
        Err(_) => false,
    }
}

/// Current status for startup diagnostics. Never panics; a resolution failure
/// is reported as "supported but not enabled".
#[must_use]
pub fn current_status() -> MtlsStatus {
    match init() {
        Ok(config) => config.status(),
        Err(_) => MtlsStatus::default(),
    }
}
