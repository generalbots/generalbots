//! Data model for Docs object ACLs (#2452).
//!
//! Plain data with no I/O and no ambient clock. `chrono` is already a crate
//! dependency and gives us a serializable, comparable instant; validity is
//! always checked against a `now` passed in by the caller.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An operation a principal may attempt on an object.
///
/// `Print`/`Copy` are separate from `Export` so a link can disable them
/// independently; all three can yield bytes, so they are gated individually.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Action {
    View,
    Print,
    Copy,
    Comment,
    Edit,
    Export,
    Share,
    /// Permission changes and ownership transfer.
    Manage,
}

/// Capability level attached to a grant.
///
/// Declaration order is privilege order: `Viewer < Commenter < Editor <
/// Owner`. Policy code relies on the derived `Ord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Role {
    Viewer,
    Commenter,
    Editor,
    Owner,
}

impl Role {
    /// Smallest role that may perform `action` on its own.
    #[must_use]
    pub const fn required_for(action: Action) -> Self {
        match action {
            Action::View | Action::Print | Action::Copy => Self::Viewer,
            Action::Comment => Self::Commenter,
            Action::Edit | Action::Export | Action::Share => Self::Editor,
            Action::Manage => Self::Owner,
        }
    }

    /// Whether this role alone is sufficient for `action`.
    #[must_use]
    pub const fn satisfies(self, action: Action) -> bool {
        (self as u8) >= (Self::required_for(action) as u8)
    }
}

/// Who is allowed to redeem a share link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkScope {
    /// Only signed-in members of the object's org may redeem the link.
    OrgOnly,
    /// Any holder of the token may redeem it (still subject to expiry,
    /// password and guest rules).
    AnyoneWithLink,
}

/// Proof carried by a link redeemer.
///
/// The password is *not* checked here: this module performs no cryptography.
/// The caller verifies the password out of band and sets
/// [`password_verified`](Self::password_verified); the policy only enforces
/// that verification happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkCredential {
    pub token: String,
    pub password_verified: bool,
}

impl LinkCredential {
    #[must_use]
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            password_verified: false,
        }
    }

    #[must_use]
    pub fn with_verified_password(mut self) -> Self {
        self.password_verified = true;
        self
    }
}

/// Policy of a share link attached to an object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkPolicy {
    pub token: String,
    pub role: Role,
    pub scope: LinkScope,
    pub expires_at: Option<DateTime<Utc>>,
    /// Hash of the optional link password. Presence means a verified password
    /// is required; the hash itself is validated by the caller.
    pub password_hash: Option<String>,
    pub allow_download: bool,
    pub allow_print: bool,
    pub allow_copy: bool,
    /// Text stamped on rendered/exported pages (DRM marker).
    pub watermark: Option<String>,
    /// True when the link is an external-guest magic link.
    pub is_guest: bool,
    /// Email the magic link is bound to; meaningful only for guest links.
    pub guest_email: Option<String>,
    /// Optional email-domain allowlist for guests.
    pub allowed_domains: Vec<String>,
}

impl LinkPolicy {
    /// Build an org-only, view-only link that expires at `expires_at`.
    #[must_use]
    pub fn new(token: impl Into<String>, role: Role, expires_at: Option<DateTime<Utc>>) -> Self {
        Self {
            token: token.into(),
            role,
            scope: LinkScope::OrgOnly,
            expires_at,
            password_hash: None,
            allow_download: false,
            allow_print: false,
            allow_copy: false,
            watermark: None,
            is_guest: false,
            guest_email: None,
            allowed_domains: Vec::new(),
        }
    }
}

/// Who is asking for access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Principal {
    /// A signed-in member of an org, with the groups they belong to (Zitadel).
    Member {
        id: String,
        org_id: String,
        groups: Vec<String>,
    },
    /// An external guest identified by the email its magic link was sent to.
    Guest {
        email: String,
        org_id: String,
        link: LinkCredential,
    },
    /// A link holder with no verified identity.
    Anonymous {
        org_id: String,
        link: LinkCredential,
    },
}

impl Principal {
    #[must_use]
    pub fn member(id: impl Into<String>, org_id: impl Into<String>, groups: Vec<String>) -> Self {
        Self::Member {
            id: id.into(),
            org_id: org_id.into(),
            groups,
        }
    }

    #[must_use]
    pub fn guest(
        email: impl Into<String>,
        org_id: impl Into<String>,
        link: LinkCredential,
    ) -> Self {
        Self::Guest {
            email: email.into(),
            org_id: org_id.into(),
            link,
        }
    }

    #[must_use]
    pub fn anonymous(org_id: impl Into<String>, link: LinkCredential) -> Self {
        Self::Anonymous {
            org_id: org_id.into(),
            link,
        }
    }

    #[must_use]
    pub fn member_id(&self) -> Option<&str> {
        match self {
            Self::Member { id, .. } => Some(id.as_str()),
            _ => None,
        }
    }

    #[must_use]
    pub fn org_id(&self) -> &str {
        match self {
            Self::Member { org_id, .. } | Self::Guest { org_id, .. } | Self::Anonymous { org_id, .. } => {
                org_id.as_str()
            }
        }
    }

    #[must_use]
    pub fn link_credential(&self) -> Option<&LinkCredential> {
        match self {
            Self::Member { .. } => None,
            Self::Guest { link, .. } | Self::Anonymous { link, .. } => Some(link),
        }
    }
}

/// The complete ACL of one Docs object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acl {
    pub org_id: String,
    pub owner: String,
    pub user_grants: BTreeMap<String, Role>,
    pub group_grants: BTreeMap<String, Role>,
    pub link: Option<LinkPolicy>,
    /// Master switch for external-guest access to this object.
    pub allow_guests: bool,
}

impl Acl {
    #[must_use]
    pub fn new(org_id: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            org_id: org_id.into(),
            owner: owner.into(),
            user_grants: BTreeMap::new(),
            group_grants: BTreeMap::new(),
            link: None,
            allow_guests: false,
        }
    }

    pub fn grant_user(&mut self, id: impl Into<String>, role: Role) {
        self.user_grants.insert(id.into(), role);
    }

    pub fn grant_group(&mut self, group: impl Into<String>, role: Role) {
        self.group_grants.insert(group.into(), role);
    }

    pub fn revoke_user(&mut self, id: &str) {
        self.user_grants.remove(id);
    }

    pub fn revoke_group(&mut self, group: &str) {
        self.group_grants.remove(group);
    }

    pub fn set_link(&mut self, link: LinkPolicy) {
        self.link = Some(link);
    }

    /// Revocation is immediate and server-side: dropping the policy makes
    /// every token for this object unusable, whatever any UI still displays.
    pub fn revoke_link(&mut self) {
        self.link = None;
    }

    /// Move ownership without touching comments, versions or the audit trail.
    ///
    /// Those artifacts live in their own stores (and in [`AuditLog`](super::AuditLog));
    /// an ACL transfer only changes who holds the owner capability. The old
    /// owner is demoted to `Editor` so their work stays editable, and any
    /// grant the new owner had is removed because ownership supersedes it.
    pub fn transfer_ownership(&mut self, new_owner: impl Into<String>) {
        let new_owner = new_owner.into();
        if new_owner == self.owner {
            return;
        }
        let previous_owner = std::mem::replace(&mut self.owner, new_owner.clone());
        self.user_grants.remove(&new_owner);
        self.user_grants
            .entry(previous_owner)
            .or_insert(Role::Editor);
    }
}

/// Explicit outcome of a policy check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Allow(Reason),
    Deny(Reason),
}

impl Decision {
    #[must_use]
    pub const fn allow(reason: Reason) -> Self {
        Self::Allow(reason)
    }

    #[must_use]
    pub const fn deny(reason: Reason) -> Self {
        Self::Deny(reason)
    }

    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow(_))
    }

    #[must_use]
    pub const fn reason(&self) -> &Reason {
        match self {
            Self::Allow(reason) | Self::Deny(reason) => reason,
        }
    }
}

/// Why a decision was reached. Kept explicit so audit and callers can act on
/// the exact cause instead of a generic "forbidden".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    Owner,
    UserGrant(Role),
    GroupGrant(String, Role),
    LinkGrant(Role),
    NoIdentity,
    CrossTenant,
    NoGrant,
    NoLink,
    LinkTokenMismatch,
    LinkExpired,
    OrgOnly,
    PasswordRequired,
    GuestsDisabled,
    GuestEmailMismatch,
    GuestOnlyLink,
    GuestActionNotAllowed(Action),
    DomainNotAllowed,
    DownloadDisabled,
    PrintDisabled,
    CopyDisabled,
    InsufficientRole { required: Role, actual: Role },
}
