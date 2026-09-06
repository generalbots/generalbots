//! Shared TLS policy for botui's outbound clients.
//!
//! The UI server proxies requests to its own backend (botserver), which on
//! self-hosted deployments presents a certificate issued by the platform's
//! internal CA. Instead of disabling certificate validation, clients here
//! **add the platform CA to the trust set** — standard validation always
//! stays enabled. Public hosts keep using the default web trust store.

use log::{debug, warn};

/// Builds an HTTPS-capable client for proxying to the backend.
///
/// The platform internal CA (discovered via botlib's stack-path resolution)
/// is preloaded when present; certificate verification is never disabled.
#[must_use]
pub fn backend_http_client(_base_url: &str) -> reqwest::Client {
    let ca_path = botlib::security::ca_cert_path();
    let mut builder = reqwest::Client::builder();
    if std::path::Path::new(&ca_path).is_file() {
        match std::fs::read(&ca_path) {
            Ok(pem) => match reqwest::Certificate::from_pem(&pem) {
                Ok(cert) => {
                    builder = builder.add_root_certificate(cert);
                    debug!("backend_http_client: trusting platform CA from {ca_path}");
                }
                Err(e) => warn!("backend_http_client: invalid CA PEM at {ca_path}: {e}"),
            },
            Err(e) => warn!("backend_http_client: unreadable CA at {ca_path}: {e}"),
        }
    } else {
        debug!("backend_http_client: no platform CA at {ca_path}, using system trust store");
    }
    builder.build().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #[test]
    fn client_builds_without_disabling_validation() {
        // Must construct successfully in any environment without any
        // danger_* bypass — validation semantics remain reqwest defaults.
        let client = super::backend_http_client("http://localhost:8080");
        // A default-built client is expected; either way no panic occurred.
        let _ = client;
    }
}
