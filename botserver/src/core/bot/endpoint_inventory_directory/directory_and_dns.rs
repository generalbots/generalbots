//! Directory lookup and DNS routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "DELETE",
        path: "/api/directory/groups/:group_id/delete",
        summary: "registered in crates/botcoredirectory/src/ui_api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/directory/groups/:group_id/invites/send",
        summary: "registered in crates/botcoredirectory/src/ui_api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/groups/:group_id/members",
        summary: "registered in crates/botcoredirectory/src/ui_api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/directory/groups/:group_id/members/add",
        summary: "registered in crates/botcoredirectory/src/ui_api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/directory/groups/:group_id/members/remove",
        summary: "registered in crates/botcoredirectory/src/ui_api.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/directory/groups/:group_id/update",
        summary: "registered in crates/botcoredirectory/src/ui_api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/groups/:id",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/directory/groups/create",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/groups/list",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/groups/list/pagination",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/groups/search",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/organizations/list",
        summary: "registered in crates/botcoredirectory/src/ui_api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/services/status",
        summary: "registered in crates/botcoredirectory/src/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users",
        summary: "registered in crates/botcoredirectory/src/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users/:id",
        summary: "registered in crates/botcoredirectory/src/api.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/directory/users/:id",
        summary: "registered in crates/botcoredirectory/src/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users/:id/activity",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/directory/users/:id/delete",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/directory/users/:id/deprovision",
        summary: "registered in crates/botcoredirectory/src/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users/:id/edit",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users/:id/permissions",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users/:id/profile",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users/:id/security",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users/:user_id/roles",
        summary: "registered in crates/botcoredirectory/src/ui_api.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/directory/users/:user_id/update",
        summary: "registered in crates/botcoredirectory/src/ui_api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/directory/users/create",
        summary: "registered in crates/botcoredirectory/src/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users/list",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users/list/pagination",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/directory/users/provision",
        summary: "registered in crates/botcoredirectory/src/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/directory/users/search",
        summary: "registered in crates/botcoredirectory/src/ui_fragments.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/dns/:id/edit",
        summary: "registered in src/main_module/routes/misc_handlers.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/dns/list",
        summary: "registered in src/main_module/routes/misc_handlers.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/dns/register",
        summary: "registered in src/main_module/routes/misc_handlers.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/dns/remove",
        summary: "registered in src/main_module/routes/misc_handlers.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/dns/search",
        summary: "registered in src/main_module/routes/misc_handlers.rs",
    },
];
