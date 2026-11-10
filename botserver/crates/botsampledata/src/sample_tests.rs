//! Offline tests for the dedicated sample tenant (issue #2075). The scope ids
//! are the contract that keeps demo data out of real tenants.

use super::*;
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use argon2::Argon2;

#[test]
fn the_scope_ids_are_the_published_constants() {
    let scope = SampleScope::build();

    assert_eq!(scope.tenant_id, SAMPLE_TENANT_ID);
    assert_eq!(scope.org_id, SAMPLE_ORG_ID);
    assert_eq!(scope.branch_id, SAMPLE_BRANCH_ID);
    assert_eq!(scope.bot_id, SAMPLE_BOT_ID);
    assert_eq!(scope.user_id, SAMPLE_USER_ID);
}

#[test]
fn the_string_forms_match_the_uuids() {
    let scope = SampleScope::build();

    assert_eq!(scope.org_str, SAMPLE_ORG_ID.to_string());
    assert_eq!(scope.branch_str, SAMPLE_BRANCH_ID.to_string());
    assert_eq!(scope.bot_str, SAMPLE_BOT_ID.to_string());
    assert_eq!(scope.user_str, SAMPLE_USER_ID.to_string());
}

#[test]
fn the_scope_is_deterministic_across_builds() {
    let first = SampleScope::build();
    let second = SampleScope::build();

    assert_eq!(first.tenant_id, second.tenant_id);
    assert_eq!(first.org_str, second.org_str);
    assert_eq!(first.user_str, second.user_str);
}

#[test]
fn every_scope_id_is_distinct_and_never_nil() {
    let scope = SampleScope::build();
    let ids = [
        scope.tenant_id,
        scope.org_id,
        scope.branch_id,
        scope.bot_id,
        scope.user_id,
    ];

    for id in ids {
        assert!(!id.is_nil());
    }
    for (index, id) in ids.iter().enumerate() {
        for other in ids.iter().skip(index + 1) {
            assert_ne!(id, other);
        }
    }
}

#[test]
fn the_scope_ids_stay_outside_the_nil_namespace() {
    // The nil UUID is the global/SaaS scope and must never host demo rows.
    let scope = SampleScope::build();
    for id in [scope.tenant_id, scope.org_id, scope.branch_id, scope.bot_id] {
        assert_ne!(id, Uuid::nil());
        assert_eq!(id.as_u128() >> 112, 0x7000);
    }
}

#[test]
fn the_demo_password_hashes_with_argon2_and_verifies() {
    let password = "gb-demo-test-password";
    let hash = hash_demo_password(password).expect("hash");

    assert!(hash.starts_with("$argon2"), "hash {hash}");
    let parsed = PasswordHash::new(&hash).expect("parse hash");
    assert!(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok());
    assert!(Argon2::default()
        .verify_password(b"wrong-password", &parsed)
        .is_err());
}

#[test]
fn hashing_the_same_password_twice_uses_a_fresh_salt() {
    let first = hash_demo_password("same-password").expect("first hash");
    let second = hash_demo_password("same-password").expect("second hash");

    assert_ne!(first, second);
}
