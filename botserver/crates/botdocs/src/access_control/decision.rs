//! The pure authorization function (#2452).
//!
//! No I/O, no clock read, no global state. A caller passes `now`, so an
//! expired-link denial is reproducible in a unit test without sleeping.

use chrono::{DateTime, Utc};

use super::{
    Action, Acl, Decision, LinkCredential, LinkScope, Principal, Reason, Role,
};

/// Decide whether `principal` may perform `action` on `acl` at instant `now`.
///
/// Precedence is intentional:
/// 1. The owner always passes, without link restrictions (full access).
/// 2. A presented link credential is authoritative — a bad or expired link is
///    denied outright rather than falling back to weaker member grants. This
///    makes revocation and expiry verifiable instead of cosmetic.
/// 3. Signed-in members use the highest of their direct and group grants.
#[must_use]
pub fn decide(
    principal: &Principal,
    action: Action,
    acl: &Acl,
    now: DateTime<Utc>,
) -> Decision {
    if principal.member_id() == Some(acl.owner.as_str()) {
        return Decision::allow(Reason::Owner);
    }

    if let Some(credential) = principal.link_credential() {
        return decide_link(principal, action, acl, credential, now);
    }

    if let Principal::Member { id, org_id, groups } = principal {
        if org_id != &acl.org_id {
            return Decision::deny(Reason::CrossTenant);
        }
        return match member_role(id, groups, acl) {
            Some(role) => decide_granted(action, role, member_reason(id, groups, acl, role)),
            None => Decision::deny(Reason::NoGrant),
        };
    }

    Decision::deny(Reason::NoIdentity)
}

fn decide_granted(action: Action, role: Role, reason: Reason) -> Decision {
    if role.satisfies(action) {
        Decision::allow(reason)
    } else {
        Decision::deny(Reason::InsufficientRole {
            required: Role::required_for(action),
            actual: role,
        })
    }
}

fn member_role(id: &str, groups: &[String], acl: &Acl) -> Option<Role> {
    let mut best = acl.user_grants.get(id).copied();
    for group in groups {
        if let Some(role) = acl.group_grants.get(group) {
            best = Some(best.map_or(*role, |current| current.max(*role)));
        }
    }
    best
}

fn member_reason(id: &str, groups: &[String], acl: &Acl, role: Role) -> Reason {
    if acl.user_grants.get(id) == Some(&role) {
        return Reason::UserGrant(role);
    }
    match groups
        .iter()
        .find(|group| acl.group_grants.get(*group) == Some(&role))
    {
        Some(group) => Reason::GroupGrant(group.clone(), role),
        None => Reason::UserGrant(role),
    }
}

fn decide_link(
    principal: &Principal,
    action: Action,
    acl: &Acl,
    credential: &LinkCredential,
    now: DateTime<Utc>,
) -> Decision {
    let Some(link) = acl.link.as_ref() else {
        return Decision::deny(Reason::NoLink);
    };

    if !constant_time_eq(&credential.token, &link.token) {
        return Decision::deny(Reason::LinkTokenMismatch);
    }
    if link.expires_at.is_some_and(|expires| now >= expires) {
        return Decision::deny(Reason::LinkExpired);
    }
    if link.scope == LinkScope::OrgOnly && principal.org_id() != acl.org_id {
        return Decision::deny(Reason::OrgOnly);
    }
    if link.password_hash.is_some() && !credential.password_verified {
        return Decision::deny(Reason::PasswordRequired);
    }

    let guest_principal = match principal {
        Principal::Guest { email, .. } => Some(email.as_str()),
        Principal::Member { .. } | Principal::Anonymous { .. } => None,
    };

    if link.is_guest {
        if !acl.allow_guests {
            return Decision::deny(Reason::GuestsDisabled);
        }
        let Some(email) = guest_principal else {
            return Decision::deny(Reason::GuestOnlyLink);
        };
        match link.guest_email.as_deref() {
            Some(bound) if bound.eq_ignore_ascii_case(email) => {}
            _ => return Decision::deny(Reason::GuestEmailMismatch),
        }
        if !link.allowed_domains.is_empty() {
            let Some(domain) = email_domain(email) else {
                return Decision::deny(Reason::DomainNotAllowed);
            };
            if !link
                .allowed_domains
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(domain))
            {
                return Decision::deny(Reason::DomainNotAllowed);
            }
        }
        // Guests can read, comment, print and copy (subject to link flags) but
        // never change the object or its permissions, whatever the link role.
        if matches!(
            action,
            Action::Edit | Action::Export | Action::Share | Action::Manage
        ) {
            return Decision::deny(Reason::GuestActionNotAllowed(action));
        }
    } else if guest_principal.is_some() {
        return Decision::deny(Reason::GuestOnlyLink);
    }

    // Capability flags gate byte-producing actions even when the link role
    // would otherwise allow them — a viewer link cannot smuggle an export.
    match action {
        Action::Export if !link.allow_download => return Decision::deny(Reason::DownloadDisabled),
        Action::Print if !link.allow_print => return Decision::deny(Reason::PrintDisabled),
        Action::Copy if !link.allow_copy => return Decision::deny(Reason::CopyDisabled),
        _ => {}
    }

    decide_granted(action, link.role, Reason::LinkGrant(link.role))
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (&a, &b) in left.iter().zip(right.iter()) {
        difference |= a ^ b;
    }
    difference == 0
}

fn email_domain(email: &str) -> Option<&str> {
    let (_, domain) = email.rsplit_once('@')?;
    let domain = domain.trim();
    if domain.is_empty() {
        None
    } else {
        Some(domain)
    }
}
