//! Post-login redirect validation (#2130).
//!
//! The `redirect` query parameter is attacker-controlled and ends up in a
//! `Location` header, which makes it an open-redirect (phishing) primitive.
//! Only two shapes are accepted:
//!
//! * a **relative path** starting with a single `/` — the only case the UI
//!   needs (`/settings`, `/suite/desktop`);
//! * an **absolute http(s) URL whose host is allowlisted** — for deployments
//!   that legitimately hop between suite and cloud origins.
//!
//! Everything else (`//host`, `/\host`, `https://evil.example`, `javascript:`,
//! `data:`, control characters, backslashes) collapses to [`DEFAULT_REDIRECT`].

/// Where a rejected redirect lands.
pub const DEFAULT_REDIRECT: &str = "/";
/// Longest accepted target; anything longer is not a real in-app route.
const MAX_REDIRECT_LEN: usize = 2048;
/// Environment variable with extra allowlisted hosts, comma separated.
const ALLOWED_HOSTS_ENV: &str = "OAUTH_REDIRECT_ALLOWED_HOSTS";

/// Which absolute targets may be redirected to.
#[derive(Debug, Clone, Default)]
pub struct RedirectPolicy {
    allowed_hosts: Vec<String>,
}

impl RedirectPolicy {
    /// Builds a policy from an explicit host allowlist (case-insensitive,
    /// `host` or `host:port`).
    pub fn new(hosts: impl IntoIterator<Item = String>) -> Self {
        let mut allowed_hosts: Vec<String> = hosts
            .into_iter()
            .map(|host| host.trim().to_ascii_lowercase())
            .filter(|host| !host.is_empty())
            .collect();
        allowed_hosts.sort_unstable();
        allowed_hosts.dedup();
        Self { allowed_hosts }
    }

    /// Allowlist derived from the deployment's own base URL, so an absolute
    /// redirect back to this very origin is accepted.
    pub fn from_base_url(base_url: &str) -> Self {
        Self::new(host_of(base_url).into_iter().collect())
    }

    /// Adds the hosts listed in `OAUTH_REDIRECT_ALLOWED_HOSTS`.
    pub fn with_env_allowlist(self) -> Self {
        let Ok(raw) = std::env::var(ALLOWED_HOSTS_ENV) else {
            return self;
        };
        let extra = raw.split(',').map(|host| host.to_string());
        let mut hosts = self.allowed_hosts;
        hosts.extend(Self::new(extra).allowed_hosts);
        Self::new(hosts)
    }

    pub fn allowed_hosts(&self) -> &[String] {
        &self.allowed_hosts
    }

    /// Validates `raw`, returning it when safe or [`DEFAULT_REDIRECT`] when not.
    pub fn sanitize(&self, raw: Option<&str>) -> String {
        let Some(raw) = raw else {
            return DEFAULT_REDIRECT.to_string();
        };
        if raw.is_empty() || raw.len() > MAX_REDIRECT_LEN {
            return DEFAULT_REDIRECT.to_string();
        }
        // Control characters, whitespace and backslashes are never part of a
        // legitimate target, and each of them can be used to smuggle a host
        // (`/\evil.example` is `//evil.example` for browsers).
        if raw
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
        {
            return DEFAULT_REDIRECT.to_string();
        }

        if raw.starts_with('/') {
            // `//host` and `///host` are protocol-relative URLs, not paths.
            if raw.starts_with("//") {
                return DEFAULT_REDIRECT.to_string();
            }
            return raw.to_string();
        }

        match self.allowed_absolute(raw) {
            Some(target) => target,
            None => DEFAULT_REDIRECT.to_string(),
        }
    }

    /// Returns the URL when it is an absolute http(s) target on an allowlisted
    /// host, `None` otherwise.
    fn allowed_absolute(&self, raw: &str) -> Option<String> {
        let (scheme, rest) = raw.split_once("://")?;
        if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
            return None;
        }
        // The authority ends at the first path/query/fragment delimiter.
        let authority_end = rest
            .find(['/', '?', '#'])
            .unwrap_or(rest.len());
        let authority = &rest[..authority_end];
        // Credentials in the authority (`https://allowed@evil`) and empty
        // authorities are refused outright.
        if authority.is_empty() || authority.contains('@') {
            return None;
        }
        let host = authority.to_ascii_lowercase();
        if self.allowed_hosts.iter().any(|allowed| *allowed == host) {
            Some(raw.to_string())
        } else {
            None
        }
    }
}

/// Convenience wrapper for callers that accept relative paths only.
pub fn safe_redirect_target(raw: Option<&str>) -> String {
    RedirectPolicy::default().sanitize(raw)
}

/// Extracts the `host[:port]` from an absolute URL, lowercased.
fn host_of(url: &str) -> Option<String> {
    let (_, rest) = url.split_once("://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    Some(authority.to_ascii_lowercase())
}
