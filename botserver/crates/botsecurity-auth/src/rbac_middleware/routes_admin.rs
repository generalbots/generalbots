use super::*;

pub(crate) fn admin_routes() -> Vec<RoutePermission> {
    vec![
        RoutePermission::new("/api/users", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/{id}", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/{id}", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/{id}", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Groups management
        RoutePermission::new("/api/groups/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/groups/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/groups/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/groups/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Bot management (create/delete)
        RoutePermission::new("/api/bots", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/bots/{id}", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/bots/{id}", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/bots/{id}/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/bots/{id}/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Analytics (admin view)
        RoutePermission::new("/api/analytics/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into(), "Moderator".into()]),
        RoutePermission::new("/api/ui/analytics/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into(), "Moderator".into()]),

        // Monitoring
        RoutePermission::new("/api/monitoring/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/monitoring/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Audit logs
        RoutePermission::new("/api/audit/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/audit/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Security settings
        RoutePermission::new("/api/security/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/security/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/security/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/security/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Admin panel
        RoutePermission::new("/api/admin/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/admin/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/admin/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/admin/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/admin/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Attendant (customer service)
        RoutePermission::new("/api/attendant/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into(), "Moderator".into()]),
        RoutePermission::new("/api/attendant/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into(), "Moderator".into()]),
        RoutePermission::new("/api/ui/attendant/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into(), "Moderator".into()]),

        // Organization settings
        RoutePermission::new("/api/organizations/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/organizations/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/organizations/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Compliance — the Compliance suite app (checks, issues, audit-log,
        // risks, training, dashboard, frameworks) is shown in the desktop to
        // every logged-in user; reads and record creation must not be gated
        // behind an Admin role or the whole app 403s for normal users.
        // Sensitive remediation stays enforced at handler level.
        RoutePermission::new("/api/compliance/**", "GET", ""),
        RoutePermission::new("/api/compliance/**", "POST", ""),
        RoutePermission::new("/api/compliance/**", "PUT", ""),
        RoutePermission::new("/api/compliance/**", "DELETE", ""),

        // Timeclock / Attendance (suite apps mounted on the API router)
        RoutePermission::new("/api/timeclock/**", "GET", ""),
        RoutePermission::new("/api/timeclock/**", "POST", ""),
        RoutePermission::new("/api/timeclock/**", "PUT", ""),
        RoutePermission::new("/api/timeclock/**", "DELETE", ""),
        RoutePermission::new("/api/attendance/**", "GET", ""),
        RoutePermission::new("/api/attendance/**", "POST", ""),

        // Sales / CRM deals
        RoutePermission::new("/api/sales/**", "GET", ""),
        RoutePermission::new("/api/sales/**", "POST", ""),
        RoutePermission::new("/api/sales/**", "PUT", ""),
        RoutePermission::new("/api/sales/**", "DELETE", ""),
        RoutePermission::new("/api/catalog/**", "GET", ""),

        // Desktop / VDI (VNC + RDP) — handlers enforce authentication and
        // per-session ownership (can_access_session); the middleware must not
        // default-deny the whole /api/desktop prefix or the app 403s for
        // every logged-in user.
        RoutePermission::new("/api/desktop/**", "GET", ""),
        RoutePermission::new("/api/desktop/**", "POST", ""),
        RoutePermission::new("/api/desktop/**", "PUT", ""),
        RoutePermission::new("/api/desktop/**", "DELETE", ""),

        // Templates / Minutes / ITSM / Legal / ERP
        RoutePermission::new("/api/templates/**", "GET", ""),
        RoutePermission::new("/api/templates/**", "POST", ""),
        RoutePermission::new("/api/templates/**", "PUT", ""),
        RoutePermission::new("/api/templates/**", "DELETE", ""),
        RoutePermission::new("/api/minutes/**", "GET", ""),
        RoutePermission::new("/api/minutes/**", "POST", ""),
        RoutePermission::new("/api/itsm/**", "GET", ""),
        RoutePermission::new("/api/itsm/**", "POST", ""),
        RoutePermission::new("/api/itsm/**", "PUT", ""),
        RoutePermission::new("/api/itsm/**", "DELETE", ""),
        RoutePermission::new("/api/legal/**", "GET", ""),
        RoutePermission::new("/api/legal/**", "POST", ""),
        RoutePermission::new("/api/erp/**", "GET", ""),
        RoutePermission::new("/api/erp/**", "POST", ""),
        RoutePermission::new("/api/erp/**", "PUT", ""),
        RoutePermission::new("/api/erp/**", "DELETE", ""),

        // Plan / Projects / Resources / Workflow
        RoutePermission::new("/api/plan/**", "GET", ""),
        RoutePermission::new("/api/plan/**", "POST", ""),
        RoutePermission::new("/api/plan/**", "PUT", ""),
        RoutePermission::new("/api/plan/**", "DELETE", ""),
        RoutePermission::new("/api/resources/**", "GET", ""),
        RoutePermission::new("/api/resources/**", "POST", ""),
        RoutePermission::new("/api/resources/**", "PUT", ""),
        RoutePermission::new("/api/resources/**", "DELETE", ""),
        RoutePermission::new("/api/workflow/**", "GET", ""),
        RoutePermission::new("/api/workflow/**", "POST", ""),

        // Deployment / Domains / Features / Reports
        RoutePermission::new("/api/deployment/**", "GET", ""),
        RoutePermission::new("/api/deployment/**", "POST", ""),
        RoutePermission::new("/api/deployment/**", "PUT", ""),
        RoutePermission::new("/api/deployment/**", "DELETE", ""),
        RoutePermission::new("/api/domains/**", "GET", ""),
        RoutePermission::new("/api/domains/**", "POST", ""),
        RoutePermission::new("/api/domains/**", "PUT", ""),
        RoutePermission::new("/api/domains/**", "DELETE", ""),
        RoutePermission::new("/api/features/**", "GET", ""),
        RoutePermission::new("/api/features/**", "POST", ""),
        RoutePermission::new("/api/reports/**", "GET", ""),
        RoutePermission::new("/api/reports/**", "POST", ""),

        // Setup / System / Governance / Ops
        RoutePermission::new("/api/setup/**", "GET", ""),
        RoutePermission::new("/api/setup/**", "POST", ""),
        RoutePermission::new("/api/system/**", "GET", ""),
        RoutePermission::new("/api/system/**", "POST", ""),
        RoutePermission::new("/api/governance/**", "GET", ""),
        RoutePermission::new("/api/governance/**", "POST", ""),
        RoutePermission::new("/api/ops/**", "GET", ""),
        RoutePermission::new("/api/ops/**", "POST", ""),

        // AI / Speech / Services / Activity
        RoutePermission::new("/api/ai/**", "GET", ""),
        RoutePermission::new("/api/ai/**", "POST", ""),
        RoutePermission::new("/api/speech/**", "GET", ""),
        RoutePermission::new("/api/speech/**", "POST", ""),
        RoutePermission::new("/api/services/**", "GET", ""),
        RoutePermission::new("/api/services/**", "POST", ""),
        RoutePermission::new("/api/activity/**", "GET", ""),
        RoutePermission::new("/api/activity/**", "POST", ""),

        // KB / OAuth connect + callback (browser redirects carry no token)
        RoutePermission::new("/api/kb/**", "GET", ""),
        RoutePermission::new("/api/kb/**", "POST", ""),
        RoutePermission::new("/api/oauth/**", "GET", ""),
        RoutePermission::new("/api/oauth/**", "POST", ""),

        // Fiscal Brazil (NF-e/CTE/NFS-e forms) + legacy PT connector endpoints
        RoutePermission::new("/api/brazil/**", "GET", ""),
        RoutePermission::new("/api/brazil/**", "POST", ""),
        RoutePermission::new("/api/pedidos/**", "GET", ""),
        RoutePermission::new("/api/categorias/**", "GET", ""),

        // Browser
        RoutePermission::new("/api/browser/**", "GET", ""),
        RoutePermission::new("/api/browser/**", "POST", ""),
        RoutePermission::new("/api/browser/**", "DELETE", ""),

        // Terminal (sandboxed PTY sessions; any authenticated user)
        RoutePermission::new("/api/terminal/**", "GET", ""),
        RoutePermission::new("/api/terminal/**", "POST", ""),

        // People / Directory
        RoutePermission::new("/api/people/**", "GET", ""),
        RoutePermission::new("/api/people/**", "POST", ""),
        RoutePermission::new("/api/people/**", "PUT", ""),
        RoutePermission::new("/api/people/**", "DELETE", ""),

        // HR
        RoutePermission::new("/api/hr/**", "GET", ""),
        RoutePermission::new("/api/hr/**", "POST", ""),
        RoutePermission::new("/api/hr/**", "PUT", ""),
        RoutePermission::new("/api/hr/**", "DELETE", ""),
        RoutePermission::new("/api/ui/hr/**", "GET", ""),

        // Vision
        RoutePermission::new("/api/vision/**", "GET", ""),
        RoutePermission::new("/api/vision/**", "POST", ""),
        RoutePermission::new("/api/ui/vision/**", "GET", ""),

        // POS / Retail
        RoutePermission::new("/api/pos/**", "GET", ""),
        RoutePermission::new("/api/pos/**", "POST", ""),
        RoutePermission::new("/api/pos/**", "PUT", ""),
        RoutePermission::new("/api/pos/**", "DELETE", ""),
        RoutePermission::new("/api/retail/**", "GET", ""),
        RoutePermission::new("/api/retail/**", "POST", ""),
        RoutePermission::new("/api/retail/**", "PUT", ""),
        RoutePermission::new("/api/retail/**", "DELETE", ""),
        RoutePermission::new("/api/ui/pos/**", "GET", ""),
        RoutePermission::new("/api/ui/retail/**", "GET", ""),

        // Handoff (customer service)
        RoutePermission::new("/api/handoff/**", "GET", ""),
        RoutePermission::new("/api/handoff/**", "POST", ""),
        RoutePermission::new("/api/handoff/**", "PUT", ""),
        RoutePermission::new("/api/handoff/**", "DELETE", ""),
        RoutePermission::new("/api/ui/handoff/**", "GET", ""),

        // KYC / Biometry
        RoutePermission::new("/api/kyc/**", "GET", ""),
        RoutePermission::new("/api/kyc/**", "POST", ""),
        RoutePermission::new("/api/kyc/**", "PUT", ""),
        RoutePermission::new("/api/kyc/**", "DELETE", ""),
        RoutePermission::new("/api/biometry/**", "GET", ""),
        RoutePermission::new("/api/biometry/**", "POST", ""),
        RoutePermission::new("/api/biometry/**", "PUT", ""),
        RoutePermission::new("/api/biometry/**", "DELETE", ""),
        RoutePermission::new("/api/ui/kyc/**", "GET", ""),
        RoutePermission::new("/api/ui/biometry/**", "GET", ""),

        // Fraud
        RoutePermission::new("/api/fraud/**", "GET", ""),
        RoutePermission::new("/api/fraud/**", "POST", ""),
        RoutePermission::new("/api/fraud/**", "PUT", ""),
        RoutePermission::new("/api/fraud/**", "DELETE", ""),
        RoutePermission::new("/api/ui/fraud/**", "GET", ""),

        // Integrations
        RoutePermission::new("/api/integrations/**", "GET", ""),
        RoutePermission::new("/api/integrations/**", "POST", ""),
        RoutePermission::new("/api/integrations/**", "PUT", ""),
        RoutePermission::new("/api/integrations/**", "DELETE", ""),
        RoutePermission::new("/api/ui/integrations/**", "GET", ""),

        // User self-service (profile, security, storage)
        RoutePermission::new("/api/ui/user/**", "GET", ""),
        RoutePermission::new("/api/ui/user/**", "POST", ""),
        RoutePermission::new("/api/ui/user/**", "PUT", ""),
        RoutePermission::new("/api/ui/user/**", "DELETE", ""),

        // Workspace pages (blocks)
        RoutePermission::new("/api/ui/pages/**", "GET", ""),
        RoutePermission::new("/api/ui/pages/**", "POST", ""),
        RoutePermission::new("/api/ui/pages/**", "PUT", ""),
        RoutePermission::new("/api/ui/pages/**", "DELETE", ""),

        // Tax (Brazilian NF-e/CT-e/NFS-e/SPED)
        RoutePermission::new("/api/tax/**", "GET", ""),
        RoutePermission::new("/api/tax/**", "POST", ""),
        RoutePermission::new("/api/tax/**", "PUT", ""),
        RoutePermission::new("/api/tax/**", "DELETE", ""),
        RoutePermission::new("/api/ui/tax/**", "GET", ""),
        RoutePermission::new("/api/fiscal/**", "GET", ""),
        RoutePermission::new("/api/fiscal/**", "POST", ""),
        RoutePermission::new("/api/banking/**", "GET", ""),
        RoutePermission::new("/api/banking/**", "POST", ""),
        RoutePermission::new("/api/banking/**", "PUT", ""),
        RoutePermission::new("/api/banking/**", "DELETE", ""),

        // Directory / Users & Groups management
        RoutePermission::new("/users/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/users/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/users/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/users/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/groups/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/groups/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/groups/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/groups/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // SCIM 2.0 endpoints (Azure AD sync)
        RoutePermission::new("/scim/v2/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/scim/v2/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/scim/v2/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/scim/v2/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Directory management (nested under /api/directory/)
        RoutePermission::new("/api/directory/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/directory/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/directory/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/directory/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

    ]
}

pub(crate) fn rbac_self_service_routes() -> Vec<RoutePermission> {
    vec![
        RoutePermission::new("/api/rbac/my-permissions", "GET", ""),
        RoutePermission::new("/api/rbac/check", "POST", ""),
        RoutePermission::new("/api/rbac/users/{user_id}/permissions", "GET", ""),

    ]
}

pub(crate) fn super_admin_routes() -> Vec<RoutePermission> {
    vec![
        RoutePermission::new("/api/rbac/**", "GET", "")
            .with_roles(vec!["SuperAdmin".into()]),
        RoutePermission::new("/api/rbac/**", "POST", "")
            .with_roles(vec!["SuperAdmin".into()]),
        RoutePermission::new("/api/rbac/**", "PUT", "")
            .with_roles(vec!["SuperAdmin".into()]),
        RoutePermission::new("/api/rbac/**", "DELETE", "")
            .with_roles(vec!["SuperAdmin".into()]),
    ]
}
