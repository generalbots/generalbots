"use strict";

(function (namespace) {
    function announce(root, message) {
        namespace.announce(root, message);
    }

    function testFailureMessage(error) {
        var status = error && typeof error.status === "number" ? error.status : 0;
        if (status === 503) {
            return "Connection test failed: credential vault unavailable";
        }
        if (status === 404) {
            return "Connection test failed: connection no longer exists";
        }
        return "Connection test failed";
    }

    function revokeFailureMessage(error) {
        var status = error && typeof error.status === "number" ? error.status : 0;
        if (status === 503) {
            return "Revoke incomplete: credential vault unavailable";
        }
        if (status === 404) {
            return "Revoke skipped: connection no longer exists";
        }
        return "Revoke failed; the connection was left unchanged";
    }

    namespace.loadConnected = async function (root) {
        var state = namespace.getState(root);
        state.connectedRequest += 1;
        var requestId = state.connectedRequest;
        state.connectedStatus = "loading";
        namespace.renderConnectedLoading(root);
        await namespace.fetchContext(root);
        var url = namespace.connectionsUrl();
        if (requestId !== state.connectedRequest) {
            return;
        }
        if (!url) {
            state.connectors = [];
            state.connectedStatus = "error";
            namespace.renderConnectedUnavailable(root);
            return;
        }
        try {
            var payload = await namespace.fetchJson(url);
            if (requestId !== state.connectedRequest) {
                return;
            }
            state.connectors = namespace.normalizeConnections(payload);
            state.connectedStatus = "ready";
            namespace.renderConnected(root);
        } catch (error) {
            if (requestId === state.connectedRequest) {
                state.connectors = [];
                state.connectedStatus = "error";
                namespace.renderConnectedError(root);
            }
        }
    };

    namespace.testConnection = async function (root, connectionId, button) {
        var url = namespace.connectionActionUrl(connectionId, "/test");
        if (!url) {
            announce(root, "Connection test needs a workspace context");
            return;
        }
        var originalLabel = button.textContent;
        button.disabled = true;
        button.textContent = "Testing...";
        try {
            var outcome = await namespace.fetchJson(url, { method: "POST", body: {} });
            announce(root, "Test finished: " + namespace.text(outcome.outcome, "unknown"));
        } catch (error) {
            announce(root, testFailureMessage(error));
        } finally {
            button.disabled = false;
            button.textContent = originalLabel;
            namespace.loadConnected(root);
        }
    };

    namespace.revokeConnection = async function (root, connectionId) {
        var url = namespace.connectionActionUrl(connectionId, "");
        if (!url) {
            announce(root, "Revoke needs a workspace context");
            return;
        }
        if (!window.confirm("Revoke this connection? Its stored credentials will be deleted.")) {
            return;
        }
        try {
            await namespace.fetchJson(url, { method: "DELETE" });
            announce(root, "Connection revoked");
        } catch (error) {
            announce(root, revokeFailureMessage(error));
        }
        namespace.loadConnected(root);
    };

    namespace.loadCatalog = async function (root) {
        var state = namespace.getState(root);
        state.catalogRequest += 1;
        var requestId = state.catalogRequest;
        state.catalogStatus = "loading";
        namespace.renderCatalogLoading(root);
        try {
            var payload = await namespace.fetchJson(namespace.catalogUrl);
            if (requestId !== state.catalogRequest) {
                return;
            }
            var catalog = namespace.normalizeCatalog(payload);
            state.providers = catalog.providers;
            state.providerMap = new Map(catalog.providers.map(function (provider) { return [provider.id, provider]; }));
            state.categories = catalog.categories;
            if (state.category !== "all" && !state.categories.some(function (category) {
                return category.value.toLowerCase() === state.category.toLowerCase();
            })) {
                state.category = "all";
            }
            state.totals = catalog.totals;
            state.catalogStatus = "ready";
            namespace.renderExplore(root);
            announce(root, "Integration catalog loaded");
        } catch (error) {
            if (requestId === state.catalogRequest) {
                state.catalogStatus = "error";
                namespace.renderCatalogError(root, error);
                announce(root, "Integration catalog could not be loaded");
            }
        }
    };

    function setTab(root, name, focusTab) {
        var state = namespace.getState(root);
        var tabs = Array.from(root.querySelectorAll("[data-tab]"));
        var panels = Array.from(root.querySelectorAll("[data-panel]"));
        state.activeTab = name;
        tabs.forEach(function (tab) {
            var selected = tab.dataset.tab === name;
            tab.setAttribute("aria-selected", String(selected));
            tab.tabIndex = selected ? 0 : -1;
            if (selected && focusTab) {
                tab.focus();
            }
        });
        panels.forEach(function (panel) { panel.hidden = panel.dataset.panel !== name; });
        if (name === "connected" && state.connectedStatus === "idle") {
            namespace.loadConnected(root);
        }
    }

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
        } else if (action === "retry-catalog") {
            namespace.loadCatalog(root);
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
        var state = namespace.createState(root);
        state.connectedStatus = "idle";
        configureAria(root, state);
        root.addEventListener("click", function (event) { handleClick(root, event); });
        root.addEventListener("input", function (event) { handleInput(root, event); });
        root.addEventListener("submit", function (event) {
            if (event.target.matches("[data-connect-form]")) {
                event.preventDefault();
                namespace.submitConnectForm(root, event.target);
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
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
