"use strict";

(function (namespace) {
    function query(root, selector) {
        return root.querySelector(selector);
    }

    function initials(name) {
        var parts = namespace.text(name, "IN").split(/\s+/).filter(Boolean);
        return parts.slice(0, 2).map(function (part) { return part.charAt(0); }).join("").toUpperCase();
    }

    function tone(provider) {
        var hash = Array.from(provider.id).reduce(function (total, char) {
            return total + char.charCodeAt(0);
        }, 0);
        return "is-tone-" + ((hash % 6) + 1);
    }

    function implementationCount(provider) {
        return provider.actions.filter(function (action) { return action.implemented === true; }).length;
    }

    function availability(provider) {
        if (provider.llm_available) {
            return { label: "Available to assistants", className: "is-ready" };
        }
        return { label: "Configuration requires adapter", className: "is-planned" };
    }

    function skeletonCards(count) {
        return Array.from({ length: count }, function () {
            return '<div class="integrations-skeleton-card">' +
                '<div class="integrations-skeleton-line is-mark"></div>' +
                '<div class="integrations-skeleton-line is-medium"></div>' +
                '<div class="integrations-skeleton-line"></div>' +
                '<div class="integrations-skeleton-line is-short"></div>' +
                '</div>';
        }).join("");
    }

    namespace.renderCatalogLoading = function (root) {
        var catalog = query(root, "[data-catalog-view]");
        var categories = query(root, "[data-category-filters]");
        var notice = query(root, "[data-explore-notice]");
        query(root, "[data-result-status]").textContent = "Loading providers";
        catalog.setAttribute("aria-busy", "true");
        catalog.innerHTML = '<div class="integrations-skeleton-grid" aria-hidden="true">' + skeletonCards(6) + '</div>';
        categories.setAttribute("aria-busy", "true");
        categories.innerHTML = "";
        query(root, "[data-category-panel]").removeAttribute("aria-labelledby");
        notice.innerHTML = "";
        namespace.renderCatalogHealth(root);
    };

    namespace.renderCatalogHealth = function (root) {
        var state = namespace.getState(root);
        var health = query(root, "[data-catalog-health]");
        var healthText = query(root, "[data-catalog-health-text]");
        health.classList.remove("is-ready", "is-error");
        if (state.catalogStatus === "ready") {
            var actions = state.providers.reduce(function (total, provider) { return total + provider.action_count; }, 0);
            health.classList.add("is-ready");
            healthText.textContent = state.providers.length + " providers / " + actions + " actions";
        } else if (state.catalogStatus === "error") {
            health.classList.add("is-error");
            healthText.textContent = "Catalog unavailable";
        } else {
            healthText.textContent = "Loading catalog";
        }
    };

    function matchesProvider(provider, state) {
        var queryText = state.search.toLowerCase();
        var actionText = provider.actions.map(function (action) {
            return [action.name, action.label, action.verb, action.summary].join(" ");
        }).join(" ");
        var haystack = [provider.name, provider.id, provider.category, provider.module, provider.strategy, actionText]
            .join(" ").toLowerCase();
        var categoryMatch = state.category === "all" || provider.category.toLowerCase() === state.category.toLowerCase();
        var priority = provider.priority.toLowerCase();
        var status = provider.status.toLowerCase();
        var statusMatch = state.statusFilter === "all" ||
            (state.statusFilter === "must" && (priority === "must" || status === "must")) ||
            (state.statusFilter === "available" && (provider.llm_available || ["available", "ready", "implemented", "active"].indexOf(status) !== -1));
        return (!queryText || haystack.indexOf(queryText) !== -1) && categoryMatch && statusMatch;
    }

    function categoryButtons(state) {
        var allCount = state.providers.length;
        var buttons = [{ value: "all", label: "All", count: allCount }].concat(state.categories);
        return buttons.map(function (category, index) {
            var selected = state.category.toLowerCase() === category.value.toLowerCase();
            var count = category.count === null || category.count === undefined ? "" : " " + category.count;
            return '<button type="button" role="tab" id="' + state.uid + '-category-tab-' + index + '" ' +
                'data-category="' + namespace.escapeHtml(category.value) + '" aria-selected="' + selected + '" ' +
                'aria-controls="' + state.uid + '-category-panel" tabindex="' + (selected ? "0" : "-1") + '">' +
                namespace.escapeHtml(category.label) + namespace.escapeHtml(count) + '</button>';
        }).join("");
    }

    function selectedCategory(state) {
        if (state.category === "all") {
            return { label: "All", count: state.providers.length };
        }
        return state.categories.find(function (category) {
            return category.value.toLowerCase() === state.category.toLowerCase();
        }) || { label: namespace.titleCase(state.category), count: state.providers.length };
    }

    function providerCard(provider) {
        var available = availability(provider);
        var implemented = implementationCount(provider);
        var priorityClass = provider.priority.toLowerCase() === "must" ? " is-must" : "";
        var actionLabel = provider.action_count + (implemented ? " / " + implemented + " live" : " defined");
        var accessibleLabel = [provider.name, namespace.titleCase(provider.category), namespace.titleCase(provider.priority) + " priority",
            namespace.titleCase(provider.status), namespace.titleCase(provider.auth.method, "Authentication not specified"),
            provider.action_count + " actions", provider.module + " module", available.label, "View details"].join(", ");
        return '<button class="integrations-provider-card" type="button" data-provider-id="' + namespace.escapeHtml(provider.id) + '" ' +
            'aria-label="' + namespace.escapeHtml(accessibleLabel) + '">' +
            '<span class="integrations-card-head">' +
                '<span class="integrations-provider-mark ' + tone(provider) + '" aria-hidden="true">' + namespace.escapeHtml(initials(provider.name)) + '</span>' +
                '<span class="integrations-card-badges">' +
                    '<span class="integrations-pill' + priorityClass + '">' + namespace.escapeHtml(namespace.titleCase(provider.priority)) + '</span>' +
                    '<span class="integrations-pill">' + namespace.escapeHtml(namespace.titleCase(provider.category)) + '</span>' +
                '</span>' +
            '</span>' +
            '<span class="integrations-card-copy">' +
                '<span class="integrations-card-title-row"><span class="integrations-card-title">' + namespace.escapeHtml(provider.name) + '</span></span>' +
                '<span class="integrations-card-description">' + namespace.escapeHtml(provider.strategy) + '</span>' +
            '</span>' +
            '<span class="integrations-card-facts">' +
                '<span class="integrations-card-fact"><span class="integrations-card-fact-label">Status</span><span class="integrations-card-fact-value">' + namespace.escapeHtml(namespace.titleCase(provider.status)) + '</span></span>' +
                '<span class="integrations-card-fact"><span class="integrations-card-fact-label">Auth</span><span class="integrations-card-fact-value">' + namespace.escapeHtml(namespace.titleCase(provider.auth.method, "Not specified")) + '</span></span>' +
                '<span class="integrations-card-fact"><span class="integrations-card-fact-label">Actions</span><span class="integrations-card-fact-value">' + namespace.escapeHtml(actionLabel) + '</span></span>' +
                '<span class="integrations-card-fact"><span class="integrations-card-fact-label">App module</span><span class="integrations-card-fact-value">' + namespace.escapeHtml(provider.module) + '</span></span>' +
            '</span>' +
            '<span class="integrations-card-footer">' +
                '<span class="integrations-availability ' + available.className + '">' + namespace.escapeHtml(available.label) + '</span>' +
                '<span class="integrations-card-link">View details</span>' +
            '</span>' +
        '</button>';
    }

    function firstUseNotice(state) {
        if (state.exploreTouched || !state.providers.length) {
            return "";
        }
        return '<div class="integrations-guide">' +
            '<span class="integrations-guide-mark" aria-hidden="true">01</span>' +
            '<div><strong>Review before configuring</strong>' +
            '<p>Open a provider to inspect its actions, secret-field metadata, and least-privilege guidance. This catalog never saves credentials.</p></div>' +
        '</div>';
    }

    function stateView(mark, title, copy, action, actionLabel, isError) {
        return '<div class="integrations-state' + (isError ? ' is-error' : '') + '">' +
            '<span class="integrations-state-mark" aria-hidden="true">' + mark + '</span>' +
            '<h3>' + namespace.escapeHtml(title) + '</h3><p>' + namespace.escapeHtml(copy) + '</p>' +
            (action ? '<button class="integrations-button" type="button" data-action="' + action + '">' + namespace.escapeHtml(actionLabel) + '</button>' : '') +
        '</div>';
    }

    namespace.renderExplore = function (root) {
        var state = namespace.getState(root);
        var catalog = query(root, "[data-catalog-view]");
        var filtered = state.providers.filter(function (provider) { return matchesProvider(provider, state); });
        var categoryList = query(root, "[data-category-filters]");
        var categoryPanel = query(root, "[data-category-panel]");
        var activeCategory = selectedCategory(state);
        categoryList.innerHTML = categoryButtons(state);
        categoryList.setAttribute("aria-busy", "false");
        categoryPanel.setAttribute("aria-labelledby", query(root, '[data-category][aria-selected="true"]').id);
        query(root, "[data-explore-notice]").innerHTML = firstUseNotice(state);
        query(root, "[data-result-status]").textContent = filtered.length + " of " + activeCategory.count +
            " providers in " + activeCategory.label;
        catalog.setAttribute("aria-busy", "false");
        if (!state.providers.length) {
            catalog.innerHTML = stateView("0", "The catalog is empty", "No providers were returned. Refresh after catalog configuration is available.", "retry-catalog", "Refresh catalog", false);
        } else if (!filtered.length) {
            catalog.innerHTML = stateView("0", "No matching integrations", "Try a broader search or clear the category and availability filters.", "clear-filters", "Clear filters", false);
        } else {
            catalog.innerHTML = '<div class="integrations-catalog-grid ' + (state.view === "list" ? "is-list" : "") + '">' +
                filtered.map(providerCard).join("") + '</div>';
        }
        namespace.renderCatalogHealth(root);
    };

    namespace.renderCatalogError = function (root) {
        var catalog = query(root, "[data-catalog-view]");
        var categories = query(root, "[data-category-filters]");
        categories.innerHTML = "";
        categories.setAttribute("aria-busy", "false");
        query(root, "[data-category-panel]").removeAttribute("aria-labelledby");
        query(root, "[data-explore-notice]").innerHTML = "";
        query(root, "[data-result-status]").textContent = "Provider catalog unavailable";
        catalog.setAttribute("aria-busy", "false");
        catalog.innerHTML = stateView("!", "We could not load the catalog", "The provider service did not respond. Connected services remain independent and may still be available.", "retry-catalog", "Try again", true);
        namespace.renderCatalogHealth(root);
    };

    function formatDate(value) {
        if (!value) {
            return "Never";
        }
        var date = new Date(value);
        if (Number.isNaN(date.getTime())) {
            return "Not reported";
        }
        return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(date);
    }

    function healthFor(connector) {
        var raw = namespace.text(connector.health || connector.last_test_status || connector.test_status || connector.status,
            connector.active === true ? "healthy" : "not reported");
        var key = raw.toLowerCase();
        var className = ["healthy", "ok", "active", "connected", "passing"].indexOf(key) !== -1 ? "is-healthy" :
            (["failed", "error", "unhealthy", "offline"].indexOf(key) !== -1 ? "is-error" : "is-warning");
        return { label: namespace.titleCase(raw), className: className };
    }

    namespace.renderConnectedLoading = function (root) {
        var view = query(root, "[data-connected-view]");
        query(root, "[data-connected-status]").textContent = "Loading connected services";
        view.setAttribute("aria-busy", "true");
        view.innerHTML = '<div class="integrations-skeleton-grid" aria-hidden="true">' + skeletonCards(3) + '</div>';
    };

    namespace.renderConnected = function (root) {
        var state = namespace.getState(root);
        var view = query(root, "[data-connected-view]");
        query(root, "[data-connected-status]").textContent = state.connectors.length + " connected service" + (state.connectors.length === 1 ? "" : "s");
        view.setAttribute("aria-busy", "false");
        if (!state.connectors.length) {
            view.innerHTML = stateView("0", "No connected services yet", "Explore the catalog to review providers. Secure connection setup becomes available only through a documented adapter.", "show-explore", "Explore providers", false);
            return;
        }
        var rows = state.connectors.map(function (connector) {
            var health = healthFor(connector);
            var name = namespace.text(connector.name, "Unnamed connector");
            var type = namespace.text(connector.type || connector.kind || connector.connector_type || connector.provider, "Connector");
            var lastSync = connector.last_sync || connector.last_synced_at || connector.last_sync_at;
            var lastTest = connector.last_test || connector.last_test_at || connector.tested_at;
            return '<tr><td data-label="Service"><span class="integrations-connected-name">' + namespace.escapeHtml(name) + '</span>' +
                '<span class="integrations-connected-type">' + namespace.escapeHtml(namespace.titleCase(type)) + '</span></td>' +
                '<td data-label="Health"><span class="integrations-health-badge ' + health.className + '">' + namespace.escapeHtml(health.label) + '</span></td>' +
                '<td data-label="Last sync">' + namespace.escapeHtml(formatDate(lastSync)) + '</td>' +
                '<td data-label="Last test">' + namespace.escapeHtml(formatDate(lastTest)) + '</td></tr>';
        }).join("");
        view.innerHTML = '<div class="integrations-table-wrap"><table class="integrations-connected-table">' +
            '<thead><tr><th>Service</th><th>Health</th><th>Last sync</th><th>Last test</th></tr></thead>' +
            '<tbody>' + rows + '</tbody></table></div>' +
            '<p class="integrations-readonly-note">Connection management is read-only here because no reliable write-action contract is exposed to this view.</p>';
    };

    namespace.renderConnectedError = function (root) {
        var view = query(root, "[data-connected-view]");
        query(root, "[data-connected-status]").textContent = "Connected services unavailable";
        view.setAttribute("aria-busy", "false");
        view.innerHTML = stateView("!", "Connected services could not be loaded", "This is a local connector-service error. The provider catalog in Explore is still available.", "retry-connected", "Try again", true);
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
