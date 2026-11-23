//! Pure authorization predicates for meeting recordings (#2732).
//!
//! These decisions are I/O-free so the host rule and the tenant-scope rule can
//! be unit-tested directly: no database, no async runtime, no HTTP context.

use axum::http::HeaderMap;
use uuid::Uuid;

use botsecurity::auth::AuthenticatedUser;

/// Only a real host/co-host role may control a recording. Client-supplied
/// labels (and absent roles) never grant access.
pub(crate) fn is_host_role(role: Option<&str>) -> bool {
    matches!(role, Some("host" | "co_host" | "co-host" | "presenter"))
}

/// Verified tenant scope of the caller. Both fields come from signed JWT
/// claims or the auth principal — never from a client body field or header.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CallerScope {
    pub(crate) org_id: Option<Uuid>,
    pub(crate) branch_id: Option<Uuid>,
}

impl CallerScope {
    pub(crate) fn from_request(user: &AuthenticatedUser, headers: &HeaderMap) -> Self {
        Self {
            org_id: botsecurity::tenant::org_from_claims(headers).or(user.organization_id),
            branch_id: botsecurity::tenant::branch_from_claims(headers),
        }
    }
}

/// The tenant columns of the room that owns a recording.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RoomScope {
    pub(crate) room_id: Uuid,
    pub(crate) org_id: Uuid,
    pub(crate) branch_id: Uuid,
}

/// Verified claims confine the caller to one org and branch. Nil columns are
/// the legacy unscoped room case and stay visible.
pub(crate) fn room_in_scope(caller: &CallerScope, room: &RoomScope) -> bool {
    if matches!(caller.org_id, Some(org) if !room.org_id.is_nil() && room.org_id != org) {
        return false;
    }
    if matches!(caller.branch_id, Some(branch) if !room.branch_id.is_nil() && room.branch_id != branch)
    {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_real_host_roles_control_recordings() {
        for role in ["host", "co_host", "co-host", "presenter"] {
            assert!(is_host_role(Some(role)), "{role} must control recording");
        }
        for role in ["participant", "viewer", "HOST", "host ", ""] {
            assert!(
                !is_host_role(Some(role)),
                "{role} must not control recording"
            );
        }
        assert!(
            !is_host_role(None),
            "missing role must not control recording"
        );
    }

    #[test]
    fn scope_predicate_confines_verified_claims() {
        let room_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let branch_id = Uuid::new_v4();
        let room = RoomScope {
            room_id,
            org_id,
            branch_id,
        };
        let owner = CallerScope {
            org_id: Some(org_id),
            branch_id: Some(branch_id),
        };
        assert!(room_in_scope(&owner, &room));
        let foreign_org = CallerScope {
            org_id: Some(Uuid::new_v4()),
            branch_id: Some(branch_id),
        };
        assert!(!room_in_scope(&foreign_org, &room));
        let foreign_branch = CallerScope {
            org_id: Some(org_id),
            branch_id: Some(Uuid::new_v4()),
        };
        assert!(!room_in_scope(&foreign_branch, &room));
        let legacy = RoomScope {
            room_id,
            org_id: Uuid::nil(),
            branch_id: Uuid::nil(),
        };
        assert!(room_in_scope(&owner, &legacy));
    }
}
