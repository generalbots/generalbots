//! Tests for the single SaaS JWT secret resolver (#2140).
//!
//! Every branch is driven through the injected resolver ([`resolve_secret`] /
//! [`resolve_with`]) so a debug binary can assert release behaviour without a
//! release build and without touching the process environment.

use super::*;
use crate::verified_jwt::{verify_hs256_jwt, JwtVerifyError};

const CONFIGURED: &str = "configured-secret-0123456789abcdefghijklmnop";
const FROM_ENV_SAAS: &str = "saas-env-secret-0123456789abcdefghijklmnop";
const FROM_ENV_JWT: &str = "jwt-env-secret-0123456789abcdefghijklmnop";

const RELEASE: JwtSecretPolicy = JwtSecretPolicy {
    debug_build: false,
    allow_dev_fallback: false,
    production: true,
};

const DEBUG: JwtSecretPolicy = JwtSecretPolicy {
    debug_build: true,
    allow_dev_fallback: false,
    production: false,
};

const DEBUG_WITH_FALLBACK: JwtSecretPolicy = JwtSecretPolicy {
    debug_build: true,
    allow_dev_fallback: true,
    production: false,
};

fn accepts_persist(_secret: &str) -> Result<(), String> {
    Ok(())
}

fn empty_inputs() -> JwtSecretInputs<'static> {
    JwtSecretInputs::default()
}

fn claims() -> serde_json::Value {
    serde_json::json!({
        "sub": "user-1",
        "exp": chrono::Utc::now().timestamp() + 3600,
    })
}

#[test]
fn configured_secrets_win_in_priority_order() {
    let cases = [
        (
            JwtSecretInputs {
                config_file_value: Some(CONFIGURED),
                env_saas_jwt_secret: Some(FROM_ENV_SAAS),
                env_jwt_secret: Some(FROM_ENV_JWT),
            },
            JwtSecretSource::ConfigFile,
            CONFIGURED,
        ),
        (
            JwtSecretInputs {
                config_file_value: None,
                env_saas_jwt_secret: Some(FROM_ENV_SAAS),
                env_jwt_secret: Some(FROM_ENV_JWT),
            },
            JwtSecretSource::EnvSaasJwtSecret,
            FROM_ENV_SAAS,
        ),
        (
            JwtSecretInputs {
                config_file_value: Some("   "),
                env_saas_jwt_secret: None,
                env_jwt_secret: Some(FROM_ENV_JWT),
            },
            JwtSecretSource::EnvJwtSecret,
            FROM_ENV_JWT,
        ),
    ];

    for (inputs, expected_source, expected_secret) in cases {
        let resolved = resolve_secret(inputs, RELEASE);
        assert_eq!(resolved.source, expected_source);
        assert_eq!(resolved.secret, expected_secret);
        assert!(!resolved.is_public_placeholder());
    }
}

#[test]
fn placeholders_are_treated_as_not_configured() {
    let inputs = JwtSecretInputs {
        config_file_value: Some(DEV_FALLBACK_SECRET),
        env_saas_jwt_secret: Some("embedded-change-me-in-production"),
        env_jwt_secret: Some(DEV_FALLBACK_SECRET),
    };
    let resolved = resolve_secret(inputs, RELEASE);
    assert_eq!(resolved.source, JwtSecretSource::Generated);
    assert!(!resolved.is_public_placeholder());
}

#[test]
fn a_release_build_never_returns_the_public_constant() {
    for inputs in [
        empty_inputs(),
        JwtSecretInputs {
            config_file_value: Some(DEV_FALLBACK_SECRET),
            ..Default::default()
        },
    ] {
        let resolved = resolve_secret(inputs, RELEASE);
        assert_ne!(resolved.secret, DEV_FALLBACK_SECRET);
        assert_eq!(resolved.source, JwtSecretSource::Generated);
        assert_eq!(resolved.secret.len(), GENERATED_SECRET_LEN);
    }
}

#[test]
fn a_debug_build_without_the_opt_in_does_not_use_the_public_constant() {
    let resolved = resolve_secret(empty_inputs(), DEBUG);
    assert_eq!(resolved.source, JwtSecretSource::Generated);
    assert_ne!(resolved.secret, DEV_FALLBACK_SECRET);
}

#[test]
fn the_public_constant_requires_the_explicit_debug_opt_in() {
    let resolved = resolve_secret(empty_inputs(), DEBUG_WITH_FALLBACK);
    assert_eq!(resolved.source, JwtSecretSource::DevFallback);
    assert_eq!(resolved.secret, DEV_FALLBACK_SECRET);
    assert!(resolved.is_public_placeholder());
}

#[test]
fn a_generated_secret_is_persisted_and_returned() {
    let mut persisted = String::new();
    let resolved = resolve_with(empty_inputs(), RELEASE, |secret: &str| {
        persisted = secret.to_string();
        Ok(())
    })
    .expect("generated secret");

    assert_eq!(resolved.source, JwtSecretSource::Generated);
    assert_eq!(persisted, resolved.secret);
    assert_ne!(persisted, DEV_FALLBACK_SECRET);
}

#[test]
fn the_dev_fallback_is_never_persisted() {
    let resolved = resolve_with(empty_inputs(), DEBUG_WITH_FALLBACK, |_secret: &str| {
        Err("must not be called".to_string())
    })
    .expect("dev fallback does not persist");
    assert_eq!(resolved.source, JwtSecretSource::DevFallback);
}

#[test]
fn production_fails_closed_when_the_secret_cannot_be_persisted() {
    let error = resolve_with(empty_inputs(), RELEASE, |_secret: &str| {
        Err("read-only file system".to_string())
    })
    .expect_err("a production deployment without a persisted secret must not start");

    match error {
        JwtSecretError::NotPersisted { reason } => assert!(reason.contains("read-only")),
    }
}

#[test]
fn development_continues_with_an_ephemeral_secret_when_persistence_fails() {
    let resolved = resolve_with(empty_inputs(), DEBUG, |_secret: &str| {
        Err("read-only file system".to_string())
    })
    .expect("a development box keeps the in-memory secret");

    assert_eq!(resolved.source, JwtSecretSource::Generated);
    assert_ne!(resolved.secret, DEV_FALLBACK_SECRET);
}

#[test]
fn a_token_signed_with_the_public_constant_is_rejected_after_a_random_secret() {
    let resolved = resolve_with(empty_inputs(), RELEASE, accepts_persist).expect("secret");
    let forged = crate::verified_jwt::test_support::mint_hs256(&claims(), DEV_FALLBACK_SECRET);
    assert_eq!(
        verify_hs256_jwt(&forged, &resolved.secret),
        Err(JwtVerifyError::InvalidSignature)
    );

    // The same token is what a duplicate resolver with the public fallback
    // would have accepted — which is exactly the bug this module closes.
    assert!(verify_hs256_jwt(&forged, DEV_FALLBACK_SECRET).is_ok());

    let genuine = crate::verified_jwt::test_support::mint_hs256(&claims(), &resolved.secret);
    assert!(verify_hs256_jwt(&genuine, &resolved.secret).is_ok());
}

#[test]
fn minter_and_verifier_agree_on_every_source() {
    let cases = [
        (
            JwtSecretInputs {
                config_file_value: Some(CONFIGURED),
                ..Default::default()
            },
            RELEASE,
            CONFIGURED,
        ),
        (
            JwtSecretInputs {
                env_saas_jwt_secret: Some(FROM_ENV_SAAS),
                ..Default::default()
            },
            RELEASE,
            FROM_ENV_SAAS,
        ),
        (
            JwtSecretInputs {
                env_jwt_secret: Some(FROM_ENV_JWT),
                ..Default::default()
            },
            RELEASE,
            FROM_ENV_JWT,
        ),
    ];

    for (inputs, policy, expected) in cases {
        let minted_with = resolve_with(inputs, policy, accepts_persist).expect("mint side");
        let verified_with = resolve_with(inputs, policy, accepts_persist).expect("verify side");
        assert_eq!(minted_with.secret, expected);
        assert_eq!(minted_with.secret, verified_with.secret);

        let token = crate::verified_jwt::test_support::mint_hs256(&claims(), &minted_with.secret);
        let verified = verify_hs256_jwt(&token, &verified_with.secret).expect("verified");
        assert_eq!(verified.sub, "user-1");
    }
}
