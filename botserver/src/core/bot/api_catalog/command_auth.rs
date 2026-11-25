//! Typed RBAC gate for catalog commands (#2448 criterion 5).
//!
//! The per-verb `admin_only` flag is enforced here, in the trusted in-process
//! execution path — never by prompt text, tool availability or an input filter.
//! Role-resolution failures deny explicitly (fail closed) and the client only
//! ever receives a fixed, sanitized message.

use super::*;

/// Denial reasons. Kept typed so every caller renders the same stable,
/// non-sensitive message instead of interpolating a command name, REST path or
/// a database error into the chat reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CommandAuthError {
    AdminRequired,
    RoleUnresolved,
}

impl CommandAuthError {
    pub(super) fn sanitized_message(self) -> &'static str {
        match self {
            Self::AdminRequired => "command denied: admin role required",
            Self::RoleUnresolved => "command denied: caller role could not be resolved",
        }
    }
}

impl From<CommandAuthError> for String {
    fn from(error: CommandAuthError) -> Self {
        error.sanitized_message().to_string()
    }
}

/// Pure decision table — no DB, no I/O. `None` means the caller's role could
/// not be resolved and is always denied for an admin-gated verb.
pub(super) fn authorize_admin_command(
    role: Option<&str>,
    admin_only: bool,
) -> Result<(), CommandAuthError> {
    if !admin_only {
        return Ok(());
    }
    match role {
        Some(crate::security::user_role::ROLE_ADMIN) => Ok(()),
        Some(_) => Err(CommandAuthError::AdminRequired),
        None => Err(CommandAuthError::RoleUnresolved),
    }
}

/// True when the catalog marks the named verb admin-only.
pub(super) fn requires_admin(name: &str) -> bool {
    commands_list()
        .iter()
        .any(|c| c.name == name && c.admin_only)
}

/// Resolves the role for an admin-gated call. The shared resolver degrades to
/// `user` on a pool failure; probing the pool first lets this gate report a
/// genuine unresolved-role denial instead of silently treating it as a user.
pub(super) fn resolve_gated_role(state: &Arc<AppState>, user_id: Uuid) -> Option<String> {
    state.conn.get().ok()?;
    let role = crate::security::user_role::resolve_user_role(&state.conn, user_id);
    (!role.is_empty()).then_some(role)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::user_role::{ROLE_ADMIN, ROLE_USER};

    /// #2448 — an admin may run an admin-only verb; a regular user may not.
    #[test]
    fn admin_allowed_non_admin_denied() {
        assert_eq!(authorize_admin_command(Some(ROLE_ADMIN), true), Ok(()));
        assert_eq!(
            authorize_admin_command(Some(ROLE_USER), true),
            Err(CommandAuthError::AdminRequired)
        );
    }

    /// #2448 — a failure to resolve the role denies (fail closed), while a
    /// non-gated verb stays callable by everyone.
    #[test]
    fn unresolved_role_denied_and_plain_verbs_open() {
        assert_eq!(
            authorize_admin_command(None, true),
            Err(CommandAuthError::RoleUnresolved)
        );
        assert_eq!(authorize_admin_command(None, false), Ok(()));
        assert_eq!(authorize_admin_command(Some(ROLE_USER), false), Ok(()));
        assert_eq!(
            CommandAuthError::AdminRequired.sanitized_message(),
            "command denied: admin role required"
        );
    }

    /// #2752 — non-admins are rejected before any handler runs.
    #[test]
    fn state_changing_commands_require_admin() {
        for name in [
            "database.rows.insert",
            "database.rows.update",
            "database.rows.import",
            "database.rows.delete",
            "database.rows.batch_delete",
            "database.table.create",
            "database.column.add",
            "database.connections.create",
            "compliance.framework.create",
            "compliance.control.add",
            "compliance.evidence.attach",
            "integrations.api-keys.create",
            "integrations.webhooks.create",
            "docs.track_changes.accept_all",
            "sheet.worksheets.delete",
        ] {
            assert!(requires_admin(name), "{name} is not gated for admin");
        }
        assert!(!requires_admin("database.tables.list"));
    }

    /// #2752 — adding a database verb without `admin_only: true` fails here.
    #[cfg(feature = "database")]
    #[test]
    fn every_database_verb_is_admin_only() {
        for name in database_verbs::declared_verbs() {
            assert!(
                requires_admin(name),
                "database verb {name} needs admin_only"
            );
        }
    }
}
