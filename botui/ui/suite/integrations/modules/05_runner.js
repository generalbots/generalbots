"use strict";

// Action runner (#939/#2352): execute implemented provider actions from the
// provider detail panel with local run history, re-run, diff, safe outcome
// rendering, and automation prefill. Delegated events keep this independent
// from the render pipeline.

(function (namespace) {
    var RUN_DELEGATED = false;
    var HISTORY_LIMIT = 5;
    var BODY_PREVIEW_CHARS = 2000;

    function t(key, fallback, params) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    }

    function findRoot(node) {
        return node && node.closest ? node.closest("[data-integrations-app]") : null;
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
    function historyKey(providerSlug, actionKey) {
        return "gb-integrations-run-history:" + String(providerSlug || "") + ":" + String(actionKey || "");
    }
    function readHistory(providerSlug, actionKey) {
        try {
            var raw = sessionStorage.getItem(historyKey(providerSlug, actionKey));
            var parsed = raw ? JSON.parse(raw) : [];
            return Array.isArray(parsed) ? parsed : [];
        } catch (_e) {
            return [];
        }
    }

    function writeHistory(providerSlug, actionKey, runs) {
        try {
            sessionStorage.setItem(historyKey(providerSlug, actionKey), JSON.stringify((runs || []).slice(0, HISTORY_LIMIT)));
        } catch (_e) {
            // sessionStorage may be unavailable/full; history is best-effort.
        }
    }

    function stringifyData(data) {
        try {
            if (typeof data === "string") return data;
            return JSON.stringify(data, null, 2);
        } catch (_e) {
            return String(data);
        }
    }

    function summarizeDiff(previous, current) {
        if (!previous) return t("integrations-runner-no-previous", "No previous run to compare.");
        var lines = [];
        if ((previous.summary || "") !== (current.summary || "")) {
            lines.push(t("integrations-runner-summary-changed", "Summary changed."));
            lines.push("- " + String(previous.summary || t("integrations-runner-empty", "(empty)")));
            lines.push("+ " + String(current.summary || t("integrations-runner-empty", "(empty)")));
        } else {
            lines.push(t("integrations-runner-summary-unchanged", "Summary unchanged."));
        }
        var prevBody = stringifyData(previous.data);
        var currBody = stringifyData(current.data);
        if (prevBody === currBody) {
            lines.push(t("integrations-runner-body-unchanged", "Response body unchanged."));
            return lines.join("\n");
        }
        var prevLines = prevBody.split("\n");
        var currLines = currBody.split("\n");
        var max = Math.max(prevLines.length, currLines.length);
        var shown = 0;
        for (var i = 0; i < max && shown < 12; i += 1) {
            var a = prevLines[i];
            var b = currLines[i];
            if (a === b) continue;
            if (a !== undefined) { lines.push("- " + a); shown += 1; }
            if (shown >= 12) break;
            if (b !== undefined) { lines.push("+ " + b); shown += 1; }
        }
        if (shown >= 12) lines.push(t("integrations-runner-diff-truncated", "… diff truncated"));
        return lines.join("\n");
    }

    function historyMarkup(providerSlug, actionKey, selectedIndex) {
        var runs = readHistory(providerSlug, actionKey);
        if (!runs.length) {
            return '<div class="integrations-run-history" data-run-history><p class="integrations-readonly-note">' +
                namespace.escapeHtml(t("integrations-runner-no-history", "No previous runs in this session.")) + '</p></div>';
        }
        var selected = typeof selectedIndex === "number" ? selectedIndex : 0;
        if (selected < 0 || selected >= runs.length) selected = 0;
        var current = runs[selected] || null;
        var previous = runs[selected + 1] || null;
        var body = current ? stringifyData(current.data) : "";
        var truncated = body.length > BODY_PREVIEW_CHARS;
        var preview = truncated
            ? body.slice(0, BODY_PREVIEW_CHARS) + "\n… " + t("integrations-runner-response-truncated", "response truncated")
            : body;
        var items = runs.map(function (run, index) {
            var when = run.at ? new Date(run.at).toLocaleTimeString() : t("integrations-runner-run-label", "run");
            var label = when + " — " + String(run.summary || t("integrations-runner-outcome", "Outcome")).slice(0, 80);
            return '<li><button type="button" class="integrations-button integrations-button-subtle" data-run-history-select="' + index + '"' +
                (index === selected ? ' aria-current="true"' : '') + '>' + namespace.escapeHtml(label) + '</button> ' +
                '<button type="button" class="integrations-button integrations-button-subtle" data-run-rerun="' + index + '">' +
                namespace.escapeHtml(t("integrations-runner-rerun", "Re-run with same params")) + '</button></li>';
        }).join("");
        var diff = current ? summarizeDiff(previous, current) : "";
        return '<div class="integrations-run-history" data-run-history data-history-selected="' + selected + '">' +
            '<div class="integrations-run-history-head"><strong>' + namespace.escapeHtml(t("integrations-runner-history-title", "Run history")) + '</strong>' +
            '<span class="integrations-readonly-note">' + namespace.escapeHtml(t("integrations-runner-history-count", "Last { $count } in this session", { count: runs.length })) + '</span></div>' +
            '<ul class="integrations-run-history-list">' + items + '</ul>' +
            (current ? '<div class="integrations-run-outcome-view" data-run-outcome-view>' +
                '<p class="integrations-run-summary">' + namespace.escapeHtml(current.summary || "") +
                (current.truncated === true ? ' <span class="integrations-pill">' + namespace.escapeHtml(t("integrations-runner-truncated", "Truncated")) + '</span>' : '') + '</p>' +
                '<div class="integrations-run-actions"><button type="button" class="integrations-button integrations-button-subtle" data-run-toggle-body>' +
                namespace.escapeHtml(t("integrations-runner-show-body", "Show response body")) + '</button>' +
                ' <button type="button" class="integrations-button integrations-button-subtle" data-run-toggle-diff>' +
                namespace.escapeHtml(t("integrations-runner-show-diff", "Diff vs previous")) + '</button></div>' +
                '<pre class="integrations-run-data" data-run-body hidden>' + namespace.escapeHtml(preview) + '</pre>' +
                '<pre class="integrations-run-data" data-run-diff hidden>' + namespace.escapeHtml(diff) + '</pre>' +
                (truncated ? '<p class="integrations-readonly-note">' +
                    namespace.escapeHtml(t("integrations-runner-body-hidden-note", "Large response body hidden by default and previewed.")) + '</p>' : '') +
            '</div>' : '') +
            '</div>';
    }

    function renderOutcomeArea(article, providerSlug, actionKey, selectedIndex) {
        var outcomeNode = article.querySelector("[data-run-outcome]");
        if (!outcomeNode) return;
        outcomeNode.innerHTML = historyMarkup(providerSlug, actionKey, selectedIndex);
        outcomeNode.hidden = false;
    }

    function runnerMarkup(actionKey, params) {
        var inputs = params.map(function (param) {
            var id = "run-" + actionKey.replace(/\./g, "-") + "-" + param.name;
            if (param.kind === "json") {
                return '<label class="integrations-run-field" for="' + id + '">' +
                    namespace.escapeHtml(param.name + (param.required ? " *" : "")) +
                    '<textarea id="' + id + '" data-run-param="' + namespace.escapeHtml(param.name) +
                    '" data-kind="json" rows="4" placeholder=\'' +
                    namespace.escapeHtml(t("integrations-runner-json-placeholder", "{ \"example\": true }")) + '\'></textarea></label>';
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
            '<p class="integrations-inline-progress" data-run-progress hidden role="status" aria-live="polite">' +
            namespace.escapeHtml(t("integrations-runner-running", "Running against the live provider...")) + '</p>' +
            '<div class="integrations-run-actions">' +
            '<button class="integrations-button" type="submit">' + namespace.escapeHtml(t("integrations-runner-execute", "Execute")) + '</button>' +
            '<button class="integrations-button integrations-button-subtle" type="button" data-action="save-run-as-automation">' +
            namespace.escapeHtml(t("integrations-runner-save-automation", "Save as automation")) + '</button>' +
            '<button class="integrations-button integrations-button-subtle" type="button" data-action="close-runner">' +
            namespace.escapeHtml(t("action-close", "Close")) + '</button>' +
            '</div></form>' +
            '<div class="integrations-run-outcome" data-run-outcome hidden role="status" aria-live="polite" aria-atomic="true"></div>' +
            '</div>';
    }

    function toggleRunner(article, actionKey, params, providerSlug) {
        var existing = article.querySelector("[data-runner]");
        if (existing) {
            existing.remove();
            return;
        }
        article.insertAdjacentHTML("beforeend", runnerMarkup(actionKey, params));
        var runner = article.querySelector("[data-runner]");
        runner.hidden = false;
        renderOutcomeArea(article, providerSlug, actionKey, 0);
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
                } catch (_error) {
                    invalid = t("integrations-runner-json-invalid", "{ $field } must be valid JSON", { field: input.dataset.runParam });
                }
                return;
            }
            params[input.dataset.runParam] = value;
        });
        return { params: params, invalid: invalid };
    }

    function fillFormParams(form, params) {
        if (!form || !params || typeof params !== "object") return;
        Array.prototype.forEach.call(form.querySelectorAll("[data-run-param]"), function (input) {
            var name = input.dataset.runParam;
            if (!(name in params)) return;
            var value = params[name];
            if (input.dataset.kind === "json") {
                try { input.value = JSON.stringify(value, null, 2); } catch (_e) { input.value = String(value); }
            } else {
                input.value = value == null ? "" : String(value);
            }
        });
    }

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
