//! Ownership and capability primitives for browser sessions (#2879).
//!
//! Browser sessions live in one process-global map. Without an owner binding
//! any caller that learns a session id can drive another tenant's page (run
//! JavaScript, read cookies/tokens, close the tab). This module holds the
//! small, side-effect-free checks used by the API layer so they can be tested
//! without launching a browser.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identity of the principal a browser session is bound to (#2879).
///
/// A session is visible and usable only by the exact owner recorded at
/// creation. `org_id`/`branch_id` are the tenant/workspace scope carried by
/// the signed token; they are kept so a caller cannot cross tenant lines by
/// re-using a leaked session id.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SessionOwner {
    #[serde(default)]
    pub owner_user_id: Uuid,
    #[serde(default)]
    pub org_id: Option<Uuid>,
    #[serde(default)]
    pub branch_id: Option<Uuid>,
}

/// Pure ownership check used by both the tab list and the session lookup.
///
/// A session with no recorded owner is legacy/unclaimed and is never visible
/// or usable by anyone: the check fails closed.
pub fn owner_allows(stored: Option<&SessionOwner>, caller: &SessionOwner) -> bool {
    match stored {
        Some(owner) => owner == caller,
        None => false,
    }
}

/// Mint a fresh unguessable capability (UUID v4, 122 bits of entropy).
///
/// The capability is returned to the creator at session creation and must be
/// presented on every subsequent session-scoped request. It is never exposed
/// through the tab snapshot (`BrowserTab`).
pub fn new_capability() -> String {
    Uuid::new_v4().to_string()
}

/// Constant-time capability verification (#2879).
///
/// A missing capability never matches. Comparison goes through the workspace
/// constant-time helper so a wrong guess cannot be refined by timing.
pub fn capability_allows(stored: &str, presented: Option<&str>) -> bool {
    match presented {
        Some(value) => botsecurity_core::constant_time::eq_str(stored, value),
        None => false,
    }
}

/// The full pure decision applied by `get_session` before it hands out a
/// session (#2879). Both the owner and the capability must match; either
/// mismatch is what makes the API answer `403 Forbidden`.
pub fn session_access_allowed(
    stored_owner: Option<&SessionOwner>,
    stored_capability: &str,
    caller_owner: &SessionOwner,
    caller_capability: Option<&str>,
) -> bool {
    owner_allows(stored_owner, caller_owner)
        && capability_allows(stored_capability, caller_capability)
}

#[cfg(test)]
mod tests {
    use super::{
        capability_allows, new_capability, owner_allows, session_access_allowed, SessionOwner,
    };
    use uuid::Uuid;

    fn owner(user: &str, org: &str, branch: &str) -> SessionOwner {
        SessionOwner {
            owner_user_id: Uuid::parse_str(user).unwrap_or_else(|_| Uuid::nil()),
            org_id: Uuid::parse_str(org).ok(),
            branch_id: Uuid::parse_str(branch).ok(),
        }
    }

    const ALICE_USER: &str = "11111111-1111-4111-8111-111111111111";
    const ALICE_ORG: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const ALICE_BRANCH: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
    const BOB_USER: &str = "22222222-2222-4222-8222-222222222222";

    #[test]
    fn same_owner_is_allowed() {
        let alice = owner(ALICE_USER, ALICE_ORG, ALICE_BRANCH);
        assert!(owner_allows(Some(&alice), &alice));
    }

    #[test]
    fn foreign_owner_is_rejected() {
        let alice = owner(ALICE_USER, ALICE_ORG, ALICE_BRANCH);
        let bob = owner(BOB_USER, ALICE_ORG, ALICE_BRANCH);
        assert!(!owner_allows(Some(&alice), &bob));
    }

    #[test]
    fn different_tenant_scope_is_rejected_even_for_same_user_id() {
        let scoped = owner(ALICE_USER, ALICE_ORG, ALICE_BRANCH);
        let other_branch = owner(ALICE_USER, ALICE_ORG, BOB_USER);
        assert!(!owner_allows(Some(&scoped), &other_branch));
    }

    #[test]
    fn unowned_session_is_never_accessible() {
        let alice = owner(ALICE_USER, ALICE_ORG, ALICE_BRANCH);
        assert!(!owner_allows(None, &alice));
    }

    #[test]
    fn capabilities_are_unguessable_and_unique() {
        let first = new_capability();
        let second = new_capability();
        assert_ne!(first, second);
        assert_eq!(first.len(), 36);
    }

    #[test]
    fn capability_check_requires_exact_match() {
        let capability = new_capability();
        assert!(capability_allows(&capability, Some(&capability)));
        assert!(!capability_allows(&capability, None));
        assert!(!capability_allows(&capability, Some("wrong-capability")));
    }

    /// Two-user regression (#2879): Alice owns a session and knows its
    /// capability. Bob learns the session id but neither owns it nor knows the
    /// capability — every session route must therefore answer 403.
    #[test]
    fn foreign_user_cannot_use_an_owned_session() {
        let alice = owner(ALICE_USER, ALICE_ORG, ALICE_BRANCH);
        let bob = owner(BOB_USER, ALICE_ORG, ALICE_BRANCH);
        let capability = new_capability();

        assert!(session_access_allowed(
            Some(&alice),
            &capability,
            &alice,
            Some(&capability)
        ));
        assert!(!session_access_allowed(
            Some(&alice),
            &capability,
            &bob,
            Some(&capability)
        ));
        assert!(!session_access_allowed(
            Some(&alice),
            &capability,
            &bob,
            None
        ));
    }
}
