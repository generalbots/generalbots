//! Meetings, monitoring, channel and ops routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "PUT",
        path: "/api/minutes/documents/:id",
        summary: "registered in crates/botminutes/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/minutes/forms/action",
        summary: "registered in crates/botuifragments/src/ui_fragments/minutes_app.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/minutes/forms/action/:id/done",
        summary: "registered in crates/botuifragments/src/ui_fragments/minutes_app.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/minutes/forms/document/:id",
        summary: "registered in crates/botuifragments/src/ui_fragments/minutes_app.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/minutes/forms/meeting",
        summary: "registered in crates/botuifragments/src/ui_fragments/minutes_app.rs",
    },
    ApiEndpoint {
        method: "PATCH",
        path: "/api/minutes/forms/meeting/:id",
        summary: "registered in crates/botminutes/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/minutes/forms/meeting/start/:id",
        summary: "registered in crates/botminutes/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/minutes/forms/sign/:id",
        summary: "registered in crates/botuifragments/src/ui_fragments/minutes_app.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/minutes/meetings",
        summary: "registered in crates/botminutes/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/minutes/transcripts",
        summary: "registered in crates/botminutes/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/alerts",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/alerts/:id",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/monitoring/alerts/:id/acknowledge",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/monitoring/alerts/:id/silence",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/monitoring/alerts/acknowledge-all",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/alerts/active",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/alerts/count",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/alerts/history",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/alerts/history/export",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/alerts/rules",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/alerts/rules/:id",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "PATCH",
        path: "/api/monitoring/alerts/rules/:id",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/monitoring/alerts/rules/:id",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/alerts/summary",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/charts/cpu",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/charts/memory",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/export",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/health/checks",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/health/dependencies",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/health/incidents",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/health/last-incident",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/health/overview",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/health/response-time",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/health/status",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/health/uptime",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/health/uptime-history",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/health/uptime-percent",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/logs/services",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/metrics",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/metrics/last-sync",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/quick/cpu",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/quick/disk",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/quick/memory",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/quick/network",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/quick/requests",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/resources/cpu",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/resources/disk",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/resources/disk/partitions",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/resources/memory",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/resources/network",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/resources/network/interfaces",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/resources/processes",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/resources/system",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/monitoring/system/status",
        summary: "registered in crates/botmonitoring/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/msteams/messages",
        summary: "registered in crates/botmsteams/src/webhook.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/msteams/send",
        summary: "registered in crates/botmsteams/src/webhook.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/o365/calendar",
        summary: "registered in crates/botm365/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/o365/onedrive",
        summary: "registered in crates/botm365/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/o365/settings",
        summary: "registered in crates/botm365/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/o365/sharepoint",
        summary: "registered in crates/botm365/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/oauth/github/connect",
        summary: "registered in crates/botsettings/src/settings_api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/oauth/google/connect",
        summary: "registered in crates/botsettings/src/settings_api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/oauth/microsoft/connect",
        summary: "registered in crates/botsettings/src/settings_api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ops/endpoints/performance",
        summary: "registered in crates/botsettings/src/ops.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ops/errors/recent",
        summary: "registered in crates/botsettings/src/ops.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ops/health",
        summary: "registered in crates/botsettings/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ops/metrics/requests",
        summary: "registered in crates/botsettings/src/ops.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ops/metrics/summary",
        summary: "registered in crates/botsettings/src/ops.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ops/services/health",
        summary: "registered in crates/botsettings/src/ops.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ops/status",
        summary: "registered in crates/botsettings/src/ops.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ops/traces",
        summary: "registered in crates/botsettings/src/ops.rs",
    },
];
