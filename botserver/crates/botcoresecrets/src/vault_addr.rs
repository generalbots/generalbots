//! Single source of truth for the local Vault address (#1960).
//!
//! The desktop stack starts Vault with `tls_disable = true` on Windows: no
//! client or CA certificates are provisioned there, and `register_secrets`
//! writes `api_addr = "http://127.0.0.1:8200"`. Any caller that guesses
//! `https://` therefore probes a TLS listener that does not exist and times
//! out — the health check reports "vault not running" on a perfectly healthy
//! process.
//!
//! Linux and macOS keep the TLS listener, so the scheme is platform-specific
//! here and nowhere else.

/// Address of the local Vault listener for the current platform.
#[must_use]
pub fn local_vault_addr() -> &'static str {
    if cfg!(target_os = "windows") {
        "http://127.0.0.1:8200"
    } else {
        "https://127.0.0.1:8200"
    }
}

/// Resolves an explicit `VAULT_ADDR` value, falling back to the platform
/// default. Blank values are treated as "not set", which is what the wizard
/// produces on a first run.
#[must_use]
pub fn pick_vault_addr(explicit: Option<&str>) -> String {
    explicit
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| local_vault_addr().to_string())
}

/// `VAULT_ADDR` from the environment, or the platform default when unset.
#[must_use]
pub fn vault_addr_from_env() -> String {
    pick_vault_addr(std::env::var("VAULT_ADDR").ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_addr_matches_platform_tls_setting() {
        let addr = local_vault_addr();
        if cfg!(target_os = "windows") {
            assert_eq!(addr, "http://127.0.0.1:8200");
        } else {
            assert_eq!(addr, "https://127.0.0.1:8200");
        }
    }

    #[test]
    fn explicit_address_wins() {
        assert_eq!(
            pick_vault_addr(Some("http://10.0.0.5:8200")),
            "http://10.0.0.5:8200"
        );
    }

    #[test]
    fn missing_or_blank_address_falls_back() {
        assert_eq!(pick_vault_addr(None), local_vault_addr());
        assert_eq!(pick_vault_addr(Some("   ")), local_vault_addr());
    }
}
