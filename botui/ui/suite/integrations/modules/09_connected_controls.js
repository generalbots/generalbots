"use strict";

/* #2339 — search and sort for the connected inventory. Filtering works on a
   copy so the connector list stays authoritative for diagnostics and lifecycle
   helpers. */
(function (namespace) {
    function t(key, fallback, params) {
        // #2339 — forward interpolation params (e.g. { $ms }) to gbT; the
        // two-arg wrapper dropped them, leaving placeholders/values blank in
        // last-test diagnostics and source labels.
        return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    }

    var originalCreateState = namespace.createState;
    if (typeof originalCreateState === "function") {
        namespace.createState = function (root) {
            var state = originalCreateState(root);
            state.connectedQuery = "";
            state.connectedSort = "status";
            return state;
        };
    }

    namespace.visibleConnections = function (state) {
        var rows = Array.isArray(state && state.connectors) ? state.connectors.slice() : [];
        var query = String((state && state.connectedQuery) || "").trim().toLowerCase();
        if (query) {
            rows = rows.filter(function (row) {
                var scopes = Array.isArray(row.granted_scopes) ? row.granted_scopes.join(" ") : "";
                return [row.display_name, row.provider_slug, row.auth_kind, row.status, row.id, scopes]
                    .join(" ").toLowerCase().indexOf(query) !== -1;
            });
        }
        var weight = {
            active: 0, healthy: 0, ok: 0, connected: 0,
            warning: 1, revoked: 2, failed: 2, error: 2, unhealthy: 2, expired: 2
        };
        rows.sort(function (a, b) {
            var sort = (state && state.connectedSort) || "status";
            if (sort === "name") {
                return String(a.display_name || a.provider_slug).localeCompare(String(b.display_name || b.provider_slug));
            }
            if (sort === "tested") {
                return String(b.last_tested_at || "").localeCompare(String(a.last_tested_at || ""));
            }
            if (sort === "status") {
                var aw = weight[String(a.status).toLowerCase()];
                var bw = weight[String(b.status).toLowerCase()];
                return (aw === undefined ? 1 : aw) - (bw === undefined ? 1 : bw);
            }
            return 0;
        });
        return rows;
    };

    namespace.connectedNoResults = function (state) {
        return namespace.stateView("0",
            t("integrations-connected-no-results", "No connected services match your search."),
            t("integrations-connected-no-results-hint", "Try a broader term or clear the search field."),
            "clear-connected-search", t("action-clear", "Clear"), false);
    };

    var originalRenderConnected = namespace.renderConnected;
    if (typeof originalRenderConnected === "function") {
        namespace.renderConnected = function (root) {
            var state = namespace.getState(root);
            if (!state) {
                originalRenderConnected(root);
                return;
            }
            var full = state.connectors;
            var filtered = namespace.visibleConnections(state);
            var view = namespace.query(root, "[data-connected-view]");
            state.connectors = filtered;
            try {
                originalRenderConnected(root);
            } finally {
                state.connectors = full;
            }
            if (full.length && !filtered.length && view) {
                view.innerHTML = namespace.connectedNoResults(state);
            }
        };
    }

    namespace.installConnectedControls = function (root) {
        if (root.dataset.connectedControlsReady === "true") {
            return;
        }
        root.dataset.connectedControlsReady = "true";
        var search = root.querySelector("[data-connected-search]");
        var sort = root.querySelector("[data-connected-sort]");
        if (search) {
            search.addEventListener("input", function () {
                var state = namespace.getState(root);
                if (state) {
                    state.connectedQuery = search.value.trim();
                    namespace.renderConnected(root);
                }
            });
        }
        if (sort) {
            sort.addEventListener("change", function () {
                var state = namespace.getState(root);
                if (state) {
                    state.connectedSort = sort.value;
                    namespace.renderConnected(root);
                }
            });
        }
        root.addEventListener("click", function (event) {
            var clear = event.target && event.target.closest
                ? event.target.closest('[data-action="clear-connected-search"]')
                : null;
            if (!clear || !root.contains(clear)) {
                return;
            }
            var state = namespace.getState(root);
            if (!state) {
                return;
            }
            state.connectedQuery = "";
            if (search) {
                search.value = "";
            }
            namespace.renderConnected(root);
            if (search) {
                search.focus();
            }
        });
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
