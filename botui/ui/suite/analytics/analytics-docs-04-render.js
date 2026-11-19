"use strict";

/**
 * DOM renderer for the Documents tab (#2507). All API-derived strings are
 * assigned with textContent; no document content or identity is ever injected
 * as markup.
 */
(function () {
    var ns = window.AnalyticsDocs;
    if (!ns) return;

    var STATUS_MESSAGES = {
        loading: ["analytics-docs-status-loading", "Loading document analytics…"],
        empty: ["analytics-docs-status-empty", "No data available"],
        error: ["analytics-docs-status-error", "Document analytics are unavailable"],
    };

    function statusElement() {
        return document.getElementById("docsStatus");
    }

    ns.setStatus = function (kind) {
        var element = statusElement();
        if (!element) return;
        element.className = "docs-status" + (kind === "loading" ? " gb-state-loading" : "") +
            (kind === "error" ? " gb-state-error" : "");
        var entry = STATUS_MESSAGES[kind];
        element.textContent = entry ? ns.tr(entry[0], entry[1]) : "";
    };

    function messageRow(tbody, colspan, message, isError) {
        tbody.textContent = "";
        var row = document.createElement("tr");
        row.className = "docs-message" + (isError ? " gb-state-error" : "");
        var cell = document.createElement("td");
        cell.colSpan = colspan;
        cell.setAttribute("role", "status");
        cell.textContent = message;
        row.appendChild(cell);
        tbody.appendChild(row);
    }

    ns.markAllTables = function (kind) {
        var entry = STATUS_MESSAGES[kind] || STATUS_MESSAGES.empty;
        ns.SECTIONS.forEach(function (section) {
            section.tables.forEach(function (table) {
                var tbody = document.querySelector('[data-docs-body="' + table.id + '"]');
                if (!tbody) return;
                tbody.textContent = "";
                var row = document.createElement("tr");
                row.className = "docs-message" + (kind === "error" ? " gb-state-error" : "");
                var cell = document.createElement("td");
                cell.colSpan = table.columns.length;
                cell.setAttribute("role", "status");
                if (kind === "loading") {
                    var spinner = document.createElement("span");
                    spinner.className = "docs-loading";
                    spinner.setAttribute("aria-hidden", "true");
                    cell.appendChild(spinner);
                    cell.appendChild(document.createTextNode(" "));
                }
                cell.appendChild(document.createTextNode(ns.tr(entry[0], entry[1])));
                row.appendChild(cell);
                tbody.appendChild(row);
            });
        });
    };

    ns.renderScopes = function (scopes) {
        ns.SCOPE_FIELDS.forEach(function (field) {
            var select = document.getElementById(field.id);
            if (!select) return;
            var previous = select.value;
            var values = scopes[field.name] || [];
            while (select.options.length > 1) select.remove(1);
            values.forEach(function (value) {
                var option = document.createElement("option");
                var label = ns.safeLabel(value);
                option.value = label;
                option.textContent = label;
                select.appendChild(option);
            });
            var stillPresent = Array.prototype.some.call(select.options, function (option) {
                return option.value === previous;
            });
            select.value = stillPresent ? previous : "";
        });
    };

    ns.renderFreshness = function (generatedAt) {
        var element = document.getElementById("docsFreshness");
        if (!element) return;
        if (!generatedAt) {
            element.classList.remove("docs-stale");
            element.textContent = ns.tr("analytics-docs-freshness-unknown", "Freshness unknown");
            return;
        }
        var age = Date.now() - generatedAt;
        var stale = age > ns.FRESHNESS_MAX_MS || age < -60000;
        var time = window.i18n && typeof window.i18n.formatDate === "function"
            ? window.i18n.formatDate(generatedAt, { hour: "2-digit", minute: "2-digit" })
            : (typeof window.gbFormatTime === "function" ? window.gbFormatTime(generatedAt) : "");
        element.classList.toggle("docs-stale", stale);
        element.textContent = stale
            ? ns.tr("analytics-docs-freshness-stale", "Data may be stale (as of { $time })", { time: time })
            : ns.tr("analytics-docs-freshness", "Data as of { $time }", { time: time });
    };

    ns.render = function (model) {
        var display = ns.buildDisplay(model);
        ns.lastDisplay = display;
        ns.lastModel = model;
        ns.renderScopes(model.scopes);

        ns.SECTIONS.forEach(function (section) {
            section.tables.forEach(function (table) {
                var tbody = document.querySelector('[data-docs-body="' + table.id + '"]');
                if (!tbody) return;
                var data = display[table.id];
                if (!data.rows.length) {
                    messageRow(
                        tbody,
                        table.columns.length,
                        ns.tr("analytics-docs-status-empty", "No data available"),
                        false,
                    );
                    return;
                }
                tbody.textContent = "";
                data.rows.forEach(function (cells) {
                    var row = document.createElement("tr");
                    cells.forEach(function (text, index) {
                        var cell = document.createElement("td");
                        cell.textContent = text;
                        var column = table.columns[index];
                        if (column.type !== "text") cell.className = "docs-number";
                        if (text === ns.suppressedMarker()) {
                            cell.classList.add("docs-suppressed");
                            cell.setAttribute("title", ns.suppressionNote());
                        }
                        row.appendChild(cell);
                    });
                    tbody.appendChild(row);
                });
            });
        });

        ns.renderFreshness(model.generatedAt);
        ns.setStatus("ready");
    };
})();
