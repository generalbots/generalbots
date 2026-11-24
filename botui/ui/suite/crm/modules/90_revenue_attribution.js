"use strict";
/* #2548 — honest per-contact revenue-attribution panel for the CRM
 * Campaigns view. Self-contained plain-script IIFE (no build, no ES module,
 * no CDN). It reads the existing, read-only
 * GET /api/crm/revenue/attribution endpoint and never invents a number: a
 * missing API, a failed request or a null money field renders an explicit
 * "unavailable" state instead of zeros or made-up rows.
 *
 * Inert when the CRM window is absent. Tolerates HTMX window re-targeting by
 * re-mounting a missing panel on swap/deep-link events and by keeping all
 * delegated handlers on `document`. Uses only existing i18n keys with English
 * gbT fallbacks — it adds no catalog key and touches no existing CRM module. */
(function () {
    var PANEL_ID = "crm-revenue-attribution-panel";
    var STYLE_ID = "crm-revenue-attribution-style";
    var ENDPOINT = "/api/crm/revenue/attribution";
    var APP_SELECTOR = '[data-gb-app="crm"]';
    var HOST_SELECTOR = "#crm-campaigns-view";
    var BOUND_KEY = "__crmRevenueAttributionBound";

    var state = { report: null, loaded: false, requestId: 0 };

    function tr(key, fallback, params) {
        if (typeof window.gbT === "function") return window.gbT(key, fallback, params);
        return fallback;
    }

    function el(tag, className, text) {
        var node = document.createElement(tag);
        if (className) node.className = className;
        if (text !== undefined && text !== null) node.textContent = String(text);
        return node;
    }

    function clear(node) {
        while (node && node.firstChild) node.removeChild(node.firstChild);
    }

    function firstText(values) {
        for (var i = 0; i < values.length; i += 1) {
            var value = values[i];
            if (value !== null && value !== undefined && String(value).trim() !== "") {
                return String(value);
            }
        }
        return "";
    }

    /* The CRM app lives in the desktop shell. Without it there is nothing to
     * decorate, so the script is a strict no-op (no style, no DOM, no fetch). */
    function findHost() {
        var app = document.querySelector(APP_SELECTOR);
        if (!app) return null;
        var host = app.querySelector ? app.querySelector(HOST_SELECTOR) : null;
        if (!host && document.querySelector) host = document.querySelector(HOST_SELECTOR);
        return host || app;
    }

    function ensureStyle() {
        if (!document.head || document.getElementById(STYLE_ID)) return;
        var style = document.createElement("style");
        style.id = STYLE_ID;
        style.textContent = [
            ".crm-attr-panel{margin:.75rem 0;padding:.6rem .8rem;border:1px solid var(--gb-border,var(--palette-d0d7de));border-radius:6px}",
            ".crm-attr-head{display:flex;align-items:center;justify-content:space-between;gap:.5rem;flex-wrap:wrap}",
            ".crm-attr-title{margin:0;font-size:.95rem;font-weight:600}",
            ".crm-attr-status{font-size:.8rem;opacity:.85;min-height:1.1em}",
            ".crm-attr-status[data-state=unavailable]{color:var(--gb-danger,var(--palette-cd4242))}",
            ".crm-attr-total{display:flex;gap:.5rem;align-items:baseline;margin:.4rem 0;flex-wrap:wrap}",
            ".crm-attr-total-label{font-weight:600}",
            ".crm-attr-total-value{font-variant-numeric:tabular-nums}",
            ".crm-attr-table{width:100%;border-collapse:collapse;font-size:.85rem}",
            ".crm-attr-table th,.crm-attr-table td{text-align:start;padding:.3rem .5rem;border-bottom:1px solid var(--gb-border,var(--palette-d0d7de))}",
            ".crm-attr-amount{font-variant-numeric:tabular-nums}",
            ".crm-attr-unavailable,.crm-attr-empty{margin:.4rem 0 0;font-size:.85rem}"
        ].join("\n");
        document.head.appendChild(style);
    }

    function buildPanel() {
        var panel = el("section", "crm-attr-panel");
        panel.id = PANEL_ID;
        panel.setAttribute("role", "region");
        panel.setAttribute("aria-label", tr("analytics-revenue-summary", "Revenue Summary"));

        var head = el("div", "crm-attr-head");
        head.appendChild(el("h2", "crm-attr-title", tr("analytics-revenue-summary", "Revenue Summary")));
        var button = el("button", "btn-secondary crm-attr-refresh", tr("action-refresh", "Refresh"));
        button.type = "button";
        button.setAttribute("data-crm-attr-refresh", "1");
        button.setAttribute("aria-label", tr("action-refresh", "Refresh"));
        head.appendChild(button);
        panel.appendChild(head);

        var status = el("div", "crm-attr-status");
        status.setAttribute("role", "status");
        status.setAttribute("aria-live", "polite");
        status.setAttribute("aria-atomic", "true");
        panel.appendChild(status);

        var body = el("div", "crm-attr-body");
        panel.appendChild(body);
        panel._crmAttrParts = { status: status, body: body, button: button };
        return panel;
    }

    function setStatus(panel, message, kind) {
        var status = panel._crmAttrParts.status;
        status.textContent = message || "";
        status.setAttribute("data-state", kind || "");
    }

    function unavailableText() {
        return tr("crm-revenue-unavailable", "Revenue attribution is unavailable.");
    }

    function renderUnavailable(panel, message) {
        var text = message || unavailableText();
        var body = panel._crmAttrParts.body;
        clear(body);
        var note = el("p", "crm-attr-unavailable", text);
        note.setAttribute("data-state", "unavailable");
        body.appendChild(note);
        setStatus(panel, text, "unavailable");
        panel._crmAttrParts.button.disabled = false;
    }

    function renderLoading(panel) {
        clear(panel._crmAttrParts.body);
        panel._crmAttrParts.button.disabled = true;
        setStatus(panel, tr("crm-loading", "Loading…"), "loading");
    }

    /* Format only numbers the API actually supplied. An absent or non-finite
     * amount becomes an honest "no data" label, never 0. */
    function formatAmount(value, currency) {
        if (value === null || value === undefined || value === "") {
            return tr("label-no-data", "No data available");
        }
        var number = typeof value === "number" ? value : Number(value);
        if (!isFinite(number)) return tr("label-no-data", "No data available");
        var cur = typeof currency === "string" && currency.trim() ? currency.trim() : "";
        if (cur && typeof Intl !== "undefined" && Intl.NumberFormat) {
            try {
                return new Intl.NumberFormat(undefined, { style: "currency", currency: cur }).format(number);
            } catch (error) {
                /* Unknown currency code — fall through to a plain amount. */
            }
        }
        return (cur ? cur + " " : "") + String(number);
    }

    function formatMoney(money) {
        if (!money || typeof money !== "object") return tr("label-no-data", "No data available");
        return formatAmount(money.amount, money.currency);
    }

    function normalizeRow(raw) {
        if (!raw || typeof raw !== "object") raw = {};
        var contact = firstText([raw.contact, raw.contact_name, raw.name, raw.email, raw.contact_id, raw.id]);
        var source = firstText([raw.source, raw.channel, raw.membership_source, raw.campaign_id]);
        return {
            contact: contact || tr("label-unknown", "Unknown"),
            source: source || tr("label-unknown", "Unknown"),
            amount: formatAmount(raw.amount !== undefined ? raw.amount : raw.value, raw.currency)
        };
    }

    /* Real endpoint shape: aggregate touches, each carrying `source`. Group
     * them into one row per contact/address; money stays "no data" because a
     * touch carries no amount. Explicit row arrays (if a newer API returns
     * them) take precedence and may carry their own amount. */
    function rowsFromTouches(touches) {
        var grouped = {};
        var order = [];
        touches.forEach(function (touch) {
            if (!touch || typeof touch !== "object") return;
            var contact = firstText([touch.email, touch.contact_id]);
            var key = contact || ("row:" + (touch.id || touch.sequence || order.length));
            if (!grouped[key]) {
                grouped[key] = {
                    contact: contact || tr("label-unknown", "Unknown"),
                    sources: [],
                    amount: formatAmount(touch.amount, touch.currency)
                };
                order.push(key);
            }
            var source = firstText([touch.source, touch.channel, touch.raw_event_type, touch.kind]);
            if (source && grouped[key].sources.indexOf(source) === -1) grouped[key].sources.push(source);
            if (touch.amount !== undefined && touch.amount !== null) {
                grouped[key].amount = formatAmount(touch.amount, touch.currency);
            }
        });
        return order.map(function (key) {
            var row = grouped[key];
            return {
                contact: row.contact,
                source: row.sources.join(" · ") || tr("label-unknown", "Unknown"),
                amount: row.amount
            };
        });
    }

    function extractRows(report) {
        var explicit = report.rows || report.attributions || report.contacts || report.bridge;
        if (Array.isArray(explicit)) return explicit.map(normalizeRow);
        if (Array.isArray(report.touches)) return rowsFromTouches(report.touches);
        return [];
    }

    function renderTable(panel, rows) {
        var table = el("table", "crm-attr-table");
        var head = el("thead");
        var headRow = el("tr");
        [tr("crm-col-contact", "Contact"), tr("crm-col-source", "Source"), tr("crm-col-value", "Value")]
            .forEach(function (label) {
                var th = el("th", null, label);
                th.setAttribute("scope", "col");
                headRow.appendChild(th);
            });
        head.appendChild(headRow);
        table.appendChild(head);

        var body = el("tbody");
        rows.forEach(function (row) {
            var trNode = el("tr", "crm-attr-row");
            trNode.appendChild(el("td", "crm-attr-contact", row.contact));
            trNode.appendChild(el("td", "crm-attr-source", row.source));
            trNode.appendChild(el("td", "crm-attr-amount", row.amount));
            body.appendChild(trNode);
        });
        table.appendChild(body);
        panel._crmAttrParts.body.appendChild(table);
    }

    function render(panel) {
        panel._crmAttrParts.button.disabled = false;
        if (!state.loaded || !state.report) {
            renderUnavailable(panel);
            return;
        }
        var report = state.report;
        var body = panel._crmAttrParts.body;
        var rows = extractRows(report);
        clear(body);
        body.setAttribute("data-rows", String(rows.length));

        var total = el("div", "crm-attr-total");
        total.appendChild(el("span", "crm-attr-total-label", tr("analytics-revenue-summary", "Revenue Summary")));
        total.appendChild(el("span", "crm-attr-total-value", formatMoney(report.revenue_influenced)));
        body.appendChild(total);

        if (!rows.length) {
            var empty = tr("label-no-data", "No data available");
            body.appendChild(el("p", "crm-attr-empty", empty));
            setStatus(panel, empty, "empty");
            return;
        }
        renderTable(panel, rows);
        setStatus(panel, tr("crm-contacts", "Contacts"), "ok");
    }

    function fail(panel, requestId) {
        if (requestId !== state.requestId) return;
        state.loaded = false;
        state.report = null;
        renderUnavailable(panel);
    }

    function load(panel) {
        var requestId = ++state.requestId;
        renderLoading(panel);
        if (typeof fetch !== "function") {
            fail(panel, requestId);
            return;
        }
        var promise;
        try {
            promise = fetch(ENDPOINT, {
                headers: { "Accept": "application/json" },
                credentials: "same-origin"
            });
        } catch (error) {
            fail(panel, requestId);
            return;
        }
        Promise.resolve(promise).then(function (response) {
            if (!response || response.ok === false) throw new Error("attribution request failed");
            return response.json ? response.json() : response;
        }).then(function (data) {
            if (requestId !== state.requestId) return;
            state.report = data && typeof data === "object" ? data : null;
            state.loaded = true;
            render(panel);
        }).catch(function () {
            fail(panel, requestId);
        });
    }

    /* `force` = user pressed Refresh; `reload` = fresh script instance or a
     * re-mounted panel after an HTMX swap, so cached data is not trusted. */
    function mount(force, reload) {
        var host = findHost();
        if (!host) return;
        ensureStyle();
        var panel = document.getElementById ? document.getElementById(PANEL_ID) : null;
        var created = false;
        if (!panel || !host.contains || !host.contains(panel)) {
            if (panel && panel.parentNode) panel.parentNode.removeChild(panel);
            panel = buildPanel();
            if (host.insertBefore) host.insertBefore(panel, host.firstChild);
            else host.appendChild(panel);
            created = true;
        }
        if (force || reload || created) load(panel);
        else render(panel);
    }

    function onClick(event) {
        var target = event && event.target;
        if (!target || typeof target.getAttribute !== "function") return;
        var trigger = null;
        if (typeof target.closest === "function") trigger = target.closest("[data-crm-attr-refresh]");
        if (!trigger && target.getAttribute("data-crm-attr-refresh") !== null) trigger = target;
        if (!trigger) return;
        if (event.preventDefault) event.preventDefault();
        mount(true, true);
    }

    function bindListeners() {
        if (typeof document.addEventListener !== "function") return;
        document.addEventListener("click", onClick, false);
        document.addEventListener("DOMContentLoaded", function () { mount(false, false); }, false);
        ["htmx:afterSwap", "htmx:load", "gb:deep-link"].forEach(function (name) {
            document.addEventListener(name, function () { mount(false, false); }, false);
        });
    }

    if (!window[BOUND_KEY]) {
        window[BOUND_KEY] = true;
        bindListeners();
    }
    /* A freshly executed script can never rely on module state from a previous
     * window instance, so the first mount always performs a real fetch. */
    mount(false, true);
})();
