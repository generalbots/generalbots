"use strict";

(function (namespace) {
    namespace.catalogUrl = "/api/apps/integrations/catalog";
    namespace.contextUrl = "/api/apps/integrations/context";
    namespace.context = namespace.context || { botId: "", botName: "", loaded: false, error: "" };
    namespace.allowedAuthKinds = ["api_key", "basic", "token", "access_key", "oauth2", "protocol"];
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

    namespace.authHeaders = function (extra) {
        var token = localStorage.getItem("gb-access-token") ||
            sessionStorage.getItem("gb-access-token") ||
            localStorage.getItem("management_token") || "";
        var headers = { "Accept": "application/json" };
        if (token) {
            headers["Authorization"] = "Bearer " + token;
        }
        return Object.assign(headers, extra && typeof extra === "object" ? extra : {});
    };

    namespace.fetchJson = async function (url, options) {
        var config = options && typeof options === "object" ? options : {};
        var headers = namespace.authHeaders(config.headers);
        if (config.body !== undefined) {
            headers["Content-Type"] = "application/json";
        }
        var response = await fetch(url, {
            method: config.method || "GET",
            credentials: "same-origin",
            headers: headers,
            body: config.body === undefined ? undefined : JSON.stringify(config.body)
        });
        if (!response.ok) {
            var failure = new Error("Request failed with status " + response.status);
            failure.status = response.status;
            throw failure;
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

    namespace.normalizeConnections = function (payload) {
        var items = Array.isArray(payload.items) ? payload.items : [];
        return items.filter(function (item) { return item && typeof item === "object"; })
            .map(function (record) {
                var version = Number(record.credential_version);
                return {
                    id: namespace.text(record.id, ""),
                    provider_slug: namespace.text(record.provider_slug, ""),
                    display_name: namespace.text(record.display_name, ""),
                    auth_kind: namespace.text(record.auth_kind, ""),
                    status: namespace.text(record.status, ""),
                    credential_version: Number.isFinite(version) ? version : 0,
                    last_test_status: namespace.text(record.last_test_status, ""),
                    last_tested_at: namespace.text(record.last_tested_at, ""),
                    expires_at: namespace.text(record.expires_at, "")
                };
            })
            .filter(function (row) { return row.id; });
    };

    namespace.announce = function (root, message) {
        var live = root.querySelector("[data-global-live]");
        if (!live) {
            return;
        }
        live.textContent = "";
        window.requestAnimationFrame(function () { live.textContent = message; });
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
            contextStatus: "idle",
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

    namespace.fetchContext = async function (root) {
        var state = namespace.getState(root);
        if (!state || state.contextStatus === "loading" || state.contextStatus === "ready") {
            return namespace.context;
        }
        state.contextStatus = "loading";
        try {
            var payload = await namespace.fetchJson(namespace.contextUrl);
            namespace.context.botId = namespace.text(payload.bot_id, "");
            namespace.context.botName = namespace.text(payload.bot_name, "");
            namespace.context.loaded = !!namespace.context.botId;
            namespace.context.error = "";
            state.contextStatus = "ready";
        } catch (error) {
            namespace.context.botId = "";
            namespace.context.botName = "";
            namespace.context.loaded = false;
            namespace.context.error = error && error.message ? error.message : "context unavailable";
            state.contextStatus = "error";
        }
        return namespace.context;
    };

    namespace.connectionsUrl = function () {
        var botId = namespace.text(namespace.context.botId, "");
        return botId ? "/api/bots/" + encodeURIComponent(botId) + "/integration-connections" : "";
    };

    namespace.connectionActionUrl = function (connectionId, suffix) {
        var base = namespace.connectionsUrl();
        var id = namespace.text(connectionId, "");
        return base && id ? base + "/" + encodeURIComponent(id) + (suffix || "") : "";
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
