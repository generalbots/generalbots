"use strict";

(function (namespace) {
    function announce(root, message) {
        namespace.announce(root, message);
    }

    function t(key, fallback) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
    }

    function syncStatusButtons(root, state) {
        root.querySelectorAll("[data-status]").forEach(function (button) {
            button.setAttribute("aria-pressed", String(button.dataset.status === state.statusFilter));
        });
    }
    // #2345 — shared with 04_events_b.js (both are split from 04_events.js);
    // without exporting it, the app throws on boot ("syncStatusButtons is not defined").
    namespace.syncStatusButtons = syncStatusButtons;

    /* #2100 — a filter that hides every provider must be one click away from
       showing everything again. */
    function showAllProviders(root) {
        var state = namespace.getState(root);
        state.statusFilter = "all";
        state.exploreTouched = true;
        syncStatusButtons(root, state);
        namespace.renderExplore(root);
        announce(root, "Showing all " + state.providers.length + " providers");
    }
    function csrfMessage(error) {
        if (error && (error.code === "CSRF_MISSING" || /CSRF token/i.test(String(error.message || "")))) {
            return "Security token missing. Reload this page and try again.";
        }
        return "";
    }

    function testFailureMessage(error) {
        var csrf = csrfMessage(error);
        if (csrf) return "Connection test failed: " + csrf;
        var status = error && typeof error.status === "number" ? error.status : 0;
        if (status === 503) return "Connection test failed: credential vault unavailable";
        if (status === 404) return "Connection test failed: connection no longer exists";
        return "Connection test failed";
    }
    function revokeFailureMessage(error) {
        var csrf = csrfMessage(error);
        if (csrf) return "Revoke failed: " + csrf;
        var status = error && typeof error.status === "number" ? error.status : 0;
        if (status === 503) return "Revoke incomplete: credential vault unavailable";
        if (status === 404) return "Revoke skipped: connection no longer exists";
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
            if (namespace.context.error) {
                state.connectedStatus = "error";
                namespace.renderConnectedError(root, namespace.context.error);
            } else {
                // #2100 — no active bot is a documented workspace state, not a
                // status failure.
                state.connectedStatus = "empty";
                namespace.renderConnectedNoContext(root);
            }
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
                // #1616 — keep the diagnosis in the console (endpoint and HTTP
                // status) so a failing deployment can be traced from the log,
                // while the card itself only reports an unavailable status.
                var status = error && typeof error.status === "number" ? " HTTP " + error.status : "";
                console.warn("[integrations] connected status request failed:" + status + " " + url, error);
                namespace.renderConnectedError(root, status ? "the endpoint answered" + status : "");
            }
        }
    };

    namespace.testConnection = async function (root, connectionId, button) {
        var url = namespace.connectionActionUrl(connectionId, "/test");
        if (!url) return announce(root, "Connection test needs a workspace context");
        if (button) { button.disabled = true; button.textContent = t("integrations-testing", "Testing…"); button.setAttribute("aria-busy", "true"); }
        var state = namespace.getState(root);
        try {
            var outcome = await namespace.fetchJson(url, { method: "POST", body: {} });
            var status = namespace.text(outcome.outcome || outcome.status, "unknown");
            var detail = namespace.text(outcome.detail || outcome.outcome_detail || outcome.message, "");
            var testedAt = new Date().toISOString();
            namespace.testDetails[connectionId] = { outcome: status, detail: detail, testedAt: testedAt };
            if (state && Array.isArray(state.connectors)) {
                state.connectors = state.connectors.map(function (row) {
                    if (row.id !== connectionId) return row;
                    return Object.assign({}, row, {
                        last_test_status: status,
                        last_tested_at: testedAt,
                        last_test_detail: detail
                    });
                });
                state.connectedStatus = "ready";
                namespace.renderConnected(root);
            }
            var parts = ["Connection test: " + namespace.titleCase(status, "Unknown")];
            if (detail) parts.push(detail);
            announce(root, parts.join(" — "));
            if (button && button.isConnected) { button.textContent = t("integrations-retest", "Retest"); }
        } catch (error) {
            var errDetail = namespace.text(error && (error.detail || error.outcome_detail), "");
            if (errDetail) {
                namespace.testDetails[connectionId] = { outcome: "failed", detail: errDetail, testedAt: new Date().toISOString() };
            }
            announce(root, testFailureMessage(error));
        } finally {
            if (button && button.isConnected) { button.disabled = false; button.removeAttribute("aria-busy"); }
            namespace.loadConnected(root);
        }
    };

    namespace.revokeConnection = async function (root, connectionId) {
        var url = namespace.connectionActionUrl(connectionId, "");
        if (!url) {
            announce(root, "Revoke needs a workspace context");
            return;
        }
        var message = t("integrations-revoke-confirm", "Revoke this connection? Its stored credentials will be deleted.");
        var title = t("integrations-revoke-title", "Revoke connection");
        var confirmed = await new Promise(function (resolve) {
            if (window.GBModal && typeof window.GBModal.confirm === "function") {
                window.GBModal.confirm(message, title, function () { resolve(true); }, function () { resolve(false); });
            } else {
                resolve(false);
            }
        });
        if (!confirmed) return;
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
            // #2100 — the default filter follows the payload: Preview OFF only
            // narrows to "must" when the catalog actually carries priorities,
            // so a payload without them can never hide every provider.
            if (!state.exploreTouched) {
                state.statusFilter = namespace.preferredStatusFilter(state.providers);
                syncStatusButtons(root, state);
            }
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
        if (name === "sources" && state.sourcesStatus === "idle") {
            if (!root.querySelector("[data-source-filters]").childElementCount) {
                root.querySelector("[data-source-filters]").innerHTML = namespace.sourceChips(state.activeSource);
            }
            namespace.loadSources(root);
        }
        if (name === "provider-source" && state.providerSourceStatus === "idle" &&
            typeof namespace.loadProviderDefinitions === "function") {
            namespace.loadProviderDefinitions(root);
        }
    }

})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
