//! URL policy evaluation for agentic browsing steps.

use serde::{Deserialize, Serialize};

pub const TASK_KIND_DEFAULT: &str = "default";
pub const TASK_KIND_CHECKOUT: &str = "checkout";

fn default_max_steps() -> u32 {
    60
}

fn default_max_cost_units() -> u64 {
    10_000
}

/// Browsing policy applied before any step is recorded.
///
/// `allowed_domains` semantics: empty list means allow-all-except-denied;
/// a non-empty list permits exact host matches or dot-suffix subdomains.
/// `denied_domains` always wins over `allowed_domains`.
/// `allow_credential_pages` disables the credential-phishing heuristic guard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyConfig {
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default)]
    pub denied_domains: Vec<String>,
    #[serde(default = "default_max_steps")]
    pub max_steps: u32,
    #[serde(default = "default_max_cost_units")]
    pub max_cost_units: u64,
    #[serde(default)]
    pub allow_credential_pages: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            allowed_domains: Vec::new(),
            denied_domains: Vec::new(),
            max_steps: default_max_steps(),
            max_cost_units: default_max_cost_units(),
            allow_credential_pages: false,
        }
    }
}

impl PolicyConfig {
    /// Extracts the per-task policy embedded in the task `plan` JSONB, or the
    /// documented defaults when absent.
    pub fn from_plan(plan: Option<&serde_json::Value>) -> Self {
        plan.and_then(|p| p.get("policy"))
            .and_then(|v| serde_json::from_value::<Self>(v.clone()).ok())
            .unwrap_or_default()
    }
}

/// True when `host` equals `domain` or is a dot-suffix subdomain of it.
fn host_matches(host: &str, domain: &str) -> bool {
    let host = host.trim_end_matches('.');
    let domain = domain.trim_end_matches('.');
    if domain.is_empty() {
        return false;
    }
    host == domain || host.ends_with(&format!(".{domain}"))
}

/// Parses scheme and host from a URL without external crates. Returns
/// `(scheme_lowercase, host_lowercase)`; malformed input yields `None`.
pub fn parse_url_parts(url: &str) -> Option<(String, String)> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if rest.is_empty() {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next()?;
    let authority = authority.rsplit('@').next()?;
    let host = authority.split(':').next()?;
    if host.is_empty() {
        return None;
    }
    Some((scheme, host.to_ascii_lowercase()))
}

/// Validates scheme and host against the configured domain lists.
///
/// Rules, in order: only `http`/`https` schemes are accepted; any host matching
/// a denied entry (exact or dot-suffix) is rejected; an empty allowed list
/// allows every remaining host; otherwise the host must match an allowed entry.
pub fn url_allowed(cfg: &PolicyConfig, url: &str) -> Result<(), String> {
    let Some((scheme, host)) = parse_url_parts(url) else {
        return Err(format!("malformed url rejected: {url}"));
    };
    if scheme != "http" && scheme != "https" {
        return Err(format!("scheme '{scheme}' not permitted"));
    }
    for denied in &cfg.denied_domains {
        if host_matches(&host, &denied.to_ascii_lowercase()) {
            return Err(format!("host '{host}' is denied by policy"));
        }
    }
    if cfg.allowed_domains.is_empty() {
        return Ok(());
    }
    for allowed in &cfg.allowed_domains {
        if host_matches(&host, &allowed.to_ascii_lowercase()) {
            return Ok(());
        }
    }
    Err(format!("host '{host}' is not in the allowed domains"))
}

/// Credential-phishing heuristic guard.
///
/// Documented simple heuristic: for tasks of kind `checkout`, any URL whose
/// full text contains the substring `login` (case-insensitive, covering path
/// and query) is blocked, because credential surfaces are out of scope for
/// payment flows. The block is lifted when `allow_credential_pages` is set in
/// the policy or when the task kind is not `checkout`. This is intentionally
/// conservative and substring-based; it may over-block paths such as
/// `/blog/login-guide`, which is acceptable for a control plane.
pub fn credential_guard(cfg: &PolicyConfig, url: &str, task_kind: &str) -> Result<(), String> {
    if cfg.allow_credential_pages || task_kind != TASK_KIND_CHECKOUT {
        return Ok(());
    }
    if url.to_ascii_lowercase().contains("login") {
        return Err(
            "credential surface detected ('login' in url) and blocked for checkout tasks"
                .to_string(),
        );
    }
    Ok(())
}

/// Combined pre-recording validation applied by `advance_task`.
pub fn step_allowed(cfg: &PolicyConfig, url: &str, task_kind: &str) -> Result<(), String> {
    url_allowed(cfg, url)?;
    credential_guard(cfg, url, task_kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(allowed: &[&str], denied: &[&str]) -> PolicyConfig {
        PolicyConfig {
            allowed_domains: allowed.iter().map(|s| s.to_string()).collect(),
            denied_domains: denied.iter().map(|s| s.to_string()).collect(),
            ..PolicyConfig::default()
        }
    }

    #[test]
    fn allows_exact_and_subdomain_when_allowlist_present() {
        let c = cfg(&["example.com"], &[]);
        assert!(url_allowed(&c, "https://example.com/a").is_ok());
        assert!(url_allowed(&c, "https://www.example.com/a").is_ok());
        assert!(url_allowed(&c, "https://notexample.com/").is_err());
    }

    #[test]
    fn empty_allowed_list_allows_all_except_denied() {
        let c = cfg(&[], &["evil.com"]);
        assert!(url_allowed(&c, "https://anything.org/").is_ok());
        assert!(url_allowed(&c, "https://sub.evil.com/").is_err());
        assert!(url_allowed(&c, "https://evil.com/").is_err());
    }

    #[test]
    fn denied_wins_over_allowed() {
        let c = cfg(&["example.com"], &["internal.example.com"]);
        assert!(url_allowed(&c, "https://example.com/").is_ok());
        assert!(url_allowed(&c, "https://internal.example.com/").is_err());
    }

    #[test]
    fn rejects_non_http_schemes_and_malformed_urls() {
        let c = cfg(&[], &[]);
        assert!(url_allowed(&c, "file:///etc/passwd").is_err());
        assert!(url_allowed(&c, "javascript:alert(1)").is_err());
        assert!(url_allowed(&c, "ftp://example.com/").is_err());
        assert!(url_allowed(&c, "not-a-url").is_err());
        assert!(url_allowed(&c, "").is_err());
    }

    #[test]
    fn credential_guard_blocks_login_only_for_checkout() {
        let c = PolicyConfig::default();
        assert!(credential_guard(&c, "https://shop.com/login?next=/pay", TASK_KIND_CHECKOUT).is_err());
        assert!(credential_guard(&c, "https://shop.com/pay", TASK_KIND_CHECKOUT).is_ok());
        assert!(credential_guard(&c, "https://shop.com/login", TASK_KIND_DEFAULT).is_ok());
        let open = PolicyConfig {
            allow_credential_pages: true,
            ..PolicyConfig::default()
        };
        assert!(credential_guard(&open, "https://shop.com/login", TASK_KIND_CHECKOUT).is_ok());
    }

    #[test]
    fn parse_url_parts_handles_ports_auth_and_fragments() {
        assert_eq!(
            parse_url_parts("HTTPS://User@Example.COM:8443/p?q=1#f"),
            Some(("https".to_string(), "example.com".to_string()))
        );
        assert_eq!(parse_url_parts("http://localhost:8080"), Some(("http".to_string(), "localhost".to_string())));
    }
}
