//! #2684 — plain-HTTP fallback must fail closed in a production posture.
//!
//! `listen_impl` used to degrade silently to cleartext HTTP whenever
//! `BOTSERVER_DISABLE_TLS` was set or the API cert/key were missing. HSTS is
//! only a response header (`run_axum_server`): it does nothing when the socket
//! itself is plain HTTP, so a misconfigured production box could serve the
//! full authenticated API, CSRF cookies and Bearer traffic in the clear.
//!
//! The decision is kept pure and unit-tested here. `#2615` defines the posture:
//! production is the default and only an explicit development marker lowers
//! the guard. Plain HTTP is therefore refused unless the operator either
//! started in development posture or set the narrow, auditable override below.

use std::sync::OnceLock;

/// Explicit opt-in for cleartext HTTP even while the posture is production.
/// Deliberately not a generic "TLS=false" flag: an operator must make a
/// conscious, auditable choice (dev box, TLS-terminating proxy in front).
pub(crate) const ALLOW_PLAIN_HTTP_ENV: &str = "BOTSERVER_ALLOW_PLAIN_HTTP";

/// Effective mode of the main API listener. Reported on health/status without
/// any certificate material or paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TlsMode {
    /// HTTPS: certificate and key were present and TLS was not disabled.
    Enabled,
    /// Cleartext HTTP (development posture or explicit override).
    PlainHttp,
}

impl TlsMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::PlainHttp => "plain-http",
        }
    }
}

/// Why production refused to serve plain HTTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TlsGuardError {
    PlainHttpInProduction {
        tls_disabled: bool,
        certs_present: bool,
    },
}

impl std::fmt::Display for TlsGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlainHttpInProduction {
                tls_disabled,
                certs_present,
            } => write!(
                f,
                "production posture requires TLS, but it is not active \
                 (BOTSERVER_DISABLE_TLS={tls_disabled}, certificates present={certs_present}); \
                 install the API certificate/key, set BOTSERVER_ENV=development, or set \
                 {ALLOW_PLAIN_HTTP_ENV}=1 for development only"
            ),
        }
    }
}

/// True only for the explicit `1`/`true`/`yes`/`on` opt-ins, matching the flag
/// shape used by the other security overrides (`GB_ALLOW_LAN`, JWT dev secret).
pub(crate) fn plain_http_override_enabled(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim).map(|value| value.to_ascii_lowercase()).as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
}

/// Pure guard: decide the effective TLS mode, or fail closed.
///
/// - TLS active (not disabled and both cert files present) always wins.
/// - Otherwise cleartext is allowed only outside production posture or with an
///   explicit override; a production bind with no TLS returns an error so the
///   caller refuses to start.
pub(crate) fn decide_tls_mode(
    is_production: bool,
    tls_disabled: bool,
    certs_present: bool,
    plain_http_override: bool,
) -> Result<TlsMode, TlsGuardError> {
    if !tls_disabled && certs_present {
        return Ok(TlsMode::Enabled);
    }
    if !is_production || plain_http_override {
        Ok(TlsMode::PlainHttp)
    } else {
        Err(TlsGuardError::PlainHttpInProduction {
            tls_disabled,
            certs_present,
        })
    }
}

static EFFECTIVE_MODE: OnceLock<TlsMode> = OnceLock::new();

/// Record the mode chosen at startup so health/status can report it.
pub(crate) fn record(mode: TlsMode) {
    if EFFECTIVE_MODE.set(mode).is_err() {
        log::warn!("Effective TLS mode was already recorded; ignoring a second write");
    }
}

/// Effective mode as a secret-free string for health/status output.
pub(crate) fn snapshot() -> &'static str {
    EFFECTIVE_MODE.get().map(|mode| mode.as_str()).unwrap_or("unknown")
}

#[cfg(test)]
mod tests {
    use super::{decide_tls_mode, plain_http_override_enabled, TlsGuardError, TlsMode};

    #[test]
    fn production_without_certs_fails_closed() {
        // The regression in #2684: a production bind with no certificates must
        // not silently fall back to cleartext HTTP.
        let result = decide_tls_mode(true, false, false, false);
        assert_eq!(
            result,
            Err(TlsGuardError::PlainHttpInProduction {
                tls_disabled: false,
                certs_present: false,
            })
        );
    }

    #[test]
    fn production_with_tls_disabled_fails_closed() {
        let result = decide_tls_mode(true, true, true, false);
        assert_eq!(
            result,
            Err(TlsGuardError::PlainHttpInProduction {
                tls_disabled: true,
                certs_present: true,
            })
        );
    }

    #[test]
    fn production_with_certificates_uses_https() {
        assert_eq!(
            decide_tls_mode(true, false, true, false),
            Ok(TlsMode::Enabled)
        );
    }

    #[test]
    fn production_with_explicit_override_allows_plain_http() {
        assert_eq!(
            decide_tls_mode(true, false, false, true),
            Ok(TlsMode::PlainHttp)
        );
    }

    #[test]
    fn development_without_certificates_still_works() {
        assert_eq!(
            decide_tls_mode(false, false, false, false),
            Ok(TlsMode::PlainHttp)
        );
        // Even with TLS disabled explicitly, development remains usable.
        assert_eq!(
            decide_tls_mode(false, true, false, false),
            Ok(TlsMode::PlainHttp)
        );
    }

    #[test]
    fn development_with_certificates_still_uses_https() {
        assert_eq!(
            decide_tls_mode(false, false, true, false),
            Ok(TlsMode::Enabled)
        );
    }

    #[test]
    fn only_explicit_values_enable_the_plain_http_override() {
        assert!(plain_http_override_enabled(Some("1")));
        assert!(plain_http_override_enabled(Some(" true ")));
        assert!(plain_http_override_enabled(Some("YES")));
        assert!(plain_http_override_enabled(Some("on")));
        assert!(!plain_http_override_enabled(None));
        assert!(!plain_http_override_enabled(Some("")));
        assert!(!plain_http_override_enabled(Some("false")));
        assert!(!plain_http_override_enabled(Some("0")));
        assert!(!plain_http_override_enabled(Some("garbage")));
    }
}
