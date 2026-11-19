use super::super::ApiEndpoint;

pub(super) static FILES_USER_AND_COMMERCE: &[ApiEndpoint] = &[
        // Drive / files
        ApiEndpoint {
            method: "GET",
            path: "/api/files/list",
            summary: "List drive files for the bot",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/files/write",
            summary: "Upload a file (base64 content) into a folder",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/files/download",
            summary: "Download a file from a folder",
        },
        ApiEndpoint {
            method: "DELETE",
            path: "/api/files/delete",
            summary: "Delete a file",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/files/createFolder",
            summary: "Create a drive folder",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/files/copy",
            summary: "Copy a file",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/files/move",
            summary: "Move a file",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/files/search",
            summary: "Search drive files by name",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/files/recent",
            summary: "Most recent files",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/files/quota",
            summary: "Storage quota usage",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/files/upload-binary",
            summary: "Upload a raw binary file",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/files/buckets",
            summary: "List bot buckets",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/files/shared",
            summary: "List shared folders",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/files/share",
            summary: "Share a folder",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/files/trash",
            summary: "List trashed files",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/files/trash/restore",
            summary: "Restore a trashed file",
        },
        // Products / fiscal
        ApiEndpoint {
            method: "GET",
            path: "/api/products/items",
            summary: "List product catalog items",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/products/items",
            summary: "Create a product item",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/products/services",
            summary: "List registered services",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/products/services/:id/tax",
            summary: "Compute Brazilian taxes for a service",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/products/categories",
            summary: "List product categories",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/products/pricelists",
            summary: "List price lists",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/products/stats",
            summary: "Product statistics",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/products/low-stock",
            summary: "Low-stock items",
        },
        // Banking / cash flow
        ApiEndpoint {
            method: "GET",
            path: "/api/banking/transactions",
            summary: "List bank transactions",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/banking/transactions",
            summary: "Create a bank transaction",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/banking/platforms",
            summary: "List delivery platforms",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/banking/reconcile",
            summary: "Run reconciliation",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/banking/reports",
            summary: "Banking reports",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/banking/reconcile/match",
            summary: "Manual reconciliation match",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/banking/imports/cashflow",
            summary: "Import a monthly cash-flow CSV",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/banking/diagnosis",
            summary: "Cash-flow diagnosis of the branch",
        },
        // Billing / payroll
        ApiEndpoint {
            method: "GET",
            path: "/api/billing/payroll/months",
            summary: "Monthly invoice totals as a payroll basis",
        },
        // Settings: API keys & webhooks
        ApiEndpoint {
            method: "GET",
            path: "/api/user/api-keys",
            summary: "List API keys with scopes and last-used time",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/user/api-keys",
            summary: "Create an API key (name, scopes, expiry days)",
        },
        ApiEndpoint {
            method: "DELETE",
            path: "/api/user/api-keys/:key_id",
            summary: "Revoke an API key",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/user/api-keys/:key_id/rotate",
            summary: "Rotate an API key secret",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/user/webhooks",
            summary: "List webhook endpoints",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/user/webhooks",
            summary: "Register a webhook endpoint (URL, events)",
        },
        ApiEndpoint {
            method: "DELETE",
            path: "/api/user/webhooks/:webhook_id",
            summary: "Delete a webhook endpoint",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/user/webhooks/:webhook_id/test",
            summary: "Send a signed test event to a webhook",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/user/webhooks/:webhook_id/deliveries",
            summary: "List webhook delivery history",
        },
        // Sources / connectors
        ApiEndpoint {
            method: "GET",
            path: "/api/integrations/connectors",
            summary: "List data connectors with type, status and last sync",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/integrations/connectors",
            summary: "Create or update a data connector (database, API, SaaS)",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/integrations/connectors/:id/test",
            summary: "Test connectivity for a data connector",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/integrations/connectors/:id/sync",
            summary: "Trigger an immediate sync for a data connector",
        },
        ApiEndpoint {
            method: "DELETE",
            path: "/api/integrations/connectors/:id/disconnect",
            summary: "Remove a data connector and its vaulted credentials",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/integrations/connectors/templates",
            summary: "Connector type catalog for typed configuration forms",
        },
        // Integrations: tenant-scoped connection control plane (#939)
        ApiEndpoint {
            method: "GET",
            path: "/api/bots/:bot_id/integration-connections",
            summary: "List integration connections for a bot; credentials stay in Vault",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/bots/:bot_id/integration-connections",
            summary: "Create an integration connection (provider_slug, display_name, auth_kind); secret keys are stored in Vault only",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/bots/:bot_id/integration-connections/:connection_id",
            summary: "Get one integration connection; never returns vault path or credential material",
        },
        ApiEndpoint {
            method: "DELETE",
            path: "/api/bots/:bot_id/integration-connections/:connection_id",
            summary: "Delete an integration connection and purge its Vault credentials",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/bots/:bot_id/integration-connections/:connection_id/test",
            summary: "Validate stored credentials shape (no outbound call); outcome is recorded as unverified",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/bots/:bot_id/integration-connections/:connection_id/rotate",
            summary: "Rotate connection credentials in Vault and increment credential_version",
        },
];
