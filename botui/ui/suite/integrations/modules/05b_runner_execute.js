"use strict";

// Action runner execution flow (#939/#2352): run an action against the live
// provider, prefill the automation form from a run, and the delegated
// click/submit handlers. Split out of 05_runner.js so every module stays
// under the 450-line limit; behaviour is unchanged.

(function (namespace) {
    var RUN_DELEGATED = false;
    var internals = namespace.__runnerInternals || {};
    var t = internals.t, findRoot = internals.findRoot, providerFor = internals.providerFor;
    var actionFor = internals.actionFor, normalizeParams = internals.normalizeParams;
    var readHistory = internals.readHistory, writeHistory = internals.writeHistory;
    var renderOutcomeArea = internals.renderOutcomeArea, toggleRunner = internals.toggleRunner;
    var collectParams = internals.collectParams, fillFormParams = internals.fillFormParams;

    async function execute(article, providerSlug, actionKey, form) {
        var rootNode = findRoot(article);
        var errorNode = form.querySelector("[data-run-error]");
        var progressNode = form.querySelector("[data-run-progress]");
        errorNode.hidden = true;
        progressNode.hidden = false;

        var collected = collectParams(form);
        if (collected.invalid) {
            progressNode.hidden = true;
            errorNode.textContent = collected.invalid;
            errorNode.hidden = false;
            if (rootNode) namespace.announce(rootNode, t("integrations-runner-cannot-run", "Action cannot run: { $detail }", { detail: collected.invalid }));
            return;
        }

        var botId = (namespace.context && namespace.context.botId) || "";
        if (!botId) {
            progressNode.hidden = true;
            errorNode.textContent = t("integrations-runner-need-workspace", "Open this app inside a bot workspace to run actions.");
            errorNode.hidden = false;
            if (rootNode) namespace.announce(rootNode, t("integrations-runner-need-workspace", "Open this app inside a bot workspace to run actions."));
            return;
        }

        try {
            var requestUrl = "/api/bots/" + encodeURIComponent(botId) + "/integration-actions/invoke";
            var requestBody = { provider: providerSlug, action: actionKey, params: collected.params };
            var payload;
            if (namespace && typeof namespace.fetchJson === "function") {
                payload = await namespace.fetchJson(requestUrl, { method: "POST", body: requestBody });
            } else {
                var headers = { "Content-Type": "application/json" };
                var token = (namespace && typeof namespace.currentToken === "function") ? namespace.currentToken() : "";
                if (token) headers["Authorization"] = "Bearer " + token;
                var csrf = (namespace && typeof namespace.csrfToken === "function") ? namespace.csrfToken() : "";
                if (!csrf) throw new Error(t("integrations-runner-csrf-missing", "CSRF token is missing. Reload the page and try again."));
                headers["X-CSRF-Token"] = csrf;
                var response = await fetch(requestUrl, { method: "POST", credentials: "same-origin", headers: headers, body: JSON.stringify(requestBody) });
                payload = await response.json().catch(function () { return null; });
                if (!response.ok) {
                    var err = new Error(t("integrations-runner-request-failed", "request failed"));
                    err.status = response.status;
                    err.body = payload;
                    throw err;
                }
            }
            progressNode.hidden = true;
            if (!payload || payload.outcome === undefined) {
                var detail = payload && (payload.detail || payload.error || payload.message);
                var noOutcome = detail || t("integrations-runner-no-outcome", "Action did not return an outcome.");
                errorNode.textContent = noOutcome;
                errorNode.hidden = false;
                if (rootNode) namespace.announce(rootNode, t("integrations-runner-failed", "Action failed: { $detail }", { detail: noOutcome }));
                return;
            }
            var runs = readHistory(providerSlug, actionKey);
            runs.unshift({
                at: Date.now(),
                summary: payload.outcome.summary || "",
                truncated: payload.outcome.truncated === true,
                data: payload.outcome.data,
                params: collected.params,
            });
            writeHistory(providerSlug, actionKey, runs);
            renderOutcomeArea(article, providerSlug, actionKey, 0);
            if (rootNode) namespace.announce(rootNode, t("integrations-runner-completed", "Action completed: { $summary }", { summary: payload.outcome.summary || t("integrations-runner-outcome", "Outcome") }));
        } catch (error) {
            progressNode.hidden = true;
            var detailText = error && error.body && (error.body.detail || error.body.error || error.body.message);
            var message = detailText || (error && error.message ? error.message : t("integrations-runner-network-error", "Network error while calling the connector."));
            errorNode.textContent = message;
            errorNode.hidden = false;
            if (rootNode) namespace.announce(rootNode, t("integrations-runner-failed", "Action failed: { $detail }", { detail: message }));
        }
    }

    function prefillAutomation(root, providerSlug, actionKey, params) {
        if (!root) return;
        var tab = root.querySelector('[data-tab="automations"]');
        if (tab) tab.click();
        var form = root.querySelector("[data-automation-form]");
        if (!form) return;
        form.hidden = false;
        var providerSelect = form.querySelector("[data-automation-provider]");
        if (providerSelect && providerSelect.dataset.loaded !== "1" && namespace.fillAutomationProviderSelect) {
            namespace.fillAutomationProviderSelect(providerSelect);
        }
        if (providerSelect) {
            providerSelect.value = providerSlug;
            providerSelect.dispatchEvent(new Event("change", { bubbles: true }));
        }
        var actionSelect = form.querySelector("[data-automation-action]");
        if (actionSelect) {
            actionSelect.value = actionKey;
            if (!actionSelect.value) {
                // best effort: match by suffix or label
                for (var i = 0; i < actionSelect.options.length; i += 1) {
                    if (actionSelect.options[i].value === actionKey) { actionSelect.selectedIndex = i; break; }
                }
            }
        }
        var paramsField = form.querySelector('[name="params"]');
        if (paramsField) {
            try { paramsField.value = Object.keys(params || {}).length ? JSON.stringify(params, null, 2) : ""; } catch (_e) { paramsField.value = ""; }
        }
        namespace.announce(root, t("integrations-runner-prefilled", "Automation form prefilled from runner"));
        if (providerSelect) providerSelect.focus();
    }

    function onDelegatedClick(event) {
        var target = event.target && event.target.closest ? event.target : null;
        if (!target) return;
        var runButton = target.closest("[data-run-action]");
        if (runButton) {
            var root = findRoot(runButton);
            var article = runButton.closest(".integrations-action");
            if (!root || !article) return;
            var providerSlug = runButton.dataset.providerSlug || "";
            var actionKey = runButton.dataset.runAction || "";
            var provider = providerFor(root, providerSlug);
            var action = actionFor(provider, actionKey);
            if (!action) return;
            event.preventDefault();
            toggleRunner(article, actionKey, normalizeParams(action), providerSlug);
            return;
        }
        var closeButton = target.closest('[data-action="close-runner"]');
        if (closeButton) {
            var holder = closeButton.closest(".integrations-action");
            if (holder) {
                var panel = holder.querySelector("[data-runner]");
                if (panel) panel.remove();
            }
            return;
        }
        var saveButton = target.closest('[data-action="save-run-as-automation"]');
        if (saveButton) {
            var article2 = saveButton.closest(".integrations-action");
            var root2 = findRoot(saveButton);
            var runBtn = article2 && article2.querySelector("[data-run-action]");
            var form = article2 && article2.querySelector("[data-run-form]");
            if (!article2 || !root2 || !runBtn || !form) return;
            var collected = collectParams(form);
            if (collected.invalid) {
                var errNode = form.querySelector("[data-run-error]");
                if (errNode) { errNode.textContent = collected.invalid; errNode.hidden = false; }
                return;
            }
            prefillAutomation(root2, runBtn.dataset.providerSlug || "", runBtn.dataset.runAction || "", collected.params);
            return;
        }
        var histSelect = target.closest("[data-run-history-select]");
        if (histSelect) {
            var article3 = histSelect.closest(".integrations-action");
            var runBtn3 = article3 && article3.querySelector("[data-run-action]");
            if (!article3 || !runBtn3) return;
            renderOutcomeArea(article3, runBtn3.dataset.providerSlug || "", runBtn3.dataset.runAction || "", parseInt(histSelect.getAttribute("data-run-history-select"), 10) || 0);
            return;
        }
        var rerun = target.closest("[data-run-rerun]");
        if (rerun) {
            var article4 = rerun.closest(".integrations-action");
            var runBtn4 = article4 && article4.querySelector("[data-run-action]");
            var form4 = article4 && article4.querySelector("[data-run-form]");
            if (!article4 || !runBtn4 || !form4) return;
            var idx = parseInt(rerun.getAttribute("data-run-rerun"), 10) || 0;
            var runs = readHistory(runBtn4.dataset.providerSlug || "", runBtn4.dataset.runAction || "");
            var run = runs[idx];
            if (run && run.params) fillFormParams(form4, run.params);
            form4.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
            return;
        }
        var toggleBody = target.closest("[data-run-toggle-body]");
        if (toggleBody) {
            var view = toggleBody.closest(".integrations-run-outcome-view") || toggleBody.closest("[data-run-history]");
            var body = view && view.querySelector("[data-run-body]");
            if (!body) return;
            body.hidden = !body.hidden;
            toggleBody.textContent = body.hidden
                ? t("integrations-runner-show-body", "Show response body")
                : t("integrations-runner-hide-body", "Hide response body");
            return;
        }
        var toggleDiff = target.closest("[data-run-toggle-diff]");
        if (toggleDiff) {
            var view2 = toggleDiff.closest(".integrations-run-outcome-view") || toggleDiff.closest("[data-run-history]");
            var diffEl = view2 && view2.querySelector("[data-run-diff]");
            if (!diffEl) return;
            diffEl.hidden = !diffEl.hidden;
            toggleDiff.textContent = diffEl.hidden
                ? t("integrations-runner-show-diff", "Diff vs previous")
                : t("integrations-runner-hide-diff", "Hide diff");
        }
    }

    function onDelegatedSubmit(event) {
        var form = event.target;
        if (!form || !form.matches || !form.matches("[data-run-form]")) return;
        var article = form.closest(".integrations-action");
        var runButton = article && article.querySelector("[data-run-action]");
        if (!article || !runButton) return;
        event.preventDefault();
        execute(article, runButton.dataset.providerSlug || "", runButton.dataset.runAction || "", form)
            .catch(function (err) {
                console.warn("[integrations-runner] unexpected execute failure", err);
            });
    }

    namespace.installActionRunner = function () {
        if (RUN_DELEGATED) return;
        RUN_DELEGATED = true;
        document.addEventListener("click", onDelegatedClick);
        document.addEventListener("submit", onDelegatedSubmit, true);
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
