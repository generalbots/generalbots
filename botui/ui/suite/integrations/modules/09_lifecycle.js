"use strict";

/* #2339 — make the blast radius of a revoke visible before the destructive
   confirm, using data already present in the workspace. */
(function (namespace) {
    function t(key, fallback, params) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    }

    namespace.revokeImpactText = function (root, connectionId) {
        var state = namespace.getState(root);
        var connection = state && Array.isArray(state.connectors)
            ? state.connectors.find(function (row) { return row.id === connectionId; })
            : null;
        if (!connection) {
            return "";
        }
        var slug = namespace.text(connection.provider_slug, "");
        var provider = namespace.escapeHtml(namespace.text(connection.display_name,
            slug || namespace.text(connection.id, "this connection")));
        var cards = root && slug ? root.querySelectorAll("[data-automation-id]") : [];
        var inspected = cards.length > 0;
        var automations = 0;
        Array.prototype.forEach.call(cards, function (card) {
            if (card.dataset.provider === slug) {
                automations += 1;
            }
        });
        var connections = 0;
        state.connectors.forEach(function (row) {
            if (row.id !== connectionId && row.provider_slug === slug) {
                connections += 1;
            }
        });
        if (!inspected) {
            return t("integrations-revoke-impact-unknown",
                "Dependents could not be inspected here. Revoke only if nothing else uses { $provider }.",
                { provider: provider });
        }
        return t("integrations-revoke-impact",
            "{ $provider } is used by { $automations } automation(s) and { $connections } other connection(s). They will stop working after revoke.",
            { provider: provider, automations: automations, connections: connections });
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
