//! #2452 — real role gate for every Docs export format.
//!
//! The six export handlers must be authorized by the object ACL before any
//! document bytes are read, rendered or audited. This module mirrors the ACL
//! projection in `handlers_api/ast.rs` (private to that module) using the same
//! role source — the `resource_permissions` store shared with collab — and the
//! same pure [`decide`] policy. No new role source is introduced here.
//!
//! Fail-closed rules: an unavailable/erroring permission store, an unparseable
//! persisted role, or a caller with no sufficient grant all deny. A denial is
//! the typed `docs_acl_denied` 403 already used by the AST read, never a
//! generic 404 and never the object body.
//!
//! Owner resolution: an explicit `owner` grant names the real document owner.
//! When the document has *no* persisted grants at all, the caller owns the
//! namespace the loader reads (`users/{caller}/docs/{id}.html`), so the caller
//! stays the owner — this preserves the pre-#2452 export behaviour for normal
//! owner-only documents.

use crate::access_control::{decide, Action, Acl, Decision, Principal, Reason, Role};
use crate::auth::DocUser;
use crate::state::DocState;
use axum::{http::StatusCode, Json};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::Text;

/// Resource type of a Docs object in the shared permission store.
const DOCS_RESOURCE: &str = "docs";

/// One persisted grant. `resource_permissions` is the ACL store shared by the
/// collab REST and WebSocket layers (#2452).
#[derive(QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct PermissionRow {
    #[diesel(sql_type = Text)]
    grantee_type: String,
    #[diesel(sql_type = Text)]
    grantee_id: String,
    #[diesel(sql_type = Text)]
    role: String,
}

/// Persisted role spelling → policy role. Unknown/legacy values fail closed.
fn policy_role(role: &str) -> Option<Role> {
    match role.trim().to_ascii_lowercase().as_str() {
        "viewer" => Some(Role::Viewer),
        "commenter" => Some(Role::Commenter),
        "editor" => Some(Role::Editor),
        "owner" => Some(Role::Owner),
        _ => None,
    }
}

/// Pure, DB-free projection of the persisted grants. `None` means a role could
/// not be resolved, so the caller must be denied without reading any bytes.
fn project_acl(rows: &[PermissionRow], caller_id: &str, caller_name: &str, org: &str) -> Option<Acl> {
    let mut acl = Acl::new(org, "");
    for row in rows {
        let role = policy_role(&row.role)?;
        match row.grantee_type.as_str() {
            "user" => {
                if role == Role::Owner {
                    acl.owner = row.grantee_id.clone();
                }
                acl.grant_user(row.grantee_id.clone(), role);
            }
            "group" => acl.grant_group(row.grantee_id.clone(), role),
            // Domain and unknown grantee kinds are not part of the Docs ACL
            // projection used by the AST read; ignore them rather than guess.
            _ => {}
        }
    }
    if rows.is_empty() {
        acl.owner = caller_id.to_string();
    }
    // Collab stores grants under the verified email; Docs identifies the same
    // principal by id. Copy a matching grant under the id the policy sees.
    if let Some(role) = acl.user_grants.get(caller_name).copied() {
        acl.user_grants.entry(caller_id.to_string()).or_insert(role);
    }
    Some(acl)
}

/// The same `decide()` policy the AST read uses, with `Action::Export` instead
/// of `Action::View`. No I/O and no clock of its own: `now` comes from caller.
fn decide_export(acl: &Acl, caller_id: &str, org: &str, now: DateTime<Utc>) -> Decision {
    let principal = Principal::member(caller_id.to_string(), org.to_string(), Vec::new());
    decide(&principal, Action::Export, acl, now)
}

/// Read the object's grants from the shared store. Any failure is `None`, which
/// the caller maps to a fail-closed denial.
fn load_acl(state: &DocState, doc_id: &str, user: &DocUser) -> Option<Acl> {
    let mut conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            log::warn!("docs export: permission store unavailable; denying");
            return None;
        }
    };
    let rows = diesel::sql_query(
        "SELECT grantee_type, grantee_id, role FROM resource_permissions \
         WHERE resource_type = $1 AND resource_id = $2",
    )
    .bind::<Text, _>(DOCS_RESOURCE)
    .bind::<Text, _>(doc_id)
    .load::<PermissionRow>(&mut conn);
    let rows = match rows {
        Ok(rows) => rows,
        Err(_) => {
            log::warn!("docs export: permission lookup failed; denying");
            return None;
        }
    };
    project_acl(&rows, user.id(), user.display_name(), &state.bucket_name)
}

/// Resolve the caller's effective role and decide `Action::Export` through the
/// existing ACL policy. Store/role resolution failures deny outright.
pub(super) fn export_decision(state: &DocState, user: &DocUser, doc_id: &str) -> Decision {
    let acl = match load_acl(state, doc_id, user) {
        Some(acl) => acl,
        None => return Decision::deny(Reason::NoGrant),
    };
    decide_export(&acl, user.id(), &state.bucket_name, Utc::now())
}

/// Stable, non-sensitive code for a policy denial (mirrors `ast.rs`). The
/// document id, owner, tenant and raw error text are never in the response.
fn reason_code(reason: &Reason) -> &'static str {
    match reason {
        Reason::Owner | Reason::UserGrant(_) | Reason::GroupGrant(_, _) | Reason::LinkGrant(_) => {
            "allowed"
        }
        Reason::NoIdentity => "no-identity",
        Reason::CrossTenant => "cross-tenant",
        Reason::NoGrant => "no-grant",
        Reason::NoLink | Reason::LinkTokenMismatch | Reason::LinkExpired => "link-invalid",
        Reason::OrgOnly => "org-only",
        Reason::PasswordRequired => "password-required",
        Reason::GuestsDisabled
        | Reason::GuestOnlyLink
        | Reason::GuestEmailMismatch
        | Reason::DomainNotAllowed => "guest-restricted",
        Reason::GuestActionNotAllowed(_) | Reason::InsufficientRole { .. } => "insufficient-role",
        Reason::DownloadDisabled | Reason::PrintDisabled | Reason::CopyDisabled => {
            "capability-disabled"
        }
    }
}

/// Non-sensitive reason token for logging; never carries identity or payload.
pub(super) fn denial_code(reason: &Reason) -> &'static str {
    reason_code(reason)
}

/// Typed refusal carrying a machine-readable cause, never the object body.
pub(super) fn acl_denied(reason: &Reason) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": "Document not found or access denied",
            "code": "docs_acl_denied",
            "reason": reason_code(reason),
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: DateTime<Utc> = DateTime::UNIX_EPOCH;
    const ORG: &str = "tenant-a";

    fn row(kind: &str, id: &str, role: &str) -> PermissionRow {
        PermissionRow {
            grantee_type: kind.to_string(),
            grantee_id: id.to_string(),
            role: role.to_string(),
        }
    }

    /// An unresolved projection becomes an ownerless ACL, which `decide()`
    /// denies — the same fail-closed path the live handler uses. Assertions
    /// below still detect a wrongly resolved role.
    fn acl_for(rows: &[PermissionRow], caller_id: &str, caller_name: &str) -> Acl {
        match project_acl(rows, caller_id, caller_name, ORG) {
            Some(acl) => acl,
            None => Acl::new(ORG, ""),
        }
    }

    #[test]
    fn editor_role_exports_and_no_grants_keeps_owner_export() {
        let rows = [
            row("user", "alice", "owner"),
            row("user", "bob@example.com", "editor"),
        ];
        let acl = acl_for(&rows, "bob", "bob@example.com");
        assert!(decide_export(&acl, "bob", ORG, NOW).is_allowed());

        // No persisted grants: the caller owns the namespace the loader reads.
        let acl = acl_for(&[], "alice", "Alice");
        assert!(decide_export(&acl, "alice", ORG, NOW).is_allowed());
    }

    #[test]
    fn viewer_role_gets_typed_denial_not_404() {
        let rows = [
            row("user", "alice", "owner"),
            row("user", "bob", "viewer"),
        ];
        let acl = acl_for(&rows, "bob", "bob");
        let decision = decide_export(&acl, "bob", ORG, NOW);
        assert!(!decision.is_allowed());
        assert_eq!(
            decision.reason(),
            &Reason::InsufficientRole {
                required: Role::Editor,
                actual: Role::Viewer,
            }
        );
        let (status, body) = acl_denied(decision.reason());
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body.0["code"], "docs_acl_denied");
        assert_eq!(body.0["reason"], "insufficient-role");
        assert!(body.0.get("content").is_none());
    }

    #[test]
    fn unresolved_role_fails_closed() {
        // A legacy/unknown role cannot be mapped, so no one may export.
        let rows = [row("user", "alice", "owner"), row("user", "bob", "reader")];
        assert!(project_acl(&rows, "bob", "bob", ORG).is_none());
        assert_eq!(
            decide_export(&Acl::new(ORG, ""), "bob", ORG, NOW),
            Decision::deny(Reason::NoGrant)
        );

        // A foreign caller with no grant never gets the object, regardless of
        // who the persisted owner is.
        let rows = [row("user", "alice", "owner")];
        let acl = acl_for(&rows, "carol", "carol");
        let decision = decide_export(&acl, "carol", ORG, NOW);
        assert!(!decision.is_allowed());
        assert_eq!(denial_code(decision.reason()), "no-grant");
    }
}
