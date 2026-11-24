//! Bootstrap wiring for internal service-to-service mTLS (#2590).
//!
//! Only containerized installs with provisioned certificate material switch
//! their internal HTTP clients to HTTPS + client certificates. Desktop-local
//! and single-host localhost installs are intentionally left alone: the TLS
//! integration stays uninitialized, so every existing plain loopback client
//! keeps its current behavior.

use crate::core::package_manager::InstallMode;
use std::path::PathBuf;

/// Initialize the `botsecurity` TLS integration from bootstrap state.
///
/// `is_remote_vault` is treated as a containerized-install signal because
/// production container boots do not pass `--container`; a remote Vault means
/// services run in separate containers behind their own certificates.
pub fn init_internal_tls(install_mode: &InstallMode, is_remote_vault: bool) {
    let cert_dir = PathBuf::from(botcore::shared::utils::get_stack_path())
        .join("conf/system/certificates");
    let containerized = matches!(install_mode, InstallMode::Container) || is_remote_vault;
    let mtls_enabled = botsecurity::mtls_policy::should_enable_mtls(containerized, &cert_dir);

    if mtls_enabled {
        log::info!(
            "Internal service mTLS enabled (cert dir: {})",
            cert_dir.display()
        );
    }
    if let Err(e) = botsecurity::integration::init_tls_integration(mtls_enabled, Some(cert_dir)) {
        log::warn!("Failed to initialize TLS integration: {e}");
    }
}
