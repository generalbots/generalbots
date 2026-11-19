"use strict";

/* #2346 — catalog source/provenance vocabulary. Loaded before 01_state.js so
   the namespace helper exists before any catalog is normalized, but nothing is
   called at load time. installSourceEnrichment() is invoked by the catalog
   source module (which loads after 01_state.js) to decorate its normalizers. */
(function (namespace) {
    namespace.providersUrl = "/api/apps/integrations/providers";

    /* Today every catalog provider is a Rust seed compiled into the binary, so
       that is the truthful fallback until a backend sends an explicit source.
       Unknown values stay unknown instead of being relabelled as first-party. */
    namespace.sourceKind = function (value) {
        var kind = String(value === null || value === undefined ? "" : value).trim().toLowerCase();
        if (kind === "seeded:hardcoded" || kind === "seeded" || kind === "hardcoded" || kind === "seed") {
            return "seeded:hardcoded";
        }
        if (kind === "operator" || kind === "seeded:operator") {
            return "operator";
        }
        if (kind === "community" || kind === "seeded:community") {
            return "community";
        }
        return kind || "seeded:hardcoded";
    };

    namespace.installSourceEnrichment = function () {
        if (namespace.sourceEnrichmentInstalled === true) {
            return;
        }
        namespace.sourceEnrichmentInstalled = true;

        var normalizeProvider = namespace.normalizeProvider;
        namespace.normalizeProvider = function (provider) {
            var normalized = normalizeProvider(provider);
            var source = provider && typeof provider === "object" ? provider : {};
            normalized.source = namespace.sourceKind(source.source || source.provenance);
            normalized.actions = Array.isArray(normalized.actions) ? normalized.actions.map(function (action) {
                return Object.assign({}, action, { source: namespace.sourceKind(action.source || action.provenance) });
            }) : [];
            return normalized;
        };

        var createState = namespace.createState;
        namespace.createState = function (root) {
            var state = createState(root);
            state.providerSourceStatus = "idle";
            state.providerSourceRequest = 0;
            state.providerDefinitions = [];
            state.providerSourceEditable = false;
            state.providerSourceMessage = "";
            state.editingProviderId = "";
            state.editingProviderOriginal = null;
            return state;
        };
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
