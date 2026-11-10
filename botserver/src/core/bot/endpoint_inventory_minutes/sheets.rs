//! Spreadsheet routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "POST",
        path: "/api/setup/configure",
        summary: "registered in crates/botapi/src/system.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/setup/status",
        summary: "registered in crates/botapi/src/system.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/:id",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/:id/collaborators",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/:sheet_id/presence",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/:sheet_id/selections",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/:sheet_id/typing",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/ai",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/ai/batch-evaluate",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/ai/evaluate-prompt",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/array-formula",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/array-formula/delete",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/cell",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/chart",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/chart/delete",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/comment",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/comment/delete",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/comment/reply",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/comment/resolve",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/comments",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/conditional-format",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/data-validation",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/delete",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/export",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/external-link",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/external-link/refresh",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/external-link/remove",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/external-links",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/filter",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/filter/clear",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/format",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/formula",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/freeze",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/import",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/list",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/load",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/load-from-drive",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/lock-cells",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/mentions/:user_id",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/merge",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/named-range",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/named-range/delete",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/named-range/update",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/named-ranges",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/named-ranges/export",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/named-ranges/import",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/new",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/note",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/pivot",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/protect",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/range",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/save",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sheet/search",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/share",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/sort",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/unmerge",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/unprotect",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/validate-cell",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/worksheet-meta",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/worksheets/add",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/worksheets/delete",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/worksheets/rename",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sheet/worksheets/switch",
        summary: "registered in crates/botsheet/src/routes.rs",
    },
];
