//! Project, access-control, reporting and sales routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "GET",
        path: "/api/projects",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/projects/:project_id",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/projects/:project_id",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/projects/:project_id/critical-path",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/projects/:project_id/gantt",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/projects/:project_id/tasks",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/projects/:project_id/tasks",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/projects/:project_id/timeline",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/rbac/check",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/groups",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/groups/:group_id/roles",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/groups/search",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/groups/{group_id}",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/rbac/groups/{group_id}/roles/{role_id}",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/my-permissions",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/permissions",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/roles",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/roles/search",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/roles/{role_id}",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/roles/{role_id}/permissions",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/users",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/users/:user_id/groups",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/users/:user_id/permissions",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/users/:user_id/roles",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/rbac/users/search",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/rbac/users/{user_id}/groups/{group_id}",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/rbac/users/{user_id}/roles/{role_id}",
        summary: "registered in crates/botsettings/src/rbac/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/billing/aging",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/billing/monthly",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/billing/payments",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/billing/revenue",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/crm/conversion",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/crm/forecast",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/crm/pipeline",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/crm/won-lost",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/support/ai-rate",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/support/by-category",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/support/by-priority",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/reports/support/resolution-time",
        summary: "registered in crates/botanalytics/src/reports.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/retail/branches",
        summary: "registered in src/retail/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/retail/promotions",
        summary: "registered in src/retail/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/retail/stock",
        summary: "registered in src/retail/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/retail/suppliers",
        summary: "registered in src/retail/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/retail/top-products",
        summary: "registered in src/retail/mod.rs",
    },
    // #2540 — canonical successors of the retired /api/sales/quotes and
    // /api/sales/orders. Quotes and orders had no CRM equivalent before the
    // consolidation and keep their existing botsales handlers.
    ApiEndpoint {
        method: "GET",
        path: "/api/crm/orders",
        summary: "registered in crates/botsales/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/crm/orders",
        summary: "registered in crates/botsales/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/crm/quotes",
        summary: "registered in crates/botsales/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/crm/quotes",
        summary: "registered in crates/botsales/src/routes.rs",
    },
    // #2540 — /api/sales/* are 410 tombstones. Kept in the inventory so the
    // model can resolve an old path to its canonical /api/crm/* replacement.
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/activities",
        summary: "410 Gone — use /api/crm/activities",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/contacts",
        summary: "410 Gone — use /api/crm/contacts",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/deals",
        summary: "410 Gone — use /api/crm/deals",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/sales/deals/:id",
        summary: "410 Gone — use /api/crm/deals/:id",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sales/email/send",
        summary: "410 Gone — use /api/crm/email/send",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sales/email/webhook",
        summary: "410 Gone — use /api/crm/email/track/open",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/forecast",
        summary: "410 Gone — use /api/crm/stats",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sales/forecast/ai-predict",
        summary: "410 Gone — use /api/crm/ai/generate",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/funnel",
        summary: "410 Gone — use /api/crm/pipeline",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/funnel/conversion-rates",
        summary: "410 Gone — use /api/crm/stats/conversion-rate",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sales/funnel/move",
        summary: "410 Gone — use /api/crm/deals/:id",
    },
];
