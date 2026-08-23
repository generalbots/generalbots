"use strict";

// Action runner (#939 wave): execute an implemented provider action from the
// provider detail panel and inspect the redacted outcome. Delegated events
// keep this module independent from the render pipeline.

(function (namespace) {
    var RUN_DELEGATED = false;

    function findRoot(node) {
        return node.closest ? node.closest("[data-integrations-app]") : null;
    }

    function providerFor(root, slug) {
        var state = namespace.getState(root);
        if (!state || !state.providerMap) return null;
        return state.providerMap.get(slug) || null;
    }

    function actionFor(provider, key) {
        if (!provider || !Array.isArray(provider.actions)) return null;
        for (var index = 0; index < provider.actions.length; index += 1) {
            if (provider.actions[index].name === key) return provider.actions[index];
        }
        return null;
    }

    function normalizeParams(action) {
        var raw = Array.isArray(action.params) ? action.params : [];
        return raw.map(function (param) {
            return {
                name: String(param.name || param.key || ""),
                kind: String(param.type || "string").toLowerCase(),
                required: param.required === true,
            };
        }).filter(function (param) { return param.name; });
    }

    function runnerMarkup(actionKey, params) {
        var inputs = params.map(function (param) {
            var id = "run-" + actionKey.replace(/\./g, "-") + "-" + param.name;
            if (param.kind === "json") {
                return '<label class="integrations-run-field" for="' + id + '">' +
                    namespace.escapeHtml(param.name + (param.required ? " *" : "")) +
                    '<textarea id="' + id + '" data-run-param="' + namespace.escapeHtml(param.name) +
                    '" data-kind="json" rows="4" placeholder=\'{ "example": true }\'></textarea></label>';
            }
            return '<label class="integrations-run-field" for="' + id + '">' +
                namespace.escapeHtml(param.name + (param.required ? " *" : "")) +
                '<input id="' + id + '" type="text" data-run-param="' + namespace.escapeHtml(param.name) +
                '" data-kind="text" autocomplete="off"></label>';
        }).join("");
        return '<div class="integrations-runner" data-runner hidden>' +
            '<form data-run-form novalidate>' +
            '<div class="integrations-run-grid">' + inputs + '</div>' +
            '<p class="integrations-inline-error" data-run-error hidden role="alert"></p>' +
            '<p class="integrations-inline-progress" data-run-progress hidden role="status">Running against the live provider...</p>' +
            '<div class="integrations-run-actions">' +
            '<button class="integrations-button" type="submit">Execute</button>' +
            '<button class="integrations-button integrations-button-subtle" type="button" data-action="close-runner">Close</button>' +
            '</div></form>' +
            '<div class="integrations-run-outcome" data-run-outcome hidden></div>' +
            '</div>';
    }

    function toggleRunner(article, actionKey, params) {
        var existing = article.querySelector("[data-runner]");
        if (existing) {
            existing.remove();
            return;
        }
        article.insertAdjacentHTML("beforeend", runnerMarkup(actionKey, params));
        var runner = article.querySelector("[data-runner]");
        runner.hidden = false;
        var firstInput = runner.querySelector("input, textarea");
        if (firstInput) firstInput.focus();
    }

    function collectParams(form) {
        var params = {};
        var invalid = "";
        Array.prototype.forEach.call(form.querySelectorAll("[data-run-param]"), function (input) {
            var value = input.value.trim();
            if (!value) return;
            if (input.dataset.kind === "json") {
                try {
                    params[input.dataset.runParam] = JSON.parse(value);
                } catch (error) {
                    invalid = input.dataset.runParam + " must be valid JSON";
                }
                return;
            }
            params[input.dataset.runParam] = value;
        });
        return { params: params, invalid: invalid };
    }

    function outcomeMarkup(outcome) {
        var payload;
        try {
            payload = JSON.stringify(outcome.data, null, 2);
        } catch (error) {
            payload = String(outcome.data);
        }
        return '<p class="integrations-run-summary">' + namespace.escapeHtml(outcome.summary || "") +
            (outcome.truncated === true ? ' <span class="integrations-pill">Truncated</span>' : "") +
            '</p><pre class="integrations-run-data">' + namespace.escapeHtml(payload) + "</pre>";
    }

    async function execute(article, providerSlug, actionKey, form) {
        var errorNode = form.querySelector("[data-run-error]");
        var progressNode = form.querySelector("[data-run-progress]");
        var outcomeNode = article.querySelector("[data-run-outcome]");
        errorNode.hidden = true;
        progressNode.hidden = false;

        var collected = collectParams(form);
        if (collected.invalid) {
            progressNode.hidden = true;
            errorNode.textContent = collected.invalid;
            errorNode.hidden = false;
            return;
        }

        var botId = (namespace.context && namespace.context.botId) || "";
        if (!botId) {
            progressNode.hidden = true;
            errorNode.textContent = "Open this app inside a bot workspace to run actions.";
            errorNode.hidden = false;
            return;
        }

        try {
            var response = await fetch("/api/bots/" + encodeURIComponent(botId) + "/integration-actions/invoke", {
                method: "POST",
                credentials: "same-origin",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    provider: providerSlug,
                    action: actionKey,
                    params: collected.params,
                }),
            });
            var payload = await response.json().catch(function () { return null; });
            progressNode.hidden = true;
            if (!response.ok || !payload || payload.outcome === undefined) {
                var detail = payload && (payload.detail || payload.error || payload.message);
                errorNode.textContent = detail || ("Request failed with status " + response.status);
                errorNode.hidden = false;
                return;
            }
            outcomeNode.innerHTML = outcomeMarkup(payload.outcome);
            outcomeNode.hidden = false;
        } catch (error) {
            progressNode.hidden = true;
            errorNode.textContent = "Network error while calling the connector.";
            errorNode.hidden = false;
        }
    }

    function onDelegatedClick(event) {
        var target = event.target.closest ? event.target : null;
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
            toggleRunner(article, actionKey, normalizeParams(action));
            return;
        }
        var closeButton = target.closest('[data-action="close-runner"]');
        if (closeButton) {
            var holder = closeButton.closest(".integrations-action");
            if (holder) {
                var panel = holder.querySelector("[data-runner]");
                if (panel) panel.remove();
            }
        }
    }

    function onDelegatedSubmit(event) {
        var form = event.target;
        if (!form.matches || !form.matches("[data-run-form]")) return;
        var article = form.closest(".integrations-action");
        var button = event.submitter || form.querySelector('button[type="submit"]');
        var runButton = article && article.querySelector("[data-run-action]");
        if (!article || !runButton) return;
        event.preventDefault();
        execute(article, runButton.dataset.providerSlug || "", runButton.dataset.runAction || "", form)
            .catch(function () {});
    }

    namespace.installActionRunner = function () {
        if (RUN_DELEGATED) return;
        RUN_DELEGATED = true;
        document.addEventListener("click", onDelegatedClick);
        document.addEventListener("submit", onDelegatedSubmit, true);
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
