"use strict";
/* Split from 04_events.js for #2305 (450-line limit). */
(function (namespace) {
    function clearFilters(root) {
        var state = namespace.getState(root);
        state.search = "";
        state.category = "all";
        state.statusFilter = "all";
        var search = root.querySelector("[data-search]");
        search.value = "";
        root.querySelectorAll("[data-status]").forEach(function (button) {
            button.setAttribute("aria-pressed", String(button.dataset.status === "all"));
        });
        namespace.renderExplore(root);
        search.focus();
    }

    function setCategory(root, value, focusTab) {
        var state = namespace.getState(root);
        state.category = value;
        state.exploreTouched = true;
        namespace.renderExplore(root);
        if (focusTab) {
            var selected = root.querySelector('[data-category][aria-selected="true"]');
            selected.focus();
            selected.scrollIntoView({ block: "nearest", inline: "nearest" });
        }
    }

    function handleAction(root, action, button) {
        var state = namespace.getState(root);
        if (action === "refresh") {
            if (state.activeTab === "connected") {
                namespace.loadConnected(root);
            } else {
                namespace.loadCatalog(root);
            }
        } else if (action === "refresh-connected" || action === "retry-connected") {
            namespace.loadConnected(root);
        } else if (action === "refresh-sources" || action === "retry-sources") {
            namespace.loadSources(root);
        } else if (action === "mcp-toggle" && button && button.dataset.mcpName) {
            namespace.toggleMcpServer(root, button.dataset.mcpName, button.dataset.mcpEnabled === "true");
        } else if (action === "mcp-remove" && button && button.dataset.mcpName) {
            if (typeof namespace.removeMcpServer === "function") {
                namespace.removeMcpServer(root, button.dataset.mcpName);
            }
        } else if (action === "retry-catalog") {
            namespace.loadCatalog(root);
        } else if (action === "show-all-providers") {
            showAllProviders(root);
        } else if (action === "clear-filters") {
            clearFilters(root);
        } else if (action === "show-explore") {
            setTab(root, "explore", true);
        } else if (action === "close-details") {
            namespace.closeDetails(root);
        } else if (action === "retry-detail" && state.selectedProviderId) {
            namespace.openDetails(root, state.selectedProviderId);
        } else if (action === "test-connection" && button && button.dataset.connectionId) {
            namespace.testConnection(root, button.dataset.connectionId, button);
        } else if (action === "rotate-connection" && button && button.dataset.connectionId) {
            if (typeof namespace.openRotateConnection === "function") {
                namespace.openRotateConnection(root, button.dataset.connectionId, button.dataset.providerSlug || "");
            } else {
                announce(root, "Rotate action is unavailable");
            }
        } else if (action === "revoke-connection" && button && button.dataset.connectionId) {
            namespace.revokeConnection(root, button.dataset.connectionId);
        }
    }

    function handleClick(root, event) {
        var tab = event.target.closest("[data-tab]");
        if (tab && root.contains(tab)) {
            setTab(root, tab.dataset.tab, false);
            return;
        }
        var chip = event.target.closest("[data-source]");
        if (chip && root.contains(chip)) {
            namespace.setSource(root, chip.dataset.source);
            return;
        }
        var actionButton = event.target.closest("[data-action]");
        if (actionButton && root.contains(actionButton)) {
            handleAction(root, actionButton.dataset.action, actionButton);
            return;
        }
        var status = event.target.closest("[data-status]");
        if (status && root.contains(status)) {
            var statusState = namespace.getState(root);
            statusState.statusFilter = status.dataset.status;
            statusState.exploreTouched = true;
            root.querySelectorAll("[data-status]").forEach(function (button) {
                button.setAttribute("aria-pressed", String(button === status));
            });
            namespace.renderExplore(root);
            return;
        }
        var category = event.target.closest("[data-category]");
        if (category && root.contains(category)) {
            setCategory(root, category.dataset.category, true);
            return;
        }
        var view = event.target.closest("[data-view]");
        if (view && root.contains(view)) {
            var viewState = namespace.getState(root);
            viewState.view = view.dataset.view;
            root.querySelectorAll("[data-view]").forEach(function (button) {
                button.setAttribute("aria-pressed", String(button === view));
            });
            namespace.renderExplore(root);
            return;
        }
        var provider = event.target.closest("[data-provider-id]");
        if (provider && root.contains(provider)) {
            namespace.openDetails(root, provider.dataset.providerId);
            root.querySelector("[data-explore-notice]").innerHTML = "";
        }
    }

    function handleInput(root, event) {
        if (!event.target.matches("[data-search]")) {
            return;
        }
        var state = namespace.getState(root);
        state.search = event.target.value.trim();
        state.exploreTouched = true;
        namespace.renderExplore(root);
    }

    function handleTabKeys(root, event) {
        var current = event.target.closest("[data-tab]");
        if (!current || !root.contains(current)) {
            return;
        }
        var tabs = Array.from(root.querySelectorAll("[data-tab]"));
        var index = tabs.indexOf(current);
        var next = index;
        if (event.key === "ArrowRight") {
            next = (index + 1) % tabs.length;
        } else if (event.key === "ArrowLeft") {
            next = (index - 1 + tabs.length) % tabs.length;
        } else if (event.key === "Home") {
            next = 0;
        } else if (event.key === "End") {
            next = tabs.length - 1;
        } else {
            return;
        }
        event.preventDefault();
        setTab(root, tabs[next].dataset.tab, true);
    }

    function handleCategoryTabKeys(root, event) {
        var current = event.target.closest("[data-category]");
        if (!current || !root.contains(current)) {
            return;
        }
        var tabs = Array.from(root.querySelectorAll("[data-category]"));
        var index = tabs.indexOf(current);
        var next = index;
        if (event.key === "ArrowRight") {
            next = (index + 1) % tabs.length;
        } else if (event.key === "ArrowLeft") {
            next = (index - 1 + tabs.length) % tabs.length;
        } else if (event.key === "Home") {
            next = 0;
        } else if (event.key === "End") {
            next = tabs.length - 1;
        } else {
            return;
        }
        event.preventDefault();
        setCategory(root, tabs[next].dataset.category, true);
    }

    function configureAria(root, state) {
        root.querySelectorAll("[data-tab]").forEach(function (tab) {
            var name = tab.dataset.tab;
            var panel = root.querySelector('[data-panel="' + name + '"]');
            tab.id = state.uid + "-tab-" + name;
            panel.id = state.uid + "-panel-" + name;
            tab.setAttribute("aria-controls", panel.id);
            panel.setAttribute("aria-labelledby", tab.id);
        });
        var title = root.querySelector("[data-detail-title]");
        var detail = root.querySelector("[data-detail-panel]");
        root.querySelector("[data-category-panel]").id = state.uid + "-category-panel";
        title.id = state.uid + "-detail-title";
        detail.setAttribute("aria-labelledby", title.id);
    }

    namespace.initializeRoot = function (root) {
        if (root.dataset.integrationsReady === "true") {
            return;
        }
        root.dataset.integrationsReady = "true";
        try {
            start(root);
        } catch (error) {
            // #2100 — never leave a blank pane: an exception during init (a
            // stale cached module, a missing fragment node, a bad payload)
            // renders an actionable card with a reload action.
            console.error("[integrations] initialization failed", error);
            try {
                namespace.renderCatalogUnavailable(root, error);
            } catch (fallbackError) {
                console.error("[integrations] fallback state failed", fallbackError);
            }
        }
    };

    function start(root) {
        var state = namespace.createState(root);
        state.connectedStatus = "idle";
        state.rotateTarget = null;
        // BBSIA #1542: preview OFF exposes only production-ready connectors.
        // #2100: the value is re-resolved from the payload in `loadCatalog`,
        // because a catalog without priorities would match nothing here.
        state.statusFilter = "all";
        namespace.syncStatusButtons(root, state);
        window.addEventListener("gb-preview-changed", function () {
            var next = namespace.preferredStatusFilter(state.providers);
            if (state.statusFilter === next) return;
            state.statusFilter = next;
            namespace.syncStatusButtons(root, state);
            namespace.renderExplore(root);
        });
        configureAria(root, state);
        root.addEventListener("click", function (event) { handleClick(root, event); });
        root.addEventListener("input", function (event) { handleInput(root, event); });
        root.addEventListener("submit", function (event) {
            if (event.target.matches("[data-connect-form]")) {
                event.preventDefault();
                namespace.submitConnectForm(root, event.target);
            } else if (event.target.matches("[data-oauth-client-form]")) {
                event.preventDefault();
                namespace.saveOAuthClient(root, event.target);
            }
        });
        root.addEventListener("keydown", function (event) {
            namespace.trapDetailFocus(root, event);
            handleTabKeys(root, event);
            handleCategoryTabKeys(root, event);
            if (event.key === "Escape" && event.target.matches("[data-search]") && event.target.value) {
                clearFilters(root);
            }
        });
        namespace.renderConnectedLoading(root);
        namespace.loadCatalog(root);
        namespace.fetchContext(root).then(function () { namespace.loadConnected(root); });
    }
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
