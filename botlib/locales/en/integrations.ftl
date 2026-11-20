# Integrations — actionable connection test result (#2345)
integrations-not-tested = Not tested yet
integrations-not-tested-hint = Run a test to verify stored credentials.
integrations-last-tested = Last tested
integrations-retest = Retest
integrations-test-now = Test
integrations-testing = Testing…
integrations-rotate-renew = Rotate / renew
integrations-rotate-unavailable = Credential rotation is unavailable in this build.
integrations-credential-expires-soon = Credential expires soon
integrations-credential-expired = Credential expired
integrations-revoke-confirm = Revoke this connection? Its stored credentials will be deleted.
integrations-revoke-title = Revoke connection

# semantic i18n key (#2542)
integrations-connection-catalog = Connection catalog

# semantic i18n key (#2542)
integrations-connection-catalog-desc = Review trusted connections that can power chat and other apps. Inspect every capability and permission before an adapter is configured.

# semantic i18n key (#2542)
integrations-loading-catalog = Loading catalog

# semantic i18n key (#2542)
integrations-explore-tab = Explore

# semantic i18n key (#2542)
integrations-automations-tab = Automations

# semantic i18n key (#2542)
integrations-search-providers-actions = Search providers and actions

# semantic i18n key (#2542)
integrations-availability = Availability

# semantic i18n key (#2542)
integrations-availability-must = Must

# semantic i18n key (#2542)
integrations-loading-providers = Loading providers

# semantic i18n key (#2542)
integrations-view-cards = Cards

# semantic i18n key (#2542)
integrations-view-list = List

# semantic i18n key (#2542)
integrations-runtime-inventory-eyebrow = Runtime inventory

# semantic i18n key (#2542)
integrations-connected-services-title = Connected services

# semantic i18n key (#2542)
integrations-connected-services-desc = Health and recent activity reported by the connector service.

# semantic i18n key (#2542)
integrations-refresh-connected = Refresh connected

# semantic i18n key (#2542)
integrations-loading-connected-services = Loading connected services

# semantic i18n key (#2542)
integrations-scheduled-execution-eyebrow = Scheduled execution

# semantic i18n key (#2542)
integrations-scheduled-execution-desc = Run an integration action on a repeating schedule using the vault-backed connection.

# semantic i18n key (#2542)
integrations-new-automation = New automation

# semantic i18n key (#2542)
integrations-provider-label = Provider *

# semantic i18n key (#2542)
integrations-action-label = Action *

# semantic i18n key (#2542)
integrations-schedule-label = Schedule *

# semantic i18n key (#2542)
integrations-every-5-minutes = Every 5 minutes

# semantic i18n key (#2542)
integrations-every-15-minutes = Every 15 minutes

# semantic i18n key (#2542)
integrations-hourly = Hourly

# semantic i18n key (#2542)
integrations-daily = Daily

# semantic i18n key (#2542)
integrations-weekly = Weekly

# semantic i18n key (#2542)
integrations-parameters-json = Parameters (JSON object)

# semantic i18n key (#2542)
integrations-create-automation = Create automation

# semantic i18n key (#2542)
integrations-loading-automations = Loading automations

# semantic i18n key (#2542)
integrations-inbound-ingestion-eyebrow = Inbound ingestion

# semantic i18n key (#2542)
integrations-inbound-ingestion-desc = MCP servers, repositories, models, mail accounts and BYOK keys that feed the agent with external context.

# semantic i18n key (#2542)
integrations-refresh-sources = Refresh sources

# semantic i18n key (#2542)
integrations-loading-data-sources = Loading data sources

# semantic i18n key (#2542)
integrations-provider-detail-eyebrow = Provider detail

# semantic i18n key (#2542)
integrations-integration-details-title = Integration details
# semantic i18n key (#2542)
integrations-integration-views-aria-label = Integration views
integrations-search-placeholder = Search AWS, messaging, storage...
integrations-search-aria-label = Search integration providers and actions
integrations-filter-availability-aria-label = Filter by availability
integrations-catalog-layout-aria-label = Catalog layout
integrations-integration-categories-aria-label = Integration categories
integrations-close-details-aria-label = Close integration details

# #2346 — catalog source transparency and provider editor
integrations-catalog-source-tab = Catalog source
integrations-catalog-source-eyebrow = Catalog provenance
integrations-catalog-source-title = Where providers come from
integrations-catalog-source-desc = Inspect where every provider definition comes from and manage entries added by operators or the community.
integrations-refresh-catalog-source = Refresh source
integrations-loading-catalog-source = Loading catalog source
integrations-source-seeded = Compiled seed
integrations-source-operator = Operator
integrations-source-community = Community
integrations-source-unknown = Unspecified
integrations-source-seeded-desc = Compiled into the server binary from Rust seed data. Changing it requires a code change and a redeploy.
integrations-source-operator-desc = Added by an operator through the provider editor and stored on the server.
integrations-source-community-desc = Contributed by the community. Review its actions before storing credentials.
integrations-source-unknown-desc = The catalog did not report where this definition came from.
integrations-source-id-required = Provider ID and name are required.
integrations-source-actions-invalid = Actions must be a valid JSON array.
integrations-source-saved = Provider saved.
integrations-source-deleted = Provider deleted.
integrations-source-save-failed = The provider could not be saved.
integrations-source-delete-failed = The provider could not be deleted.
integrations-source-delete-confirm = Delete this provider? It will disappear from the catalog immediately.
integrations-source-delete-confirm-title = Delete provider
integrations-source-none = No provider definitions are available.
integrations-source-col-provider = Provider
integrations-source-col-category = Category
integrations-source-col-source = Source
integrations-source-col-module = Module
integrations-source-actions = Actions
integrations-source-col-manage = Manage
integrations-source-id = Provider ID
integrations-source-name = Name
integrations-source-category = Category
integrations-source-module = Module
integrations-source-status = Status
integrations-source-credentials-url = Credentials URL
integrations-source-docs-url = Official docs URL
integrations-source-setup-steps = Setup steps (one per line)
integrations-source-guidance = Guidance
integrations-source-new-provider = New provider
integrations-source-save-provider = Save provider
integrations-source-editor-hint = Provider definitions are stored on the server. Changes appear in the catalog immediately after saving.
integrations-source-readonly-title = Source transparency
integrations-source-readonly-desc = This build serves a compile-time catalog. Provider definitions live in botserver/src/apps/integration_catalog/providers/*.rs and are expanded at boot; adding one needs a code change and a redeploy.
integrations-source-error-title = Provider definitions unavailable
integrations-source-error = The provider definitions endpoint did not answer. The list below is the compiled catalog, shown so the source of every provider stays visible.
integrations-source-provider-list-title = Provider definitions
integrations-source-edit-provider = Edit
integrations-source-delete-provider = Delete
integrations-source-detail-title = Source and provenance
integrations-source-detail-source = Source

integrations-automation-param-invalid = Check the value for { $field } ({ $type }).
integrations-automation-param-required = { $field } is required.
integrations-connected-no-results = No connected services match your search.
integrations-connected-no-results-hint = Try a broader term or clear the search field.
integrations-connected-search = Search connected services
integrations-connected-sort = Sort connected services
integrations-copy-diagnostics = Copy diagnostics
integrations-diagnostics-detail = Detail: { $detail }
integrations-diagnostics-latency = Latency: { $ms } ms
integrations-diagnostics-outcome = Outcome: { $outcome }
integrations-diagnostics-provider = Provider: { $slug }
integrations-diagnostics-title = Connection diagnostics
integrations-revoke-impact = { $provider } is used by { $automations } automation(s) and { $connections } other connection(s). They will stop working after revoke.
integrations-revoke-impact-unknown = Dependents could not be inspected here. Revoke only if nothing else uses { $provider }.
integrations-source-approval-required = Approval required: { $count } server(s)
integrations-source-approved = All registered servers are approved
integrations-source-changed = Changed since approval: { $count } server(s)
integrations-source-last-sync = Last sync: { $when }
integrations-source-sync-error = Last sync error: { $error }

integrations-source-remove-confirm = Remove MCP server { $name }? Its tools and UI resources will no longer be available.
integrations-source-remove-title = Remove MCP server
integrations-source-removed = MCP server { $name } removed
integrations-source-remove-failed = MCP server removal failed
