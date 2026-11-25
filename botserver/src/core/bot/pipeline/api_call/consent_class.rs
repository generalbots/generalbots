//! Derives the consent action class from a command name.

/// Read-only verbs get `"read"`, destructive verbs get `"delete"`, everything
/// else defaults to `"update"`.
#[cfg(feature = "consent")]
pub(super) fn consent_action_class(name: &str) -> &'static str {
    let verb = name.rsplit('.').next().unwrap_or(name);
    // #2448 criterion 4 — bulk/destructive office verbs must raise the
    // destructive marker even though their trailing segment is not itself a
    // delete word (`docs.track_changes.accept_all`, `slides.transition.all`,
    // `replace-all`, bulk merge, any `*.delete*`).
    if is_bulk_destructive(name, verb) {
        return "delete";
    }
    match verb {
        "find" | "list" | "search" | "get" | "status" | "read" | "dashboard" | "diagnosis"
        | "stats" | "open" => "read",
        "delete" | "batch_delete" | "revoke" => "delete",
        _ => "update",
    }
}

/// Recognises destructive bulk verbs from the command name alone (#2448).
/// Pure name classification — no document content is inspected.
#[cfg(feature = "consent")]
fn is_bulk_destructive(name: &str, verb: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    // Every `*.delete*` shape: delete, batch_delete, bulk_delete, delete_all.
    if lower.contains("delete") {
        return true;
    }
    let lower_verb = verb.to_ascii_lowercase();
    if lower_verb.contains("replace") || lower_verb.contains("merge") {
        return true;
    }
    // docs.track_changes.accept_all / reject_all.
    if lower_verb == "accept_all"
        || lower_verb == "acceptall"
        || lower_verb == "reject_all"
        || lower_verb == "rejectall"
    {
        return true;
    }
    // slides.transition.all — "all" alone is too generic, require its family.
    if lower.contains("transition.all") || lower.contains("transition_all") {
        return true;
    }
    false
}

#[cfg(all(test, feature = "consent"))]
mod tests {
    use super::consent_action_class;

    #[test]
    fn bulk_destructive_verbs_get_delete_class() {
        let destructive = [
            "docs.track_changes.accept_all",
            "docs.track_changes.reject_all",
            "slides.transition.all",
            "docs.replace-all",
            "docs.replace_all",
            "docs.bulk_merge",
            "slides.merge_all",
            "database.rows.bulk_delete",
            "drive.items.delete_all",
        ];
        for name in destructive {
            assert_eq!(
                consent_action_class(name),
                "delete",
                "{name} should be delete"
            );
        }
        // Regression: generic `all` and read families must not be promoted.
        assert_eq!(consent_action_class("products.list.all"), "update");
        assert_eq!(consent_action_class("drive.list"), "read");
    }
}
