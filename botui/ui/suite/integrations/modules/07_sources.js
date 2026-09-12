"use strict";

/* Data Sources tab (#1352) — folds the Sources ingestion capabilities into the
   Integrations app. Every call targets an endpoint that already exists:
     /api/sources/mcp            JSON   MCP server catalog
     /api/sources/mcp/:name/…    JSON   enable/disable a server
     /api/ui/sources/repositories  HTML fragment
     /api/ui/sources/models        HTML fragment
     /api/ui/sources/accounts      HTML fragment (IMAP accounts)
     /api/ui/sources/api-keys      HTML fragment (BYOK keys)
   No new botserver routes are introduced. */

(function (namespace) {
    namespace.sources = [
        { id: "mcp", label: "MCP Servers", endpoint: "/api/sources/mcp", kind: "json" },
        { id: "repositories", label: "Repositories", endpoint: "/api/ui/sources/repositories", kind: "fragment" },
        { id: "models", label: "Models", endpoint: "/api/ui/sources/models", kind: "fragment" },
        { id: "accounts", label: "Accounts", endpoint: "/api/ui/sources/accounts", kind: "fragment" },
        { id: "api-keys", label: "API Keys", endpoint: "/api/ui/sources/api-keys", kind: "fragment" }
    ];

    namespace.sourceDefinition = function (id) {
        return namespace.sources.find(function (source) { return source.id === id; }) || namespace.sources[0];
    };

    namespace.normalizeMcpServers = function (payload) {
        var items = [];
        if (payload && Array.isArray(payload.data)) {
            items = payload.data;
        } else if (payload && Array.isArray(payload.items)) {
            items = payload.items;
        } else if (Array.isArray(payload)) {
            items = payload;
        }
        return items.filter(function (item) { return item && typeof item === "object"; })
            .map(function (server) {
                var tools = Number(server.tools_count);
                return {
                    name: namespace.text(server.name, "Unnamed server"),
                    description: namespace.text(server.description, ""),
                    server_type: namespace.text(server.server_type, "stdio"),
                    status: namespace.text(server.status, "unknown"),
                    enabled: server.enabled === true,
                    tools_count: Number.isFinite(tools) ? tools : 0,
                    requires_approval: server.requires_approval === true
                };
            });
    };

    function statusBadge(value, healthy, warning) {
        var key = String(value).toLowerCase();
        var tone = [healthy].indexOf(key) !== -1 ? "is-healthy" :
            (["error", "failed", "disabled"].indexOf(key) !== -1 ? "is-error" : "is-warning");
        return '<span class="integrations-health-badge ' + tone + '">' +
            namespace.escapeHtml(namespace.titleCase(value, "Unknown")) + '</span>';
    }

    namespace.renderSources = function (root) {
        var state = namespace.getState(root);
        var view = namespace.query(root, "[data-sources-view]");
        var status = namespace.query(root, "[data-sources-status]");
        if (!view) {
            return;
        }
        view.setAttribute("aria-busy", "false");
        if (state.sourcesStatus === "loading") {
            status.textContent = "Loading " + namespace.sourceDefinition(state.activeSource).label;
            view.innerHTML = '<div class="integrations-skeleton-grid" aria-hidden="true">' + namespace.skeletonCards(3) + '</div>';
            return;
        }
        if (state.sourcesStatus === "error") {
            status.textContent = "Data sources unavailable";
            view.innerHTML = namespace.stateView("!", "Data sources could not be loaded", "The sources service did not respond. Connections and the provider catalog remain available.", "retry-sources", "Try again", true);
            return;
        }

        var definition = namespace.sourceDefinition(state.activeSource);
        if (definition.kind === "fragment") {
            status.textContent = definition.label + " served by the sources service";
            view.innerHTML = '<div class="integrations-sources-fragment">' + state.sourcesHtml + '</div>';
            return;
        }

        var servers = state.mcpServers;
        status.textContent = servers.length + " MCP server" + (servers.length === 1 ? "" : "s");
        if (!servers.length) {
            view.innerHTML = namespace.stateView("0", "No MCP servers registered", "Register an MCP server to let the agent call external tools over the Model Context Protocol.", "", "", false);
            return;
        }
        view.innerHTML = '<div class="integrations-sources-grid">' + servers.map(mcpCard).join("") + '</div>';
    };

    /* query, skeletonCards and stateView come from 02_render.js via the
       namespace — defined once, reused here. */

    function mcpCard(server) {
        var toggle = '<button class="integrations-button integrations-button-subtle" type="button" data-action="mcp-toggle"' +
            ' data-mcp-name="' + namespace.escapeHtml(server.name) + '"' +
            ' data-mcp-enabled="' + (server.enabled ? "true" : "false") + '">' +
            (server.enabled ? "Disable" : "Enable") + '</button>';
        return '<article class="integrations-source-card">' +
            '<header class="integrations-source-card-header">' +
            '<span class="integrations-source-icon" aria-hidden="true">' + (server.enabled ? "🟢" : "⚪") + '</span>' +
            '<div><h3>' + namespace.escapeHtml(server.name) + '</h3>' +
            '<p class="integrations-cell-subtext">' + namespace.escapeHtml(namespace.titleCase(server.server_type, "Server")) +
            (server.requires_approval ? " · approval required" : "") + '</p></div>' +
            statusBadge(server.enabled ? "active" : "disabled", "active") +
            '</header>' +
            '<p class="integrations-source-description">' + namespace.escapeHtml(server.description || "No description provided.") + '</p>' +
            '<footer class="integrations-source-card-footer">' +
            '<span class="integrations-faint-text">' + server.tools_count + ' tool' + (server.tools_count === 1 ? "" : "s") + '</span>' +
            '<span class="integrations-row-actions">' + toggle + '</span>' +
            '</footer>' +
            '</article>';
    }

    namespace.renderSourcesLoading = function (root) {
        var state = namespace.getState(root);
        state.sourcesStatus = "loading";
        namespace.renderSources(root);
    };

    function fetchFragment(url) {
        return fetch(url, { credentials: "same-origin", headers: namespace.authHeaders({ "Accept": "text/html" }) })
            .then(function (response) {
                if (!response.ok) {
                    var failure = new Error("Request failed with status " + response.status);
                    failure.status = response.status;
                    throw failure;
                }
                return response.text();
            });
    }

    namespace.setSource = function (root, id) {
        var state = namespace.getState(root);
        state.activeSource = id;
        state.sourcesStatus = "idle";
        var chips = root.querySelectorAll("[data-source]");
        chips.forEach(function (chip) {
            chip.setAttribute("aria-pressed", String(chip.dataset.source === id));
        });
        namespace.loadSources(root);
    };

    namespace.loadSources = async function (root) {
        var state = namespace.getState(root);
        state.sourcesRequest += 1;
        var requestId = state.sourcesRequest;
        var definition = namespace.sourceDefinition(state.activeSource);
        namespace.renderSourcesLoading(root);
        try {
            if (definition.kind === "json") {
                var payload = await namespace.fetchJson(definition.endpoint);
                if (requestId !== state.sourcesRequest) {
                    return;
                }
                state.mcpServers = namespace.normalizeMcpServers(payload);
            } else {
                var html = await fetchFragment(definition.endpoint);
                if (requestId !== state.sourcesRequest) {
                    return;
                }
                state.sourcesHtml = html;
            }
            state.sourcesStatus = "ready";
            namespace.renderSources(root);
            namespace.announce(root, definition.label + " loaded");
        } catch (error) {
            if (requestId === state.sourcesRequest) {
                state.sourcesStatus = "error";
                namespace.renderSources(root);
                namespace.announce(root, definition.label + " could not be loaded");
            }
        }
    };

    namespace.toggleMcpServer = async function (root, name, enabled) {
        var target = "/api/sources/mcp/" + encodeURIComponent(name) + "/" + (enabled ? "disable" : "enable");
        try {
            await namespace.fetchJson(target, { method: "POST", body: {} });
            namespace.announce(root, "MCP server " + (enabled ? "disabled" : "enabled"));
        } catch (error) {
            namespace.announce(root, "MCP server update failed");
        }
        namespace.loadSources(root);
    };

    namespace.sourceChips = function (activeId) {
        return namespace.sources.map(function (source) {
            return '<button class="integrations-chip" type="button" data-source="' + source.id + '"' +
                ' aria-pressed="' + String(source.id === activeId) + '">' + namespace.escapeHtml(source.label) + '</button>';
        }).join("");
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
