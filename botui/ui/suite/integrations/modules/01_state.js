"use strict";

(function (namespace) {
    namespace.catalogUrl = "/api/apps/integrations/catalog";
    namespace.connectedUrl = "/api/integrations/connectors";
    namespace.states = namespace.states || new WeakMap();
    namespace.instanceCount = namespace.instanceCount || 0;
    namespace.categoryOrder = [
        "featured",
        "productivity",
        "developer",
        "startups",
        "small_business",
        "finance",
        "social_messaging",
        "lifestyle"
    ];

    namespace.escapeHtml = function (value) {
        return String(value === null || value === undefined ? "" : value)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#39;");
    };

    namespace.text = function (value, fallback) {
        if (typeof value === "string" && value.trim()) {
            return value.trim();
        }
        if (typeof value === "number" && Number.isFinite(value)) {
            return String(value);
        }
        return fallback || "";
    };

    namespace.titleCase = function (value, fallback) {
        var text = namespace.text(value, fallback || "Unknown");
        return text.replace(/[-_]+/g, " ").replace(/\b\w/g, function (letter) {
            return letter.toUpperCase();
        });
    };

    namespace.safeUrl = function (value) {
        if (typeof value !== "string" || !value.trim()) {
            return "";
        }
        try {
            var url = new URL(value, window.location.origin);
            return url.protocol === "http:" || url.protocol === "https:" ? url.href : "";
        } catch (error) {
            return "";
        }
    };

    namespace.fetchJson = async function (url) {
        var response = await fetch(url, {
            credentials: "same-origin",
            headers: { "Accept": "application/json" }
        });
        if (!response.ok) {
            throw new Error("Request failed with status " + response.status);
        }
        var payload = await response.json();
        if (!payload || typeof payload !== "object") {
            throw new Error("Invalid response payload");
        }
        return payload;
    };

    namespace.normalizeProvider = function (provider) {
        var source = provider && typeof provider === "object" ? provider : {};
        var auth = source.auth && typeof source.auth === "object" ? source.auth : {};
        var actions = Array.isArray(source.actions) ? source.actions.filter(function (action) {
            return action && typeof action === "object";
        }) : [];
        var count = Number(source.action_count);
        return {
            id: namespace.text(source.id, namespace.text(source.name, "provider")),
            name: namespace.text(source.name, namespace.titleCase(source.id, "Provider")),
            category: namespace.text(source.category, "Other"),
            strategy: namespace.text(source.strategy, "Review capabilities and setup guidance."),
            status: namespace.text(source.status, "planned"),
            priority: namespace.text(source.priority, "standard"),
            module: namespace.text(source.module, "Core"),
            official_docs: namespace.text(source.official_docs, ""),
            auth: auth,
            actions: actions,
            action_count: Number.isFinite(count) ? count : actions.length,
            llm_available: source.llm_available === true
        };
    };

    function normalizeCategories(rawCategories, providers) {
        var values = [];
        if (Array.isArray(rawCategories)) {
            values = rawCategories;
        } else if (rawCategories && typeof rawCategories === "object") {
            values = Object.keys(rawCategories).map(function (key) {
                var entry = rawCategories[key];
                return entry && typeof entry === "object" ? Object.assign({ id: key }, entry) : key;
            });
        }
        if (!values.length) {
            values = providers.map(function (provider) { return provider.category; });
        }
        var seen = new Set();
        return values.reduce(function (categories, category) {
            var value = typeof category === "string" ? category : namespace.text(
                category.id || category.value || category.slug || category.name,
                "Other"
            );
            var key = value.toLowerCase();
            if (!seen.has(key)) {
                seen.add(key);
                var providerCount = providers.filter(function (provider) {
                    return provider.category.toLowerCase() === key;
                }).length;
                categories.push({
                    value: value,
                    label: typeof category === "string" ? namespace.titleCase(category) : namespace.text(category.label || category.name, namespace.titleCase(value)),
                    count: typeof category === "object" && Number.isFinite(Number(category.count)) ? Number(category.count) : providerCount
                });
            }
            return categories;
        }, []).sort(function (a, b) {
            var aIndex = namespace.categoryOrder.indexOf(a.value.toLowerCase());
            var bIndex = namespace.categoryOrder.indexOf(b.value.toLowerCase());
            aIndex = aIndex === -1 ? namespace.categoryOrder.length : aIndex;
            bIndex = bIndex === -1 ? namespace.categoryOrder.length : bIndex;
            return aIndex === bIndex ? a.label.localeCompare(b.label) : aIndex - bIndex;
        });
    }

    namespace.normalizeCatalog = function (payload) {
        if (!Array.isArray(payload.providers)) {
            throw new Error("Catalog providers are unavailable");
        }
        var providers = payload.providers.map(namespace.normalizeProvider);
        return {
            providers: providers,
            categories: normalizeCategories(payload.categories, providers),
            totals: payload.totals && typeof payload.totals === "object" ? payload.totals : {}
        };
    };

    namespace.normalizeConnectors = function (payload) {
        var items = Array.isArray(payload.connectors) ? payload.connectors : payload.items;
        if (!Array.isArray(items)) {
            throw new Error("Connector list is unavailable");
        }
        return items.filter(function (item) { return item && typeof item === "object"; });
    };

    namespace.createState = function (root) {
        namespace.instanceCount += 1;
        var state = {
            uid: "integrations-" + namespace.instanceCount,
            root: root,
            activeTab: "explore",
            providers: [],
            providerMap: new Map(),
            categories: [],
            totals: {},
            connectors: [],
            catalogStatus: "loading",
            connectedStatus: "loading",
            search: "",
            category: "all",
            statusFilter: "all",
            view: "grid",
            exploreTouched: false,
            selectedProviderId: "",
            detailStatus: "idle",
            detailRequest: 0,
            catalogRequest: 0,
            connectedRequest: 0,
            lastFocus: null
        };
        namespace.states.set(root, state);
        return state;
    };

    namespace.getState = function (root) {
        return namespace.states.get(root);
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
