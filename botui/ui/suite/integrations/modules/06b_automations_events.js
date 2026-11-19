"use strict";

// Automations delegated events: click/change/submit handlers, the immediate
// run toggle and the install hook. Split out of 06_automations.js so every
// module stays under the 450-line limit; behaviour is unchanged.

(function (namespace) {
    var BOUND = false;
    var internals = namespace.__automationInternals || {};
    var t = internals.t, root = internals.root, botId = internals.botId, base = internals.base;
    var fillProviderSelect = internals.fillProviderSelect, syncActions = internals.syncActions;
    var runAutomationNow = internals.runAutomationNow, pauseAutomation = internals.pauseAutomation;
    var clearHistory = internals.clearHistory, deleteAutomation = internals.deleteAutomation;
    var syncPauseButton = internals.syncPauseButton, loadAutomations = internals.loadAutomations;

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
        var runNowButton = target.closest('[data-action="run-automation-now"]');
        if (runNowButton && app.contains(runNowButton)) {
            var runArticle = runNowButton.closest("[data-automation-id]");
            if (runArticle) { event.preventDefault(); void runAutomationNow(app, runArticle); }
            return;
        }
        var pauseButton = target.closest('[data-action="pause-automation"]');
        if (pauseButton && app.contains(pauseButton)) {
            event.preventDefault();
            void pauseAutomation(app, pauseButton.closest("[data-automation-id]"), pauseButton);
            return;
        }
        var clearButton = target.closest('[data-action="clear-automation-history"]');
        if (clearButton && app.contains(clearButton)) {
            event.preventDefault();
            clearHistory(app, clearButton.closest("[data-automation-id]"), clearButton);
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
            syncPauseButton(article.querySelector('[data-action="pause-automation"]'), enabled);
            namespace.announce(app, enabled
                ? t("integrations-automation-enabled-announce", "Automation enabled.")
                : t("integrations-automation-disabled-announce", "Automation disabled."));
        } catch (error) {
            input.checked = !enabled;
            namespace.announce(app, t("integrations-automation-update-failed", "Could not update the automation."));
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
                errorNode.textContent = t("integrations-automation-json-invalid", "Parameters must be valid JSON.");
                errorNode.hidden = false;
                return;
            }
        }
        if (!providerSlug || !actionKey || !botId()) {
            errorNode.textContent = t("integrations-automation-required", "Provider and action are required.");
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
            if (app) namespace.announce(app, t("integrations-automation-created", "Automation created"));
            loadAutomations(root().querySelector("[data-automations-view]"), root().querySelector("[data-automation-status]"));
        }).catch(function () {
            var app = root();
            errorNode.textContent = t("integrations-automation-create-failed", "Could not create the automation.");
            errorNode.hidden = false;
            if (app) namespace.announce(app, t("integrations-automation-create-failed", "Could not create the automation."));
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
