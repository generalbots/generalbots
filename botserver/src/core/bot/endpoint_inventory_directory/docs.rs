//! Document and domain routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/:doc_id/collaborators",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/:doc_id/presence",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/:doc_id/selections",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/:doc_id/typing",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/:id",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/ai",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/ai/custom",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/ai/expand",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/ai/improve",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/ai/simplify",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/ai/summarize",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/ai/translate",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/autosave",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/comment",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/comment/delete",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/comment/reply",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/comment/resolve",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/comments",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/compare",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/delete",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/endnote",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/endnote/delete",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/endnote/update",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/endnotes",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/export/docx",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/export/html",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/export/md",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/export/pdf",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/export/txt",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/footnote",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/footnote/delete",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/footnote/update",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/footnotes",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/import",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/list",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/load",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/mentions/:user_id",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/new",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/open-from-drive",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/outline",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/save",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/search",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/style",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/style/apply",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/style/delete",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/style/update",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/styles",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/template/blank",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/template/letter",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/template/meeting",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/template/report",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/toc/generate",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/toc/update",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/docs/track-changes",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/track-changes/accept-reject",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/track-changes/accept-reject-all",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/docs/track-changes/enable",
        summary: "registered in crates/botdocs/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/domains/resolve",
        summary: "registered in crates/botcloud/src/api.rs",
    },
];
