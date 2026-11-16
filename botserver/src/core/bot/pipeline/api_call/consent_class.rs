//! Derives the consent action class from a command name.

/// Read-only verbs get `"read"`, destructive verbs get `"delete"`, everything
/// else defaults to `"update"`.
#[cfg(feature = "consent")]
pub(super) fn consent_action_class(name: &str) -> &'static str {
    let verb = name.rsplit('.').next().unwrap_or(name);
    match verb {
        "find" | "list" | "search" | "get" | "status" | "read" | "dashboard"
        | "diagnosis" | "stats" | "open" => "read",
        "delete" | "batch_delete" | "revoke" => "delete",
        _ => "update",
    }
}
