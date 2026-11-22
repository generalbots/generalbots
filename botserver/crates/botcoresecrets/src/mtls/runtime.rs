//! Runtime transport adapters for the resolved mTLS configuration (#2590).
//!
//! These helpers only rewrite URL strings and carry file *paths*. They never
//! read, copy or log certificate/key bytes. A plaintext configuration is a
//! no-op so single-host/desktop-local installs keep their loopback behaviour.

use super::config::{MtlsConfig, RolloutStage, TransportMode};

/// True when data clients must use a verified TLS transport.
#[must_use]
pub fn requires_tls(config: &MtlsConfig) -> bool {
    config.mode == TransportMode::MutualTls && config.stage >= RolloutStage::ServerTls
}

/// Client-side material paths (never contents), if a client must present a
/// certificate. `None` means "keep the current unauthenticated client".
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientTlsPaths {
    /// CA bundle used to verify the peer server certificate.
    pub ca_cert: String,
    /// PEM certificate presented by this client.
    pub client_cert: String,
    /// PEM private key paired with `client_cert`.
    pub client_key: String,
}

impl ClientTlsPaths {
    /// Select the paths relevant to the configured rollout stage.
    #[must_use]
    pub fn from_config(config: &MtlsConfig) -> Option<Self> {
        if !requires_tls(config) {
            return None;
        }
        let paths = Self {
            ca_cert: config.ca_cert_path.clone(),
            client_cert: config.client_cert_path.clone(),
            client_key: config.client_key_path.clone(),
        };
        (paths.has_ca() || paths.has_identity()).then_some(paths)
    }

    /// Whether a custom CA must be added to the trust store.
    #[must_use]
    pub fn has_ca(&self) -> bool {
        !self.ca_cert.is_empty()
    }

    /// Whether a client identity is available at this stage.
    #[must_use]
    pub fn has_identity(&self) -> bool {
        !self.client_cert.is_empty() && !self.client_key.is_empty()
    }
}

/// Percent-encode a value for a libpq connection URI. Unreserved bytes and the
/// path separator are kept so certificate paths remain readable in diagnostics
/// that do not print the value itself.
fn encode_uri_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(char::from(byte));
            }
            other => {
                out.push('%');
                out.push_str(&format!("{other:02X}"));
            }
        }
    }
    out
}

/// Append fixed query parameters to a connection URL without touching any
/// existing query string.
fn append_params(base: &str, params: &[(&str, String)]) -> String {
    if params.is_empty() {
        return base.to_string();
    }
    let mut out = base.to_string();
    let mut separator = if base.contains('?') { '&' } else { '?' };
    for (key, value) in params {
        out.push(separator);
        separator = '&';
        out.push_str(key);
        out.push('=');
        out.push_str(&encode_uri_value(value));
    }
    out
}

/// Build the libpq connection URL for the configured transport.
///
/// `verify-full` requires a CA; the client certificate/key are added only once
/// the rollout reaches client-certificate verification.
#[must_use]
pub fn secure_database_url(base: &str, config: &MtlsConfig) -> String {
    if !requires_tls(config) {
        return base.to_string();
    }
    let mut params: Vec<(&str, String)> = vec![("sslmode", "verify-full".to_string())];
    if !config.ca_cert_path.is_empty() {
        params.push(("sslrootcert", config.ca_cert_path.clone()));
    }
    if config.stage >= RolloutStage::ClientCertVerification {
        if !config.client_cert_path.is_empty() {
            params.push(("sslcert", config.client_cert_path.clone()));
        }
        if !config.client_key_path.is_empty() {
            params.push(("sslkey", config.client_key_path.clone()));
        }
    }
    append_params(base, &params)
}

/// Build the cache (Redis/Valkey) URL for the configured transport.
#[must_use]
pub fn secure_cache_url(base: &str, config: &MtlsConfig) -> String {
    if !requires_tls(config) {
        return base.to_string();
    }
    match base.strip_prefix("redis://") {
        Some(rest) => format!("rediss://{rest}"),
        None => base.to_string(),
    }
}

/// Build an internal HTTP service URL for the configured transport.
#[must_use]
pub fn secure_http_url(base: &str, config: &MtlsConfig) -> String {
    if !requires_tls(config) {
        return base.to_string();
    }
    match base.strip_prefix("http://") {
        Some(rest) => format!("https://{rest}"),
        None => base.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mtls::config::TransportMode;
    use std::collections::BTreeMap;

    fn config(mode: &str, stage: &str) -> MtlsConfig {
        let mut vault = BTreeMap::new();
        vault.insert("mtls_mode".to_string(), mode.to_string());
        vault.insert("mtls_stage".to_string(), stage.to_string());
        vault.insert("mtls_ca_cert".to_string(), "/vault/ca.pem".to_string());
        vault.insert(
            "mtls_server_cert".to_string(),
            "/vault/server.pem".to_string(),
        );
        vault.insert(
            "mtls_server_key".to_string(),
            "/vault/server.key".to_string(),
        );
        vault.insert(
            "mtls_client_cert".to_string(),
            "/vault/client.pem".to_string(),
        );
        vault.insert(
            "mtls_client_key".to_string(),
            "/vault/client.key".to_string(),
        );
        MtlsConfig::from_sources(&vault, &BTreeMap::new())
    }

    #[test]
    fn plaintext_config_leaves_all_urls_untouched() {
        let plain = config("plaintext", "plaintext-disabled");
        assert!(!requires_tls(&plain));
        let url = "postgres://user:pass@host:5432/db";
        assert_eq!(secure_database_url(url, &plain), url);
        assert_eq!(
            secure_cache_url("redis://host:6379/0", &plain),
            "redis://host:6379/0"
        );
        assert_eq!(
            secure_http_url("http://service:8080", &plain),
            "http://service:8080"
        );
        assert!(ClientTlsPaths::from_config(&plain).is_none());
        let default = MtlsConfig::default();
        assert!(!requires_tls(&default));
        assert_eq!(secure_database_url(url, &default), url);
        assert_eq!(
            secure_http_url("http://service:8080", &default),
            "http://service:8080"
        );
    }

    #[test]
    fn server_tls_forces_verified_database_url_without_client_cert() {
        let secure = config("mtls", "server-tls");
        let url = secure_database_url("postgres://user:pass@host:5432/db", &secure);
        assert_eq!(
            url, "postgres://user:pass@host:5432/db?sslmode=verify-full&sslrootcert=/vault/ca.pem",
            "credentials preserved, sslmode=verify-full and sslrootcert appended"
        );
        assert!(url.contains("sslmode=verify-full"));
        assert!(url.contains("sslrootcert=/vault/ca.pem"));
        assert!(!url.contains("sslcert="));
        assert!(!url.contains("sslkey="));
        let paths = ClientTlsPaths::from_config(&secure);
        assert!(paths.is_some());
        if let Some(paths) = paths {
            assert!(paths.has_ca());
            assert!(!paths.has_identity());
        }
    }

    #[test]
    fn client_cert_verification_adds_identity_paths() {
        let secure = config("mtls", "client-cert-verification");
        let url = secure_database_url("postgresql://host/db?application_name=gb", &secure);
        assert!(
            url.contains("?application_name=gb&"),
            "existing query preserved: {url}"
        );
        assert!(url.contains("sslcert=/vault/client.pem"));
        assert!(url.contains("sslkey=/vault/client.key"));
        let paths = ClientTlsPaths::from_config(&secure);
        assert!(paths.is_some());
        if let Some(paths) = paths {
            assert!(paths.has_identity());
        }
    }

    #[test]
    fn cache_and_http_urls_upgrade_scheme_only() {
        let secure = config("mtls", "server-tls");
        assert!(requires_tls(&secure));
        assert_eq!(
            secure_cache_url("redis://:secret@cache:6379/2", &secure),
            "rediss://:secret@cache:6379/2"
        );
        assert_eq!(
            secure_cache_url("redis://appuser:apppass@cache:6379/2", &secure),
            "rediss://appuser:apppass@cache:6379/2"
        );
        assert_eq!(
            secure_http_url("http://service.internal:8080/api", &secure),
            "https://service.internal:8080/api"
        );
        assert_eq!(
            secure_http_url("https://service:8080", &secure),
            "https://service:8080"
        );
        assert_eq!(
            secure_cache_url("unix:///tmp/redis.sock", &secure),
            "unix:///tmp/redis.sock"
        );
    }

    #[test]
    fn mode_parser_drives_transport_selection() {
        assert_eq!(
            config("required", "server-tls").mode,
            TransportMode::MutualTls
        );
        assert!(!requires_tls(&config("mtls", "cert-provisioning")));
        assert!(!requires_tls(&config("plaintext", "plaintext-disabled")));
        assert!(requires_tls(&config("mtls", "server-tls")));
        assert!(requires_tls(&config("mtls", "client-cert-verification")));
        assert!(requires_tls(&config("mtls", "plaintext-disabled")));
        assert!(ClientTlsPaths::from_config(&config("mtls", "cert-provisioning")).is_none());
    }

    #[test]
    fn validation_errors_never_leak_certificate_paths_or_bytes() {
        let sentinel = "/tmp/gb-mtls-SECRET-MARKER/ca.pem";
        let secret_bytes = "-----BEGIN CERTIFICATE-----\nSECRET\n-----END CERTIFICATE-----";

        let mut missing = BTreeMap::new();
        missing.insert("mtls_mode".to_string(), "mtls".to_string());
        missing.insert("mtls_stage".to_string(), "server-tls".to_string());
        let message = match MtlsConfig::from_sources(&missing, &BTreeMap::new()).validate() {
            Ok(()) => "unexpectedly valid".to_string(),
            Err(error) => error.to_string(),
        };
        assert!(message.contains("ca_cert"), "{message}");
        assert!(message.contains("server_cert"), "{message}");
        assert!(!message.contains('/'), "{message}");
        assert!(!message.contains(secret_bytes), "{message}");

        let mut bad_file = BTreeMap::new();
        bad_file.insert("mtls_mode".to_string(), "mtls".to_string());
        bad_file.insert("mtls_stage".to_string(), "server-tls".to_string());
        bad_file.insert("mtls_ca_cert".to_string(), sentinel.to_string());
        bad_file.insert(
            "mtls_server_cert".to_string(),
            "/definitely/not/a/generalbots/server.pem".to_string(),
        );
        bad_file.insert(
            "mtls_server_key".to_string(),
            "/definitely/not/a/generalbots/server.key".to_string(),
        );
        let message = match MtlsConfig::from_sources(&bad_file, &BTreeMap::new()).validate() {
            Ok(()) => "unexpectedly valid".to_string(),
            Err(error) => error.to_string(),
        };
        assert!(message.starts_with("mTLS "), "{message}");
        assert!(!message.contains(sentinel), "{message}");
        assert!(!message.contains("SECRET-MARKER"), "{message}");
        assert!(!message.contains(secret_bytes), "{message}");
    }

    #[test]
    fn uri_values_are_percent_encoded() {
        let mut vault = BTreeMap::new();
        vault.insert("mtls_mode".to_string(), "mtls".to_string());
        vault.insert("mtls_stage".to_string(), "server-tls".to_string());
        vault.insert("mtls_ca_cert".to_string(), "/vault/my ca.pem".to_string());
        let secure = MtlsConfig::from_sources(&vault, &BTreeMap::new());
        let url = secure_database_url("postgres://host/db", &secure);
        assert!(url.contains("sslrootcert=/vault/my%20ca.pem"));
    }
}
