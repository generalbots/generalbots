//! Regression tests for #2134 — an OAuth login must never be decided by the
//! email string alone.

use crate::oauth_identity::{
    normalize_email, resolve_oauth_identity, EmailOwner, LocalIdentityFacts, NewOAuthAccount,
    OAuthResolution,
};
use uuid::Uuid;

fn owner(is_admin: bool, has_password: bool) -> EmailOwner {
    EmailOwner {
        user_id: Uuid::new_v4(),
        is_admin,
        has_password,
        has_bound_identity: false,
    }
}

fn linked_owner() -> EmailOwner {
    EmailOwner {
        user_id: Uuid::new_v4(),
        is_admin: false,
        has_password: false,
        has_bound_identity: true,
    }
}

fn facts(linked: Option<Uuid>, email_owner: Option<EmailOwner>) -> LocalIdentityFacts {
    LocalIdentityFacts {
        linked_user_id: linked,
        email_owner,
    }
}

#[test]
fn provider_subject_match_logs_in() {
    let user_id = Uuid::new_v4();
    let resolution =
        resolve_oauth_identity(None, false, &facts(Some(user_id), None));

    assert_eq!(resolution, OAuthResolution::Login(user_id));
}

#[test]
fn provider_subject_match_wins_over_a_conflicting_email() {
    // The subject is the identity: an email owned by somebody else cannot
    // divert a bound subject to a different account.
    let linked = Uuid::new_v4();
    let resolution = resolve_oauth_identity(
        Some("someone@example.com"),
        true,
        &facts(Some(linked), Some(owner(false, true))),
    );

    assert_eq!(resolution, OAuthResolution::Login(linked));
}

#[test]
fn email_only_match_does_not_log_in() {
    // The account exists with a password: no session, link flow only.
    let resolution = resolve_oauth_identity(
        Some("victim@example.com"),
        true,
        &facts(None, Some(owner(false, true))),
    );

    match resolution {
        OAuthResolution::LinkRequired { email_owner } => {
            assert!(email_owner.has_password);
        }
        other => panic!("email match must not log in, got {other:?}"),
    }
}

#[test]
fn unverified_email_does_not_link() {
    // Even a password-less account is not adopted on an unverified email.
    let resolution = resolve_oauth_identity(
        Some("victim@example.com"),
        false,
        &facts(None, Some(owner(false, false))),
    );

    assert!(matches!(
        resolution,
        OAuthResolution::LinkRequired { .. }
    ));
}

#[test]
fn second_provider_cannot_hijack_an_existing_account() {
    // Provider B returns the victim's (verified) email: still no session.
    let resolution = resolve_oauth_identity(
        Some("admin@example.com"),
        true,
        &facts(None, Some(owner(false, true))),
    );

    assert!(matches!(
        resolution,
        OAuthResolution::LinkRequired { .. }
    ));
}

#[test]
fn admin_account_is_never_auto_linked() {
    // No password set (OAuth-only admin) and a verified email: still refused.
    let resolution = resolve_oauth_identity(
        Some("admin@example.com"),
        true,
        &facts(None, Some(owner(true, false))),
    );

    assert!(matches!(
        resolution,
        OAuthResolution::LinkRequired { .. }
    ));
}

#[test]
fn verified_email_adopts_a_passwordless_oauth_account() {
    // An account created by an earlier OAuth flow has no other credential, so
    // adopting it on a provider-verified email cannot deprive its owner.
    let existing = owner(false, false);
    let resolution = resolve_oauth_identity(
        Some("user@example.com"),
        true,
        &facts(None, Some(existing.clone())),
    );

    assert_eq!(resolution, OAuthResolution::Adopt(existing.user_id));
}

#[test]
fn an_account_with_a_bound_identity_is_not_adopted() {
    // A second identity must never stack onto an account that already has one:
    // the owner has to link it explicitly while authenticated.
    let resolution = resolve_oauth_identity(
        Some("user@example.com"),
        true,
        &facts(None, Some(linked_owner())),
    );

    assert!(matches!(
        resolution,
        OAuthResolution::LinkRequired { .. }
    ));
}

#[test]
fn unknown_identity_is_created_without_admin_rights() {
    let resolution = resolve_oauth_identity(
        Some("new@example.com"),
        true,
        &facts(None, None),
    );

    assert_eq!(
        resolution,
        OAuthResolution::Create {
            email: Some("new@example.com".to_string())
        }
    );
}

#[test]
fn unverified_email_is_not_stored_as_the_account_email() {
    let resolution = resolve_oauth_identity(
        Some("claimed@example.com"),
        false,
        &facts(None, None),
    );

    assert_eq!(resolution, OAuthResolution::Create { email: None });
}

#[test]
fn normalize_email_trims_and_lowercases() {
    assert_eq!(
        normalize_email(Some("  User@Example.COM ")),
        Some("user@example.com".to_string())
    );
    assert_eq!(normalize_email(Some("")), None);
    assert_eq!(normalize_email(Some("   ")), None);
    assert_eq!(normalize_email(Some("not-an-address")), None);
    assert_eq!(normalize_email(None), None);
}

#[test]
fn case_differences_do_not_bypass_the_email_owner_check() {
    // The lookup is fed the normalized address, so the stored casing cannot
    // turn an email match into a fresh account.
    let normalized = normalize_email(Some("ViCtIm@Example.com"));
    assert_eq!(normalized.as_deref(), Some("victim@example.com"));

    let resolution = resolve_oauth_identity(
        normalized.as_deref(),
        true,
        &facts(None, Some(owner(false, true))),
    );

    assert!(matches!(
        resolution,
        OAuthResolution::LinkRequired { .. }
    ));
}

#[test]
fn a_created_account_is_never_an_administrator() {
    let account = NewOAuthAccount::for_subject(
        "github",
        "42",
        Some("user@example.com"),
        Some("Octo Cat"),
    );

    assert!(!account.is_admin);
    assert_eq!(account.email, "user@example.com");
}

#[test]
fn a_created_account_never_stores_an_unverified_email() {
    let account = NewOAuthAccount::for_subject("discord", "123", None, Some("someone"));

    assert!(!account.email.is_empty());
    assert!(account.email.ends_with("@oauth.local"));
    assert!(!account.email.contains("victim"));
}

#[test]
fn usernames_keep_the_full_provider_subject() {
    // Discord snowflakes share long prefixes; truncating the subject to a few
    // characters would collide on `users.username` (UNIQUE).
    let first = NewOAuthAccount::for_subject("discord", "123456789012345678", None, Some("same"));
    let second = NewOAuthAccount::for_subject("discord", "123456789012345679", None, Some("same"));

    assert_ne!(first.username, second.username);
    assert!(first.username.contains("123456789012345678"));
}

#[test]
fn a_nameless_identity_still_gets_a_stable_username() {
    let account = NewOAuthAccount::for_subject("google", "109876543210", None, None);

    assert_eq!(account.username, "google_109876543210");
}
