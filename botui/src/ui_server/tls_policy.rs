//! Shared TLS policy for botui's outbound proxy clients.
//!
//! The UI server proxies requests to its own backend (botserver), which on
//! self-hosted deployments presents an internal certificate. Certificate
//! validation may only be relaxed for loopback/private backends — traffic
//! that never crosses an untrusted network. Public hosts always get full
//! validation.

use log::warn;
use url::Url;

/// Whether certificate errors may be tolerated for this backend URL.
///
/// Allowed only when the traffic cannot cross an untrusted network:
/// loopback addresses and RFC 1918 / link-local / unique-local targets.
fn tolerant_tls_allowed_for(base: &str) -> bool {
    let Ok(parsed) = Url::parse(base) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let host = host.trim_start_matches('[').trim_end_matches(']');
    host == "localhost"
        || host.starts_with("127.")
        || host == "::1"
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("169.254.")
        || is_private_172(host)
        || host.starts_with("fc")
        || host.starts_with("fd")
        || host.starts_with("fe80")
}

/// 172.16.0.0/12 check (172.16.x.x through 172.31.x.x).
fn is_private_172(host: &str) -> bool {
    host.strip_prefix("172.")
        .and_then(|rest| rest.split('.').next())
        .and_then(|octet| octet.parse::<u8>().ok())
        .is_some_and(|o| (16..=31).contains(&o))
}

/// Builds an HTTPS-capable client for proxying to the backend.
///
/// Internal certificates are accepted only for loopback/private backends;
/// the choice is logged so operators can spot unexpected configurations.
#[must_use]
pub fn backend_http_client(base_url: &str) -> reqwest::Client {
    let allow_insecure = tolerant_tls_allowed_for(base_url);
    if allow_insecure {
        warn!("Backend uses internal TLS: accepting internal certificates for {base_url}");
    }
    let mut builder = reqwest::Client::builder();
    if allow_insecure {
        builder = builder.danger_accept_invalid_certs(true);
    }
    builder.build().unwrap_or_default()
}

/// Whether an internal-certificate WebSocket connection to this URL is
/// acceptable (loopback/private targets only). Used by the WS proxies.
#[must_use]
pub fn ws_insecure_tls_allowed(backend_url: &str) -> bool {
    tolerant_tls_allowed_for(backend_url)
}

#[cfg(test)]
mod tests {
    use super::tolerant_tls_allowed_for as tolerant;

    #[test]
    fn allows_loopback() {
        assert!(tolerant("http://localhost:8080"));
        assert!(tolerant("https://127.0.0.1:8080"));
        assert!(tolerant("https://[::1]:8080"));
    }

    #[test]
    fn allows_private_ranges() {
        assert!(tolerant("https://10.0.3.10:8080"));
        assert!(tolerant("https://192.168.1.5:8080"));
        assert!(tolerant("https://172.20.0.2:8080"));
    }

    #[test]
    fn rejects_public_hosts() {
        assert!(!tolerant("https://api.generalbots.org"));
        assert!(!tolerant("https://chat.pragmatismo.com.br"));
        assert!(!tolerant("https://172.32.0.1"));
        assert!(!tolerant("https://11.0.0.1"));
    }

    #[test]
    fn rejects_garbage() {
        assert!(!tolerant("not a url"));
        assert!(!tolerant(""));
    }
}
