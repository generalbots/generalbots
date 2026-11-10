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

    /* Shared with the Data Sources tab (07_sources.js) — one definition, no copies. */
    namespace.stateView = stateView;
    namespace.query = query;
    namespace.skeletonCards = skeletonCards;
    // Exposed for the consent test: the "Shared" cell is the consent contract
    // (which scopes, and who else can use the connection).
    namespace.sharedCell = sharedCell;

    function statusFilterLabel(value) {
        if (value === "must") {
            return "production-ready";
        }
        if (value === "available") {
            return "available";
        }
        return "all";
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
        // #2100 — the count names the active filter, so an empty pane is never
        // mistaken for an empty catalog.
        query(root, "[data-result-status]").textContent = filtered.length + " of " + state.providers.length +
            " providers · " + activeCategory.label + " · " + statusFilterLabel(state.statusFilter);
        catalog.setAttribute("aria-busy", "false");
        if (!state.providers.length) {
            catalog.innerHTML = stateView("0", "The catalog is empty",
                "The provider service answered but returned no providers. Refresh to try again; if this build ships without the connector catalog, configure one on the server.",
                "retry-catalog", "Refresh catalog", false);
        } else if (!filtered.length && state.statusFilter !== "all" && !state.search) {
            catalog.innerHTML = stateView("0", "No providers in this filter",
                "All " + state.providers.length + " providers are hidden by the \"" +
                statusFilterLabel(state.statusFilter) + "\" availability filter.",
                "show-all-providers", "Show all " + state.providers.length + " providers", false);
        } else if (!filtered.length) {
            catalog.innerHTML = stateView("0", "No matching integrations", "Try a broader search or clear the category and availability filters.", "clear-filters", "Clear filters", false);
        } else {
            catalog.innerHTML = '<div class="integrations-catalog-grid ' + (state.view === "list" ? "is-list" : "") + '">' +
                filtered.map(providerCard).join("") + '</div>';
        }
        namespace.renderCatalogHealth(root);
    };

    namespace.renderCatalogError = function (root, failure) {
        var catalog = query(root, "[data-catalog-view]");
        var categories = query(root, "[data-category-filters]");
        categories.innerHTML = "";
        categories.setAttribute("aria-busy", "false");
        query(root, "[data-category-panel]").removeAttribute("aria-labelledby");
        query(root, "[data-explore-notice]").innerHTML = "";
        query(root, "[data-result-status]").textContent = "Provider catalog unavailable";
        catalog.setAttribute("aria-busy", "false");
        var status = failure && failure.status;
        var detail = status ? " The request answered HTTP " + status + "." : "";
        if (status === 401 || status === 403) {
            var loginUrl = (window.GB_LOGIN_URL || "/login") + "?next=" +
                encodeURIComponent(location.pathname + location.search);
            catalog.innerHTML = stateView("!", "Sign in to view the catalog",
                "Integration catalogs are tenant-scoped and require an authenticated session.",
                "goto-login", "Sign in", false);
            var action = catalog.querySelector('[data-action="goto-login"]');
            if (action) action.addEventListener("click", function () {
                window.location.href = loginUrl;
            });
        } else if (status === 404) {
            catalog.innerHTML = stateView("!", "This build does not expose the catalog API",
                "The Integrations app called " + namespace.catalogUrl + " and the server has no such route." +
                detail + " This is a build/profile problem, not an empty catalog: rebuild with the integrations feature and reload the app.",
                "retry-catalog", "Try again", true);
        } else if (status && status >= 500) {
            catalog.innerHTML = stateView("!", "We could not load the catalog", "The provider service returned an error." +
                detail + " Connected services remain independent and may still be available.",
                "retry-catalog", "Try again", true);
        } else {
            catalog.innerHTML = stateView("!", "The provider service is unreachable",
                "The Integrations app could not reach " + namespace.catalogUrl + "." + detail +
                " This is a backend/network failure, not an empty catalog. Connected services remain independent and may still be available.",
                "retry-catalog", "Try again", true);
        }
        namespace.renderCatalogHealth(root);
    };

    /* #2100 — last line of defence: if the app's own initialization throws
       (a stale cached module, a missing fragment node, an unexpected payload)
       the pane used to stay blank with no explanation. Say what happened and
       offer a retry instead. */
    namespace.renderCatalogUnavailable = function (root, failure) {
        var catalog = query(root, "[data-catalog-view]");
        var categories = query(root, "[data-category-filters]");
        if (categories) {
            categories.innerHTML = "";
            categories.setAttribute("aria-busy", "false");
        }
        var panel = query(root, "[data-category-panel]");
        if (panel) {
            panel.removeAttribute("aria-labelledby");
        }
        var notice = query(root, "[data-explore-notice]");
        if (notice) {
            notice.innerHTML = "";
        }
        var statusNode = query(root, "[data-result-status]");
        if (statusNode) {
            statusNode.textContent = "Integrations could not start";
        }
        if (!catalog) {
            return;
        }
        catalog.setAttribute("aria-busy", "false");
        var reason = failure && failure.message ? failure.message : "unknown error";
        catalog.innerHTML = stateView("!", "Integrations could not start",
            "The app failed to initialize (" + reason + "). Reload the app; if the problem persists, the shipped module bundle is inconsistent.",
            "retry-catalog", "Reload catalog", true);
        var healthText = query(root, "[data-catalog-health-text]");
        var health = query(root, "[data-catalog-health]");
        if (health && healthText) {
            health.classList.remove("is-ready");
            health.classList.add("is-error");
            healthText.textContent = "Integrations unavailable";
        }
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

    function statusBadge(value) {
        var key = String(value).toLowerCase();
        var className = ["active", "healthy", "ok", "connected"].indexOf(key) !== -1 ? "is-healthy" :
            (["revoked", "failed", "error", "unhealthy", "expired"].indexOf(key) !== -1 ? "is-error" : "is-warning");
        return '<span class="integrations-health-badge ' + className + '">' + namespace.escapeHtml(namespace.titleCase(value, "Unknown")) + '</span>';
    }

    namespace.renderConnectedLoading = function (root) {
        var view = query(root, "[data-connected-view]");
        query(root, "[data-connected-status]").textContent = "Loading connected services";
        view.setAttribute("aria-busy", "true");
        view.innerHTML = '<div class="integrations-skeleton-grid" aria-hidden="true">' + skeletonCards(3) + '</div>';
    };

    function testCell(row) {
        if (!row.last_test_status) {
            return '<span class="integrations-faint-text">Not tested</span>';
        }
        return statusBadge(row.last_test_status) +
            '<span class="integrations-cell-subtext">' + namespace.escapeHtml(formatDate(row.last_tested_at)) + '</span>';
    }

    // GB7-18 — the consent view: what the connection may read or do, and who
    // else can use it. Empty scopes are shown honestly instead of hidden.
    function sharedCell(row) {
        var scopes = Array.isArray(row.granted_scopes) ? row.granted_scopes : [];
        var visibility = row.visibility === "branch" ? "branch workspace" : row.visibility === "private" ? "only you" : "";
        var scopeText = scopes.length
            ? scopes.slice(0, 3).join(", ") + (scopes.length > 3 ? " +" + (scopes.length - 3) : "")
            : "No scopes recorded";
        var title = (scopes.length ? "Scopes: " + scopes.join(", ") : "No scopes recorded") +
            (visibility ? " · Visible to: " + visibility : "");
        return '<span class="integrations-shared" title="' + namespace.escapeHtml(title) + '">' +
            namespace.escapeHtml(scopeText) +
            (visibility ? '<span class="integrations-cell-subtext">' + namespace.escapeHtml(visibility) + '</span>' : "") +
            '</span>';
    }

    function connectionRow(row) {
        var name = row.display_name || namespace.titleCase(row.provider_slug, "Connection");
        var expires = row.expires_at ? formatDate(row.expires_at) : "No expiry";
        var actions = '<div class="integrations-row-actions">' +
            '<button class="integrations-button integrations-button-subtle" type="button" data-action="test-connection" data-connection-id="' + namespace.escapeHtml(row.id) + '">Test</button>' +
            '<button class="integrations-button integrations-button-subtle" type="button" data-action="revoke-connection" data-connection-id="' + namespace.escapeHtml(row.id) + '">Revoke</button>' +
            '</div>';
        return '<tr><td data-label="Service"><span class="integrations-connected-name">' + namespace.escapeHtml(name) + '</span>' +
            '<span class="integrations-connected-type">' + namespace.escapeHtml(namespace.titleCase(row.provider_slug, "Provider")) + ' / ' + namespace.escapeHtml(namespace.titleCase(row.auth_kind, "Auth")) + '</span></td>' +
            '<td data-label="Status">' + statusBadge(row.status) + '</td>' +
            '<td data-label="Shared">' + sharedCell(row) + '</td>' +
            '<td data-label="Last test">' + testCell(row) + '</td>' +
            '<td data-label="Credential">v' + namespace.escapeHtml(String(row.credential_version)) + '</td>' +
            '<td data-label="Expires">' + namespace.escapeHtml(expires) + '</td>' +
            '<td data-label="Actions">' + actions + '</td></tr>';
    }

    namespace.renderConnected = function (root) {
        var state = namespace.getState(root);
        var view = query(root, "[data-connected-view]");
        query(root, "[data-connected-status]").textContent = state.connectors.length + " connected service" + (state.connectors.length === 1 ? "" : "s");
        view.setAttribute("aria-busy", "false");
        if (!state.connectors.length) {
            view.innerHTML = stateView("0", "No connected services yet", "Open an available provider in Explore and use Connect to store its credentials in the secure vault.", "show-explore", "Explore providers", false);
            return;
        }
        var rows = state.connectors.map(connectionRow).join("");
        view.innerHTML = '<div class="integrations-table-wrap"><table class="integrations-connected-table">' +
            '<thead><tr><th>Service</th><th>Status</th><th>Shared</th><th>Last test</th><th>Credential</th><th>Expires</th><th>Actions</th></tr></thead>' +
            '<tbody>' + rows + '</tbody></table></div>' +
            '<p class="integrations-readonly-note">Credentials stay in the server-side vault; this inventory only shows sanitized metadata.</p>';
    };

    // #1616 — a failed status request is NOT a connection state. The card must
    // say the status is unavailable and offer a retry, never imply that the
    // user's accounts are disconnected or that the service reported an error.
    namespace.renderConnectedUnavailable = function (root) {
        var view = query(root, "[data-connected-view]");
        query(root, "[data-connected-status]").textContent = "Status unavailable";
        view.setAttribute("aria-busy", "false");
        view.innerHTML = stateView("!", "Status unavailable", "The workspace context service did not answer, so connections cannot be listed right now. This is a status check failure, not a disconnected account. The provider catalog in Explore remains available.", "retry-connected", "Check again", true);
    };

    namespace.renderConnectedError = function (root, reason) {
        var view = query(root, "[data-connected-view]");
        query(root, "[data-connected-status]").textContent = "Status unavailable";
        view.setAttribute("aria-busy", "false");
        var copy = "The connection service did not answer this request, so the list cannot be shown. " +
            "This is a status check failure, not a disconnected account.";
        if (reason) {
            copy += " Reported cause: " + reason + ".";
        }
        view.innerHTML = stateView("!", "Status unavailable", copy, "retry-connected", "Check again", true);
    };

    /* #2100 — the context endpoint documents a no-workspace answer
       (`bot_id: null` + `reason: "no_active_bot"`). That is not a failure:
       say so, and keep Explore usable. */
    namespace.renderConnectedNoContext = function (root) {
        var view = query(root, "[data-connected-view]");
        query(root, "[data-connected-status]").textContent = "No active bot/workspace context";
        view.setAttribute("aria-busy", "false");
        view.innerHTML = stateView("0", "No active bot/workspace context",
            "Connections belong to a bot. This installation has no active bot yet, so there is nothing to list; the provider catalog in Explore is still available.",
            "retry-connected", "Check again", false);
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
