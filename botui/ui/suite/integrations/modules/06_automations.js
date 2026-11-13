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
        var app = root();
        try {
            var payload = await namespace.fetchJson(base(), { credentials: "same-origin" });
            var items = Array.isArray(payload.automations) ? payload.automations : [];
            statusNode.textContent = items.length ? "" : "No automations yet.";
            view.removeAttribute("aria-busy");
            view.innerHTML = items.length ? items.map(rowMarkup).join("") : "";
            if (app) namespace.announce(app, items.length ? items.length + " automation" + (items.length === 1 ? "" : "s") + " loaded" : "No automations yet");
        } catch (error) {
            statusNode.textContent = "Automations are unavailable right now.";
            view.removeAttribute("aria-busy");
            if (app) namespace.announce(app, "Automations are unavailable right now");
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

    function confirmWithModal(message) {
        return new Promise(function (resolve) {
            if (window.GBModal && typeof window.GBModal.confirm === "function") {
                window.GBModal.confirm(message, "Confirm", function () { resolve(true); }, function () { resolve(false); });
                return;
            }
            resolve(false);
        });
    }

    async function deleteAutomation(app, article, id, button) {
        if (!app || !article || !id || !button) return;
        if (button.disabled || button.dataset.deleting === "1") return;

        var confirmed = await confirmWithModal("Delete this automation?");
        if (!confirmed) return;

        button.disabled = true;
        button.dataset.deleting = "1";
        button.setAttribute("aria-busy", "true");

        try {
            await namespace.fetchJson(base() + "/" + encodeURIComponent(id), { method: "DELETE" });
            article.remove();
            var statusNode = app.querySelector("[data-automation-status]");
            var remaining = app.querySelectorAll("[data-automation-id]").length;
            if (!remaining && statusNode) statusNode.textContent = "No automations yet.";
            namespace.announce(app, "Automation deleted.");
        } catch (error) {
            button.disabled = false;
            delete button.dataset.deleting;
            button.removeAttribute("aria-busy");
            namespace.announce(app, "Could not delete the automation.");
        }
    }

    function onDelegatedClick(event) {
        var target = event.target;
        if (!target || !target.closest) return;
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
        var deleteButton = target.closest('[data-action="delete-automation"]');
        if (deleteButton && app.contains(deleteButton)) {
            var article = deleteButton.closest("[data-automation-id]");
            var id = article && article.dataset.automationId;
            if (!id) return;
            event.preventDefault();
            void deleteAutomation(app, article, id, deleteButton);
        }
    }

    async function toggleAutomation(app, article, id, input, enabled) {
        if (!app || !article || !id || !input) return;
        if (input.disabled || input.dataset.toggling === "1") return;

        input.disabled = true;
        input.dataset.toggling = "1";
        try {
            await namespace.fetchJson(base() + "/" + encodeURIComponent(id), {
                method: "PATCH",
                body: { enabled: enabled }
            });
            namespace.announce(app, enabled ? "Automation enabled." : "Automation disabled.");
        } catch (error) {
            input.checked = !enabled;
            namespace.announce(app, "Could not update the automation.");
        } finally {
            input.disabled = false;
            delete input.dataset.toggling;
        }
    }

    function onDelegatedChange(event) {
        var input = event.target;
        if (!input || !input.matches) return;
        if (input.matches("[data-automation-provider]")) {
            var providerForm = input.closest("[data-automation-form]");
            if (providerForm) syncActions(providerForm);
            return;
        }
        if (input.matches("[data-automation-toggle]")) {
            var app = input.closest("[data-integrations-app]");
            var article = input.closest("[data-automation-id]");
            var id = article && article.dataset.automationId;
            if (!app || !id) return;
            void toggleAutomation(app, article, id, input, input.checked);
        }
    }

    function onDelegatedSubmit(event) {
        var form = event.target;
        if (!form || !form.matches || !form.matches("[data-automation-form]")) return;
        event.preventDefault();
        var errorNode = form.querySelector("[data-automation-error]");
        if (!errorNode) return;
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
            var app = root();
            form.reset();
            form.hidden = true;
            if (app) namespace.announce(app, "Automation created");
            var view = root().querySelector("[data-automations-view]");
            loadAutomations(view, root().querySelector("[data-automation-status]"));
        }).catch(function () {
            var app = root();
            errorNode.textContent = "Could not create the automation.";
            errorNode.hidden = false;
            if (app) namespace.announce(app, "Could not create the automation");
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
