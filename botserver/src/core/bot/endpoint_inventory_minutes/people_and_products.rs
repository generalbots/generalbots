//! Organization, people, plan and product routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "POST",
        path: "/api/organizations",
        summary: "registered in src/main_module/routes/api_setup.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/organizations/current",
        summary: "registered in src/main_module/routes/api_setup.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/organizations/current/audit",
        summary: "registered in src/main_module/routes/api_setup.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/organizations/current/branding",
        summary: "registered in src/main_module/routes/api_setup.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/organizations/current/contact",
        summary: "registered in src/main_module/routes/api_setup.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/organizations/current/export",
        summary: "registered in src/main_module/routes/api_setup.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/organizations/current/settings",
        summary: "registered in src/main_module/routes/api_setup.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/organizations/current/stats",
        summary: "registered in src/main_module/routes/api_setup.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/pages/current",
        summary: "registered in crates/botworkspaces/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/pages/{page_id}",
        summary: "registered in crates/botworkspaces/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/people",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/people/:id",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/people/:id/reports",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/people/:id/skills",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/people/departments",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/people/skills",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/people/stats",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/people/teams",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/people/teams/:id",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/people/teams/:id/members",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/people/teams/:team_id/members/:person_id",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/people/time-off",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/people/time-off/:id/approve",
        summary: "registered in crates/botpeople/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/plan/:plan_id",
        summary: "registered in crates/botplan/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/plan",
        summary: "server-minted plan id, owner from the verified caller (botplan)",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/plan/:plan_id",
        summary: "owner-only plan delete (botplan)",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/plan/:plan_id/typing",
        summary: "registered in crates/botplan/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/plan/task",
        summary: "registered in crates/botplan/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/plan/task/delete",
        summary: "registered in crates/botplan/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/plan/task/update",
        summary: "registered in crates/botplan/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/plan/{plan_id}/collaborators",
        summary: "registered in crates/botplan/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/plan/{plan_id}/presence",
        summary: "registered in crates/botplan/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/player/:bot_id/info/*path",
        summary: "registered in crates/botplayer/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/player/:bot_id/stream/*path",
        summary: "registered in crates/botplayer/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/player/:bot_id/thumbnail/*path",
        summary: "registered in crates/botplayer/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/player/formats",
        summary: "registered in crates/botplayer/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/pos/orders",
        summary: "registered in crates/botpos/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/pos/orders",
        summary: "registered in crates/botpos/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/pos/orders/:id",
        summary: "registered in crates/botpos/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/pos/products",
        summary: "registered in crates/botpos/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/pos/products",
        summary: "registered in crates/botpos/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/pos/sales",
        summary: "registered in crates/botproducts/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/pos/sales/validate-stock",
        summary: "registered in crates/botproducts/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/product",
        summary: "registered in src/main_module/routes/api_setup.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/products/categories",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/products/inventory/adjust",
        summary: "registered in crates/botproducts/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/products/inventory/build",
        summary: "registered in crates/botproducts/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/products/inventory/quote/:id",
        summary: "registered in crates/botproducts/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/products/items",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/products/items/:id",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/products/items/:id/movements",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/products/items/:id/stock",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/products/low-stock",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/products/price-lists",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/products/pricelists",
        summary: "registered in src/main_module/routes/misc_handlers.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/products/services",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/products/services/:id",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/products/services/:id/tax",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/products/stats",
        summary: "registered in crates/botproducts/src/lib.rs",
    },
];
