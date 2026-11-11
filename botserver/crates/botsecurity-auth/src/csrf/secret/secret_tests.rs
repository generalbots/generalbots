//! CSRF signing-secret tests (#2141).
//!
//! Split out of `secret.rs` so the production module carries no placeholder
//! literal: the retired public constant lives here, on purpose, as the value the
//! new resolver must reject.

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
