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
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/activities",
        summary: "registered in crates/botsales/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/contacts",
        summary: "registered in crates/botsales/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/deals",
        summary: "registered in crates/botsales/src/routes.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/sales/deals/:id",
        summary: "registered in crates/botsales/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sales/email/send",
        summary: "registered in crates/botcontacts/src/routes_sales.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sales/email/webhook",
        summary: "registered in crates/botcontacts/src/routes_sales.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/forecast",
        summary: "registered in crates/botcontacts/src/routes_sales.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sales/forecast/ai-predict",
        summary: "registered in crates/botcontacts/src/routes_sales.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/funnel",
        summary: "registered in crates/botcontacts/src/routes_sales.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/sales/funnel/conversion-rates",
        summary: "registered in crates/botcontacts/src/routes_sales.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sales/funnel/move",
        summary: "registered in crates/botcontacts/src/routes_sales.rs",
    },
];
