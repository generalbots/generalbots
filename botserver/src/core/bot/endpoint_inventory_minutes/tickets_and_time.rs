//! Ticket and timeclock routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/:id",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/:id/activities",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/:id/ai-suggest",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/tickets/:id/assign",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/tickets/:id/close",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/:id/comments",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/:id/full",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/tickets/:id/reopen",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/tickets/:id/resolve",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/tickets/:id/status",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/canned",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/categories",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/cis",
        summary: "registered in crates/bottickets/src/itsm.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/cis/:id",
        summary: "registered in crates/bottickets/src/itsm.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/kb",
        summary: "registered in crates/bottickets/src/itsm.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/kb/:id",
        summary: "registered in crates/bottickets/src/itsm.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/overdue",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/sla",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/stats",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tickets/tags",
        summary: "registered in crates/bottickets/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/timeclock/clock",
        summary: "registered in crates/bottimeclock/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/timeclock/forms/break-end",
        summary: "registered in crates/botuifragments/src/ui_fragments/timeclock.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/timeclock/forms/break-start",
        summary: "registered in crates/botuifragments/src/ui_fragments/timeclock.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/timeclock/forms/clock-in",
        summary: "registered in crates/botuifragments/src/ui_fragments/timeclock.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/timeclock/forms/clock-out",
        summary: "registered in crates/botuifragments/src/ui_fragments/timeclock.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/timeclock/forms/justification",
        summary: "registered in crates/botuifragments/src/ui_fragments/timeclock.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/timeclock/forms/overtime",
        summary: "registered in crates/botuifragments/src/ui_fragments/timeclock.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/timeclock/forms/overtime/:id/approve",
        summary: "registered in crates/botuifragments/src/ui_fragments/timeclock.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/timeclock/overtime",
        summary: "registered in crates/bottimeclock/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/timeclock/overtime/:id/approve",
        summary: "registered in crates/bottimeclock/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/timeclock/records",
        summary: "registered in crates/bottimeclock/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/timeclock/reports",
        summary: "registered in crates/bottimeclock/src/routes.rs",
    },
];
