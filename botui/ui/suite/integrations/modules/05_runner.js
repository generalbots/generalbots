"use strict";

// Action runner (#939/#2352): execute implemented provider actions from the
// provider detail panel with local run history, re-run, diff, safe outcome
// rendering, and automation prefill. Delegated events keep this independent
// from the render pipeline.

(function (namespace) {
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
            // #2329 — one time shape across the suite, from the suite preference.
            var when = run.at && window.gbFormatTime
                ? window.gbFormatTime(run.at)
                : t("integrations-runner-run-label", "run");
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

    // Shared with 05b_runner_execute.js (same namespace) so the execute
    // flow and the delegated handlers live in their own module.
    namespace.__runnerInternals = {
        t: t, findRoot: findRoot, providerFor: providerFor, actionFor: actionFor,
        normalizeParams: normalizeParams, readHistory: readHistory, writeHistory: writeHistory,
        renderOutcomeArea: renderOutcomeArea, toggleRunner: toggleRunner,
        collectParams: collectParams, fillFormParams: fillFormParams,
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
