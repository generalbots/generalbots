//! #2811 — signup email verification trust tests.

use super::signup_email_verified;

#[test]
fn forged_or_missing_verified_flag_never_verifies() {
    assert!(!signup_email_verified(None, "user@example.com"));
    assert!(!signup_email_verified(Some("attacker@evil.example"), "user@example.com"));
    assert!(!signup_email_verified(Some(""), "user@example.com"));
}

#[test]
fn only_the_idp_echo_for_the_same_address_verifies() {
    assert!(signup_email_verified(Some("user@example.com"), "user@example.com"));
    assert!(signup_email_verified(Some(" USER@Example.com "), "user@example.com"));
}
