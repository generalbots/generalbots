//! Caller identity checks and identifier sanitising for the VM subsystem.

use axum::http::StatusCode;
use botschema::users;
use diesel::prelude::*;
use uuid::Uuid;

use crate::AgentService;

use super::{bad_request, internal};

/// Guest identities minted by the anonymous chat path. They are created per
/// page load (`guest_<uuid>@anonymous.local`), so they must never own a
/// container: one container per guest exhausted the production host disk.
const ANONYMOUS_EMAIL_DOMAIN: &str = "@anonymous.local";
const ANONYMOUS_USERNAME_PREFIX: &str = "guest_";

fn is_anonymous_email(email: &str) -> bool {
    let lower = email.to_ascii_lowercase();
    lower
        .strip_suffix(ANONYMOUS_EMAIL_DOMAIN)
        .is_some_and(|local| local.starts_with(ANONYMOUS_USERNAME_PREFIX))
}

/// Anonymous visitors must never provision a container (#1596). The guest row
/// is minted by `/api/auth` on every anonymous page load, so provisioning one
/// container per guest leaks disk until the host is full. Only signed-in
/// accounts (real email rows) may own an agent workspace.
///
/// A caller whose identity has no `users` row is likewise refused: the websocket
/// pipeline passes `Uuid::nil()` for sessions it could not attribute, and each
/// of those used to spin up an `ag-u00000000-*` container (production host disk
/// exhaustion). Single-user/desktop builds that run without accounts opt out
/// explicitly with `AGENT_ALLOW_UNREGISTERED_IDENTITIES=1`.
pub(super) fn require_authenticated_identity(
    state: &AgentService,
    user_id: &Uuid,
) -> Result<(), (StatusCode, String)> {
    // Single-user/desktop installs run without accounts: the opt-out covers
    // guest identities and nil callers too, otherwise a fresh local install can
    // never reach the agent workspace.
    if allow_unregistered_identities() {
        return Ok(());
    }
    if user_id.is_nil() {
        tracing::warn!("agent workspace provisioning denied for nil identity");
        return Err((
            StatusCode::FORBIDDEN,
            "Sign in to use the agent workspace".to_string(),
        ));
    }

    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
    let email = users::table
        .filter(users::id.eq(*user_id))
        .select(users::email)
        .first::<String>(&mut conn)
        .optional()
        .map_err(|e| internal(format!("users lookup: {e}")))?;

    match email {
        Some(email) if is_anonymous_email(&email) => {
            tracing::warn!("agent workspace provisioning denied for anonymous identity {user_id}");
            Err((
                StatusCode::FORBIDDEN,
                "Sign in to use the agent workspace".to_string(),
            ))
        }
        Some(_) => Ok(()),
        None => {
            if allow_unregistered_identities() {
                return Ok(());
            }
            tracing::warn!(
                "agent workspace provisioning denied for identity {user_id} without a users row"
            );
            Err((
                StatusCode::FORBIDDEN,
                "Sign in to use the agent workspace".to_string(),
            ))
        }
    }
}

/// Opt-out for single-user/desktop installs that run without accounts.
pub(crate) fn allow_unregistered_identities() -> bool {
    matches!(
        std::env::var("AGENT_ALLOW_UNREGISTERED_IDENTITIES").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

pub fn sanitize_id(raw: &str) -> Result<String, (StatusCode, String)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(bad_request("Invalid identifier"));
    }
    let clean: String = trimmed
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if clean.is_empty() {
        return Err(bad_request("Invalid identifier"));
    }
    Ok(clean)
}

/// `ag-u{user8}-{rand4}` — short, unique, DNS-safe container name.
pub(super) fn generate_vm_name(user_id: &Uuid) -> String {
    let user8: String = user_id.simple().to_string().chars().take(8).collect();
    let rand4: String = Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(4)
        .collect();
    format!("ag-u{user8}-{rand4}")
}

#[cfg(test)]
mod tests {
    use super::is_anonymous_email;

    #[test]
    fn guest_identities_are_anonymous() {
        assert!(is_anonymous_email("guest_1b0d9ece@anonymous.local"));
        assert!(is_anonymous_email("GUEST_ABC@ANONYMOUS.LOCAL"));
    }

    #[test]
    fn real_accounts_are_not_anonymous() {
        assert!(!is_anonymous_email("rodriguez@pragmatismo.com.br"));
        assert!(!is_anonymous_email("guest_reader@example.com"));
        assert!(!is_anonymous_email("someone@anonymous.local.example.com"));
        assert!(!is_anonymous_email(""));
    }
}
