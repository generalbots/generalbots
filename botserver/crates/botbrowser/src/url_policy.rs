//! Shared navigation URL policy for the in-app browser (#2878).
//!
//! Browser navigation is an SSRF boundary: the target comes from an API caller
//! (or a VM tab), Chromium follows it — including redirects — and the
//! `/extract`, `/screenshot` and `/stream` routes then return the page contents.
//! Every navigation therefore goes through [`validate_navigation_target`]
//! before `page.goto`, and the *final* URL is validated through the same
//! function again before any content is read.
//!
//! The static/network checks are layered on the shared SSRF guard
//! (`botsecurity_core::validation`): http(s) only, no userinfo, no obfuscated
//! or non-global IP literals, no local/metadata hostnames, no control-plane
//! ports, and DNS answers must all be global.

use botsecurity_core::validation::{resolve_checked, validate_url_ssrf_strict};
use url::Url;

/// Generic reason returned to API callers. Never echo the rejected host: this
/// string crosses the HTTP boundary and must not become an SSRF oracle.
const POLICY_REJECTION: &str = "navigation target rejected by URL policy";

/// Exact hostnames that are internal admin/control-plane surfaces even when a
/// resolver happens to point them at a global address. The shared guard already
/// lists the loopback/metadata names; these cover the platform's own services.
const INTERNAL_ADMIN_HOSTS: &[&str] = &[
    "alm",
    "forgejo",
    "botserver",
    "admin",
    "console",
    "vault",
    "otterio",
    "kubernetes",
];

/// Subdomains reserved for this platform's admin surfaces.
const INTERNAL_ADMIN_HOST_PREFIXES: &[&str] = &["alm.", "forgejo.", "botserver."];

/// Suffixes that only resolve through a local search domain or a cluster.
const INTERNAL_ADMIN_HOST_SUFFIXES: &[&str] = &[
    ".internal",
    ".local",
    ".localhost",
    ".localdomain",
    ".home.arpa",
    ".cluster.local",
    ".svc",
];

/// Ports that expose a General Bots admin/control-plane HTTP surface. An admin
/// path on one of these ports is off-limits even on a global-looking host.
const ADMIN_PORTS: &[u16] = &[3000, 4000, 4747, 5000, 5858, 8080];

/// Path prefixes that only exist on our own admin surfaces. They are refused
/// only in combination with an [`ADMIN_PORTS`] port, so public sites that
/// legitimately expose `/api/...` or `/admin` on 80/443 remain navigable.
const ADMIN_PATH_PREFIXES: &[&str] = &[
    "/api/",
    "/admin",
    "/cloud/partials/",
    "/suite/admin/",
    "/manage",
    "/console",
    "/dashboard",
    "/vault/",
];

/// Static check: scheme, credentials, host shape and the internal-admin
/// denylist. No DNS, so it is safe to call from synchronous request handlers.
/// The returned [`Url`] is the normalized target callers should connect to.
pub(crate) fn validate_navigation_url(raw: &str) -> Result<Url, String> {
    let parsed = match validate_url_ssrf_strict(raw) {
        Ok(url) => url,
        Err(e) => {
            log::warn!("Blocked browser navigation URL: {}", e.detail());
            return Err(POLICY_REJECTION.to_string());
        }
    };
    if is_internal_admin_url(&parsed) {
        log::warn!("Blocked browser navigation to internal admin URL");
        return Err(POLICY_REJECTION.to_string());
    }
    Ok(parsed)
}

/// Full check used around `page.goto`: the static policy plus DNS resolution,
/// so a hostname that answers with any loopback/private/link-local/metadata
/// address is refused before Chromium connects (and before a redirect is
/// followed, by re-running it on the final URL).
pub(crate) async fn validate_navigation_target(raw: &str) -> Result<Url, String> {
    let parsed = validate_navigation_url(raw)?;
    if let Err(e) = resolve_checked(&parsed).await {
        log::warn!(
            "Blocked browser navigation after DNS resolution: {}",
            e.detail()
        );
        return Err(POLICY_REJECTION.to_string());
    }
    Ok(parsed)
}

fn is_internal_admin_url(url: &Url) -> bool {
    let host = match url.host_str() {
        Some(h) => h
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim_end_matches('.')
            .to_ascii_lowercase(),
        None => return true,
    };
    if INTERNAL_ADMIN_HOSTS.contains(&host.as_str())
        || INTERNAL_ADMIN_HOST_PREFIXES
            .iter()
            .any(|prefix| host.starts_with(prefix))
        || INTERNAL_ADMIN_HOST_SUFFIXES
            .iter()
            .any(|suffix| host.ends_with(suffix))
    {
        return true;
    }
    if let Some(port) = url.port_or_known_default() {
        if ADMIN_PORTS.contains(&port) {
            let path = url.path().to_ascii_lowercase();
            if ADMIN_PATH_PREFIXES
                .iter()
                .any(|prefix| path.starts_with(prefix))
            {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Public destinations that must stay navigable.
    const ALLOWED: &[&str] = &[
        "https://example.com/",
        "http://example.org/path?query=1#fragment",
        "https://www.example.com/api/docs", // public API docs on 443, not our admin port
        "https://chat.example.com:8080/",   // admin port but no admin path
    ];

    /// Adversarial corpus: schemes, credentials, loopback/private/link-local/
    /// metadata/obfuscated literals and our own admin surfaces.
    const BLOCKED: &[&str] = &[
        "file:///etc/passwd",
        "ftp://example.com/file",
        "javascript:alert(1)",
        "data:text/html,<h1>x</h1>",
        "http://user:pass@example.com/",
        "",
        "http://127.0.0.1/",
        "http://127.0.0.1:8080/admin",
        "http://localhost:3000/suite/admin/index.html",
        "http://[::1]/",
        "http://10.0.0.1/",
        "http://192.168.1.1/",
        "http://172.16.0.1/",
        "http://0.0.0.0/",
        "http://169.254.169.254/latest/meta-data/",
        "http://metadata.google.internal/computeMetadata/v1/",
        "http://[fd00::1]/",
        "http://[fe80::1]/",
        "http://[::ffff:127.0.0.1]/",
        "http://2130706433/",
        "http://0x7f000001/",
        "http://127.1/",
        "http://example.internal/",
        "http://example.local/",
        "http://alm.example.com/admin",
        "http://forgejo.example.com/",
        "http://8.8.8.8:4747/admin",
        "http://8.8.8.8:8080/api/browser/session",
        "http://example.com/a b",
    ];

    #[test]
    fn public_urls_pass_static_policy() {
        for url in ALLOWED {
            assert!(
                validate_navigation_url(url).is_ok(),
                "expected public URL to be allowed: {url}"
            );
        }
    }

    #[test]
    fn adversarial_urls_are_blocked_by_static_policy() {
        for url in BLOCKED {
            assert!(
                validate_navigation_url(url).is_err(),
                "expected adversarial URL to be blocked: {url}"
            );
        }
    }

    #[test]
    fn rejection_reason_never_echoes_the_host() {
        let reason = validate_navigation_url("http://127.0.0.1:8080/admin").unwrap_err();
        assert!(!reason.contains("127.0.0.1"));
        assert!(!reason.contains("8080"));
        assert_eq!(reason, POLICY_REJECTION);
    }

    #[tokio::test]
    async fn full_policy_accepts_a_global_ip_literal() {
        // IP literals skip DNS, so this test is deterministic and offline.
        assert!(validate_navigation_target("http://8.8.8.8/").await.is_ok());
    }

    #[tokio::test]
    async fn full_policy_rejects_loopback_and_admin_urls() {
        for url in [
            "http://127.0.0.1/",
            "http://169.254.169.254/latest/meta-data/",
            "http://8.8.8.8:8080/admin",
        ] {
            assert!(
                validate_navigation_target(url).await.is_err(),
                "expected full policy to block: {url}"
            );
        }
    }
}
