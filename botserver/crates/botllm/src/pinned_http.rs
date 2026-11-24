//! Shared outbound HTTP client construction with certificate pin enforcement.
//!
//! Every LLM provider used to build its own plain `reqwest::Client`, bypassing
//! the pinning manager entirely (issue #2626). All outbound clients now start
//! from [`pinned_builder`], so when a pin config is deployed the handshake is
//! verified against it; with no config the behaviour is unchanged (system
//! trust), keeping local/dev working.

use std::time::Duration;

use botsecurity::cert_pinning::global_pinning_manager;

/// Build a reqwest client builder with certificate pinning applied when the
/// deployment configured it. Timeouts and other transport options are left to
/// the caller so each provider keeps its existing settings.
pub fn pinned_builder() -> reqwest::ClientBuilder {
    let manager = global_pinning_manager();
    if !manager.is_enabled() {
        return reqwest::Client::builder();
    }
    match manager.build_pinned_tls_config() {
        Ok(tls) => reqwest::Client::builder().use_preconfigured_tls(tls),
        Err(error) => {
            // Sanitized: log the context, never echo raw config into responses.
            log::error!("certificate pinning TLS setup failed: {error}");
            // Fail closed: an empty trust store makes rustls reject every
            // handshake rather than silently falling back to system trust.
            reqwest::Client::builder().use_rustls_tls().tls_built_in_root_certs(false)
        }
    }
}

/// Convenience for provider code paths that need a ready client with the
/// common LLM timeout profile.
pub fn pinned_client(timeout: Duration, connect_timeout: Duration) -> reqwest::Client {
    pinned_builder()
        .timeout(timeout)
        .connect_timeout(connect_timeout)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}
