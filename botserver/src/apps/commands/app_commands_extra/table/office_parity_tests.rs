//! #2448 — parity guard for the review/collaboration verbs added for Office
//! chat/agent automation. Names and paths are data here so a rename or a route
//! drift breaks the build instead of silently degrading a chat command into a
//! deep link.

/// (command name, app id, HTTP method, registered route) — stable contract.
const EXPECTED: &[(&str, &str, &str, &str)] = &[
    (
        "docs.comment.add",
        "docs",
        "POST",
        "/api/docs/comment",
    ),
    (
        "docs.comment.reply",
        "docs",
        "POST",
        "/api/docs/comment/reply",
    ),
    (
        "docs.comment.resolve",
        "docs",
        "POST",
        "/api/docs/comment/resolve",
    ),
    (
        "docs.comment.delete",
        "docs",
        "POST",
        "/api/docs/comment/delete",
    ),
    ("docs.comments.list", "docs", "GET", "/api/docs/comments"),
    (
        "docs.track_changes.list",
        "docs",
        "GET",
        "/api/docs/track-changes",
    ),
    (
        "docs.track_changes.accept_reject",
        "docs",
        "POST",
        "/api/docs/track-changes/accept-reject",
    ),
    (
        "docs.track_changes.accept_all",
        "docs",
        "POST",
        "/api/docs/track-changes/accept-reject-all",
    ),
    ("docs.style.create", "docs", "POST", "/api/docs/style"),
    (
        "docs.style.update",
        "docs",
        "POST",
        "/api/docs/style/update",
    ),
    (
        "docs.style.delete",
        "docs",
        "POST",
        "/api/docs/style/delete",
    ),
    ("docs.styles.list", "docs", "GET", "/api/docs/styles"),
    ("docs.toc.update", "docs", "POST", "/api/docs/toc/update"),
    ("sheet.comment.add", "sheet", "POST", "/api/sheet/comment"),
    (
        "sheet.comment.reply",
        "sheet",
        "POST",
        "/api/sheet/comment/reply",
    ),
    (
        "sheet.comment.resolve",
        "sheet",
        "POST",
        "/api/sheet/comment/resolve",
    ),
    (
        "sheet.comment.delete",
        "sheet",
        "POST",
        "/api/sheet/comment/delete",
    ),
    (
        "sheet.comments.list",
        "sheet",
        "POST",
        "/api/sheet/comments",
    ),
    (
        "sheet.worksheets.delete",
        "sheet",
        "POST",
        "/api/sheet/worksheets/delete",
    ),
    (
        "slides.transition.all",
        "slides",
        "POST",
        "/api/slides/transition/all",
    ),
    (
        "slides.media.update",
        "slides",
        "POST",
        "/api/slides/media/update",
    ),
    (
        "slides.presenter.end",
        "slides",
        "POST",
        "/api/slides/presenter/end",
    ),
];

#[test]
fn office_review_verbs_are_registered_with_stable_names_and_paths() {
    for (name, app, method, path) in EXPECTED {
        let command = crate::apps::commands::command_by_name(name)
            .unwrap_or_else(|| panic!("{name} is not registered"));
        assert_eq!(command.app, *app, "{name} targets the wrong app");

        let binding = crate::apps::commands::verb_endpoint(name)
            .unwrap_or_else(|| panic!("{name} is not bound to an endpoint"));
        assert_eq!(binding.method, *method, "{name} uses the wrong method");
        assert_eq!(binding.path, *path, "{name} binds the wrong path");
    }
}

/// The catalog has no dedicated scope/confirmation/undo columns, so the
/// contract lives in the summary injected into the LLM prompt. Guard that it
/// stays there — an agent must be able to read the permission and undo story.
#[test]
fn office_review_summaries_carry_scope_confirmation_and_undo() {
    for (name, _, _, _) in EXPECTED {
        let summary = crate::apps::commands::command_by_name(name)
            .unwrap_or_else(|| panic!("{name} is not registered"))
            .summary;
        assert!(summary.contains("scope:"), "{name} has no scope metadata");
        assert!(
            summary.contains("confirm:"),
            "{name} has no confirmation metadata"
        );
        assert!(summary.contains("undo:"), "{name} has no undo metadata");
    }
}

/// Destructive and bulk verbs must advertise the preview-then-confirm step in
/// the prompt so a chat agent never executes them on a bare request.
#[test]
fn destructive_office_review_verbs_require_confirmation() {
    const DESTRUCTIVE: &[&str] = &[
        "docs.comment.delete",
        "docs.track_changes.accept_all",
        "docs.style.delete",
        "sheet.comment.delete",
        "sheet.worksheets.delete",
        "slides.transition.all",
    ];
    for name in DESTRUCTIVE {
        let summary = crate::apps::commands::command_by_name(name)
            .unwrap_or_else(|| panic!("{name} is not registered"))
            .summary;
        assert!(
            summary.contains("confirm: required"),
            "{name} must require explicit confirmation"
        );
    }
}
