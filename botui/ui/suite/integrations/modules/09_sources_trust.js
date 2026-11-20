"use strict";

/* #2339 — per-source trust summary: approval/change posture and the last
   successful or failed sync, rendered above the source view. No secret values
   are read. */
(function (namespace) {
    function t(key, fallback, params) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    }

    namespace.markSourceSync = function (root, id, error) {
        var state = namespace.getState(root);
        if (!state) {
            return;
        }
        state.sourceSync = state.sourceSync || {};
        state.sourceSync[id] = { at: Date.now(), error: error || "" };
    };

    namespace.renderSourceTrust = function (root) {
        var node = root && root.querySelector("[data-source-trust]");
        var state = root && namespace.getState(root);
        if (!node || !state) {
            return;
        }
        var definition = namespace.sourceDefinition(state.activeSource);
        var lines = [];
        if (definition.id === "mcp") {
            var servers = state.mcpServers || [];
            var pending = servers.filter(function (server) { return server.requires_approval === true; }).length;
            var changed = servers.filter(function (server) {
                return (server.ui_resources || []).some(function (resource) { return resource.pin_changed === true; });
            }).length;
            lines.push(pending
                ? t("integrations-source-approval-required", "Approval required: { $count } server(s)", { count: pending })
                : t("integrations-source-approved", "All registered servers are approved"));
            if (changed) {
                lines.push(t("integrations-source-changed", "Changed since approval: { $count } server(s)", { count: changed }));
            }
        }
        var sync = state.sourceSync && state.sourceSync[definition.id];
        if (sync) {
            var when = sync.at && window.gbFormatDateTime
                ? window.gbFormatDateTime(sync.at)
                : String(sync.at);
            lines.push(t("integrations-source-last-sync", "Last sync: { $when }", { when: when }));
            if (sync.error) {
                lines.push(t("integrations-source-sync-error", "Last sync error: { $error }", { error: sync.error }));
            }
        }
        node.textContent = lines.join(" · ");
    };

    /* #2339 — a confirmed remove path for MCP servers, using the existing
       DELETE /api/sources/mcp/:name route. The server name is escaped because
       the floating confirm renders its message as HTML. */
    namespace.removeMcpServer = async function (root, name) {
        var safeName = namespace.escapeHtml(String(name || ""));
        var message = t("integrations-source-remove-confirm",
            "Remove MCP server { $name }? Its tools and UI resources will no longer be available.",
            { name: safeName });
        var title = t("integrations-source-remove-title", "Remove MCP server");
        var confirmed = await new Promise(function (resolve) {
            if (window.GBModal && typeof window.GBModal.confirm === "function") {
                window.GBModal.confirm(message, title,
                    function () { resolve(true); }, function () { resolve(false); });
                return;
            }
            // #2297 — no themed modal: keep the server unchanged.
            resolve(false);
        });
        if (!confirmed) {
            return;
        }
        try {
            await namespace.fetchJson("/api/sources/mcp/" + encodeURIComponent(name), { method: "DELETE" });
            namespace.announce(root, t("integrations-source-removed",
                "MCP server { $name } removed", { name: safeName }));
        } catch (error) {
            namespace.announce(root, t("integrations-source-remove-failed", "MCP server removal failed"));
        }
        namespace.loadSources(root);
    };

    var originalRenderSources = namespace.renderSources;
    if (typeof originalRenderSources === "function") {
        namespace.renderSources = function (root) {
            originalRenderSources(root);
            namespace.renderSourceTrust(root);
        };
    }
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
