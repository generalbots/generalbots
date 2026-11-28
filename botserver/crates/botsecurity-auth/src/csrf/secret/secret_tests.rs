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
        resolve_secret(None, None, true, false),
        Err(CsrfSecretError::MissingInProduction)
    ));
    // Whitespace is absence, not a secret.
    assert!(matches!(
        resolve_secret(Some("   "), Some(""), true, false),
        Err(CsrfSecretError::MissingInProduction)
    ));
}

#[test]
fn missing_secret_generates_an_ephemeral_one_in_development() {
    let first = resolve_secret(None, None, false, false).expect("development secret");
    let second = resolve_secret(None, None, false, false).expect("development secret");

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
    let error = resolve_secret(Some("too-short"), None, false, false).expect_err("must reject");
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
        false,
    )
    .expect("env secret");
    assert_eq!(from_env.source(), CsrfSecretSource::Env);
    assert_eq!(from_env.bytes(), GOOD_SECRET.as_bytes());

    let from_vault = resolve_secret(None, Some(GOOD_SECRET), true, false).expect("vault secret");
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

    let resolved = resolve_secret(Some(GOOD_SECRET), None, true, false).expect("resolved secret");
    let current = CsrfManager::new(Default::default(), resolved.bytes()).expect("manager");

    assert!(
        !current.verify_signed_token(&forged),
        "a token minted with the retired public constant must not validate"
    );
}

#[test]
fn debug_output_never_contains_the_secret() {
    let secret = resolve_secret(Some(GOOD_SECRET), None, true, false).expect("secret");
    let rendered = format!("{secret:?}");
    assert!(!rendered.contains(GOOD_SECRET));
    assert!(rendered.contains("redacted"));
}

// --- Desktop edition (gblite) tests ---

#[test]
fn desktop_edition_generates_secret_when_none_configured_in_production() {
    let resolved =
        resolve_secret(None, None, true, true).expect("desktop edition must not refuse to start");
    assert_eq!(resolved.source(), CsrfSecretSource::DesktopGenerated);
    // 64 random bytes hex-encoded = 128 characters = 128 bytes as UTF-8.
    assert_eq!(resolved.bytes().len(), 128);
    // The value is valid hex (all ASCII).
    assert!(resolved.bytes().iter().all(|b| b.is_ascii_hexdigit()));
}

#[test]
fn desktop_edition_uses_env_when_present() {
    let resolved = resolve_secret(Some(GOOD_SECRET), None, true, true).expect("env secret");
    assert_eq!(resolved.source(), CsrfSecretSource::Env);
    assert_eq!(resolved.bytes(), GOOD_SECRET.as_bytes());
}

#[test]
fn desktop_edition_uses_vault_when_present() {
    let resolved = resolve_secret(None, Some(GOOD_SECRET), true, true).expect("vault secret");
    assert_eq!(resolved.source(), CsrfSecretSource::Vault);
    assert_eq!(resolved.bytes(), GOOD_SECRET.as_bytes());
}

#[test]
fn desktop_generated_secret_is_unique_per_call() {
    let first = resolve_secret(None, None, true, true).expect("first");
    let second = resolve_secret(None, None, true, true).expect("second");
    assert_ne!(first.bytes(), second.bytes());
}

#[test]
fn desktop_edition_blank_values_still_generates() {
    let resolved =
        resolve_secret(Some("  "), Some(""), true, true).expect("desktop with blanks");
    assert_eq!(resolved.source(), CsrfSecretSource::DesktopGenerated);
}

#[test]
fn non_desktop_production_still_refuses_without_secret() {
    assert!(matches!(
        resolve_secret(None, None, true, false),
        Err(CsrfSecretError::MissingInProduction)
    ));
}

#[test]
fn desktop_generated_as_vault_string_returns_hex() {
    let resolved = resolve_secret(None, None, true, true).expect("desktop");
    let hex = resolved.as_vault_string().expect("desktop source has vault string");
    assert_eq!(hex.len(), 128);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn non_desktop_as_vault_string_returns_none() {
    let resolved = resolve_secret(Some(GOOD_SECRET), None, true, false).expect("env");
    assert!(resolved.as_vault_string().is_none());
}
