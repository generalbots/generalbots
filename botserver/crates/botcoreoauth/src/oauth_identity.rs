//! OAuth identity resolution — the security-critical decision of the callback.
//!
//! #2134: the callback used to look the local user up by the `email` string the
//! provider returned and mint a session for that row. Anyone able to present an
//! arbitrary email at any enabled provider could therefore take over an
//! existing account (including administrators). Identity is now the provider
//! subject `(provider, provider_user_id)` persisted in `oauth_account_links`;
//! an email is only ever used to *propose* a link, never to log in.
//!
//! The rules below are pure so they can be tested without a database. The
//! callback gathers [`LocalIdentityFacts`] and asks
//! [`resolve_oauth_identity`] what to do; only [`OAuthResolution::Login`] and
//! [`OAuthResolution::Adopt`] may mint a session.

use uuid::Uuid;

/// `users.password_hash` marker written by the OAuth provisioning path. A row
/// carrying it was created by an OAuth flow and holds no password credential.
pub const OAUTH_NO_PASSWORD_HASH: &str = "OAUTH_USER_NO_PASSWORD";

/// The local account that already owns the provider email.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailOwner {
    pub user_id: Uuid,
    /// `users.is_admin` — an administrator is never auto-linked.
    pub is_admin: bool,
    /// `false` when the row was created by an OAuth flow (no password set).
    pub has_password: bool,
}

/// Local facts the resolver needs, gathered in one round trip by the callback.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalIdentityFacts {
    /// `oauth_account_links.user_id` for `(provider, provider_user_id)`.
    pub linked_user_id: Option<Uuid>,
    /// `users.id` owning the normalized email, if any.
    pub email_owner: Option<EmailOwner>,
}

/// Decision for the callback. Only `Login` and `Adopt` may mint a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthResolution {
    /// The provider subject is already bound to this account.
    Login(Uuid),
    /// A password-less account created by an earlier OAuth flow owns this
    /// verified email: bind the subject to it and log in.
    Adopt(Uuid),
    /// No local identity exists: create one (never an administrator).
    Create { email: Option<String> },
    /// An existing local account owns this email. Refuse the login and send the
    /// user through the authenticated link flow (Settings → Accounts).
    LinkRequired { email_owner: EmailOwner },
}

/// Trims and lowercases a provider email for comparison, rejecting empty
/// strings and values that cannot be an address.
pub fn normalize_email(raw: Option<&str>) -> Option<String> {
    let trimmed = raw?.trim();
    if trimmed.is_empty() || !trimmed.contains('@') {
        return None;
    }
    Some(trimmed.to_lowercase())
}

/// Decides what the callback may do with a provider identity.
///
/// * The provider subject is the identity: a bound subject always wins.
/// * An email alone never logs anybody in. When it matches an existing account
///   the callback must refuse and require the authenticated link flow.
/// * The single email-based exception is adopting a password-less account that
///   an earlier OAuth flow created, and only when the provider asserted the
///   email as verified. Such an account has no other credential, so nobody can
///   be deprived of access by the adoption.
/// * Administrators are never adopted automatically, verified email or not.
pub fn resolve_oauth_identity(
    email: Option<&str>,
    email_verified: bool,
    facts: &LocalIdentityFacts,
) -> OAuthResolution {
    if let Some(user_id) = facts.linked_user_id {
        return OAuthResolution::Login(user_id);
    }

    let Some(owner) = facts.email_owner.clone() else {
        let email = if email_verified {
            email.map(str::to_string)
        } else {
            None
        };
        return OAuthResolution::Create { email };
    };

    // An unverified email proves nothing about the caller.
    if !email_verified {
        return OAuthResolution::LinkRequired { email_owner: owner };
    }
    // An administrator identity is only ever attached by an explicit action.
    if owner.is_admin {
        return OAuthResolution::LinkRequired { email_owner: owner };
    }
    // A password account belongs to whoever knows the password: linking it is
    // the owner's decision, taken from Settings while authenticated.
    if owner.has_password {
        return OAuthResolution::LinkRequired { email_owner: owner };
    }

    OAuthResolution::Adopt(owner.user_id)
}
