//! #2613 — deploy-gateway listener bind policy.
//!
//! The gateway performs privileged Incus/Caddy operations with no per-user
//! auth or RBAC. Binding it to `0.0.0.0` therefore exposes those operations on
//! every host interface. The default is loopback; external binding requires
//! BOTH an explicit, narrowly-scoped opt-in AND a real per-deployment secret.

use std::net::SocketAddr;

/// Single-purpose opt-in. Deliberately not a generic `BIND_ADDRESS` — an
/// operator must make a conscious, auditable choice to widen exposure.
pub(super) const EXTERNAL_BIND_ENV: &str = "GB_DEPLOY_GATEWAY_EXTERNAL_BIND";

const GATEWAY_PORT: u16 = 5860;

/// Only an explicit `1` / `true` / `yes` opts in. Anything else — including an
/// unset, empty or malformed value — keeps the listener on loopback.
pub(super) fn external_bind_enabled(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim).map(|value| value.to_ascii_lowercase()).as_deref(),
        Some("1" | "true" | "yes")
    )
}

/// Threat model: host-network exposure of an unauthenticated-by-RBAC control
/// plane would let any reachable peer start/stop containers or push artifacts.
/// Loopback is safe by default; `0.0.0.0` is reachable only when the operator
/// explicitly opted in and a strong per-deployment secret was provisioned.
pub(super) fn gateway_bind_addr(external_opt_in: bool, key_configured: bool) -> SocketAddr {
    if external_opt_in && key_configured {
        SocketAddr::from(([0, 0, 0, 0], GATEWAY_PORT))
    } else {
        SocketAddr::from(([127, 0, 0, 1], GATEWAY_PORT))
    }
}

#[cfg(test)]
mod tests {
    use super::{external_bind_enabled, gateway_bind_addr};

    #[test]
    fn defaults_to_loopback() {
        let addr = gateway_bind_addr(false, true);
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert_eq!(addr.port(), 5860);
    }

    #[test]
    fn opt_in_without_configured_key_stays_on_loopback() {
        let addr = gateway_bind_addr(true, false);
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    #[test]
    fn external_bind_requires_both_conditions() {
        let addr = gateway_bind_addr(true, true);
        assert!(addr.ip().is_unspecified());
        assert_eq!(addr.port(), 5860);
    }

    #[test]
    fn only_explicit_values_enable_external_bind() {
        assert!(external_bind_enabled(Some("1")));
        assert!(external_bind_enabled(Some(" true ")));
        assert!(external_bind_enabled(Some("YES")));
        assert!(!external_bind_enabled(None));
        assert!(!external_bind_enabled(Some("")));
        assert!(!external_bind_enabled(Some("false")));
        assert!(!external_bind_enabled(Some("0")));
        assert!(!external_bind_enabled(Some("garbage")));
    }
}
