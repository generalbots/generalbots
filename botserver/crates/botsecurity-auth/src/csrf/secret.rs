//! CSRF signing-secret resolution (#2141).
//!
//! The CSRF double-submit proof is only as strong as the HMAC secret behind it.
//! This module owns the rule that decides which secret signs tokens:
//!
//! | Source | When | Notes |
//! |---|---|---|
//! | `CSRF_SECRET` (env) | first choice | required for multi-instance deployments |
//! | Vault `secret/gbo/system/security` `csrf_secret` | when the env var is absent | canonical for a deployed node |
//! | ephemeral random | development only | 32 bytes per process, never persisted |
//! | refuse to start | production without env/Vault | fail closed |
//!
//! There is deliberately **no** built-in constant: a public default makes every
//! token forgeable by anyone who has read the repository.
//!
//! ## Rotation
//!
//! 1. Write the new value to Vault:
//!    `vault kv put secret/gbo/system/security csrf_secret="<64+ random chars>"`
//!    (or set `CSRF_SECRET` in the service environment).
//! 2. Restart botserver. The secret is read once at boot, so the restart is the
//!    rotation boundary; tokens minted before it stop validating.
//! 3. For a rolling rotation across instances, run every instance with the same
//!    new secret before draining the old ones, otherwise a token minted by one
//!    node fails on another.
//! 4. Never reuse the retired value. A 32-byte minimum is enforced here, but
//!    generate at least 32 bytes of entropy (for example
//!    `openssl rand -base64 48`).

use anyhow::Result;
use rand::Rng;
use std::fmt;

/// Environment variable that carries the CSRF signing secret.
pub const ENV_CSRF_SECRET: &str = "CSRF_SECRET";

/// Minimum accepted length for a configured secret, in bytes.
///
/// HMAC-SHA256 keys shorter than the hash output gain nothing from the extra
/// HMAC round; 32 bytes is the shortest length that uses the full strength.
pub const MIN_SECRET_BYTES: usize = 32;

/// Where the effective signing secret came from. Never carries the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrfSecretSource {
    Env,
    Vault,
    /// Random per process; only used when neither env nor Vault is configured.
    Ephemeral,
}

impl CsrfSecretSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Vault => "vault",
            Self::Ephemeral => "ephemeral",
        }
    }
}

/// A resolved signing secret plus its origin.
#[derive(Clone)]
pub struct CsrfSecret {
    bytes: Vec<u8>,
    source: CsrfSecretSource,
}

impl CsrfSecret {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn source(&self) -> CsrfSecretSource {
        self.source
    }
}

impl fmt::Debug for CsrfSecret {
    /// Prints the origin only, so a debug log can never leak the key.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CsrfSecret")
            .field("source", &self.source)
            .field(
                "bytes",
                &format_args!("<redacted {} bytes>", self.bytes.len()),
            )
            .finish()
    }
}

/// Why a signing secret could not be resolved. Every variant is fatal at boot:
/// there is no fallback that keeps signing with a known value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CsrfSecretError {
    #[error(
        "no CSRF signing secret is configured in production: set {ENV_CSRF_SECRET} \
         or Vault secret/gbo/system/security csrf_secret"
    )]
    MissingInProduction,
    #[error("{origin} CSRF secret is {length} bytes; at least {minimum} are required")]
    TooShort {
        origin: &'static str,
        length: usize,
        minimum: usize,
    },
}

/// Resolve the CSRF signing secret.
///
/// Pure on purpose: the caller supplies the candidate values and the deployment
/// mode, so the fail-closed rule is testable without touching process globals.
///
/// * `env` — value of [`ENV_CSRF_SECRET`], if set.
/// * `vault` — value read from Vault, if any.
/// * `production` — refuse to mint an ephemeral secret when `true`.
pub fn resolve_secret(
    env: Option<&str>,
    vault: Option<&str>,
    production: bool,
) -> Result<CsrfSecret, CsrfSecretError> {
    let configured = [
        (env, CsrfSecretSource::Env),
        (vault, CsrfSecretSource::Vault),
    ]
    .into_iter()
    .find_map(|(value, source)| {
        value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| (value, source))
    });

    if let Some((value, source)) = configured {
        if value.len() < MIN_SECRET_BYTES {
            return Err(CsrfSecretError::TooShort {
                source: source.as_str(),
                length: value.len(),
                minimum: MIN_SECRET_BYTES,
            });
        }
        return Ok(CsrfSecret {
            bytes: value.as_bytes().to_vec(),
            source,
        });
    }

    if production {
        return Err(CsrfSecretError::MissingInProduction);
    }

    Ok(CsrfSecret {
        bytes: random_secret(),
        source: CsrfSecretSource::Ephemeral,
    })
}

/// 32 bytes from the process CSPRNG. Development and single-node installs only:
/// a restart invalidates every token minted before it.
fn random_secret() -> Vec<u8> {
    let mut rng = rand::rng();
    (0..MIN_SECRET_BYTES).map(|_| rng.random()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csrf::CsrfManager;

    /// The exact constant that used to be the built-in default. Kept here, in a
    /// test, as the thing the new code must reject — never in production code.
    const LEGACY_PUBLIC_SECRET: &str = "dev-csrf-secret-change-in-production-minimum-32-bytes";

    const GOOD_SECRET: &str = "a-production-csrf-secret-with-enough-entropy-0123456789";

    #[test]
    fn missing_secret_refuses_to_mint_in_production() {
        assert!(matches!(
            resolve_secret(None, None, true),
            Err(CsrfSecretError::MissingInProduction)
        ));
        // Whitespace is absence, not a secret.
        assert!(matches!(
            resolve_secret(Some("   "), Some(""), true),
            Err(CsrfSecretError::MissingInProduction)
        ));
    }

    #[test]
    fn missing_secret_generates_an_ephemeral_one_in_development() {
        let first = resolve_secret(None, None, false).expect("development secret");
        let second = resolve_secret(None, None, false).expect("development secret");

        assert_eq!(first.source(), CsrfSecretSource::Ephemeral);
        assert_eq!(first.bytes().len(), MIN_SECRET_BYTES);
        assert_ne!(
            first.bytes(),
            second.bytes(),
            "each process boot gets its own"
        );
        assert_ne!(first.bytes(), LEGACY_PUBLIC_SECRET.as_bytes());
    }

    #[test]
    fn a_short_secret_fails_closed_instead_of_degrading() {
        let error = resolve_secret(Some("too-short"), None, false).expect_err("must reject");
        assert!(matches!(
            error,
            CsrfSecretError::TooShort {
                origin: "env",
                length: 9,
                minimum: MIN_SECRET_BYTES
            }
        ));
    }

    #[test]
    fn environment_wins_over_vault_and_vault_is_used_when_env_is_absent() {
        let from_env = resolve_secret(
            Some(GOOD_SECRET),
            Some("vault-secret-that-is-long-enough-0123456789"),
            true,
        )
        .expect("env secret");
        assert_eq!(from_env.source(), CsrfSecretSource::Env);
        assert_eq!(from_env.bytes(), GOOD_SECRET.as_bytes());

        let from_vault = resolve_secret(None, Some(GOOD_SECRET), true).expect("vault secret");
        assert_eq!(from_vault.source(), CsrfSecretSource::Vault);
        assert_eq!(from_vault.bytes(), GOOD_SECRET.as_bytes());
    }

    #[test]
    fn a_token_signed_with_the_legacy_public_default_is_rejected() {
        let legacy = CsrfManager::from_secret(LEGACY_PUBLIC_SECRET).expect("legacy manager");
        let forged = legacy.generate_signed_token();

        // Sanity: the legacy constant does verify its own tokens, which is
        // exactly why it must never be the runtime default.
        assert!(legacy.verify_signed_token(&forged));

        let resolved = resolve_secret(Some(GOOD_SECRET), None, true).expect("resolved secret");
        let current = CsrfManager::new(Default::default(), resolved.bytes()).expect("manager");

        assert!(
            !current.verify_signed_token(&forged),
            "a token minted with the retired public constant must not validate"
        );
    }

    #[test]
    fn debug_output_never_contains_the_secret() {
        let secret = resolve_secret(Some(GOOD_SECRET), None, true).expect("secret");
        let rendered = format!("{secret:?}");
        assert!(!rendered.contains(GOOD_SECRET));
        assert!(rendered.contains("redacted"));
    }
}
