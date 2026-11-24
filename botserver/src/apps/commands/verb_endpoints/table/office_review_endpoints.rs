use super::{VerbEndpoint, bind};

/// #2448 — REST bindings for the review/collaboration/formatting verbs declared
/// in `app_commands_extra/table/office_review_ops.rs`. Every path here is an
/// already-registered route from `endpoint_inventory::all_routes()`; the parity
/// unit test in the command table asserts name, method and path stay in sync.
///
/// `file_id` is the catalogue's uniform targeting key and is renamed onto the
/// handler's field (`doc_id`, `id`, `presentation_id`) so an agent call always
/// names `{app, file_id, selection/range}` explicitly rather than "current
/// document" (the #2448 side-by-side targeting contract).
pub(super) static OFFICE_REVIEW_ENDPOINTS: &[VerbEndpoint] = &[
    // ——— Docs ———
    bind(
        "docs.comment.reply",
        "POST",
        "/api/docs/comment/reply",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.comment.resolve",
        "POST",
        "/api/docs/comment/resolve",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.comment.delete",
        "POST",
        "/api/docs/comment/delete",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.comments.list",
        "GET",
        "/api/docs/comments",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.track_changes.list",
        "GET",
        "/api/docs/track-changes",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.track_changes.accept_reject",
        "POST",
        "/api/docs/track-changes/accept-reject",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.track_changes.accept_all",
        "POST",
        "/api/docs/track-changes/accept-reject-all",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.style.create",
        "POST",
        "/api/docs/style",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.style.update",
        "POST",
        "/api/docs/style/update",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.style.delete",
        "POST",
        "/api/docs/style/delete",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.styles.list",
        "GET",
        "/api/docs/styles",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.toc.update",
        "POST",
        "/api/docs/toc/update",
        &[("file_id", "doc_id")],
        &[],
    ),
    // ——— Sheets ———
    bind(
        "sheet.comment.add",
        "POST",
        "/api/sheet/comment",
        &[("file_id", "sheet_id")],
        &[],
    ),
    bind(
        "sheet.comment.reply",
        "POST",
        "/api/sheet/comment/reply",
        &[("file_id", "sheet_id")],
        &[],
    ),
    bind(
        "sheet.comment.resolve",
        "POST",
        "/api/sheet/comment/resolve",
        &[("file_id", "sheet_id")],
        &[],
    ),
    bind(
        "sheet.comment.delete",
        "POST",
        "/api/sheet/comment/delete",
        &[("file_id", "sheet_id")],
        &[],
    ),
    bind(
        "sheet.comments.list",
        "POST",
        "/api/sheet/comments",
        &[("file_id", "sheet_id")],
        &[],
    ),
    bind(
        "sheet.worksheets.delete",
        "POST",
        "/api/sheet/worksheets/delete",
        &[("file_id", "id")],
        &[],
    ),
    // ——— Slides ———
    bind(
        "slides.transition.all",
        "POST",
        "/api/slides/transition/all",
        &[("file_id", "presentation_id")],
        &[],
    ),
    bind(
        "slides.media.update",
        "POST",
        "/api/slides/media/update",
        &[("file_id", "presentation_id"), ("slide", "slide_index")],
        &[],
    ),
    bind(
        "slides.presenter.end",
        "POST",
        "/api/slides/presenter/end",
        &[("file_id", "presentation_id")],
        &[],
    ),
];
