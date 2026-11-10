//! Editor routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "GET",
        path: "/api/editor/file/*path",
        summary: "registered in crates/boteditor/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/editor/files",
        summary: "registered in crates/boteditor/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/editor/format",
        summary: "registered in crates/boteditor/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/editor/magic",
        summary: "registered in crates/boteditor/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/editor/redo",
        summary: "registered in crates/boteditor/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/editor/save",
        summary: "registered in crates/boteditor/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/editor/save-as",
        summary: "registered in crates/boteditor/src/routes.rs",
    },
];
