"use strict";

// Automations panel: create, list, toggle and delete scheduled runs of
// implemented integration actions. Delegated events keep this independent
// from the render pipeline.

(function (namespace) {
    var BOUND = false;

    function root() {
        return document.querySelector("[data-integrations-app]");
    }

    function stateOf(node) {
        return namespace.getState(root());
    }

    function botId() {
        return (namespace.context && namespace.context.botId) || "";
    }

    function base() {
        return botId() ? "/api/bots/" + encodeURIComponent(botId()) + "/integration-automations" : "";
    }

    function implementedProviders(state) {
        if (!state || !state.providerMap) return [];
        var out = [];
        state.providerMap.forEach(function (provider) {
            var actions = (provider.actions || []).filter(function (a) { return a.implemented === true; });
            if (actions.length) out.push({ provider: provider, actions: actions });
        });
        return out.sort(function (a, b) {
            return String(a.provider.name).localeCompare(String(b.provider.name));
        });
    }

    function fillProviderSelect(select) {
        var entries = implementedProviders(stateOf(select));
        select.innerHTML = '<option value="">Select a provider</option>' + entries.map(function (entry) {
            return '<option value="' + namespace.escapeHtml(entry.provider.id) + '">' +
                namespace.escapeHtml(entry.provider.name || entry.provider.id) + '</option>';
        }).join("");
        select.dataset.loaded = "1";
    }

    function syncActions(form) {
        var providerSelect = form.querySelector("[data-automation-provider]");
        var actionSelect = form.querySelector("[data-automation-action]");
        var entries = implementedProviders(stateOf(form));
        var slug = providerSelect.value;
        var match = null;
        for (var i = 0; i < entries.length; i += 1) {
            if (entries[i].provider.id === slug) { match = entries[i]; break; }
        }
        if (!match) {
            actionSelect.innerHTML = '<option value="">Select an action</option>';
            return;
        }
        actionSelect.innerHTML = match.actions.map(function (action) {
            return '<option value="' + namespace.escapeHtml(action.name) + '">' +
                namespace.escapeHtml(action.name) + '</option>';
        }).join("");
    }

    async function loadAutomations(view, statusNode) {
        view.setAttribute("aria-busy", "true");
        if (!botId()) {
            statusNode.textContent = "Open this app inside a bot workspace to manage automations.";
            view.innerHTML = "";
            return;
        }
        try {
            var payload = await namespace.fetchJson(base(), { credentials: "same-origin" });
            var items = Array.isArray(payload.automations) ? payload.automations : [];
            statusNode.textContent = items.length ? "" : "No automations yet.";
            view.removeAttribute("aria-busy");
            view.innerHTML = items.length ? items.map(rowMarkup).join("") : "";
        } catch (error) {
            statusNode.textContent = "Automations are unavailable right now.";
            view.removeAttribute("aria-busy");
        }
    }

    function rowMarkup(item) {
        var checked = item.enabled === true ? " checked" : "";
        return '<article class="integrations-automation" data-automation-id="' + namespace.escapeHtml(item.id) + '">' +
            '<div class="integrations-automation-head"><strong>' +
            namespace.escapeHtml(item.provider_slug + "." + item.action_key.split(".").slice(1).join(".") || item.action_key) +
            '</strong><label class="integrations-automation-toggle"><input type="checkbox" data-automation-toggle' + checked + '> enabled</label></div>' +
            '<p class="integrations-run-summary">' + namespace.escapeHtml(item.schedule) +
            (item.last_run_at ? ' · last run ' + namespace.escapeHtml(String(item.last_run_at).replace("T", " ").slice(0, 16)) : ' · never run') +
            '</p>' +
            (item.last_outcome ? '<p class="integrations-automation-outcome">' + namespace.escapeHtml(item.last_outcome) + '</p>' : "") +
            '<div class="integrations-run-actions"><button class="integrations-button integrations-button-subtle" type="button" data-action="delete-automation">Delete</button></div>' +
            '</article>';
    }

    function onDelegatedClick(event) {
        var target = event.target;
        if (!target.closest) return;
        var app = target.closest("[data-integrations-app]");
        if (!app) return;

        if (target.matches('[data-action="new-automation"]')) {
            var form = app.querySelector("[data-automation-form]");
            form.hidden = false;
            var providerSelect = form.querySelector("[data-automation-provider]");
            if (providerSelect.dataset.loaded !== "1") fillProviderSelect(providerSelect);
            syncActions(form);
            providerSelect.focus();
            return;
        }
        if (target.matches('[data-action="cancel-automation"]')) {
            app.querySelector("[data-automation-form]").hidden = true;
            return;
        }
        if (target.matches('[data-action="delete-automation"]')) {
            var article = target.closest("[data-automation-id]");
            var id = article && article.dataset.automationId;
            if (!id) return;
            event.preventDefault();
            namespace.fetchJson(base() + "/" + encodeURIComponent(id), { method: "DELETE" })
                .then(function () {
                    article.remove();
                    var remaining = app.querySelectorAll("[data-automation-id]").length;
                    if (!remaining) app.querySelector("[data-automation-status]").textContent = "No automations yet.";
                })
                .catch(function () {});
        }
    }

    function onDelegatedChange(event) {
        var input = event.target;
        if (!input.matches) return;
        if (input.matches("[data-automation-provider]") ) {
            syncActions(input.closest("[data-automation-form]"));
            return;
        }
        if (input.matches("[data-automation-toggle]")) {
            var article = input.closest("[data-automation-id]");
            var id = article && article.dataset.automationId;
            if (!id) return;
            namespace.fetchJson(base() + "/" + encodeURIComponent(id), {
                method: "PATCH",
                body: { enabled: input.checked }
            }).catch(function () {});
        }
    }

    function onDelegatedSubmit(event) {
        var form = event.target;
        if (!form.matches || !form.matches("[data-automation-form]")) return;
        event.preventDefault();
        var errorNode = form.querySelector("[data-automation-error]");
        errorNode.hidden = true;

        var providerSlug = form.elements.provider.value;
        var actionKey = form.elements.action.value;
        var schedule = form.elements.schedule.value;
        var paramsText = (form.elements.params.value || "").trim();
        var params = {};
        if (paramsText) {
            try { params = JSON.parse(paramsText); } catch (e) {
                errorNode.textContent = "Parameters must be valid JSON.";
                errorNode.hidden = false;
                return;
            }
        }
        if (!providerSlug || !actionKey || !botId()) {
            errorNode.textContent = "Provider and action are required.";
            errorNode.hidden = false;
            return;
        }
        namespace.fetchJson(base(), {
            method: "POST",
            body: { provider: providerSlug, action: actionKey, schedule: schedule, params: params }
        }).then(function () {
            form.reset();
            form.hidden = true;
            var view = root().querySelector("[data-automations-view]");
            loadAutomations(view, root().querySelector("[data-automation-status]"));
        }).catch(function () {
            errorNode.textContent = "Could not create the automation.";
            errorNode.hidden = false;
        });
    }

    namespace.refreshAutomations = function () {
        var r = root();
        if (!r) return;
        var panel = r.querySelector('[data-panel="automations"]');
        if (!panel || panel.hidden) return;
        loadAutomations(r.querySelector("[data-automations-view]"), r.querySelector("[data-automation-status]"));
    };

    namespace.installAutomations = function () {
        if (BOUND) return;
        BOUND = true;
        document.addEventListener("click", onDelegatedClick);
        document.addEventListener("change", onDelegatedChange);
        document.addEventListener("submit", onDelegatedSubmit, true);
        document.addEventListener("click", function (event) {
            var tab = event.target.closest ? event.target.closest('[data-tab="automations"]') : null;
            if (tab) setTimeout(namespace.refreshAutomations, 0);
        });
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
