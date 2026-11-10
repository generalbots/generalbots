//! Search, security, session and settings routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "GET",
        path: "/api/search/entities",
        summary: "registered in crates/botsearch/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/search/entity",
        summary: "registered in crates/botsearch/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/search/reindex",
        summary: "registered in crates/botsearch/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/search/reindex/:source",
        summary: "registered in crates/botsearch/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/search/settings",
        summary: "registered in crates/botsearch/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/search/stats",
        summary: "registered in crates/botsearch/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/biometric/verify",
        summary: "registered in crates/botsecurity/src/routes_security.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/fix",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/security/fix/status",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/kyc/profiles",
        summary: "registered in crates/botsecurity/src/routes_security.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/security/kyc/profiles/:id",
        summary: "registered in crates/botsecurity/src/routes_security.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/kyc/profiles/:id/checks",
        summary: "registered in crates/botsecurity/src/routes_security.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/protection/:tool/auto",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/protection/:tool/install",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/security/protection/:tool/report",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/protection/:tool/run",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/protection/:tool/start",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/protection/:tool/stop",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/security/protection/clamav/quarantine",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/protection/clamav/quarantine/{id}",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/security/protection/status",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/protection/{tool}/disable",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/protection/{tool}/enable",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/security/protection/{tool}/status",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/protection/{tool}/uninstall",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/protection/{tool}/update",
        summary: "registered in crates/botsecurity-protection/src/protection/api.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/security/signatures/:id/validate",
        summary: "registered in crates/botsecurity/src/routes_security.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/signatures/requests",
        summary: "registered in crates/botsecurity/src/routes_security.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/security/signatures/requests/:id/complete",
        summary: "registered in crates/botsecurity/src/routes_security.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/services/summary",
        summary: "registered in src/main_module/routes/misc_handlers.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/sessions/current/message",
        summary: "registered in src/main_module/routes/chat_handlers.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/settings/accounts/smtp",
        summary: "registered in crates/botsettings/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/settings/search",
        summary: "registered in crates/botsettings/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/settings/security/api-keys",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/settings/security/api-keys/:key_id",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/settings/security/audit",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/settings/security/cors",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/settings/security/mfa",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/settings/security/overview",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/settings/security/password-policy",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/settings/security/rate-limit",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/settings/security/scan",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/settings/security/sessions",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/settings/security/sessions/:session_id",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/settings/security/tls",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/settings/security/users/{user_id}/sessions",
        summary: "registered in crates/botsettings/src/security_admin.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/settings/smtp/test",
        summary: "registered in crates/botsettings/src/lib.rs",
    },
];
