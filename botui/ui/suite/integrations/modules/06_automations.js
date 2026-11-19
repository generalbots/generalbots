"use strict";

// Automations panel: create, list, toggle and delete scheduled runs of
// implemented integration actions, run one immediately, and inspect a per
// automation run log. Delegated events keep this independent from the render
// pipeline.

(function (namespace) {
    var LOG_LIMIT = 5;

    function t(key, fallback, params) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    }
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
    function logKey(id) {
        return "gb-integrations-automation-log:" + String(id || "");
    }
    function readLog(id) {
        try {
            var raw = sessionStorage.getItem(logKey(id));
            var parsed = raw ? JSON.parse(raw) : [];
            return Array.isArray(parsed) ? parsed : [];
        } catch (_e) {
            return [];
        }
    }

    function writeLog(id, entries) {
        try {
            sessionStorage.setItem(logKey(id), JSON.stringify((entries || []).slice(0, LOG_LIMIT)));
        } catch (_e) {
            // sessionStorage may be unavailable/full; the log is best-effort.
        }
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
        select.innerHTML = '<option value="">' + namespace.escapeHtml(t("integrations-automation-select-provider", "Select a provider")) + '</option>' +
            entries.map(function (entry) {
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
            actionSelect.innerHTML = '<option value="">' + namespace.escapeHtml(t("integrations-automation-select-action", "Select an action")) + '</option>';
            return;
        }
        actionSelect.innerHTML = match.actions.map(function (action) {
            return '<option value="' + namespace.escapeHtml(action.name) + '">' +
                namespace.escapeHtml(action.name) + '</option>';
        }).join("");
    }

    namespace.fillAutomationProviderSelect = fillProviderSelect;
    namespace.syncAutomationActions = syncActions;

    async function loadAutomations(view, statusNode) {
        view.setAttribute("aria-busy", "true");
        if (!botId()) {
            statusNode.textContent = t("integrations-automation-need-workspace", "Open this app inside a bot workspace to manage automations.");
            view.innerHTML = "";
            return;
        }
        var app = root();
        try {
            var payload = await namespace.fetchJson(base(), { credentials: "same-origin" });
            var items = Array.isArray(payload.automations) ? payload.automations : [];
            statusNode.textContent = items.length ? "" : t("integrations-automation-empty", "No automations yet.");
            view.removeAttribute("aria-busy");
            view.innerHTML = items.length ? items.map(rowMarkup).join("") : "";
            if (app) namespace.announce(app, items.length
                ? t("integrations-automation-loaded", "{ $count } automations loaded", { count: items.length })
                : t("integrations-automation-empty", "No automations yet."));
        } catch (error) {
            statusNode.textContent = t("integrations-automation-unavailable", "Automations are unavailable right now.");
            view.removeAttribute("aria-busy");
            if (app) namespace.announce(app, t("integrations-automation-unavailable", "Automations are unavailable right now."));
        }
    }

    function fmtTime(value) {
        if (!value) return "";
        try { return String(value).replace("T", " ").slice(0, 16); } catch (_e) { return String(value); }
    }

    function stringifyData(data) {
        try {
            return typeof data === "string" ? data : JSON.stringify(data, null, 2);
        } catch (_e) {
            return String(data);
        }
    }

    function serverOutcome(item) {
        return item.last_outcome || item.last_result || item.last_error || "";
    }

    function logEntriesMarkup(item, id) {
        var rows = readLog(id).map(function (entry) {
            // #2329 — same date/time contract as the rest of the suite.
            var when = entry.at && window.gbFormatDateTime
                ? window.gbFormatDateTime(entry.at)
                : t("integrations-automation-log-entry", "run");
            var status = entry.status === "error"
                ? t("integrations-automation-status-error", "failed")
                : t("integrations-automation-status-ok", "completed");
            var body = entry.data === undefined || entry.data === null ? "" : stringifyData(entry.data);
            return '<li data-automation-log-entry data-automation-log-local="1"><strong>' + namespace.escapeHtml(when) + '</strong> — ' +
                namespace.escapeHtml(status) + ' ' + namespace.escapeHtml(entry.summary || "") +
                (body ? '<pre class="integrations-run-data">' + namespace.escapeHtml(body) + '</pre>' : "") + '</li>';
        }).join("");
        var outcome = serverOutcome(item);
        if (outcome) {
            rows += '<li data-automation-log-entry><strong>' + namespace.escapeHtml(t("integrations-automation-last-outcome", "Last outcome (server)")) +
                '</strong><pre class="integrations-run-data">' + namespace.escapeHtml(stringifyData(outcome)) + '</pre></li>';
        }
        return rows || '<p class="integrations-readonly-note">' + namespace.escapeHtml(t("integrations-automation-no-log", "No run log entries yet.")) + '</p>';
    }

    function rowMarkup(item) {
        var id = item.id || "";
        var checked = item.enabled === true ? " checked" : "";
        var providerSlug = item.provider_slug || item.provider || "";
        var actionKey = item.action_key || item.action || "";
        var title = (providerSlug && actionKey) ? (providerSlug + "." + String(actionKey).split(".").slice(1).join(".") || actionKey) : (actionKey || providerSlug || t("integrations-automation-title", "Automation"));
        var nextRun = item.next_run_at || item.next_run || "";
        var lastRun = item.last_run_at || item.last_ran_at || "";
        var lastStatus = item.last_run_status || item.last_status || "";
        var runCount = (item.run_count !== undefined && item.run_count !== null) ? item.run_count : "";
        var meta = [];
        meta.push(t("integrations-automation-schedule", "Schedule: { $schedule }", { schedule: String(item.schedule || "—") }));
        meta.push(nextRun ? t("integrations-automation-next", "next { $when }", { when: fmtTime(nextRun) }) : t("integrations-automation-next-unavailable", "next run unavailable"));
        meta.push(lastRun ? t("integrations-automation-last", "last { $when }", { when: fmtTime(lastRun) }) : t("integrations-automation-never", "never run"));
        if (lastStatus) meta.push(t("integrations-automation-status", "status { $status }", { status: String(lastStatus) }));
        if (runCount !== "") meta.push(t("integrations-automation-runs", "{ $count } runs", { count: runCount }));
        var pauseLabel = item.enabled === true ? t("integrations-automation-pause", "Pause") : t("integrations-automation-resume", "Resume");
        var pauseTitle = (id && botId())
            ? t("integrations-automation-pause-title", "Pauses or resumes this automation through the enabled flag (PATCH).")
            : t("integrations-automation-pause-unsupported", "Pause is unavailable: missing automation or workspace context.");
        var localCount = readLog(id).length;
        var clearTitle = t("integrations-automation-clear-title", "Clears only this browser session's run log. The API does not support clearing server history.");
        return '<article class="integrations-automation" data-automation-id="' + namespace.escapeHtml(id) + '" data-provider="' + namespace.escapeHtml(providerSlug) + '" data-action-key="' + namespace.escapeHtml(actionKey) + '" data-params="' + namespace.escapeHtml(JSON.stringify(item.params || item.parameters || {})) + '">' +
            '<div class="integrations-automation-head"><strong>' + namespace.escapeHtml(title) + '</strong><label class="integrations-automation-toggle"><input type="checkbox" data-automation-toggle' + checked + '> ' + namespace.escapeHtml(t("integrations-automation-enabled", "enabled")) + '</label></div>' +
            '<p class="integrations-run-summary">' + namespace.escapeHtml(meta.join(" · ")) + '</p>' +
            '<details class="integrations-automation-log" data-automation-log><summary>' + namespace.escapeHtml(t("integrations-automation-run-log", "Run log")) +
            ' <span class="integrations-readonly-note">' + namespace.escapeHtml(t("integrations-automation-log-count", "{ $count } entries", { count: localCount + (serverOutcome(item) ? 1 : 0) })) + '</span></summary>' +
            '<div data-automation-log-entries>' + logEntriesMarkup(item, id) + '</div></details>' +
            '<div class="integrations-run-actions">' +
            '<button class="integrations-button integrations-button-subtle" type="button" data-action="run-automation-now">' + namespace.escapeHtml(t("integrations-automation-run-now", "Run now")) + '</button>' +
            '<button class="integrations-button integrations-button-subtle" type="button" data-action="pause-automation" data-paused="' + (item.enabled === true ? "false" : "true") + '" title="' + namespace.escapeHtml(pauseTitle) + '"' + (id && botId() ? "" : " disabled") + '>' + namespace.escapeHtml(pauseLabel) + '</button>' +
            '<button class="integrations-button integrations-button-subtle" type="button" data-action="clear-automation-history" title="' + namespace.escapeHtml(clearTitle) + '"' + (localCount ? "" : " disabled") + '>' + namespace.escapeHtml(t("integrations-automation-clear-history", "Clear history")) + '</button>' +
            '<button class="integrations-button integrations-button-subtle" type="button" data-action="delete-automation">' + namespace.escapeHtml(t("integrations-automation-delete", "Delete")) + '</button>' +
            '</div></article>';
    }

    function confirmWithModal(message) {
        return new Promise(function (resolve) {
            if (window.GBModal && typeof window.GBModal.confirm === "function") {
                window.GBModal.confirm(message, t("integrations-automation-confirm-title", "Confirm"), function () { resolve(true); }, function () { resolve(false); });
                return;
            }
            resolve(false);
        });
    }

    async function deleteAutomation(app, article, id, button) {
        if (!app || !article || !id || !button) return;
        if (button.disabled || button.dataset.deleting) return;
        button.dataset.deleting = "pending";
        button.disabled = true;
        button.setAttribute("aria-busy", "true");
        var confirmed = await confirmWithModal(t("integrations-automation-delete-confirm", "Delete this automation?"));
        if (!confirmed) {
            button.disabled = false;
            delete button.dataset.deleting;
            button.removeAttribute("aria-busy");
            return;
        }
        button.dataset.deleting = "1";
        try {
            await namespace.fetchJson(base() + "/" + encodeURIComponent(id), { method: "DELETE" });
            article.remove();
            var statusNode = app.querySelector("[data-automation-status]");
            var remaining = app.querySelectorAll("[data-automation-id]").length;
            if (!remaining && statusNode) statusNode.textContent = t("integrations-automation-empty", "No automations yet.");
            namespace.announce(app, t("integrations-automation-deleted", "Automation deleted."));
        } catch (error) {
            button.disabled = false;
            delete button.dataset.deleting;
            button.removeAttribute("aria-busy");
            namespace.announce(app, t("integrations-automation-delete-failed", "Could not delete the automation."));
        }
    }

    async function runAutomationNow(app, article) {
        if (!app || !article) return;
        var id = article.dataset.automationId || "";
        var provider = article.dataset.provider || "";
        var actionKey = article.dataset.actionKey || "";
        var params = {};
        try { params = JSON.parse(article.dataset.params || "{}") || {}; } catch (_e) { params = {}; }
        if (!botId() || !provider || !actionKey) {
            namespace.announce(app, t("integrations-automation-run-missing", "Cannot run automation: missing provider, action, or workspace context."));
            return;
        }
        namespace.announce(app, t("integrations-automation-running", "Running automation now..."));
        try {
            var payload = await namespace.fetchJson("/api/bots/" + encodeURIComponent(botId()) + "/integration-actions/invoke", {
                method: "POST",
                body: { provider: provider, action: actionKey, params: params }
            });
            var outcome = payload && payload.outcome ? payload.outcome : {};
            var entries = readLog(id);
            entries.unshift({ at: Date.now(), status: "ok", summary: outcome.summary || "", data: outcome.data });
            writeLog(id, entries);
            namespace.announce(app, t("integrations-automation-run-completed", "Automation run completed."));
            loadAutomations(app.querySelector("[data-automations-view]"), app.querySelector("[data-automation-status]"));
        } catch (error) {
            var detail = error && error.body && (error.body.detail || error.body.error || error.body.message);
            var message = detail || (error && error.message) || t("integrations-automation-request-failed", "request failed");
            var failed = readLog(id);
            failed.unshift({ at: Date.now(), status: "error", summary: message });
            writeLog(id, failed);
            namespace.announce(app, t("integrations-automation-run-failed", "Automation run failed: { $detail }", { detail: message }));
            loadAutomations(app.querySelector("[data-automations-view]"), app.querySelector("[data-automation-status]"));
        }
    }

    function syncPauseButton(button, enabled) {
        if (!button) return;
        button.dataset.paused = enabled ? "false" : "true";
        button.textContent = enabled ? t("integrations-automation-pause", "Pause") : t("integrations-automation-resume", "Resume");
    }

    async function pauseAutomation(app, article, button) {
        var id = article && article.dataset.automationId;
        if (!app || !article || !id || !button || button.disabled) return;
        var enable = button.dataset.paused === "true";
        button.disabled = true;
        button.setAttribute("aria-busy", "true");
        try {
            await namespace.fetchJson(base() + "/" + encodeURIComponent(id), { method: "PATCH", body: { enabled: enable } });
            var input = article.querySelector("[data-automation-toggle]");
            if (input) input.checked = enable;
            syncPauseButton(button, enable);
            namespace.announce(app, enable
                ? t("integrations-automation-resumed", "Automation resumed.")
                : t("integrations-automation-paused", "Automation paused."));
        } catch (error) {
            namespace.announce(app, t("integrations-automation-update-failed", "Could not update the automation."));
        } finally {
            button.disabled = false;
            button.removeAttribute("aria-busy");
        }
    }

    function clearHistory(app, article, button) {
        var id = article && article.dataset.automationId;
        if (!app || !article || !id || !button) return;
        try { sessionStorage.removeItem(logKey(id)); } catch (_e) { /* best-effort */ }
        Array.prototype.forEach.call(article.querySelectorAll("[data-automation-log-local]"), function (node) { node.remove(); });
        var entries = article.querySelector("[data-automation-log-entries]");
        if (entries && !entries.querySelector("[data-automation-log-entry]")) {
            entries.innerHTML = '<p class="integrations-readonly-note">' + namespace.escapeHtml(t("integrations-automation-no-log", "No run log entries yet.")) + '</p>';
        }
        button.disabled = true;
        namespace.announce(app, t("integrations-automation-history-cleared", "Local run log cleared."));
    }


    // Shared with 06b_automations_events.js (same namespace) so the
    // delegated handlers live in their own module.
    namespace.__automationInternals = {
        t: t, root: root, botId: botId, base: base,
        fillProviderSelect: fillProviderSelect, syncActions: syncActions,
        runAutomationNow: runAutomationNow, pauseAutomation: pauseAutomation,
        clearHistory: clearHistory, deleteAutomation: deleteAutomation,
        syncPauseButton: syncPauseButton, loadAutomations: loadAutomations,
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
