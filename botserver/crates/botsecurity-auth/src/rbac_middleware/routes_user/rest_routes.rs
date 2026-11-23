//! Remaining authenticated routes: goals, bots, CRM, billing and profile.

use super::super::RoutePermission;

pub(super) fn routes() -> Vec<RoutePermission> {
    vec![
        // Goals
        RoutePermission::new("/api/goals/**", "GET", ""),
        RoutePermission::new("/api/goals/**", "POST", ""),
        RoutePermission::new("/api/goals/**", "PUT", ""),
        RoutePermission::new("/api/goals/**", "DELETE", ""),

        // Settings (user's own settings). #2775 — the
        // /api/settings/security/* handlers (botsettings::security_admin) are
        // admin-shaped: TLS, rate-limit, CORS, MFA, password policy, API keys
        // and session revocation. The broad /api/settings/** entries below
        // also match that prefix, and manager.rs returns on the first match,
        // so the admin-gated rules must come FIRST or any authenticated user
        // slips through to the stubs. Keep the remaining settings self-service
        // scope unchanged.
        RoutePermission::new("/api/settings/security/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/settings/security/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/settings/security/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/settings/security/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/settings/**", "GET", ""),
        RoutePermission::new("/api/settings/**", "POST", ""),
        RoutePermission::new("/api/settings/**", "PUT", ""),

        // Bots (read for all authenticated users)
        RoutePermission::new("/api/bots", "GET", ""),
        RoutePermission::new("/api/bots/{id}", "GET", ""),
        RoutePermission::new("/api/bots/{id}/**", "GET", ""),
        // Integration connections and actions. The handlers resolve the
        // tenant scope from the bot id and the caller's membership, and the
        // credential material stays in Vault, so any authenticated user may
        // manage their own bot's connections. Without these rules the
        // "add a Gmail/OneDrive account" flow answered 403 before reaching
        // the handler.
        RoutePermission::new("/api/bots/{id}/integration-connections", "POST", ""),
        RoutePermission::new("/api/bots/{id}/integration-connections/**", "POST", ""),
        RoutePermission::new("/api/bots/{id}/integration-connections/**", "DELETE", ""),
        RoutePermission::new("/api/bots/{id}/integration-actions/**", "POST", ""),
        RoutePermission::new("/api/bots/{id}/integration-automations/**", "POST", ""),
        RoutePermission::new("/api/bots/{id}/integration-automations/**", "DELETE", ""),
        RoutePermission::new("/api/bots/{id}/integrations/oauth/**", "GET", ""),
        // Saving the OAuth client (the "configure once" form) is a POST.
        RoutePermission::new("/api/bots/{id}/integrations/oauth/**", "POST", ""),

        // Autotask
        RoutePermission::new("/api/autotask/**", "GET", ""),
        RoutePermission::new("/api/autotask/**", "POST", ""),
        RoutePermission::new("/api/autotask/**", "PUT", ""),
        RoutePermission::new("/api/autotask/**", "DELETE", ""),

        // Designer
        RoutePermission::new("/api/designer/**", "GET", ""),
        RoutePermission::new("/api/designer/**", "POST", ""),
        RoutePermission::new("/api/designer/**", "PUT", ""),
        RoutePermission::new("/api/designer/**", "DELETE", ""),

        // Dashboards
        RoutePermission::new("/api/dashboards/**", "GET", ""),
        RoutePermission::new("/api/dashboards/**", "POST", ""),
        RoutePermission::new("/api/dashboards/**", "PUT", ""),
        RoutePermission::new("/api/dashboards/**", "DELETE", ""),

        // DB/Table access
        RoutePermission::new("/api/db/**", "GET", ""),
        RoutePermission::new("/api/db/**", "POST", ""),
        RoutePermission::new("/api/db/**", "PUT", ""),
        RoutePermission::new("/api/db/**", "DELETE", ""),

        // CRM / Contacts
        RoutePermission::new("/api/crm/**", "GET", ""),
        RoutePermission::new("/api/crm/**", "POST", ""),
        RoutePermission::new("/api/crm/**", "PUT", ""),
        RoutePermission::new("/api/crm/**", "DELETE", ""),
        RoutePermission::new("/api/contacts/**", "GET", ""),
        RoutePermission::new("/api/contacts/**", "POST", ""),
        RoutePermission::new("/api/contacts/**", "PUT", ""),
        RoutePermission::new("/api/contacts/**", "DELETE", ""),

        // Marketing / Campaigns
        RoutePermission::new("/api/marketing/**", "GET", ""),
        RoutePermission::new("/api/marketing/**", "POST", ""),
        RoutePermission::new("/api/marketing/**", "PUT", ""),
        RoutePermission::new("/api/marketing/**", "DELETE", ""),

        // CRM Campaigns
        RoutePermission::new("/api/crm/campaigns/**", "GET", ""),
        RoutePermission::new("/api/crm/campaigns/**", "POST", ""),
        RoutePermission::new("/api/crm/campaigns/**", "PUT", ""),
        RoutePermission::new("/api/crm/campaigns/**", "DELETE", ""),
        RoutePermission::new("/api/crm/lists/**", "GET", ""),
        RoutePermission::new("/api/crm/lists/**", "POST", ""),
        RoutePermission::new("/api/crm/lists/**", "PUT", ""),
        RoutePermission::new("/api/crm/lists/**", "DELETE", ""),
        RoutePermission::new("/api/crm/templates/**", "GET", ""),
        RoutePermission::new("/api/crm/templates/**", "POST", ""),
        RoutePermission::new("/api/crm/templates/**", "PUT", ""),
        RoutePermission::new("/api/crm/templates/**", "DELETE", ""),

        // Billing / Products
        // #2671 — reads stay available to every authenticated tenant user;
        // invoice/payment/product state changes are admin-only. These entries
        // must carry the roles HERE (not in admin_routes): the route table is
        // matched in registration order and returns on the first hit.
        RoutePermission::new("/api/billing/**", "GET", ""),
        RoutePermission::new("/api/billing/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/billing/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/billing/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/products/**", "GET", ""),
        RoutePermission::new("/api/products/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/products/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/products/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        // #2671 — cloud store checkout, vouchers and organization writes.
        RoutePermission::new("/api/cloud/store/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/vouchers/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/vouchers/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/vouchers/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/organizations/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/organizations/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/organizations/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/billing/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/billing/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Tickets
        RoutePermission::new("/api/tickets/**", "GET", ""),
        RoutePermission::new("/api/tickets/**", "POST", ""),
        RoutePermission::new("/api/tickets/**", "PUT", ""),
        RoutePermission::new("/api/tickets/**", "DELETE", ""),

        // Learn
        RoutePermission::new("/api/learn/**", "GET", ""),
        RoutePermission::new("/api/learn/**", "POST", ""),

        // Social
        RoutePermission::new("/api/social/**", "GET", ""),
        RoutePermission::new("/api/social/**", "POST", ""),

        // LLM
        RoutePermission::new("/api/llm/**", "GET", ""),
        RoutePermission::new("/api/llm/**", "POST", ""),
        RoutePermission::new("/api/llm/**", "PUT", ""),
        RoutePermission::new("/api/llm/**", "DELETE", ""),

        // Email
        RoutePermission::new("/api/email/**", "GET", ""),
        RoutePermission::new("/api/email/**", "POST", ""),
        RoutePermission::new("/api/email/**", "PUT", ""),
        RoutePermission::new("/api/email/**", "DELETE", ""),

        // Messaging channels
        RoutePermission::new("/api/telegram/**", "GET", ""),
        RoutePermission::new("/api/telegram/**", "POST", ""),
        RoutePermission::new("/api/whatsapp/**", "GET", ""),
        RoutePermission::new("/api/whatsapp/**", "POST", ""),
        RoutePermission::new("/api/msteams/**", "GET", ""),
        RoutePermission::new("/api/msteams/**", "POST", ""),
        RoutePermission::new("/api/instagram/**", "GET", ""),
        RoutePermission::new("/api/instagram/**", "POST", ""),

        // Pages
        RoutePermission::new("/api/pages/**", "GET", ""),
        RoutePermission::new("/api/pages/**", "POST", ""),
        RoutePermission::new("/api/pages/**", "PUT", ""),
        RoutePermission::new("/api/pages/**", "DELETE", ""),

        // Insights
        RoutePermission::new("/api/insights/**", "GET", ""),
        RoutePermission::new("/api/insights/**", "POST", ""),

        // App logs
        RoutePermission::new("/api/app-logs/**", "GET", ""),
        RoutePermission::new("/api/app-logs/**", "POST", ""),

        // User profile (own user)
        RoutePermission::new("/api/user/**", "GET", ""),
        RoutePermission::new("/api/user/**", "PUT", ""),

        // Agent VM (per-user VM, #1580)
        RoutePermission::new("/api/agent/**", "GET", ""),
        RoutePermission::new("/api/agent/**", "POST", ""),
        RoutePermission::new("/api/agent/**", "PUT", ""),
        RoutePermission::new("/api/agent/**", "DELETE", ""),

    ]
}
