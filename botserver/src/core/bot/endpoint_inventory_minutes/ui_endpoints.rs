//! UI fragment and billing panel routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/activity/recent",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/bots/performance",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/channels/distribution",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/ui/analytics/chat",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/dashboard",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/errors/count",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/llm/tokens",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/messages/count",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/metric",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/metrics/list",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/ops/cost",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/ops/endpoints",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/ops/queue",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/ops/slo",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/ops/traffic",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/queries/top",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/response/avg",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/sessions/active",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/storage/usage",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/timeseries/messages",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/analytics/timeseries/response_time",
        summary: "registered in crates/botanalytics/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/attendant/agents",
        summary: "registered in crates/botattendant/src/ui/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/attendant/agents/online",
        summary: "registered in crates/botattendant/src/ui/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/attendant/dashboard",
        summary: "registered in crates/botattendant/src/ui/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/attendant/queues",
        summary: "registered in crates/botattendant/src/ui/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/attendant/queues/:id/stats",
        summary: "registered in crates/botattendant/src/ui/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/attendant/sessions",
        summary: "registered in crates/botattendant/src/ui/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/attendant/sessions/:id",
        summary: "registered in crates/botattendant/src/ui/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/attendant/sessions/active",
        summary: "registered in crates/botattendant/src/ui/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/attendant/sessions/count",
        summary: "registered in crates/botattendant/src/ui/mod.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/attendant/sessions/waiting",
        summary: "registered in crates/botattendant/src/ui/mod.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/ui/autotask/create",
        summary: "registered in src/main_module/routes/misc_handlers.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/dashboard/alerts",
        summary: "registered in crates/botbilling/src/billing_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/dashboard/bot-usage",
        summary: "registered in crates/botbilling/src/billing_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/dashboard/cost-breakdown",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/dashboard/metrics",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/dashboard/quotas",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/dashboard/recent-invoices",
        summary: "registered in crates/botbilling/src/billing_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/dashboard/spending-chart",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/gl/accounts",
        summary: "registered in crates/botbilling/src/erp_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/gl/balance-sheet",
        summary: "registered in crates/botbilling/src/erp_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/gl/income-statement",
        summary: "registered in crates/botbilling/src/erp_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/inventory",
        summary: "registered in crates/botbilling/src/erp_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/invoices",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/invoices/admin",
        summary: "registered in crates/botbilling/src/billing_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/invoices/export",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/payments",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/procurement",
        summary: "registered in crates/botbilling/src/erp_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/procurement/orders",
        summary: "registered in crates/botbilling/src/erp_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/quotes",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/search",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/stats/overdue",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/stats/paid-month",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/billing/stats/pending",
        summary: "registered in crates/botbilling/src/billing_ui.rs",
    },
];
